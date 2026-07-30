use crate::wal::MmapWal;
use orbit_core::{Message, OrbitError};
use std::path::Path;

pub struct WalRingBuffer {
    wal: MmapWal,
    capacity: usize,
}

impl WalRingBuffer {
    pub fn new<P: AsRef<Path>>(path: P, capacity: usize) -> Result<Self, std::io::Error> {
        let wal = MmapWal::new(path, capacity)?;
        Ok(Self { wal, capacity })
    }

    pub fn push(&mut self, msg: &Message) -> Result<(), OrbitError> {
        // If the WAL append returns a BufferOverflow, a space-grade ring buffer 
        // would safely overwrite the oldest lowest-priority blocks.
        match self.wal.append(msg) {
            Ok(_) => Ok(()),
            Err(OrbitError::BufferOverflow) => {
                // Space-grade mitigation: In a full implementation, shift the ring buffer 
                // head pointer and overwrite old HighResMedia logs.
                Err(OrbitError::BufferOverflow)
            }
            Err(e) => Err(e),
        }
    }
}