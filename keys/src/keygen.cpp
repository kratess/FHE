#include "openfhe.h"
#include "utils/serial.h"
#include "cereal/types/polymorphic.hpp"
#include "cryptocontext-ser.h"
#include "key/key-ser.h"
#include "scheme/ckksrns/ckksrns-ser.h"
#include <iostream>
#include <vector>
#include <filesystem>

using namespace lbcrypto;
using namespace std;

CryptoContext<DCRTPoly> create_context() {
    SecretKeyDist secretKeyDist = UNIFORM_TERNARY;
    SecurityLevel securityLevel = HEStd_NotSet;
    uint32_t ringDimension      = 8192;

    CCParams<CryptoContextCKKSRNS> parameters;
    parameters.SetSecretKeyDist(secretKeyDist);
    parameters.SetSecurityLevel(securityLevel);
    parameters.SetRingDim(ringDimension);

#if NATIVEINT == 128
    ScalingTechnique rescaleTech = FIXEDAUTO;
    usint dcrtBits  = 78;
    usint firstMod  = 89;
#else
    ScalingTechnique rescaleTech = FLEXIBLEAUTO;
    usint dcrtBits  = 59;
    usint firstMod  = 60;
#endif

    parameters.SetScalingModSize(dcrtBits);
    parameters.SetScalingTechnique(rescaleTech);
    parameters.SetFirstModSize(firstMod);
    parameters.SetMultiplicativeDepth(10);

    CryptoContext<DCRTPoly> cc = GenCryptoContext(parameters);
    cc->Enable(PKE);
    cc->Enable(KEYSWITCH);
    cc->Enable(LEVELEDSHE);
    cc->Enable(ADVANCEDSHE);

    cout << "[+] CryptoContext created" << endl;
    return cc;
}

KeyPair<DCRTPoly> generate_keys(CryptoContext<DCRTPoly> cc) {
    auto keyPair = cc->KeyGen();

    if (!keyPair.good())
        throw runtime_error("Key generation failed");

    cout << "[+] KeyPair generated" << endl;

    vector<int32_t> rotations = {1};
    cc->EvalRotateKeyGen(keyPair.secretKey, rotations);
    cout << "[+] Rotation keys generated" << endl;

    cc->EvalMultKeyGen(keyPair.secretKey);
    cout << "[+] EvalMult keys generated" << endl;

    cc->EvalSumKeyGen(keyPair.secretKey);
    cout << "[+] EvalSum keys generated" << endl;

    return keyPair;
}

void export_keys(CryptoContext<DCRTPoly> cc,
                 KeyPair<DCRTPoly>       keys,
                 const string&           outDir) {
    filesystem::create_directories(outDir);

    // ── Crypto context (scheme parameters) ──────────────────────────────────
    if (!Serial::SerializeToFile(outDir + "/cryptocontext.bin", cc, SerType::BINARY))
        throw runtime_error("Failed to serialize CryptoContext");
    cout << "[+] CryptoContext saved  -> " << outDir << "/cryptocontext.bin" << endl;

    // ── Public key ───────────────────────────────────────────────────────────
    if (!Serial::SerializeToFile(outDir + "/public_key.bin", keys.publicKey, SerType::BINARY))
        throw runtime_error("Failed to serialize public key");
    cout << "[+] Public key saved     -> " << outDir << "/public_key.bin" << endl;

    // ── Secret key ───────────────────────────────────────────────────────────
    if (!Serial::SerializeToFile(outDir + "/secret_key.bin", keys.secretKey, SerType::BINARY))
        throw runtime_error("Failed to serialize secret key");
    cout << "[+] Secret key saved     -> " << outDir << "/secret_key.bin" << endl;

    // ── Evaluation / relinearisation keys ───────────────────────────────────
    ofstream emkeyfile(outDir + "/eval_mult_key.bin", ios::out | ios::binary);
    if (!emkeyfile.is_open())
        throw runtime_error("Cannot open eval_mult_key.bin for writing");
    if (!cc->SerializeEvalMultKey(emkeyfile, SerType::BINARY))
        throw runtime_error("Failed to serialize EvalMultKey");
    emkeyfile.close();
    cout << "[+] EvalMult key saved   -> " << outDir << "/eval_mult_key.bin" << endl;

    // ── Rotation keys ────────────────────────────────────────────────────────
    ofstream erotfile(outDir + "/eval_rotate_key.bin", ios::out | ios::binary);
    if (!erotfile.is_open())
        throw runtime_error("Cannot open eval_rotate_key.bin for writing");
    if (!cc->SerializeEvalAutomorphismKey(erotfile, SerType::BINARY))
        throw runtime_error("Failed to serialize EvalRotateKey");
    erotfile.close();
    cout << "[+] EvalRotate key saved -> " << outDir << "/eval_rotate_key.bin" << endl;

    // ── EvalSum keys ─────────────────────────────────────────────────────────
    ofstream esumfile(outDir + "/eval_sum_key.bin", ios::out | ios::binary);
    if (!esumfile.is_open())
        throw runtime_error("Cannot open eval_sum_key.bin for writing");
    if (!cc->SerializeEvalSumKey(esumfile, SerType::BINARY))
        throw runtime_error("Failed to serialize EvalSumKey");
    esumfile.close();
    cout << "[+] EvalSum key saved    -> " << outDir << "/eval_sum_key.bin" << endl;
}

void verify_round_trip(const string& outDir) {
    cout << "\n[*] Verifying round-trip deserialization..." << endl;

    CryptoContext<DCRTPoly> cc;
    if (!Serial::DeserializeFromFile(outDir + "/cryptocontext.bin", cc, SerType::BINARY))
        throw runtime_error("Failed to deserialize CryptoContext");

    PublicKey<DCRTPoly> pubKey;
    if (!Serial::DeserializeFromFile(outDir + "/public_key.bin", pubKey, SerType::BINARY))
        throw runtime_error("Failed to deserialize public key");

    PrivateKey<DCRTPoly> secKey;
    if (!Serial::DeserializeFromFile(outDir + "/secret_key.bin", secKey, SerType::BINARY))
        throw runtime_error("Failed to deserialize secret key");

    ifstream emkeyfile(outDir + "/eval_mult_key.bin", ios::in | ios::binary);
    cc->ClearEvalMultKeys();
    if (!cc->DeserializeEvalMultKey(emkeyfile, SerType::BINARY))
        throw runtime_error("Failed to deserialize EvalMultKey");
    emkeyfile.close();

    ifstream erotfile(outDir + "/eval_rotate_key.bin", ios::in | ios::binary);
    cc->ClearEvalAutomorphismKeys();
    if (!cc->DeserializeEvalAutomorphismKey(erotfile, SerType::BINARY))
        throw runtime_error("Failed to deserialize EvalRotateKey");
    erotfile.close();

    ifstream esumfile(outDir + "/eval_sum_key.bin", ios::in | ios::binary);
    cc->ClearEvalSumKeys();
    if (!cc->DeserializeEvalSumKey(esumfile, SerType::BINARY))
        throw runtime_error("Failed to deserialize EvalSumKey");
    esumfile.close();

    cout << "[+] All keys deserialized successfully" << endl;
}

int main() {
    const string KEY_DIR = "./keys";
    
    filesystem::create_directories(KEY_DIR);

    try {
        auto cc   = create_context();
        auto keys = generate_keys(cc);

        cout << "\n[*] Exporting keys to " << KEY_DIR << "/" << endl;
        export_keys(cc, keys, KEY_DIR);

        verify_round_trip(KEY_DIR);

        cout << "\n[✓] Done. Key files in " << KEY_DIR << "/" << endl;
    } catch (const exception& e) {
        cerr << "[!] Error: " << e.what() << endl;
        return 1;
    }

    return 0;
}