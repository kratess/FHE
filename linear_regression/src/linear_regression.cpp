#include <openfhe.h>
#include <iostream>
#include <iomanip>
#include <unordered_map>
#include <chrono>
#include <vector>
#include <fstream>
#include <sstream>

using namespace lbcrypto;

constexpr bool DEBUG = false;
constexpr bool BOOTSTRAP = true;

#define PRINT_CT(var) printEncryptedVector(var, 20, #var, 4)
// #define PRINT_CT_TRANSPOSE(var) printEncryptedVector(var, 20, #var, 5)

struct FHEState {
  CryptoContext<DCRTPoly> cc;
  PublicKey<DCRTPoly> pk;
  PrivateKey<DCRTPoly> sk;

  size_t ringDim;
  size_t batchSize;
  size_t depth;
};

// Global instance
FHEState state;

struct Dataset {
  std::vector<std::vector<double>> X;
  std::vector<double> Y;
};

// Map bucket name -> start time
std::unordered_map<std::string, std::chrono::steady_clock::time_point> logBuckets;

// Track last prefix
std::string lastPrefix;

std::string formatDuration(std::chrono::steady_clock::time_point start, std::chrono::steady_clock::time_point end) {
  using namespace std::chrono;

  auto diff = end - start;

  if (diff < seconds(1)) {
    auto ms = duration_cast<milliseconds>(diff).count();
    return std::to_string(ms) + "ms";
  }

  double s = duration_cast<duration<double>>(diff).count();
  std::ostringstream out;
  out << std::fixed << std::setprecision(2) << s << "s";
  return out.str();
}

void log(const std::string& prefix, const std::string& message, const std::string& bucket = "") {
  using clock = std::chrono::steady_clock;

  // Insert blank line if prefix changed
  if (!lastPrefix.empty() && lastPrefix != prefix) {
    std::cout << std::endl;
  }
  lastPrefix = prefix;

  if (bucket.empty()) {
    std::cout << "[" << prefix << "]: " << message << std::endl;
    return;
  }

  auto it = logBuckets.find(bucket);
  if (it == logBuckets.end()) {
    // First time: start timer
    logBuckets[bucket] = clock::now();
    std::cout << "[" << prefix << "]: " << message << std::endl;
  } else {
    auto durationFormatted = formatDuration(it->second, clock::now());
    std::cout << "[" << prefix << "]: " << message << " [" << durationFormatted << "]" << std::endl;
    logBuckets.erase(it);
  }
}

std::pair<Dataset, Dataset> loadAndSplitDataset(const std::string& path, double trainRatio) {
  log("DATA", "Loading data...", "DATA");

  if (trainRatio <= 0.0 || trainRatio > 1.0) {
    throw std::invalid_argument("trainRatio must be in (0,1]");
  }

  std::ifstream file(path);
  if (!file.is_open()) {
    throw std::runtime_error("Cannot open file");
  }

  std::string line;
  std::getline(file, line);  // skip header

  std::vector<std::vector<double>> data;

  while (std::getline(file, line)) {
    std::istringstream iss(line);
    std::vector<double> row;
    double val;
    char comma;

    while (iss >> val) {
      row.push_back(val);
      iss >> comma;
    }
    data.push_back(row);
  }

  if (data.empty()) {
    throw std::runtime_error("Empty dataset");
  }

  size_t nRows = data.size();
  size_t nCols = data[0].size();
  size_t nFeatures = nCols - 1;

  size_t trainSize = static_cast<size_t>(nRows * trainRatio);

  Dataset train;
  Dataset test;

  for (size_t i = 0; i < nRows; ++i) {
    std::vector<double> x(data[i].begin(), data[i].begin() + nFeatures);
    double y = data[i].back();

    if (i < trainSize) {
      train.X.push_back(x);
      train.Y.push_back(y);
    } else {
      test.X.push_back(x);
      test.Y.push_back(y);
    }
  }

  log("DATA", "Data has been loaded", "DATA");

  return {train, test};
}

void initFHEState() {
  log("FHE", "Initializing CKKS cryptosystem", "FHE");

  // Setup CryptoContext
  SecretKeyDist secretKeyDist = UNIFORM_TERNARY;  // SPARSE_TERNARY or UNIFORM_TERNARY {-1, 0, +1}
  SecurityLevel securityLevel = HEStd_NotSet;     // If different from HEStd_NotSet, do not to set ring dimension
  uint32_t ringDimension = 8192;                  // Number of coefficients in the ring, minimum batchSize*2

  CCParams<CryptoContextCKKSRNS> parameters;
  parameters.SetSecretKeyDist(secretKeyDist);
  parameters.SetSecurityLevel(securityLevel);
  parameters.SetRingDim(ringDimension);

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

  /* Bootstrapping parameters.
   * We set a budget for the number of levels we can consume in bootstrapping for encoding and decoding, respectively.
   * Using larger numbers of levels reduces the complexity and number of rotation keys,
   * but increases the depth required for bootstrapping.
   * We must choose values smaller than ceil(log2(slots)). A level budget of {4, 4} is good for higher ring
   * dimensions (65536 and higher).
   */
  std::vector<uint32_t> levelBudget = {4, 4};

  // Note that the actual number of levels avalailable after bootstrapping before next bootstrapping
  // will be levelsAvailableAfterBootstrap - 1 because an additional level
  // is used for scaling the ciphertext before next bootstrapping (in 64-bit CKKS bootstrapping)
  uint32_t levelsAvailableAfterBootstrap = 6 * 10 + 1;
  state.depth = levelsAvailableAfterBootstrap + FHECKKSRNS::GetBootstrapDepth(levelBudget, secretKeyDist);
  parameters.SetMultiplicativeDepth(state.depth);

  log("FHE", "Initializing CryptoContext", "CryptoContext");
  state.cc = GenCryptoContext(parameters);
  log("FHE", "CryptoContext generated", "CryptoContext");

  // Enable features
  state.cc->Enable(PKE);          // Enable public key encryption functionality
  state.cc->Enable(KEYSWITCH);    // Enable key switching (required for changing ciphertext keys or performing rotations)
  state.cc->Enable(LEVELEDSHE);   // Enable leveled SHE (Somewhat Homomorphic Encryption) operations
  state.cc->Enable(ADVANCEDSHE);  // Enable advanced SHE features like multiplication, rotations, and rescaling
  state.cc->Enable(FHE);          // Enable full FHE (bootstrapping) capabilities

  state.ringDim = state.cc->GetRingDimension();
  state.batchSize = state.ringDim / 2;
  log("FHE", "batchSize " + std::to_string(state.batchSize));

  if (BOOTSTRAP) state.cc->EvalBootstrapSetup(levelBudget);

  // Generate keys
  KeyPair keyPair = state.cc->KeyGen();

  state.pk = keyPair.publicKey;
  state.sk = keyPair.secretKey;

  state.cc->EvalMultKeyGen(state.sk);
  state.cc->EvalSumKeyGen(state.sk);
  if (BOOTSTRAP) state.cc->EvalBootstrapKeyGen(state.sk, state.batchSize);

  // Generate Rotation Keys for Manual Gradient Calculation
  std::vector<int> rotationIndices;
  for (int i = 1; i <= 16; ++i) {
    rotationIndices.push_back(-i);
  }
  for (size_t i = 1; i <= state.batchSize; i *= 2) {
    rotationIndices.push_back((int)i);
    rotationIndices.push_back(-(int)i);
  }
  state.cc->EvalRotateKeyGen(state.sk, rotationIndices);

  log("FHE", "FHE initialization finished", "FHE");
}

void printEncryptedVector(const Ciphertext<DCRTPoly>& ct, size_t length = 0, const std::string& label = "", size_t split = 0) {
  if (!DEBUG) return;

  const double eps = 1e-10;
  constexpr size_t LABEL_WIDTH = 8;
  constexpr size_t VAL_WIDTH = 4;

  Plaintext pt;
  state.cc->Decrypt(state.sk, ct, &pt);

  auto complexVals = pt->GetCKKSPackedValue();
  size_t maxPrint = (length == 0 || length > complexVals.size()) ? complexVals.size() : length;

  if (!label.empty()) {
    std::cout << std::left << std::setw(LABEL_WIDTH) << label.substr(0, LABEL_WIDTH) << ": ";
  } else {
    std::cout << "Y_hat: ";
  }

  for (size_t j = 0; j < maxPrint; ++j) {
    double v = complexVals[j].real();
    if (std::abs(v) < eps) v = 0.0;

    double r2 = std::round(v * 100.0) / 100.0;

    std::ostringstream oss;
    oss << std::fixed << std::setprecision(2) << r2;
    std::string out = oss.str();

    if (out.size() > VAL_WIDTH) {
      out = out.substr(0, VAL_WIDTH);
    }

    std::cout << std::left << std::setw(VAL_WIDTH) << out;

    if (split > 1 && (j + 1) % split == 0 && (j + 1) < maxPrint) {
      std::cout << " | ";
    } else {
      std::cout << " ";
    }
  }

  std::cout << std::endl;
}

std::vector<double> flatten(const std::vector<std::vector<double>>& v) {
  size_t total = 0;
  for (const auto& row : v) total += row.size();

  std::vector<double> out;
  out.reserve(total);

  for (const auto& row : v) {
    out.insert(out.end(), row.begin(), row.end());
  }
  return out;
}

std::vector<std::vector<double>> transpose(const std::vector<std::vector<double>>& m) {
  size_t rows = m.size();
  size_t cols = m[0].size();

  std::vector<std::vector<double>> t(cols, std::vector<double>(rows));

  for (size_t i = 0; i < rows; ++i)
    for (size_t j = 0; j < cols; ++j) t[j][i] = m[i][j];

  return t;
}

void padMatrixInPlace(std::vector<std::vector<double>>& v, size_t rows, size_t cols) {
  for (auto& row : v) row.resize(cols, 0.0);

  while (v.size() < rows) v.emplace_back(cols, 0.0);
}

void padBetweenWithTrailingInPlace(std::vector<double>& v, size_t pad) {
  if (v.empty() || pad == 0) return;

  size_t n = v.size();
  std::vector<double> orig = v;  // save original values
  size_t new_size = n * (pad + 1);
  v.assign(new_size, 0.0);  // new elements initialized to 0

  // Copy backwards to avoid overwriting
  for (size_t i = n; i-- > 0;) {
    v[i * (pad + 1)] = orig[i];
  }
}

void packVectorInPlace(std::vector<double>& v, size_t blockSize) {
  if (v.empty() || blockSize == 0) return;

  size_t n = v.size();
  std::vector<double> orig = v;  // save original values
  size_t new_size = n * blockSize;
  v.assign(new_size, 0.0);  // will overwrite

  // Copy backwards and fill each block with the element
  for (size_t i = n; i-- > 0;) {
    size_t start_idx = i * blockSize;
    for (size_t j = 0; j < blockSize; ++j) {
      v[start_idx + j] = orig[i];
    }
  }
}

// Original code by OpenFHE, BSD 2-Clause License
uint32_t nextPow2(uint32_t x) { return std::pow(2, std::ceil(std::log(x) / std::log(2.0))); };

// Original code by OpenFHE, BSD 2-Clause License
template <typename T>
std::vector<T> PackVecColWise(const std::vector<T>& v, std::size_t block_size, std::size_t num_slots) {
  // Check input parameters
  std::size_t n = v.size();

  // Check power of two constraints
  if (!lbcrypto::IsPowerOfTwo(block_size)) {
    OPENFHE_THROW("BlockSize must be a power of two");
  }

  if (!lbcrypto::IsPowerOfTwo(num_slots)) {
    OPENFHE_THROW("NumSlots must be a power of two");
  }

  // Check size constraints
  if (block_size < n) {
    OPENFHE_THROW("vector of size (" + std::to_string(n) + ") is longer than size of a slot (" + std::to_string(block_size) + ")");
  }

  if (num_slots < n) {
    OPENFHE_THROW("vector is longer than total slots");
  }

  if (num_slots == n) {
    return v;
  }

  // Calculate blocks
  if (num_slots % block_size != 0) OPENFHE_THROW("num_slots % block_size");

  // Create and fill packed vector
  std::vector<T> packed(num_slots, 0);

  std::size_t total_blocks = num_slots / block_size;
  // std::size_t free_slots   = num_slots - n * total_blocks;

  // Pack the vector column-wise
  std::size_t k = 0;  // index into vector to write
  for (std::size_t i = 0; i < total_blocks; ++i) {
    for (std::size_t j = 0; j < n; ++j) {
      packed[k] = v[j];
      ++k;
    }
    k += block_size - n;  // Skip remaining slots in the block
  }

  return packed;
}
template std::vector<double> PackVecColWise(const std::vector<double>& v, std::size_t block_size, std::size_t num_slots);

void train(Dataset trainingDataset, size_t epochs = 10, double eta = 0.1) {
  log("TRAINING", "Training has been started", "TRAINING");

  auto& X = trainingDataset.X;
  /*for (auto& row : X) {
    std::vector<double> newRow;
    newRow.reserve(row.size() * 2);

    for (double val : row) {
      newRow.push_back(val);
      newRow.push_back(val * val);
    }

    row = std::move(newRow);
  }*/
  auto Y = trainingDataset.Y;

  size_t n_samples = X.size();
  size_t n_features = X[0].size();

  if (Y.size() != n_samples) {
    throw std::invalid_argument("Y size is different than database samples size");
  }

  size_t paddedFeaturesCount = nextPow2(n_features);
  // Every line is padded till next power of 2 with zeroes
  padMatrixInPlace(X, n_samples, paddedFeaturesCount);
  // Every line is padded with own elements till paddedFeaturesCount length is reached
  packVectorInPlace(Y, paddedFeaturesCount);

  const size_t requiredSlots = paddedFeaturesCount * n_samples;
  log("TRAINING", "Required slots " + std::to_string(requiredSlots) + " out of " + std::to_string(state.batchSize));

  if (requiredSlots > state.batchSize) {
    throw std::invalid_argument("Dataset X cannot be greater than batchSize = " + std::to_string(state.batchSize));
  }

  log("ENCRYPT", "Encrypting Dataset X and Y and parameters W and B...", "ENCRYPT");

  auto sumColsKey = state.cc->EvalSumColsKeyGen(state.sk);

  // Encrypt X matrix
  // Every line is padded till next power of 2 with zeroes
  std::vector<double> flatX = flatten(X);
  Plaintext ptX = state.cc->MakeCKKSPackedPlaintext(flatX);
  Ciphertext<DCRTPoly> ctX = state.cc->Encrypt(state.pk, ptX);
  PRINT_CT(ctX);

  // Encrypt X transposed
  // Matrix is padded with zeroes cols until number of cols is a power of 2
  /*std::vector<std::vector<double>> X_T = transpose(X);
  std::vector<double> flatX_T = flatten(X_T);
  Plaintext ptX_T = state.cc->MakeCKKSPackedPlaintext(flatX_T);
  Ciphertext<DCRTPoly> ctX_T = state.cc->Encrypt(state.pk, ptX_T);
  PRINT_CT_TRANSPOSE(ctX_T);*/

  // Encrypt Y vector
  // Every line is padded till next power of 2 with zeroes
  Plaintext ptY = state.cc->MakeCKKSPackedPlaintext(Y);
  auto ctY = state.cc->Encrypt(state.pk, ptY);
  PRINT_CT(ctY);

  // Model parameters
  // Padded till power of 2 then repeated until end of slots

  // Encrypt W vector
  // std::vector<double> flatW = PackVecColWise(std::vector<double>{0, 1, 2}, paddedFeaturesCount, state.batchSize);
  std::vector<double> flatW(state.batchSize, 0.0);
  Plaintext ptW = state.cc->MakeCKKSPackedPlaintext(flatW);  // weights
  auto ctW = state.cc->Encrypt(state.pk, ptW);
  PRINT_CT(ctW);

  // Encrypt B as vector (scalar indeed)
  // std::vector<double> flatB = PackVecColWise(std::vector<double>{0.45}, paddedFeaturesCount, state.batchSize);
  std::vector<double> flatB(state.batchSize, 0.0);
  Plaintext ptB = state.cc->MakeCKKSPackedPlaintext(flatB);  // bias
  auto ctB = state.cc->Encrypt(state.pk, ptB);
  PRINT_CT(ctB);

  log("ENCRYPT", "Dataset and parameters were successfully encrypted", "ENCRYPT");

  // Precomputed for performance
  double scale_factor = eta / n_samples;

  std::vector<double> maskFirstOfFeature(state.batchSize, 0.0);
  for (size_t i = 0; i < n_samples; ++i) {
    maskFirstOfFeature[i * paddedFeaturesCount] = 1.0;
  }
  Plaintext ptMaskFirstOfFeature = state.cc->MakeCKKSPackedPlaintext(maskFirstOfFeature);

  std::vector<double> maskPaddedFeatures(state.batchSize, 0.0);
  std::fill_n(maskPaddedFeatures.begin(), paddedFeaturesCount, 1.0);
  Plaintext ptMaskPaddedFeatures = state.cc->MakeCKKSPackedPlaintext(maskPaddedFeatures);

  std::vector<double> maskScaleFactorScalar(state.batchSize, 0.0);
  maskScaleFactorScalar[0] = scale_factor;
  Plaintext ptMaskScaleFactorScalar = state.cc->MakeCKKSPackedPlaintext(maskScaleFactorScalar);

  // ---------------------------
  // TRAINING LOOP
  // ---------------------------
  log("TRAINING", "Starting training loop...", "TRAINING_LOOP");
  std::cout << std::endl;

  for (size_t epoch = 0; epoch < epochs; ++epoch) {
    auto startEpoch = std::chrono::steady_clock::now();

    // ---------------------------
    // FORWARD PASS: Y_hat(ct) = X(ct) * W(ct) + B(ct)
    // ---------------------------
    PRINT_CT(ctX);
    PRINT_CT(ctW);

    auto XC = state.cc->EvalMult(ctX, ctW);
    XC = state.cc->EvalSumCols(XC, paddedFeaturesCount, *sumColsKey);

    PRINT_CT(XC);
    PRINT_CT(ctB);

    auto Y_hat = XC + ctB;

    PRINT_CT(Y_hat);

    // ---------------------------
    // ERROR COMPUTATION: E(ct) = Y_hat(ct) - Y
    // ---------------------------

    PRINT_CT(ctY);

    // As: E[0], 0..0, E[1], 0..0
    auto E = Y_hat - ctY;

    PRINT_CT(E);

    // ---------------------------
    // GRADIENT COMPUTATION (Scaled): G(ct) = X_T(from X(ct)) * E(ct)
    // ---------------------------

    auto E_clean = state.cc->EvalMult(E, ptMaskFirstOfFeature);

    // Replicate e_i across the block
    auto E_expanded = E_clean;
    for (size_t k = 1; k < paddedFeaturesCount; ++k) {
      auto E_rot = state.cc->EvalRotate(E_clean, -(int)k);
      E_expanded = state.cc->EvalAdd(E_expanded, E_rot);
    }

    PRINT_CT(E_expanded);

    // Multiply with X (Dense) to get partial products
    auto XTE = state.cc->EvalMult(ctX, E_expanded);

    PRINT_CT(XTE);

    // Sum all rows to generate X^T * E
    XTE = state.cc->EvalSumRows(XTE, paddedFeaturesCount, *sumColsKey);

    // Scale gradient
    auto G = state.cc->EvalMult(XTE, scale_factor);

    PRINT_CT(G);

    // ---------------------------
    // WEIGHT UPDATE: W(ct) = W(ct) - gradient(ct)
    // ---------------------------

    // Mask the gradient to valid features only
    auto G_features = state.cc->EvalMult(G, ptMaskPaddedFeatures);

    PRINT_CT(G_features);

    // Replicate Gradient across all sample blocks so W stays consistent
    auto ctG_FULL = G_features;
    for (size_t s = 1; s < n_samples; s *= 2) {
      auto rot = state.cc->EvalRotate(ctG_FULL, -(int)(s * paddedFeaturesCount));
      ctG_FULL = state.cc->EvalAdd(ctG_FULL, rot);
    }

    PRINT_CT(ctG_FULL);

    ctW = state.cc->EvalSub(ctW, ctG_FULL);

    PRINT_CT(ctW);

    // ---------------------------
    // BIAS UPDATE: B(ct) = B(ct) - eta * mean(E)
    // ---------------------------

    // Sum errors
    auto SumE = E_clean;  // [e0, 0.., e1, 0..]
    for (size_t s = 1; s < n_samples; s *= 2) {
      auto rot = state.cc->EvalRotate(SumE, s * paddedFeaturesCount);
      SumE = state.cc->EvalAdd(SumE, rot);
    }
    // Now SumE[0] has sum(ei).

    // Mask SumE to keep only index 0
    auto ctSumE_0 = state.cc->EvalMult(SumE, ptMaskScaleFactorScalar);

    // Replicate Bias Gradient
    auto ctB_FULL = ctSumE_0;
    for (size_t s = 1; s < n_samples; s *= 2) {
      auto rot = state.cc->EvalRotate(ctB_FULL, -(int)(s * paddedFeaturesCount));
      ctB_FULL = state.cc->EvalAdd(ctB_FULL, rot);
    }

    ctB = state.cc->EvalSub(ctB, ctB_FULL);

    PRINT_CT(ctB);

    // ---------------------------
    // MONITORING
    // ---------------------------

    auto endEpoch = std::chrono::steady_clock::now();
    auto formattedDuration = formatDuration(startEpoch, endEpoch);

    if (DEBUG) {
      double loss = 0.0;

      Plaintext ptE;
      state.cc->Decrypt(state.sk, E, &ptE);
      // E is sparse [e0, ..., e1, ...]
      // Manually parse
      auto e_vals = ptE->GetRealPackedValue();
      for (size_t i = 0; i < n_samples; ++i) {
        double err = e_vals[i * paddedFeaturesCount];
        loss += err * err;
      }
      loss /= n_samples;

      Plaintext ptB_curr;
      state.cc->Decrypt(state.sk, ctB, &ptB_curr);
      double b_val_curr = ptB_curr->GetRealPackedValue()[0];

      std::cout << "Epoch " << (epoch + 1) << " : b = " << std::fixed << std::setprecision(4) << b_val_curr << ", MSE = " << std::fixed << std::setprecision(4) << loss << " ["
                << formattedDuration << "]" << std::endl;
    } else {
      std::cout << "Epoch " << (epoch + 1) << " finished [" << formattedDuration << "]" << std::endl;
    }

    // Each epoch takes 6 levels, bootstrap requires 1 level
    if (BOOTSTRAP) {
      auto depthRemained = state.depth - ctW->GetLevel() - (ctW->GetNoiseScaleDeg() - 1);
      std::cout << "Number of levels remaining: " << depthRemained << std::endl << std::endl;

      if ((epoch + 2 < epochs && depthRemained <= 6) || (epoch + 2 == epochs && depthRemained < 6)) {
        log("BOOTSTRAP", "Started bootstrapping W and B...", "BOOTSTRAP");

        ctW = state.cc->EvalBootstrap(ctW);
        ctB = state.cc->EvalBootstrap(ctB);

        log("BOOTSTRAP", "Boostrap ended...", "BOOTSTRAP");
      }
    }
  }

  std::cout << std::endl;
  log("TRAINING", "Ended training loop...", "TRAINING_LOOP");

  // ---------------------------
  // DECRYPT FINAL RESULTS
  // ---------------------------
  log("DECRYPT", "Decrypting final model...");

  // Decrypt final weights
  Plaintext ptW_final;
  state.cc->Decrypt(state.sk, ctW, &ptW_final);
  ptW_final->SetLength(n_features);
  auto W_final = ptW_final->GetRealPackedValue();

  // Decrypt final bias (take slot 0)
  Plaintext ptB_final;
  state.cc->Decrypt(state.sk, ctB, &ptB_final);
  ptB_final->SetLength(1);
  double b_final = ptB_final->GetRealPackedValue()[0];

  // Print final regression function
  std::cout << "\nRecovered regression function:\n";
  std::cout << "y = ";
  for (size_t i = 0; i < n_features; ++i) {
    std::cout << std::fixed << std::setprecision(4) << W_final[i] << " * x" << i;
    if (i < n_features - 1) std::cout << " + ";
  }
  std::cout << " + " << std::fixed << std::setprecision(4) << b_final << "\n";

  log("TRAINING", "Training has finished", "TRAINING");
}

int main() {
  auto [trainSet, testSet] = loadAndSplitDataset("data/hospital_data_prob.csv", 1.0);

  initFHEState();
  train(trainSet, 10, 0.00001);

  return 0;
}