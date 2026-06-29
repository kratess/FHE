# OpenFHE CKKS Benchmark Suite

A comprehensive benchmarking tool for the **CKKS (Cheon-Kim-Kim-Song)** homomorphic encryption scheme using the [OpenFHE](https://openfhe.org/) library. This project provides performance analysis across various multiplicative depths and ring dimensions, including both leveled and bootstrapping configurations.

## Features

- **Broad Parameter Sweep**: Automatically benchmarks multiple configurations:
  - **Multiplicative Depth**: 1 to 512 (Leveled), 1 to 128 (Bootstrapping)
  - **Ring Dimension**: 256 to 65536 (Leveled), 256 to 16384 (Bootstrapping)
- **Comprehensive Operation Benchmarks**:
  - `ContextCreation`: Performance of setting up the CKKS environment.
  - `KeyGen`: Standard public/secret key pair generation.
  - `EvalKeyGen`: Relinearization and rotation key generation.
  - `Encrypt` / `Decrypt`: Basic encoding and encryption/decryption operations.
  - `EvalAdd` / `EvalMult`: Homomorphic addition and multiplication.
  - `Bootstrap`: Full homographic noise removal (bootstrapping).
  - `BootstrapKeyGen`: Generation of specialty keys required for bootstrapping.
- **Resource Monitoring**: Tracks execution time (CPU/Real time) and peak heap memory usage (MB).
- **Statistical Aggregates**: Repeats each benchmark 10 times and exports `mean`, `median`, `stddev`, and `cv` rows in the CSV output.
- **Visualization**: Integrated with the python-based plotting utility to generate performance graphs from CSV data.

## Benchmark Results Gallery

Below are the performance trends observed during the benchmark sweeps.

### Core Operations (Leveled)

| Execution Time | Memory Usage |
| :--- | :--- |
| ![Encrypt](../results/ckks/BM_Encrypt.png) | ![Encrypt Memory](../results/ckks/BM_Encrypt_memory.png) |
| ![Decrypt](../results/ckks/BM_Decrypt.png) | ![Decrypt Memory](../results/ckks/BM_Decrypt_memory.png) |
| ![EvalAdd](../results/ckks/BM_EvalAdd.png) | ![EvalAdd Memory](../results/ckks/BM_EvalAdd_memory.png) |
| ![EvalMult](../results/ckks/BM_EvalMult.png) | ![EvalMult Memory](../results/ckks/BM_EvalMult_memory.png) |
| ![KeyGen](../results/ckks/BM_KeyGen.png) | ![KeyGen Memory](../results/ckks/BM_KeyGen_memory.png) |
| ![EvalKeyGen](../results/ckks/BM_EvalKeyGen.png) | ![EvalKeyGen Memory](../results/ckks/BM_EvalKeyGen_memory.png) |
| ![Context Creation](../results/ckks/BM_ContextCreation.png) | ![Context Memory](../results/ckks/BM_ContextCreation_memory.png) |

### Bootstrapping

| Execution Time | Memory Usage |
| :--- | :--- |
| ![Bootstrap](../results/ckks/BM_Bootstrap.png) | ![Bootstrap Memory](../results/ckks/BM_Bootstrap_memory.png) |
| ![Bootstrap KeyGen](../results/ckks/BM_BootstrapKeyGen.png) | ![Bootstrap KeyGen Memory](../results/ckks/BM_BootstrapKeyGen_memory.png) |

## Test Environment

The benchmarks presented in the gallery were performed on the following hardware/software stack:

- **CPU**: AMD Ryzen 9800X3D
- **Memory**: 48GB DDR5 @ 6000MHz (12GB Swap)
- **OS**: Windows Subsystem for Linux (WSL)
- **Compiler**: Optimized with `-march=native`
- **Parallelism**: 
  - `GenCryptoContext`, used in `BM_ContextCreation` as test and also in the other cases to create the context without being accounted in execution time, is a **single-core operation**.
  - All other benchmark operations leverage **multi-threading** via OpenMP.

> [!IMPORTANT]
> **Memory Disclaimer**: Reported peak memory usage is obtained from the system allocator and reflects heap usage only. Values are indicative and may vary across environments (e.g., WSL vs bare metal); swap memory is not included.

---
*Note: Benchmarks were performed on a local workstation using hardware optimization (-march=native). Results may vary based on CPU architecture and available memory.*
