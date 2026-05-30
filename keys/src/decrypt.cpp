#include "openfhe.h"
#include "utils/serial.h"
#include "cryptocontext-ser.h"
#include "ciphertext-ser.h"
#include "key/key-ser.h"
#include "scheme/ckksrns/ckksrns-ser.h"
#include <iostream>
#include <vector>
#include <iomanip>
#include <fstream>
#include <cstdlib>

using namespace lbcrypto;
using namespace std;

Ciphertext<DCRTPoly> load_ct(const string& filename) {
    Ciphertext<DCRTPoly> ct;
    if (!Serial::DeserializeFromFile(filename, ct, SerType::BINARY))
        throw runtime_error("Failed to deserialize " + filename);
    return ct;
}

int main(int argc, char* argv[]) {
    const string KEY_DIR = "./keys";
    const string RESULTS_DIR = "./results";

    int total_pats = 10;
    if (argc > 1) total_pats = atoi(argv[1]);

    try {
        CryptoContext<DCRTPoly> cc;
        if (!Serial::DeserializeFromFile(KEY_DIR + "/cryptocontext.bin", cc, SerType::BINARY))
            throw runtime_error("Failed to deserialize CryptoContext");

        PrivateKey<DCRTPoly> sk;
        if (!Serial::DeserializeFromFile(KEY_DIR + "/secret_key.bin", sk, SerType::BINARY))
            throw runtime_error("Failed to deserialize Secret Key");

        // load total data
        auto ct_total_imp  = load_ct(RESULTS_DIR + "/ct_total_improvement.bin");
        auto ct_total_days = load_ct(RESULTS_DIR + "/ct_total_days.bin");
        auto ct_total_sat  = load_ct(RESULTS_DIR + "/ct_total_satisfaction.bin");

        auto decrypt_val = [&](Ciphertext<DCRTPoly> ct) {
            Plaintext pt;
            cc->Decrypt(sk, ct, &pt);
            pt->SetLength(1);
            return pt->GetRealPackedValue()[0];
        };

        double total_imp  = decrypt_val(ct_total_imp);
        double total_days = decrypt_val(ct_total_days);
        double total_sat  = decrypt_val(ct_total_sat);

        // medical analytics
        double avg_efficacy     = (total_pats > 0)  ? (total_imp / total_pats)   : 0;
        double avg_days         = (total_pats > 0)  ? (total_days / total_pats)  : 0;
        double avg_sat          = (total_pats > 0)  ? (total_sat / total_pats)   : 0;
        double improv_per_day   = (total_days > 0)  ? (total_imp / total_days)   : 0;

        cout << fixed << setprecision(6);
        cout << "\n--- [Medical Report FHE] ----------" << endl;
        cout << "Total Improvement   : " << total_imp << endl;
        cout << "Total Patients      : " << (int)total_pats << endl;
        cout << "Total Days of Trial : " << (int)total_days << endl;
        cout << "Total Satisfaction  : " << total_sat << endl;
        cout << "-------------------------------------" << endl;
        cout << "Avg Efficacy        : " << avg_efficacy << endl;
        cout << "Avg Days            : " << avg_days << endl;
        cout << "Avg Satisfaction    : " << avg_sat << endl;
        cout << "Avg Improvement/Day : " << improv_per_day << endl;

        // save for main.cpp comparison
        ofstream out(RESULTS_DIR + "/fhe_results.txt");
        out << avg_efficacy << " " << avg_days << " " << avg_sat << " " << improv_per_day << endl;
        out.close();
    } catch (const exception& e) {
        cerr << "[Decrypt] Error: " << e.what() << endl;
        return 1;
    }

    return 0;
}
