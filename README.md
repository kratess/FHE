# FHE: Fully Homomorphic Encryption

This repository contains the codebase and experimental results for my thesis focused on the benchmarking and performance evaluation of **Fully Homomorphic Encryption (FHE)** schemes. The project utilizes the [OpenFHE](https://openfhe.org/) library to conduct systematic analysis across various encryption paradigms.

## Thesis Context

The primary goal of this research is to evaluate the practical performance overheads of different FHE schemes, specifically **CKKS**, **BGV**, and **BFV** under varying security parameters and circuit depths. This suite serves as the empirical foundation for the comparative analysis presented in my thesis.

## Key Features

- **Benchmarking**: Comprehensive benchmarking of homomorphic encryption schemes, including **BGV (integer), BFV (integer), and CKKS (floating point)**. Experiments systematically sweep **multiplicative depths and ring dimensions**, while profiling **execution time and peak heap memory usage**. An **automated python pipeline** generates comparative performance graphs for analysis.
- **Linear Regression**: Test of linear regression model. _WORK IN PROGRESS_.

## Project Structure

```
.
├── benchmark/                  # Core C++ benchmark suite
│   ├── bfv/                    # BFV (Brakerski-Fan-Vercauteren) benchmarks
│   ├── bgv/                    # BGV (Brakerski-Gentry-Vaikuntanathan) benchmarks
│   ├── ckks/                   # CKKS (Cheon-Kim-Kim-Song) benchmarks
│   └── results/                # Benchmark results
├── keys/                       # Key material and helper files for local experiments
├── linear_regression/          # Linear regression experiments with FHE
├── prio3/                      # Prio3 experiments and FHE-based VDAF prototypes
│   ├── prio3_normal_main.rs    # Normal Prio3 example
│   ├── fhe-vdaf-1/             # Simpler FHE protocol that mimics Prio3
│   ├── fhe-vdaf-2/             # Sharded FHE protocol closer to Prio3
│   └── openfhe-bgv-rs/         # Rust wrapper used by the FHE prototypes
└── README.md
```

## Benchmark Results

Below are the performance trends observed during the benchmark sweeps.

### BGV (Brakerski-Gentry-Vaikuntanathan)
| Execution Time | Memory Usage |
| :--- | :--- |
| ![Encrypt](benchmark/results/bgv/BM_Encrypt.png) | ![Encrypt Memory](benchmark/results/bgv/BM_Encrypt_memory.png) |
| ![Decrypt](benchmark/results/bgv/BM_Decrypt.png) | ![Decrypt Memory](benchmark/results/bgv/BM_Decrypt_memory.png) |
| ![EvalAdd](benchmark/results/bgv/BM_EvalAdd.png) | ![EvalAdd Memory](benchmark/results/bgv/BM_EvalAdd_memory.png) |
| ![EvalMult](benchmark/results/bgv/BM_EvalMult.png) | ![EvalMult Memory](benchmark/results/bgv/BM_EvalMult_memory.png) |
| ![KeyGen](benchmark/results/bgv/BM_KeyGen.png) | ![KeyGen Memory](benchmark/results/bgv/BM_KeyGen_memory.png) |
| ![EvalKeyGen](benchmark/results/bgv/BM_EvalKeyGen.png) | ![EvalKeyGen Memory](benchmark/results/bgv/BM_EvalKeyGen_memory.png) |
| ![Context Creation](benchmark/results/bgv/BM_ContextCreation.png) | ![Context Memory](benchmark/results/bgv/BM_ContextCreation_memory.png) |

### BFV (Brakerski-Fan-Vercauteren)
| Execution Time | Memory Usage |
| :--- | :--- |
| ![Encrypt](benchmark/results/bfv/BM_Encrypt.png) | ![Encrypt Memory](benchmark/results/bfv/BM_Encrypt_memory.png) |
| ![Decrypt](benchmark/results/bfv/BM_Decrypt.png) | ![Decrypt Memory](benchmark/results/bfv/BM_Decrypt_memory.png) |
| ![EvalAdd](benchmark/results/bfv/BM_EvalAdd.png) | ![EvalAdd Memory](benchmark/results/bfv/BM_EvalAdd_memory.png) |
| ![EvalMult](benchmark/results/bfv/BM_EvalMult.png) | ![EvalMult Memory](benchmark/results/bfv/BM_EvalMult_memory.png) |
| ![KeyGen](benchmark/results/bfv/BM_KeyGen.png) | ![KeyGen Memory](benchmark/results/bfv/BM_KeyGen_memory.png) |
| ![EvalKeyGen](benchmark/results/bfv/BM_EvalKeyGen.png) | ![EvalKeyGen Memory](benchmark/results/bfv/BM_EvalKeyGen_memory.png) |
| ![Context Creation](benchmark/results/bfv/BM_ContextCreation.png) | ![Context Memory](benchmark/results/bfv/BM_ContextCreation_memory.png) |

### CKKS (Cheon-Kim-Kim-Song)
| Execution Time | Memory Usage |
| :--- | :--- |
| ![Encrypt](benchmark/results/ckks/BM_Encrypt.png) | ![Encrypt Memory](benchmark/results/ckks/BM_Encrypt_memory.png) |
| ![Decrypt](benchmark/results/ckks/BM_Decrypt.png) | ![Decrypt Memory](benchmark/results/ckks/BM_Decrypt_memory.png) |
| ![EvalAdd](benchmark/results/ckks/BM_EvalAdd.png) | ![EvalAdd Memory](benchmark/results/ckks/BM_EvalAdd_memory.png) |
| ![EvalMult](benchmark/results/ckks/BM_EvalMult.png) | ![EvalMult Memory](benchmark/results/ckks/BM_EvalMult_memory.png) |
| ![KeyGen](benchmark/results/ckks/BM_KeyGen.png) | ![KeyGen Memory](benchmark/results/ckks/BM_KeyGen_memory.png) |
| ![EvalKeyGen](benchmark/results/ckks/BM_EvalKeyGen.png) | ![EvalKeyGen Memory](benchmark/results/ckks/BM_EvalKeyGen_memory.png) |
| ![Context Creation](benchmark/results/ckks/BM_ContextCreation.png) | ![Context Memory](benchmark/results/ckks/BM_ContextCreation_memory.png) |
| ![Bootstrap](benchmark/results/ckks/BM_Bootstrap.png) | ![Bootstrap Memory](benchmark/results/ckks/BM_Bootstrap_memory.png) |
| ![Bootstrap KeyGen](benchmark/results/ckks/BM_BootstrapKeyGen.png) | ![Bootstrap KeyGen Memory](benchmark/results/ckks/BM_BootstrapKeyGen_memory.png) |

## Getting Started

### Prerequisites
- **CMake** >= 3.14
- **C++17** compliant compiler (GCC 11+, Clang 12+)
- **OpenFHE** (Latest stable)
- **Python 3** (for visualization)

### Installation, Build and Running
- **Benchmarking**: Follow guide in [`benchmark/README.md`](benchmark/README.md)

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

## License

This project is licensed under the BSD 2-Clause License - see the [LICENSE](LICENSE) file for details.
