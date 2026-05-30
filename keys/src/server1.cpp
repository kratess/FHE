#include "openfhe.h"
#include "utils/serial.h"
#include "cryptocontext-ser.h"
#include "ciphertext-ser.h"
#include "key/key-ser.h"
#include "scheme/ckksrns/ckksrns-ser.h"
#include <iostream>
#include <vector>

using namespace lbcrypto;
using namespace std;

Ciphertext<DCRTPoly> load_ct(const string& filename) {
    Ciphertext<DCRTPoly> ct;
    if (!Serial::DeserializeFromFile(filename, ct, SerType::BINARY))
        throw runtime_error("Failed to deserialize " + filename);
    return ct;
}

int main() {
    const string KEY_DIR = "./keys";
    const string RESULTS_DIR = "./results";

    try {
        CryptoContext<DCRTPoly> cc;
        if (!Serial::DeserializeFromFile(KEY_DIR + "/cryptocontext.bin", cc, SerType::BINARY))
            throw runtime_error("Failed to deserialize CryptoContext");

        Ciphertext<DCRTPoly> h1 = load_ct(RESULTS_DIR + "/h1_data.bin");
        Ciphertext<DCRTPoly> h2 = load_ct(RESULTS_DIR + "/h2_data.bin");

        // creating masks
        // layout: [B, A, D, R,    B, A, D, R,    ...]
        //         [1, 0, 0, 0,    1, 0, 0, 0,    ...]
        vector<double> data_b, data_a, data_d, data_r;
        // 8192 is the ring dimension
        // 4096 are the usable slots (ckks numbers are complex so require 2 slots each)
        // 1024 is the maximum number of patients per vector
        for (int i = 0; i < cc->GetRingDimension() / 2; ++i) {  
            data_b.push_back((i % 4 == 0) ? 1.0 : 0.0);
            data_a.push_back((i % 4 == 1) ? 1.0 : 0.0);
            data_d.push_back((i % 4 == 2) ? 1.0 : 0.0);
            data_r.push_back((i % 4 == 3) ? 1.0 : 0.0);
        }
        auto pt_b = cc->MakeCKKSPackedPlaintext(data_b);
        auto pt_a = cc->MakeCKKSPackedPlaintext(data_a);
        auto pt_d = cc->MakeCKKSPackedPlaintext(data_d);
        auto pt_r = cc->MakeCKKSPackedPlaintext(data_r);

        auto ct_b_masked = cc->EvalMult(h1, pt_b);
        auto ct_a_masked = cc->EvalMult(h1, pt_a);
        auto ct_d_masked = cc->EvalMult(h1, pt_d);
        auto ct_r_masked = cc->EvalMult(h1, pt_r);

        auto ct_b_masked2 = cc->EvalMult(h2, pt_b);
        auto ct_a_masked2 = cc->EvalMult(h2, pt_a);
        auto ct_d_masked2 = cc->EvalMult(h2, pt_d);
        auto ct_r_masked2 = cc->EvalMult(h2, pt_r);

        cout << "[Server 1] Saving masked data..." << endl;

        if (!Serial::SerializeToFile(RESULTS_DIR + "/ct_b_masked.bin", ct_b_masked, SerType::BINARY))
            throw runtime_error("Failed to serialize results/ct_b_masked.bin");
        if (!Serial::SerializeToFile(RESULTS_DIR + "/ct_a_masked.bin", ct_a_masked, SerType::BINARY))
            throw runtime_error("Failed to serialize results/ct_a_masked.bin");
        if (!Serial::SerializeToFile(RESULTS_DIR + "/ct_d_masked.bin", ct_d_masked, SerType::BINARY))
            throw runtime_error("Failed to serialize results/ct_d_masked.bin");
        if (!Serial::SerializeToFile(RESULTS_DIR + "/ct_r_masked.bin", ct_r_masked, SerType::BINARY))
            throw runtime_error("Failed to serialize results/ct_r_masked.bin");

        if (!Serial::SerializeToFile(RESULTS_DIR + "/ct_b_masked2.bin", ct_b_masked2, SerType::BINARY))
            throw runtime_error("Failed to serialize results/ct_b_masked2.bin");
        if (!Serial::SerializeToFile(RESULTS_DIR + "/ct_a_masked2.bin", ct_a_masked2, SerType::BINARY))
            throw runtime_error("Failed to serialize results/ct_a_masked2.bin");
        if (!Serial::SerializeToFile(RESULTS_DIR + "/ct_d_masked2.bin", ct_d_masked2, SerType::BINARY))
            throw runtime_error("Failed to serialize results/ct_d_masked2.bin");
        if (!Serial::SerializeToFile(RESULTS_DIR + "/ct_r_masked2.bin", ct_r_masked2, SerType::BINARY))
            throw runtime_error("Failed to serialize results/ct_r_masked2.bin");
    } catch (const exception& e) {
        cerr << "[Server 1] Error: " << e.what() << endl;
        return 1;
    }

    return 0;
}
