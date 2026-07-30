orbit-mq/
├── Cargo.toml                       # Cargo Workspace manifest
├── README.md
├── kraft.yaml                       # Unikraft build configuration
│
├── crates/
│   ├── orbit-core/                  # Core abstractions, payload schema & traits
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── message.rs           # Message headers, payload definitions, priority levels
│   │       └── error.rs             # Custom fault handling & SEU recovery
│   │
│   ├── orbit-storage/               # Zero-copy WAL & SSD persistence
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── wal.rs               # Memory-mapped file handler (memmap2)
│   │       ├── ring_buffer.rs       # Fixed-size SSD ring buffer
│   │       └── eviction.rs          # Least-Critical Unit (LCU) drop policy
│   │
│   ├── orbit-priority/              # Priority scheduling engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── heap.rs              # Binary heap with dynamic aging
│   │       └── scheduler.rs         # Queue consumer logic
│   │
│   ├── orbit-ingress/               # Microservice API listeners
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── grpc.rs              # Tonic gRPC server implementation
│   │       └── unix_socket.rs       # Raw IPC Unix domain listener
│   │
│   ├── orbit-uplink/                # Ground/Satellite egress transmitter
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── link_monitor.rs      # Transceiver state machine
│   │       └── framer.rs            # Postcard binary packetizer
│   │
│   └── orbit-broker/                # Main runnable binary / daemon CLI
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs              # Tokio runtime orchestration
│           └── config.rs            # Environment & buffer size settings
│
└── tests/
    └── integration_blackout_test.rs  # Simulates 45-min connection drop & burst sync