#include "openfhe.h"
#include "benchmark/benchmark.h"
#include <vector>
#include <map>
#include <memory>
#include <iostream>
#include <algorithm>
#include <fstream>
#include <string>
#include <malloc.h>

using namespace lbcrypto;

namespace {
constexpr int kBenchmarkRepetitions = 10;
}  // namespace

#if defined(__GLIBC__) && (__GLIBC__ > 2 || (__GLIBC__ == 2 && __GLIBC_MINOR__ >= 33))
    #define HAS_MALLINFO2
#endif

double getHeapUsageMB() {
    #ifdef HAS_MALLINFO2
        struct mallinfo2 mi = mallinfo2();
    #else
        struct mallinfo mi = mallinfo();
    #endif
        return (double)(mi.uordblks + mi.hblkhd) / (1024.0 * 1024.0);
}

struct ContextData {
    CryptoContext<DCRTPoly> cc;
    KeyPair<DCRTPoly> keys;
    int depth;
    int slots;
};

class ContextManager {
private:
    static std::unique_ptr<ContextData> currentContext;
    static std::tuple<std::string, int, int> currentParams;
public:
    static CCParams<CryptoContextBFVRNS> GetParams(int depth, int ringDim) {
        CCParams<CryptoContextBFVRNS> parameters;

        parameters.SetSecurityLevel(HEStd_NotSet);
        parameters.SetRingDim(ringDim);
        parameters.SetMultiplicativeDepth(depth);
        // We could've chose 65537 since the max ring dimension is 16384, but we use 786433 for comparison with BGV
        parameters.SetPlaintextModulus(786433); // (q-1)/m must be an integer [m = 2 * ringDim]

        return parameters;
    }

    static void Reset() {
        if (currentContext) {
            if (currentContext->cc) {
                currentContext->cc->ClearEvalMultKeys();
                currentContext->cc->ClearEvalAutomorphismKeys();
                try { currentContext->cc->ClearEvalSumKeys(); } catch(...) {}
                currentContext->cc = nullptr;
            }
            currentContext->keys.publicKey = nullptr;
            currentContext->keys.secretKey = nullptr;
            currentContext.reset();
        }
        CryptoContextFactory<DCRTPoly>::ReleaseAllContexts();
        malloc_trim(0);
    }

    static ContextData* GetContext(const std::string& opName, int depth, int ringDim) {
        auto key = std::make_tuple(opName, depth, ringDim);

        if (!currentContext || currentParams != key) {
            Reset();
            
            int slots = ringDim; // BGV/BFV have ringDim slots with PackedEncoding
            std::cout << "Generating BFV context for Depth=" << depth << ", RingDim=" << ringDim << " (Slots=" << slots << ")" << std::endl;
            auto data = std::make_unique<ContextData>();
            data->depth = depth;
            data->slots = slots;

            CCParams<CryptoContextBFVRNS> parameters = GetParams(depth, ringDim);

            data->cc = GenCryptoContext(parameters);
            data->cc->Enable(PKE);          // Enable public key encryption functionality
            data->cc->Enable(KEYSWITCH);    // Enable key switching (required for changing ciphertext keys or performing rotations)
            data->cc->Enable(LEVELEDSHE);   // Enable leveled SHE (Somewhat Homomorphic Encryption) operations
            data->cc->Enable(ADVANCEDSHE);  // Enable advanced SHE features like rotations and rescaling

            data->keys = data->cc->KeyGen();
            data->cc->EvalMultKeyGen(data->keys.secretKey); // Required for multiplication
            data->cc->EvalSumKeyGen(data->keys.secretKey);  // Required for addition over rotations

            currentContext = std::move(data);
            currentParams = key;
        }
        return currentContext.get();
    }
};

std::unique_ptr<ContextData> ContextManager::currentContext = nullptr;
std::tuple<std::string, int, int> ContextManager::currentParams = {"", 0, 0};

static void BM_ContextCreation(benchmark::State& state) {
    int depth = state.range(0);
    int ringDim = state.range(1);

    for (auto _ : state) {
        auto parameters = ContextManager::GetParams(depth, ringDim);
        auto cc = GenCryptoContext(parameters);
        cc->Enable(PKE);
        cc->Enable(KEYSWITCH);
        cc->Enable(LEVELEDSHE);
        cc->Enable(ADVANCEDSHE);
        benchmark::DoNotOptimize(cc);
    }
    
    state.counters["MB"] = benchmark::Counter(getHeapUsageMB(), static_cast<benchmark::Counter::Flags>(benchmark::Counter::kDefaults & ~benchmark::Counter::kAvgIterations));
}

static void BM_EvalKeyGen(benchmark::State& state) {
    int depth = state.range(0);
    int ringDim = state.range(1);
    ContextData* ctx = ContextManager::GetContext("EvalKeyGen", depth, ringDim);

    for (auto _ : state) {
        ctx->cc->ClearEvalMultKeys();
        ctx->cc->ClearEvalAutomorphismKeys();
        ctx->cc->EvalMultKeyGen(ctx->keys.secretKey);
        ctx->cc->EvalRotateKeyGen(ctx->keys.secretKey, {1, -1, 2});
    }
    
    state.counters["MB"] = benchmark::Counter(getHeapUsageMB(), static_cast<benchmark::Counter::Flags>(benchmark::Counter::kDefaults & ~benchmark::Counter::kAvgIterations));
}

static void BM_KeyGen(benchmark::State& state) {
    int depth = state.range(0);
    int ringDim = state.range(1);
    ContextData* ctx = ContextManager::GetContext("KeyGen", depth, ringDim);
    
    for (auto _ : state) {
        auto kp = ctx->cc->KeyGen();
        benchmark::DoNotOptimize(kp);
    }
    
    state.counters["MB"] = benchmark::Counter(getHeapUsageMB(), static_cast<benchmark::Counter::Flags>(benchmark::Counter::kDefaults & ~benchmark::Counter::kAvgIterations));
}

static void BM_Encrypt(benchmark::State& state) {
    int depth = state.range(0);
    int ringDim = state.range(1);
    ContextData* ctx = ContextManager::GetContext("Encrypt", depth, ringDim);
    std::vector<int64_t> x(ctx->slots, 1);
    Plaintext ptxt = ctx->cc->MakePackedPlaintext(x);

    for (auto _ : state) {
        auto ciphertext = ctx->cc->Encrypt(ctx->keys.publicKey, ptxt);
        benchmark::DoNotOptimize(ciphertext);
    }
    
    state.counters["MB"] = benchmark::Counter(getHeapUsageMB(), static_cast<benchmark::Counter::Flags>(benchmark::Counter::kDefaults & ~benchmark::Counter::kAvgIterations));
}

static void BM_Decrypt(benchmark::State& state) {
    int depth = state.range(0);
    int ringDim = state.range(1);
    ContextData* ctx = ContextManager::GetContext("Decrypt", depth, ringDim);
    std::vector<int64_t> x(ctx->slots, 1);
    Plaintext ptxt = ctx->cc->MakePackedPlaintext(x);
    auto ciphertext = ctx->cc->Encrypt(ctx->keys.publicKey, ptxt);
    Plaintext result;

    for (auto _ : state) {
        ctx->cc->Decrypt(ctx->keys.secretKey, ciphertext, &result);
        benchmark::DoNotOptimize(result);
    }
    
    state.counters["MB"] = benchmark::Counter(getHeapUsageMB(), static_cast<benchmark::Counter::Flags>(benchmark::Counter::kDefaults & ~benchmark::Counter::kAvgIterations));
}

static void BM_EvalAdd(benchmark::State& state) {
    int depth = state.range(0);
    int ringDim = state.range(1);
    ContextData* ctx = ContextManager::GetContext("EvalAdd", depth, ringDim);
    std::vector<int64_t> x(ctx->slots, 1);
    Plaintext ptxt = ctx->cc->MakePackedPlaintext(x);
    auto c1 = ctx->cc->Encrypt(ctx->keys.publicKey, ptxt);
    auto c2 = ctx->cc->Encrypt(ctx->keys.publicKey, ptxt);

    for (auto _ : state) {
        auto result = ctx->cc->EvalAdd(c1, c2);
        benchmark::DoNotOptimize(result);
    }
    
    state.counters["MB"] = benchmark::Counter(getHeapUsageMB(), static_cast<benchmark::Counter::Flags>(benchmark::Counter::kDefaults & ~benchmark::Counter::kAvgIterations));
}

static void BM_EvalMult(benchmark::State& state) {
    int depth = state.range(0);
    int ringDim = state.range(1);
    ContextData* ctx = ContextManager::GetContext("EvalMult", depth, ringDim);
    std::vector<int64_t> x(ctx->slots, 1);
    Plaintext ptxt = ctx->cc->MakePackedPlaintext(x);
    auto c1 = ctx->cc->Encrypt(ctx->keys.publicKey, ptxt);
    auto c2 = ctx->cc->Encrypt(ctx->keys.publicKey, ptxt);

    for (auto _ : state) {
        auto result = ctx->cc->EvalMult(c1, c2);
        benchmark::DoNotOptimize(result);
    }
    
    state.counters["MB"] = benchmark::Counter(getHeapUsageMB(), static_cast<benchmark::Counter::Flags>(benchmark::Counter::kDefaults & ~benchmark::Counter::kAvgIterations));
}

static void CustomArguments(benchmark::internal::Benchmark* b) {
    std::vector<int> depths = {1, 2, 4, 8, 16, 32, 64, 128};
    std::vector<int> ringDims = {256, 512, 1024, 2048, 4096, 8192, 16384};

    for (int d : depths) {
        for (int r : ringDims) {
            b->Args({d, r});
        }
    }
}

BENCHMARK(BM_ContextCreation)->Apply(CustomArguments)->Unit(benchmark::kMillisecond)->Repetitions(kBenchmarkRepetitions)->DisplayAggregatesOnly();
BENCHMARK(BM_EvalKeyGen)->Apply(CustomArguments)->Unit(benchmark::kMillisecond)->Repetitions(kBenchmarkRepetitions)->DisplayAggregatesOnly();
BENCHMARK(BM_KeyGen)->Apply(CustomArguments)->Unit(benchmark::kMillisecond)->Repetitions(kBenchmarkRepetitions)->DisplayAggregatesOnly();
BENCHMARK(BM_Encrypt)->Apply(CustomArguments)->Unit(benchmark::kMillisecond)->Repetitions(kBenchmarkRepetitions)->DisplayAggregatesOnly();
BENCHMARK(BM_Decrypt)->Apply(CustomArguments)->Unit(benchmark::kMillisecond)->Repetitions(kBenchmarkRepetitions)->DisplayAggregatesOnly();
BENCHMARK(BM_EvalAdd)->Apply(CustomArguments)->Unit(benchmark::kMillisecond)->Repetitions(kBenchmarkRepetitions)->DisplayAggregatesOnly();
BENCHMARK(BM_EvalMult)->Apply(CustomArguments)->Unit(benchmark::kMillisecond)->Repetitions(kBenchmarkRepetitions)->DisplayAggregatesOnly();

BENCHMARK_MAIN();
