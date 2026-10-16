# 🛰️ OrbitMQ (orbit-mq)

**OrbitMQ** is a high-performance, fault-tolerant, asynchronous message broker built in **Rust** specifically engineered for intermittent, high-latency deep space and satellite networks. 

Inspired by the networking constraints of the NASA Lunar Gateway, OrbitMQ bridges the gap between modern cloud microservices (gRPC/Unix Sockets) and extreme space physics (planet occultation blackouts, radiation-induced memory corruption, and narrow optical laser transmission windows).

---

## Performance Benchmarks: Zero-Copy WAL vs Standard I/O

OrbitMQ utilizes a memory-mapped, zero-copy architecture (`MmapWal`) to bypass OS kernel context switches during critical telemetry buffering. 

In standard Criterion benchmarks against standard file system I/O (`std::fs::File`), the zero-copy architecture demonstrates a **77.3% reduction in write latency** per packet.

```text
Telemetry_Storage_Throughput/OrbitMQ_Mmap_Append
                        time:   [652.53 ns 692.70 ns 738.04 ns]

Telemetry_Storage_Throughput/Standard_Syscall_Write
                        time:   [2.9179 µs 3.0517 µs 3.1778 µs]

                        Standard Syscall Write: ~3,051 ns (3.05 µs) per packet

                        OrbitMQ Mmap Append: ~692 ns per packet

                        Net Result: 4.4x throughput increase, saving crucial CPU cycles on power-constrained flight computers.

```

## 🏗️ Architecture & Core Features

OrbitMQ decouples the *generation* of data from its *transmission*, solving the brutal physics of space communications (e.g., planetary blockages and communication windows).

```text
[ Onboard Sensors / Cameras ] 
          │ (gRPC / Unix Domain Sockets)
          ▼
┌────────────────────────────────────────────────────────┐
│                   ORBIT-BROKER DAEMON                  │
│                                                        │
│  ┌───────────────────────┐   ┌──────────────────────┐  │
│  │ Ingress Layer         │──►│ MLFQ Priority Engine │  │
│  │ (Tonic gRPC / UDS)    │   │ (Dynamic Aging Heap) │  │
│  └───────────────────────┘   └──────────┬───────────┘  │
│                                         │              │
│                                         ▼              │
│  ┌──────────────────────────────────────────────────┐  │
│  │ Zero-Copy Storage Engine (mmap WAL & Ring Buffer)│  │
│  └──────────────────────────┬───────────────────────┘  │
└─────────────────────────────┼──────────────────────────┘
                              │ Burst Transmission Window
                              ▼
                   ((((( Space Vacuum )))))
                              │
                              ▼
                     [ GROUND STATION ]
```



### Core Engineering Features

*   **Zero-Copy Write-Ahead Log (WAL):** Uses `memmap2` to bypass standard file I/O overhead by memory-mapping files directly into application space. This prevents flash wear, reduces CPU context switching, and ensures telemetry is safely written to disk instantly to survive sudden power loss or Single Event Upsets (SEUs).
*   **Multi-Level Feedback Queue (MLFQ) with Starvation Aging:** Automatically prioritizes critical systems (`CriticalLifeSupport` > `ThrusterMetrics` > `GeneralTelemetry` > `HighResMedia`) while dynamically bumping the priority score of older messages so low-tier data never starves indefinitely.
*   **Low-Overhead Ingress Layer:** Supports both standard gRPC (`Tonic`) and high-speed local IPC via Unix Domain Sockets for low-latency internal spacecraft microservices.
*   **Smart Egress State Machine:** Constantly evaluates optical transceiver signal health and bursts queued data at maximum speed the millisecond an Earth ground station link opens.

---

## 🛠️ Tech Stack

Built as a modular Cargo Workspace, `orbit-mq` isolates system boundaries to support compilation into native Linux binaries or lightweight space Unikernels:

*   **Language:** Rust (Edition 2021/2024) — Zero-cost abstractions, memory safety, and deterministic performance.
*   **Async Runtime:** Tokio — High-performance lock-free task execution.
*   **Serialization:** Postcard — Ultra-compact binary encoding optimized for constrained bandwidth.
*   **Storage (`orbit-storage`):** `memmap2` — Zero-copy Write-Ahead Log (WAL) ring buffer mapped directly to SSD memory to protect flash lifespans.
*   **Scheduling (`orbit-priority`):** Multi-Level Feedback Queue (MLFQ) with starvation-aging algorithms.
*   **Ingress (`orbit-ingress`):** Tonic gRPC and Unix Domain Sockets (UDS) for low-latency local IPC.
*   **Uplink (`orbit-uplink`):** Hardware transceiver state machine and link-state health monitor.

---

## 📂 Workspace Repository Structure

```text
orbit-mq/
├── Cargo.toml                       # Cargo Workspace root manifest
├── Readme.md                        # Project documentation
│
├── crates/
│   ├── orbit-core/                  # Canonical Message struct, PriorityLevel enum & Postcard serialization
│   ├── orbit-storage/               # Zero-copy WAL, SSD ring-buffer & LCU eviction policies
│   ├── orbit-priority/              # MLFQ scheduling engine with starvation-aging algorithm
│   ├── orbit-ingress/               # Tonic gRPC server & Unix Domain Socket IPC listeners
│   ├── orbit-uplink/                # Optical laser state machine & link health monitor
│   ├── orbit-broker/                # Main runnable broker daemon (Tokio async runtime)
│   └── orbit-client/                # Reference gRPC telemetry publishing client
```

---

## 🚀 Quick Start & Testing

### 1. Build and Run the Broker Daemon

In your workspace root terminal (with WSL enabled if on Windows):

```bash
cargo run --bin orbit-broker
```

### 2. Run the Test Client (Simulating a Satellite Sensor)

In a separate terminal window, send a high-priority telemetry packet:

```bash
cargo run --bin orbit-client
```