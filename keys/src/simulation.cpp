#include <iostream>
#include <vector>
#include <numeric>
#include <iomanip>
#include <cstdlib>

using namespace std;

struct Patient {
    double before;  // health metric before medicine (0.0 - 1.0)
    double after;   // health metric after medicine (0.0 - 1.0)
    int days;       // number of days of trial (0-14)
    int rating;     // patient experience rating (0-5)
};

int main(int argc, char* argv[]) {
    vector<Patient> patients;

    if (argc == 1) {
        // hospital 1
        patients.push_back({0.2, 0.7, 10, 4});
        patients.push_back({0.3, 0.6, 7, 5});
        patients.push_back({0.4, 0.5, 14, 3});
        patients.push_back({0.7, 0.65, 8, 1});
        patients.push_back({0.5, 0.55, 11, 2});

        // hospital 2
        patients.push_back({0.4, 0.5, 3, 5});
        patients.push_back({0.7, 0.6, 6, 1});
        patients.push_back({0.5, 0.54, 13, 2});
        patients.push_back({0.4, 0.76, 8, 4});
        patients.push_back({0.7, 0.4, 11, 1});
    } else {
        if ((argc - 1) % 4 != 0) {
            throw runtime_error("Invalid number of arguments");
        }
        
        int num_patients = (argc - 1) / 4;
        for (int i = 0; i < num_patients; ++i) {
            double before = stod(argv[1 + i * 4]);
            double after  = stod(argv[1 + i * 4 + 1]);
            int days      = stoi(argv[1 + i * 4 + 2]);
            int rating    = stoi(argv[1 + i * 4 + 3]);
            
            patients.push_back({before, after, days, rating});
        }
    }


    double total_imp  = 0;
    int    total_pats = patients.size();
    double total_days = 0;
    double total_sat  = 0;

    for (const auto& p : patients) {
        total_imp  += (p.after - p.before);
        total_days += p.days;
        total_sat  += p.rating;
    }

    // medical analytics (Cleartext Division)
    double avg_efficacy     = (total_pats > 0)  ? (total_imp / total_pats)   : 0;
    double avg_days         = (total_pats > 0)  ? (total_days / total_pats)  : 0;
    double avg_sat          = (total_pats > 0)  ? (total_sat / total_pats)   : 0;
    double improv_per_day   = (total_days > 0)  ? (total_imp / total_days)   : 0;

    cout << fixed << setprecision(6);
    cout << "\n--- [Medical Report SIMULATION] ---" << endl;
    cout << "Total Improvement   : " << total_imp << endl;
    cout << "Total Patients      : " << (int)total_pats << endl;
    cout << "Total Days of Trial : " << (int)total_days << endl;
    cout << "Total Satisfaction  : " << total_sat << endl;
    cout << "-------------------------------------" << endl;
    cout << "Avg Efficacy        : " << avg_efficacy << endl;
    cout << "Avg Days            : " << avg_days << endl;
    cout << "Avg Satisfaction    : " << avg_sat << endl;
    cout << "Avg Improvement/Day : " << improv_per_day << endl;

    return 0;
}
