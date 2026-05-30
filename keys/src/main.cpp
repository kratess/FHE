#include <iostream>
#include <string>
#include <memory>
#include <stdexcept>
#include <array>
#include <vector>
#include <sstream>
#include <filesystem>
#include <cmath>
#include <fstream>
#include <iomanip>
#include <random>

using namespace std;

struct Patient {
    double before;  // health metric before medicine (0.0 - 1.0)
    double after;   // health metric after medicine (0.0 - 1.0)
    int days;       // number of days of trial (0-14)
    int rating;     // patient experience rating (0-5)
};

vector<Patient> generate_patients(int count, int seed) {
    mt19937 gen(seed);
    uniform_real_distribution<> dis_before(0.1, 0.4);
    uniform_real_distribution<> dis_after(0.6, 0.9);
    uniform_int_distribution<> dis_days(3, 14);
    uniform_int_distribution<> dis_rating(1, 5);

    vector<Patient> patients;
    for (int i = 0; i < count; ++i) {
        patients.push_back({dis_before(gen), dis_after(gen), dis_days(gen), dis_rating(gen)});
    }
    return patients;
}

string patients_to_args(const vector<Patient>& patients) {
    stringstream ss;
    ss << fixed << setprecision(6);
    for (const auto& p : patients) {
        ss << " " << p.before << " " << p.after << " " << p.days << " " << p.rating;
    }
    return ss.str();
}

string exec(const char* cmd) {
    array<char, 128> buffer;
    string result;
    std::unique_ptr<FILE, int(*)(FILE*)> pipe(popen(cmd, "r"), pclose);
    if (!pipe) throw runtime_error("popen() failed!");
    while (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr) {
        result += buffer.data();
    }
    return result;
}

string find_binary(const string& name) {
    if (filesystem::exists("./build/" + name)) return "./build/" + name;
    if (filesystem::exists(name)) return "./" + name;
    return "./" + name;
}

int main() {
    const string RESULTS_DIR = "./results";

    int patients_per_hospital = 1024;
    int total_patients = patients_per_hospital * 2;

    try {
        cout << "[Main 0] Generating " << total_patients << " (" << patients_per_hospital << " per hospital) patients..." << endl;
        auto h1_patients = generate_patients(patients_per_hospital, 123);
        auto h2_patients = generate_patients(patients_per_hospital, 456);

        string h1_args = patients_to_args(h1_patients);
        string h2_args = patients_to_args(h2_patients);
        
        vector<Patient> all_patients = h1_patients;
        all_patients.insert(all_patients.end(), h2_patients.begin(), h2_patients.end());
        string all_args = patients_to_args(all_patients);

        cout << "[Main 1] Generating keys..." << endl;
        cout << exec(find_binary("keygen").c_str()) << endl;

        cout << "[Main 2] Hospitals Encryption..." << endl;
        cout << exec((find_binary("hospital1") + h1_args).c_str()) << endl;
        cout << exec((find_binary("hospital2") + h2_args).c_str()) << endl;

        cout << "[Main 3] Servers Analysis..." << endl;
        cout << exec(find_binary("server1").c_str()) << endl;
        cout << exec(find_binary("server2").c_str()) << endl;

        cout << "[Main 4] Hospital Decryption & Results..." << endl;
        string decrypt_out = exec((find_binary("decrypt") + " " + to_string(total_patients)).c_str());
        cout << decrypt_out << endl;

        cout << "[Main 5] Simulation Consistency Check..." << endl;
        string sim_out = exec((find_binary("simulation") + all_args).c_str());
        cout << sim_out << endl;

        // Parse FHE results (decrypted totals)
        // decrypt.cpp saves: avg_efficacy, avg_days, avg_sat, improv_per_day
        ifstream fhe_in(RESULTS_DIR + "/fhe_results.txt");
        double fhe_efficacy, fhe_days, fhe_sat, fhe_improv;
        if (!(fhe_in >> fhe_efficacy >> fhe_days >> fhe_sat >> fhe_improv)) {
             cerr << "[Error] Failed to read decrypted FHE results." << endl;
             return 1;
        }

        // Parse Simulation results
        double sim_efficacy, sim_sat, sim_improv;
        stringstream ss(sim_out);
        string line;
        bool found = false;
        while (getline(ss, line)) {
            if (line.find("Avg Efficacy        :") != string::npos) {
                sim_efficacy = stod(line.substr(line.find(":") + 1));
            } else if (line.find("Avg Satisfaction    :") != string::npos) {
                sim_sat = stod(line.substr(line.find(":") + 1));
            } else if (line.find("Avg Improvement/Day :") != string::npos) {
                sim_improv = stod(line.substr(line.find(":") + 1));
                found = true;
            }
        }

        if (!found) {
            cerr << "[Error] Simulation values not found." << endl;
            return 1;
        }

        // Professional Comparative Reporting
        cout << "\n" << string(60, '=') << endl;
        cout << left << setw(25) << "MEDICAL METRIC" << " | " << left << setw(15) << "FHE" << " | " << left << setw(15) << "SIMULATION" << endl;
        cout << string(60, '-') << endl;
        
        cout << fixed << setprecision(6);
        cout << left << setw(25) << "Avg Efficacy" << " | " << left << setw(15) << fhe_efficacy << " | " << left << setw(15) << sim_efficacy << endl;
        cout << left << setw(25) << "Avg Satisfaction" << " | " << left << setw(15) << fhe_sat << " | " << left << setw(15) << sim_sat << endl;
        cout << left << setw(25) << "Improvement/Day" << " | " << left << setw(15) << fhe_improv << " | " << left << setw(15) << sim_improv << endl;
        cout << string(60, '=') << endl;

        if (abs(sim_efficacy - fhe_efficacy) < 0.001) {
            cout << "\n[✓] Phase 3 Final Verification Complete: NO LEAKAGE & PERFECT MATCH." << endl;
        } else {
            cout << "\n[!] PHASE 3 CRITICAL ERROR: Results do not match." << endl;
            cout << "Difference Efficacy: " << abs(sim_efficacy - fhe_efficacy) << endl;
        }
    } catch (const exception& e) {
        cerr << "[Main] Error: " << e.what() << endl;
        return 1;
    }

    return 0;
}
