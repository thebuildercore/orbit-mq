use criterion::{black_box, criterion_group, criterion_main, Criterion};
use orbit_storage::wal::MmapWal;
use orbit_core::{Message, PriorityLevel};
use std::fs::OpenOptions;
use std::io::{Write, Seek, SeekFrom}; // <-- Combined the I/O imports here
use tempfile::tempdir;

fn benchmark_wal(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    
    // 1. Setup Zero-Copy Mmap WAL (My Architecture)
    let mmap_path = dir.path().join("mmap_wal.bin");
    let mut mmap_wal = MmapWal::new(&mmap_path, 100 * 1024 * 1024).unwrap(); // 100MB pre-allocated

    // 2. Setup Standard I/O WAL (Naive Architecture)
    let stdio_path = dir.path().join("stdio_wal.bin");
    let mut std_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&stdio_path)
        .unwrap();

    // Create a dummy message
    let dummy_msg = Message::new(
        1,
        PriorityLevel::CriticalLifeSupport,
        "benchmark".to_string(),
        vec![0; 256], // Simulate a 256-byte telemetry packet
    );

    // Pre-serialize for the standard I/O benchmark
    let msg_bytes = dummy_msg.to_bytes().unwrap();
    let len_bytes = (msg_bytes.len() as u32).to_le_bytes();

    let mut group = c.benchmark_group("Telemetry_Storage_Throughput");

    // Benchmark A: Zero-Copy Mmap
    group.bench_function("OrbitMQ_Mmap_Append", |b| {
        b.iter(|| {
            // MAKE SURE the word 'match' is right here:
            match mmap_wal.append(black_box(&dummy_msg)) {
                Ok(_) => {},
                Err(orbit_core::OrbitError::BufferOverflow) => {
                    // we hit the 100mb limit! Reset the cursor to simulate the ring buffer.
                    mmap_wal.reset_write_cursor();
                    // Retry the write at the beginning of the file
                    mmap_wal.append(black_box(&dummy_msg)).unwrap();
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        })
    });

    // Benchmark B: Standard File System Write
    group.bench_function("Standard_Syscall_Write", |b| {
        let mut current_size = 0;
        let limit = 100 * 1024 * 1024; // 100mb limit to match the wal

        b.iter(|| {
            if current_size >= limit {
                // Rewind standard file to the beginning
                std_file.seek(SeekFrom::Start(0)).unwrap();
                current_size = 0;
            }
            std_file.write_all(black_box(&len_bytes)).unwrap();
            std_file.write_all(black_box(&msg_bytes)).unwrap();
            current_size += 4 + msg_bytes.len();
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_wal);
criterion_main!(benches);