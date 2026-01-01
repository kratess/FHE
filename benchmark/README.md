# OpenFHE Multi-Scheme Benchmark Suite

A comprehensive benchmarking environment for evaluating various homomorphic encryption schemes implemented in the [OpenFHE](https://openfhe.org/) library. This suite provides systematic performance analysis for **CKKS**, **BGV**, and **BFV** schemes across diverse parameter sets.

## Features

- **Multi-Scheme Support**: Unified benchmarking for CKKS (floating point), BGV (integer), and BFV (integer) schemes.
- **Broad Parameter Sweep**: Automatically scales benchmarks across:
  - **Multiplicative Depth**: Evaluates performance from shallow to deep circuits.
  - **Ring Dimension**: Measures impact of batch size (slots capacity).
- **Comprehensive Operation Benchmarking**:
  - `ContextCreation`: Setup overhead for each scheme.
  - `KeyGen`: Standard public/secret key pair generation.
  - `EvalKeyGen`: Relinearization and rotation key generation.
  - `Encrypt` / `Decrypt`: Basic encoding and encryption/decryption operations.
  - `EvalAdd` / `EvalMult`: Homomorphic addition and multiplication.
  - `Bootstrap` (CKKS): Full homographic noise removal (bootstrapping).
  - `BootstrapKeyGen` (CKKS): Generation of specialty keys required for bootstrapping.
- **Resource Monitoring**: Tracks execution time (CPU/Real time) and peak heap memory usage (MB).
- **Visualization**: Integrated with the python-based plotting utility to generate performance graphs from CSV data.

## Project Structure

```
.
├── bfv/            # BFV (Brakerski-Fan-Vercauteren) benchmarks
├── bgv/            # BGV (Brakerski-Gentry-Vaikuntanathan) benchmarks
├── ckks/           # CKKS (Cheon-Kim-Kim-Song) benchmarks
├── results/        # Benchmark results
│   ├── bfv/        # BFV performance graphs
│   ├── bgv/        # BGV performance graphs
│   ├── ckks/       # CKKS performance graphs
│   └── plot_results.py
└── README.md
```

## Prerequisites

- **CMake** >= 3.14
- **C++17** compliant compiler (GCC 11+, Clang 12+)
- **OpenFHE** (Latest stable)
- **Python 3** (for visualization)

## Installation & Build

1. **Setup**:
   ```bash
   mkdir -p build && cd build
   ```

2. **Configure (Optimized)**:
   For accurate performance results, always use the optimized Release build:
   ```bash
   cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_CXX_FLAGS="-march=native" ..
   ```

3. **Compile**:
   ```bash
   make -j$(nproc)
   ```

## Usage

### 1. Running Benchmarks
Each scheme folder provides its own executable. Export results to CSV for visualization.

```bash
# Example: Run CKKS Benchmark
./build/ckks/ckks_benchmark --benchmark_out=ckks/results.csv --benchmark_out_format=csv
```

### 2. Generating Visualizations
The centralized plotting script allows for selective plotting and data filtering.

```bash
# Generate all plots (default)
python3 results/plot_results.py

# Select a specific scheme and apply filters
python3 results/plot_results.py --scheme ckks --max-depth 16 --max-ring-dim 4096
```

**Available Arguments:**
- `--scheme`: Specific scheme to plot (`ckks`, `bgv`, or `bfv`). If omitted, all detected results are processed.
- `--max-depth`: Filters out results with multiplicative depth exceeding this value.
- `--max-ring-dim`: Filters out results with ring dimension exceeding this value.

### BGV (Brakerski-Gentry-Vaikuntanathan)
| Execution Time | Memory Usage |
| :--- | :--- |
| ![Encrypt](results/bgv/BM_Encrypt.png) | ![Encrypt Memory](results/bgv/BM_Encrypt_memory.png) |
| ![Decrypt](results/bgv/BM_Decrypt.png) | ![Decrypt Memory](results/bgv/BM_Decrypt_memory.png) |
| ![EvalAdd](results/bgv/BM_EvalAdd.png) | ![EvalAdd Memory](results/bgv/BM_EvalAdd_memory.png) |
| ![EvalMult](results/bgv/BM_EvalMult.png) | ![EvalMult Memory](results/bgv/BM_EvalMult_memory.png) |
| ![KeyGen](results/bgv/BM_KeyGen.png) | ![KeyGen Memory](results/bgv/BM_KeyGen_memory.png) |
| ![EvalKeyGen](results/bgv/BM_EvalKeyGen.png) | ![EvalKeyGen Memory](results/bgv/BM_EvalKeyGen_memory.png) |
| ![Context Creation](results/bgv/BM_ContextCreation.png) | ![Context Memory](results/bgv/BM_ContextCreation_memory.png) |

### BFV (Brakerski-Fan-Vercauteren)
| Execution Time | Memory Usage |
| :--- | :--- |
| ![Encrypt](results/bfv/BM_Encrypt.png) | ![Encrypt Memory](results/bfv/BM_Encrypt_memory.png) |
| ![Decrypt](results/bfv/BM_Decrypt.png) | ![Decrypt Memory](results/bfv/BM_Decrypt_memory.png) |
| ![EvalAdd](results/bfv/BM_EvalAdd.png) | ![EvalAdd Memory](results/bfv/BM_EvalAdd_memory.png) |
| ![EvalMult](results/bfv/BM_EvalMult.png) | ![EvalMult Memory](results/bfv/BM_EvalMult_memory.png) |
| ![KeyGen](results/bfv/BM_KeyGen.png) | ![KeyGen Memory](results/bfv/BM_KeyGen_memory.png) |
| ![EvalKeyGen](results/bfv/BM_EvalKeyGen.png) | ![EvalKeyGen Memory](results/bfv/BM_EvalKeyGen_memory.png) |
| ![Context Creation](results/bfv/BM_ContextCreation.png) | ![Context Memory](results/bfv/BM_ContextCreation_memory.png) |

### CKKS (Cheon-Kim-Kim-Song)
| Execution Time | Memory Usage |
| :--- | :--- |
| ![Encrypt](results/ckks/BM_Encrypt.png) | ![Encrypt Memory](results/ckks/BM_Encrypt_memory.png) |
| ![Decrypt](results/ckks/BM_Decrypt.png) | ![Decrypt Memory](results/ckks/BM_Decrypt_memory.png) |
| ![EvalAdd](results/ckks/BM_EvalAdd.png) | ![EvalAdd Memory](results/ckks/BM_EvalAdd_memory.png) |
| ![EvalMult](results/ckks/BM_EvalMult.png) | ![EvalMult Memory](results/ckks/BM_EvalMult_memory.png) |
| ![KeyGen](results/ckks/BM_KeyGen.png) | ![KeyGen Memory](results/ckks/BM_KeyGen_memory.png) |
| ![EvalKeyGen](results/ckks/BM_EvalKeyGen.png) | ![EvalKeyGen Memory](results/ckks/BM_EvalKeyGen_memory.png) |
| ![Context Creation](results/ckks/BM_ContextCreation.png) | ![Context Memory](results/ckks/BM_ContextCreation_memory.png) |
| ![Bootstrap](results/ckks/BM_Bootstrap.png) | ![Bootstrap Memory](results/ckks/BM_Bootstrap_memory.png) |
| ![Bootstrap KeyGen](results/ckks/BM_BootstrapKeyGen.png) | ![Bootstrap KeyGen Memory](results/ckks/BM_BootstrapKeyGen_memory.png) |

## Test Environment

Benchmarks were performed on the following hardware/software stack:
- **CPU**: AMD Ryzen 9800X3D
- **Memory**: 48GB DDR5 @ 6000MHz (12GB Swap)
- **OS**: Windows Subsystem for Linux (WSL)
- **Optimization**: `-march=native` enabled.
- **Parallelism**: 
  - `GenCryptoContext`, used in `BM_ContextCreation` as test and also in the other cases to create the context without being accounted in execution time, is a **single-core operation**.
  - All other benchmark operations leverage **multi-threading** via OpenMP for maximum performance.

> [!IMPORTANT]
> **Memory Disclaimer**: Reported peak memory usage is obtained from the system allocator and reflects heap usage only. Values are indicative and may vary across environments (e.g., WSL vs bare metal); swap memory is not included.

---
*Note: Benchmarks were performed on a local workstation using hardware optimization (-march=native). Results may vary based on CPU architecture and available memory.*