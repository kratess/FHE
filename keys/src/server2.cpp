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

        ifstream esumfile(KEY_DIR + "/eval_sum_key.bin", ios::in | ios::binary);
        if (!cc->DeserializeEvalSumKey(esumfile, SerType::BINARY))
            throw runtime_error("Failed to deserialize EvalSumKey");
        esumfile.close();

        ifstream erotfile(KEY_DIR + "/eval_rotate_key.bin", ios::in | ios::binary);
        if (!cc->DeserializeEvalAutomorphismKey(erotfile, SerType::BINARY))
            throw runtime_error("Failed to deserialize EvalRotateKey");
        erotfile.close();

        // load masked ciphertexts
        Ciphertext<DCRTPoly> ct_b_masked = load_ct(RESULTS_DIR + "/ct_b_masked.bin");
        Ciphertext<DCRTPoly> ct_a_masked = load_ct(RESULTS_DIR + "/ct_a_masked.bin");
        Ciphertext<DCRTPoly> ct_d_masked = load_ct(RESULTS_DIR + "/ct_d_masked.bin");
        Ciphertext<DCRTPoly> ct_r_masked = load_ct(RESULTS_DIR + "/ct_r_masked.bin");

        Ciphertext<DCRTPoly> ct_b_masked2 = load_ct(RESULTS_DIR + "/ct_b_masked2.bin");
        Ciphertext<DCRTPoly> ct_a_masked2 = load_ct(RESULTS_DIR + "/ct_a_masked2.bin");
        Ciphertext<DCRTPoly> ct_d_masked2 = load_ct(RESULTS_DIR + "/ct_d_masked2.bin");
        Ciphertext<DCRTPoly> ct_r_masked2 = load_ct(RESULTS_DIR + "/ct_r_masked2.bin");

        // concatenate ciphertexts
        Ciphertext<DCRTPoly> ct_b_masked_total = cc->EvalAdd(ct_b_masked, ct_b_masked2);
        Ciphertext<DCRTPoly> ct_a_masked_total = cc->EvalAdd(ct_a_masked, ct_a_masked2);
        Ciphertext<DCRTPoly> ct_d_masked_total = cc->EvalAdd(ct_d_masked, ct_d_masked2);
        Ciphertext<DCRTPoly> ct_r_masked_total = cc->EvalAdd(ct_r_masked, ct_r_masked2);

        // calculate improvement: a - b
        // layout: [I, 0, 0, 0,    I, 0, 0, 0,    ...]
        Ciphertext<DCRTPoly> ct_improvement = cc->EvalSub(cc->EvalRotate(ct_a_masked_total, 1), ct_b_masked_total);

        // calculate total improvement
        Ciphertext<DCRTPoly> ct_total_improvement = cc->EvalSum(ct_improvement, cc->GetRingDimension() / 2);

        // calculate total days of treatment
        Ciphertext<DCRTPoly> ct_total_days = cc->EvalSum(ct_d_masked_total, cc->GetRingDimension() / 2);

        // calculate total satisfaction
        Ciphertext<DCRTPoly> ct_total_satisfaction = cc->EvalSum(ct_r_masked_total, cc->GetRingDimension() / 2);

        cout << "[Server 2] Saving results..." << endl;

        if (!Serial::SerializeToFile(RESULTS_DIR + "/ct_improvement.bin", ct_improvement, SerType::BINARY))
            throw runtime_error("Failed to serialize results/ct_improvement.bin");
        if (!Serial::SerializeToFile(RESULTS_DIR + "/ct_total_improvement.bin", ct_total_improvement, SerType::BINARY))
            throw runtime_error("Failed to serialize results/ct_total_improvement.bin");
        if (!Serial::SerializeToFile(RESULTS_DIR + "/ct_total_days.bin", ct_total_days, SerType::BINARY))
            throw runtime_error("Failed to serialize results/ct_total_days.bin");
        if (!Serial::SerializeToFile(RESULTS_DIR + "/ct_total_satisfaction.bin", ct_total_satisfaction, SerType::BINARY))
            throw runtime_error("Failed to serialize results/ct_total_satisfaction.bin");
    } catch (const exception& e) {
        cerr << "[Server 2] Error: " << e.what() << endl;
        return 1;
    }

    return 0;
}
