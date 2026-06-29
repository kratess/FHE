# OpenFHE BFV Benchmark Suite

A comprehensive benchmarking tool for the **BFV (Brakerski-Fan-Vercauteren)** homomorphic encryption scheme using the [OpenFHE](https://openfhe.org/) library. This project provides performance analysis across various multiplicative depths and ring dimensions.

## Features

- **Broad Parameter Sweep**: Automatically benchmarks multiple configurations:
  - **Multiplicative Depth**: 1 to 128
  - **Ring Dimension**: 256 to 16384
- **Comprehensive Operation Benchmarks**:
  - `ContextCreation`: Performance of setting up the BFV environment.
  - `KeyGen`: Standard public/secret key pair generation.
  - `EvalKeyGen`: Relinearization and rotation key generation.
  - `Encrypt` / `Decrypt`: Basic encoding and encryption/decryption operations.
  - `EvalAdd` / `EvalMult`: Homomorphic addition and multiplication.
- **Resource Monitoring**: Tracks execution time (CPU/Real time) and peak heap memory usage (MB).
- **Statistical Aggregates**: Repeats each benchmark 10 times and exports `mean`, `median`, `stddev`, and `cv` rows in the CSV output.
- **Visualization**: Integrated with the python-based plotting utility to generate performance graphs from CSV data.

## Benchmark Results Gallery

Below are the performance trends observed during the benchmark sweeps.

### Core Operations

| Execution Time | Memory Usage |
| :--- | :--- |
| ![Encrypt](../results/bfv/BM_Encrypt.png) | ![Encrypt Memory](../results/bfv/BM_Encrypt_memory.png) |
| ![Decrypt](../results/bfv/BM_Decrypt.png) | ![Decrypt Memory](../results/bfv/BM_Decrypt_memory.png) |
| ![EvalAdd](../results/bfv/BM_EvalAdd.png) | ![EvalAdd Memory](../results/bfv/BM_EvalAdd_memory.png) |
| ![EvalMult](../results/bfv/BM_EvalMult.png) | ![EvalMult Memory](../results/bfv/BM_EvalMult_memory.png) |
| ![KeyGen](../results/bfv/BM_KeyGen.png) | ![KeyGen Memory](../results/bfv/BM_KeyGen_memory.png) |
| ![EvalKeyGen](../results/bfv/BM_EvalKeyGen.png) | ![EvalKeyGen Memory](../results/bfv/BM_EvalKeyGen_memory.png) |
| ![Context Creation](../results/bfv/BM_ContextCreation.png) | ![Context Memory](../results/bfv/BM_ContextCreation_memory.png) |

## Test Environment

- **CPU**: AMD Ryzen 9800X3D
- **Memory**: 48GB DDR5 @ 6000MHz (12GB Swap)
- **OS**: Windows Subsystem for Linux (WSL)
- **Compiler**: GCC/Clang with `-march=native`
- **Parallelism**: 
  - `GenCryptoContext`, used in `BM_ContextCreation` as test and also in the other cases to create the context without being accounted in execution time, is a **single-core operation**.
  - All other benchmarks leverage **multi-threading** via OpenMP.

> [!IMPORTANT]
> **Memory Disclaimer**: Reported peak memory usage is obtained from the system allocator and reflects heap usage only. Values are indicative and may vary across environments (e.g., WSL vs bare metal); swap memory is not included.

---
*Note: Benchmarks were performed on a local workstation using hardware optimization (-march=native). Results may vary based on CPU architecture and available memory.*
