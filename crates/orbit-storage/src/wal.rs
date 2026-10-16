use memmap2::MmapMut;
use orbit_core::{Message, OrbitError};
use std::fs::OpenOptions;
use std::path::Path;

/// A Zero-Copy Write-Ahead Log backed by SSD memory-mapping.
pub struct MmapWal {
    /// The memory-mapped file buffer
    mmap: MmapMut,
    /// Current write offset in bytes
    write_cursor: usize,
    /// Total pre-allocated size of the file
    capacity: usize,
}

impl MmapWal {
    /// Initializes a new WAL. It pre-allocates the entire file on the SSD 
    /// to prevent fragmentation and guarantee space is available.
    pub fn new<P: AsRef<Path>>(path: P, capacity: usize) -> Result<Self, std::io::Error> {
        // Open or create the file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        // Pre-allocate the file size on the SSD immediately.
        // In space, you NEVER want to dynamically grow files, as it 
        // risks running out of space during a critical telemetry surge.
        file.set_len(capacity as u64)?;

        // Ask the kernel to map this file directly into RAM
        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(Self {
            mmap,
            write_cursor: 0,
            capacity,
        })
    }

    /// Appends a new message directly to the memory-mapped buffer.
    pub fn append(&mut self, msg: &Message) -> Result<(), OrbitError> {
        // Convert the Message struct to a compact Postcard binary payload
        let bytes = msg.to_bytes().map_err(OrbitError::Serialization)?;
        let payload_len = bytes.len();

        // We need 4 bytes for the length prefix + the payload itself
        if self.write_cursor + 4 + payload_len > self.capacity {
            return Err(OrbitError::BufferOverflow);
        }

        // 1. Write the Length Prefix (so we know how many bytes to read later)
        let len_bytes = (payload_len as u32).to_le_bytes();
        self.mmap[self.write_cursor..self.write_cursor + 4].copy_from_slice(&len_bytes);
        self.write_cursor += 4;

        // 2. Write the Payload
        self.mmap[self.write_cursor..self.write_cursor + payload_len].copy_from_slice(&bytes);
        self.write_cursor += payload_len;

        Ok(())
    }

    /// Forces the kernel to flush the memory map to the physical SSD.
    /// In a real architecture, you'd call this periodically or right before an outage.
    pub fn flush_to_disk(&self) -> Result<(), std::io::Error> {
        self.mmap.flush()
    }

    /// Resets the write cursor to the beginning of the file, allowing continuous overwrites.
    pub fn reset_write_cursor(&mut self) {
        self.write_cursor = 0;
    }

    /// Reads the priority of the oldest message at the start of the WAL.
    pub fn peek_oldest_priority(&self) -> Option<orbit_core::PriorityLevel> {
        if self.write_cursor == 0 {
            return None; // WAL is empty
        }
        
        // Read the 4-byte length prefix of the first message
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&self.mmap[0..4]);
        let len = u32::from_le_bytes(len_bytes) as usize;
        
        if len == 0 || 4 + len > self.capacity {
            return None;
        }
        
        // Extract the payload and deserialize it to check priority
        let payload = &self.mmap[4..4+len];
        
        // Assuming your Message struct has a from_bytes method matching to_bytes
        if let Ok(msg) = Message::from_bytes(payload) {
            Some(msg.priority)
        } else {
            None
        }
    }
}