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
    bool isBootstrapping;
};

class ContextManager {
private:
    static std::unique_ptr<ContextData> currentContext;
    static std::tuple<std::string, int, int, bool> currentParams;
public:
    struct BootstrapConfig {
        /* Bootstrapping parameters.
        * We set a budget for the number of levels we can consume in bootstrapping for encoding and decoding, respectively.
        * Using larger numbers of levels reduces the complexity and number of rotation keys,
        * but increases the depth required for bootstrapping.
        * We must choose values smaller than ceil(log2(slots)). A level budget of {4, 4} is good for higher ring
        * dimensions (65536 and higher).
        */
        static inline const std::vector<uint32_t> levelBudget = {4, 4};

        // Note that the actual number of levels avalailable after bootstrapping before next bootstrapping
        // will be levelsAvailableAfterBootstrap - 1 because an additional level
        // is used for scaling the ciphertext before next bootstrapping (in 64-bit CKKS bootstrapping)
        static constexpr uint32_t levelsAvailableAfterBootstrap = 8;
    };

    static CCParams<CryptoContextCKKSRNS> GetParams(int depth, int ringDim, bool bootstrapping) {
        CCParams<CryptoContextCKKSRNS> parameters;

        SecretKeyDist secretKeyDist = UNIFORM_TERNARY;  // SPARSE_TERNARY or UNIFORM_TERNARY {-1, 0, +1}
        SecurityLevel securityLevel = HEStd_NotSet;     // If different from HEStd_NotSet, do not to set ring dimension
        
        parameters.SetSecretKeyDist(secretKeyDist);
        parameters.SetSecurityLevel(securityLevel);
        parameters.SetRingDim(ringDim);
        
        // Don't change, only expert users should modify
        #if NATIVEINT == 128
            ScalingTechnique rescaleTech = FIXEDAUTO;
            usint dcrtBits = 78;
            usint firstMod = 89;
        #else
            ScalingTechnique rescaleTech = FLEXIBLEAUTO;
            usint dcrtBits = 59;
            usint firstMod = 60;
        #endif

        parameters.SetScalingModSize(dcrtBits);
        parameters.SetScalingTechnique(rescaleTech);
        parameters.SetFirstModSize(firstMod);

        if (bootstrapping) {
            parameters.SetMultiplicativeDepth(
                BootstrapConfig::levelsAvailableAfterBootstrap +
                FHECKKSRNS::GetBootstrapDepth(
                    BootstrapConfig::levelBudget,
                    secretKeyDist
                )
            );
        } else {
            parameters.SetMultiplicativeDepth(depth);
        }
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

    static ContextData* GetContext(const std::string& opName, int depth, int ringDim, bool bootstrapping) {
        auto key = std::make_tuple(opName, depth, ringDim, bootstrapping);

        // If context doesn't exist or params changed, create new one
        if (!currentContext || currentParams != key) {
            Reset();
            
            int slots = ringDim / 2; // CKKS packs real + imaginary
            std::cout << "Generating context for Depth=" << depth << ", RingDim=" << ringDim << " (Slots=" << slots << "), Bootstrap=" << bootstrapping << std::endl;
            auto data = std::make_unique<ContextData>();
            data->depth = depth;
            data->slots = slots;
            data->isBootstrapping = bootstrapping;

            CCParams<CryptoContextCKKSRNS> parameters = GetParams(depth, ringDim, bootstrapping);

            data->cc = GenCryptoContext(parameters);
            data->cc->Enable(PKE);          // Enable public key encryption functionality
            data->cc->Enable(KEYSWITCH);    // Enable key switching (required for changing ciphertext keys or performing rotations)
            data->cc->Enable(LEVELEDSHE);   // Enable leveled SHE (Somewhat Homomorphic Encryption) operations
            data->cc->Enable(ADVANCEDSHE);  // Enable advanced SHE features like rotations and rescaling

            if (bootstrapping) {
                data->cc->Enable(FHE);  // Enable full FHE (bootstrapping) capabilities
            }

            data->keys = data->cc->KeyGen();
            data->cc->EvalMultKeyGen(data->keys.secretKey); // Required for multiplication
            data->cc->EvalSumKeyGen(data->keys.secretKey);  // Required for addition over rotations
            if (bootstrapping) {
                // Bootstrapping Setup
                data->cc->EvalBootstrapSetup(BootstrapConfig::levelBudget);
                data->cc->EvalBootstrapKeyGen(data->keys.secretKey, slots);
            }

            currentContext = std::move(data);
            currentParams = key;
        }
        return currentContext.get();
    }
};

std::unique_ptr<ContextData> ContextManager::currentContext = nullptr;
std::tuple<std::string, int, int, bool> ContextManager::currentParams = {"", 0, 0, false};

static void BM_ContextCreation(benchmark::State& state) {
    int depth = state.range(0);
    int ringDim = state.range(1);

    for (auto _ : state) {
        auto parameters = ContextManager::GetParams(depth, ringDim, false);
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
    ContextData* ctx = ContextManager::GetContext("EvalKeyGen", depth, ringDim, false);

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
    ContextData* ctx = ContextManager::GetContext("KeyGen", depth, ringDim, false);
    
    for (auto _ : state) {
        auto kp = ctx->cc->KeyGen();
        benchmark::DoNotOptimize(kp);
    }
    
    state.counters["MB"] = benchmark::Counter(getHeapUsageMB(), static_cast<benchmark::Counter::Flags>(benchmark::Counter::kDefaults & ~benchmark::Counter::kAvgIterations));
}

static void BM_Encrypt(benchmark::State& state) {
    int depth = state.range(0);
    int ringDim = state.range(1);
    ContextData* ctx = ContextManager::GetContext("Encrypt", depth, ringDim, false);
    std::vector<double> x(ctx->slots, 1.0);
    Plaintext ptxt = ctx->cc->MakeCKKSPackedPlaintext(x);

    for (auto _ : state) {
        auto ciphertext = ctx->cc->Encrypt(ctx->keys.publicKey, ptxt);
        benchmark::DoNotOptimize(ciphertext);
        ciphertext = nullptr;
    }
    
    state.counters["MB"] = benchmark::Counter(getHeapUsageMB(), static_cast<benchmark::Counter::Flags>(benchmark::Counter::kDefaults & ~benchmark::Counter::kAvgIterations));
}

static void BM_Decrypt(benchmark::State& state) {
    int depth = state.range(0);
    int ringDim = state.range(1);
    ContextData* ctx = ContextManager::GetContext("Decrypt", depth, ringDim, false);
    std::vector<double> x(ctx->slots, 1.0);
    Plaintext ptxt = ctx->cc->MakeCKKSPackedPlaintext(x);
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
    ContextData* ctx = ContextManager::GetContext("EvalAdd", state.range(0), state.range(1), false);
    std::vector<double> x(ctx->slots, 1.0);
    Plaintext ptxt = ctx->cc->MakeCKKSPackedPlaintext(x);
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
    ContextData* ctx = ContextManager::GetContext("EvalMult", depth, ringDim, false);
    std::vector<double> x(ctx->slots, 1.0);
    Plaintext ptxt = ctx->cc->MakeCKKSPackedPlaintext(x);
    auto c1 = ctx->cc->Encrypt(ctx->keys.publicKey, ptxt);
    auto c2 = ctx->cc->Encrypt(ctx->keys.publicKey, ptxt);

    for (auto _ : state) {
        auto result = ctx->cc->EvalMult(c1, c2);
        benchmark::DoNotOptimize(result);
        result = nullptr;
    }
    
    state.counters["MB"] = benchmark::Counter(getHeapUsageMB(), static_cast<benchmark::Counter::Flags>(benchmark::Counter::kDefaults & ~benchmark::Counter::kAvgIterations));
}

static void BM_BootstrapKeyGen(benchmark::State& state) {
    int depth = state.range(0);
    int ringDim = state.range(1);
    ContextData* ctx = ContextManager::GetContext("BootstrapKeyGen", depth, ringDim, true);

    for (auto _ : state) {
        ctx->cc->ClearEvalMultKeys();
        ctx->cc->ClearEvalAutomorphismKeys();
        ctx->cc->EvalBootstrapKeyGen(ctx->keys.secretKey, ctx->slots);
    }
    
    state.counters["MB"] = benchmark::Counter(getHeapUsageMB(), static_cast<benchmark::Counter::Flags>(benchmark::Counter::kDefaults & ~benchmark::Counter::kAvgIterations));
}

static void BM_Bootstrap(benchmark::State& state) {
    int depth = state.range(0);
    int ringDim = state.range(1);
    ContextData* ctx = ContextManager::GetContext("Bootstrap", depth, ringDim, true);
    std::vector<double> x(ctx->slots, 1.0);
    Plaintext ptxt = ctx->cc->MakeCKKSPackedPlaintext(x);
    auto ciphertext = ctx->cc->Encrypt(ctx->keys.publicKey, ptxt);
    
    for (auto _ : state) {
        try {
            auto result = ctx->cc->EvalBootstrap(ciphertext);
            benchmark::DoNotOptimize(result);
            result = nullptr;
        } catch (const std::exception& e) {
            std::string err = "Bootstrap failed: ";
            err += e.what();
            state.SkipWithError(err.c_str());
            break; 
        }
    }
    ciphertext = nullptr;
    ptxt = nullptr;

    state.counters["MB"] = benchmark::Counter(getHeapUsageMB(), static_cast<benchmark::Counter::Flags>(benchmark::Counter::kDefaults & ~benchmark::Counter::kAvgIterations));
}

static void CustomArguments(benchmark::internal::Benchmark* b) {
    std::vector<int> depths = {1, 2, 4, 8, 16, 32, 64, 128, 256, 512};
    std::vector<int> ringDims = {256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536};

    for (int d : depths) {
        for (int r : ringDims) {
            b->Args({d, r});
        }
    }
}

static void BootstrapArguments(benchmark::internal::Benchmark* b) {
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

BENCHMARK(BM_BootstrapKeyGen)->Apply(BootstrapArguments)->Unit(benchmark::kMillisecond)->Repetitions(kBenchmarkRepetitions)->DisplayAggregatesOnly();
BENCHMARK(BM_Bootstrap)->Apply(BootstrapArguments)->Unit(benchmark::kMillisecond)->Repetitions(kBenchmarkRepetitions)->DisplayAggregatesOnly();

BENCHMARK_MAIN();
