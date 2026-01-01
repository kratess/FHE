#include <openfhe.h>
#include <iostream>
#include <iomanip>
#include <chrono>
#include <vector>
#include <fstream>
#include <sstream>
#include <Eigen/Dense>

int main() {
  // Start timer
  auto start = std::chrono::high_resolution_clock::now();

  std::ifstream file("data/mock_data.csv");
  std::vector<std::vector<double>> data;
  std::string line;
  std::getline(file, line);  // Skip header
  while (std::getline(file, line)) {
    std::istringstream iss(line);
    double val;
    std::vector<double> row;
    char comma;
    while (iss >> val) {
      row.push_back(val);
      iss >> comma;
    }
    data.push_back(row);
  }
  file.close();
  Eigen::MatrixXd _data(data.size(), data[0].size());
  for (size_t i = 0; i < data.size(); ++i)
    for (size_t j = 0; j < data[i].size(); ++j) _data(i, j) = data[i][j];
  Eigen::VectorXd Y = _data.col(3);
  Eigen::MatrixXd X = _data.leftCols(3);

  std::cout << "--- DATA\n";

  std::cout << "X size: " << X.size() << "\n";
  std::cout << "---\n";
  std::cout << "Y size: " << Y.size() << "\n";

  // ---------------------------
  // MODEL PARAMETERS
  // ---------------------------
  Eigen::VectorXd W = Eigen::VectorXd::Zero(X.cols());  // weights
  double b = 0;                                         // bias
  double eta = 0.01;                                    // learning rate
  size_t epochs = 100;                                  // number of epochs

  // ---------------------------
  // COMPUTATION
  // ---------------------------

  Eigen::MatrixXd X_T = X.transpose();
  /*std::cout << "--- X_T\n";
  std::cout << X_T << "\n";*/

  // ---------------------------
  // TRAINING LOOP
  // ---------------------------
  for (size_t epoch = 0; epoch < epochs; ++epoch) {
    // Forward pass: compute predictions
    Eigen::VectorXd Y_hat = X * W;
    Y_hat.array() += b;

    // std::cout << "--- Y_hat\n";
    // std::cout << Y_hat << "\n";

    // Compute error vector
    Eigen::VectorXd E = Y_hat - Y;

    // std::cout << "--- E\n";
    // std::cout << E << "\n";

    // Compute gradients and update weights
    Eigen::VectorXd gradient = X_T * E;  // Matrix multiplication: X_T × E
    gradient *= (eta / X.rows());        // Scale by learning rate / batch size
    W -= gradient;                       // Update weights

    // Update bias
    double b_grad = E.sum() / X.rows();
    // std::cout << "--- b_grad " << b_grad << "\n";
    b -= eta * b_grad;
    // std::cout << "--- b " << b << "\n";

    // Compute and print mean squared error for this epoch
    double loss = E.squaredNorm() / E.size();
    std::cout << "Epoch " << (epoch + 1) << " : b = " << std::fixed
              << std::setprecision(2) << b << ", MSE = " << std::fixed
              << std::setprecision(2) << loss << "\n";
    // std::cout << "---\n";
  }

  // Print final regression function
  std::cout << "\nRecovered regression function:\n";
  std::cout << "y = ";
  for (int i = 0; i < W.size(); ++i) {
    std::cout << std::fixed << std::setprecision(4) << W[i] << " * x" << i;
    if (i < W.size() - 1) std::cout << " + ";
  }
  std::cout << " + " << std::fixed << std::setprecision(4) << b << "\n";

  // End timer
  auto end = std::chrono::high_resolution_clock::now();

  // Compute duration in microseconds
  auto duration =
      std::chrono::duration_cast<std::chrono::milliseconds>(end - start)
          .count();

  std::cout << "Elapsed time: " << duration << " ms\n";

  return 0;
}