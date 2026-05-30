#include "openfhe.h"
#include "utils/serial.h"
#include "cryptocontext-ser.h"
#include "key/key-ser.h"
#include "ciphertext-ser.h"
#include <iostream>
#include <vector>
#include <filesystem>

using namespace lbcrypto;
using namespace std;

struct Patient {
    double before;  // health metric before medicine (0.0 - 1.0)
    double after;   // health metric after medicine (0.0 - 1.0)
    int days;       // number of days of trial (0-14)
    int rating;     // patient experience rating (0-5)
};

void encrypt_and_save(CryptoContext<DCRTPoly> cc, PublicKey<DCRTPoly> pubKey, const vector<double>& values, const string& filename) {
    auto pt = cc->MakeCKKSPackedPlaintext(values);
    auto ct = cc->Encrypt(pubKey, pt);
    if (!Serial::SerializeToFile(filename, ct, SerType::BINARY))
        throw runtime_error("Failed to serialize " + filename);
}

int main(int argc, char* argv[]) {
    const string KEY_DIR = "./keys";
    const string RESULTS_DIR = "./results";

    filesystem::create_directories(RESULTS_DIR);

    vector<double> data;

    if (argc == 1) {
        vector<Patient> patients;

        patients.push_back({0.2, 0.7, 10, 4});
        patients.push_back({0.3, 0.6, 7, 5});
        patients.push_back({0.4, 0.5, 14, 3});
        patients.push_back({0.7, 0.65, 8, 1});
        patients.push_back({0.5, 0.55, 11, 2});

        for (const auto& p : patients) {
            data.push_back(p.before);
            data.push_back(p.after);
            data.push_back(static_cast<double>(p.days));
            data.push_back(static_cast<double>(p.rating));
        }
    } else {
        if ((argc - 1) % 4 != 0) {
            throw runtime_error("Invalid number of arguments");
        }

        // data passed as args
        for (int i = 1; i < argc; ++i) {
            data.push_back(atof(argv[i]));
        }
    }

    try {
        CryptoContext<DCRTPoly> cc;
        if (!Serial::DeserializeFromFile(KEY_DIR + "/cryptocontext.bin", cc, SerType::BINARY))
            throw runtime_error("Failed to deserialize CryptoContext");

        PublicKey<DCRTPoly> pubKey;
        if (!Serial::DeserializeFromFile(KEY_DIR + "/public_key.bin", pubKey, SerType::BINARY))
            throw runtime_error("Failed to deserialize Public Key");

        cout << "[Hospital 1] Encrypting " << (data.size() / 4) << " patient records" << endl;

        encrypt_and_save(cc, pubKey, data, RESULTS_DIR + "/h1_data.bin");
    } catch (const exception& e) {
        cerr << "[Hospital 1] Error: " << e.what() << endl;
        return 1;
    }

    return 0;
}
