use serde::{Deserialize, Serialize};

/// Priority levels for space telemetry and payloads.
/// Lower numerical value = Higher priority transmission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum PriorityLevel {
    /// Oxygen levels, hull pressure, radiation warnings
    CriticalLifeSupport = 0,
    /// Attitude adjustment, orbital mechanics, thrusters
    ThrusterMetrics = 1,
    /// Standard housekeeping logs, power diagnostics
    GeneralTelemetry = 2,
    /// High-res manufacturing optics, science payloads
    HighResMedia = 3,
}

/// The atomic unit of data passed through orbit-mq.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Unique 64-bit sequence identifier
    pub id: u64,
    /// Priority tier driving the scheduling heap
    pub priority: PriorityLevel,
    /// Epoch timestamp when generated on spacecraft (milliseconds)
    pub timestamp_ms: u64,
    /// Originating microservice (e.g., "life-support-daemon")
    pub source: String,
    /// Raw binary or encoded payload
    pub payload: Vec<u8>,
}

impl Message {
    pub fn new(id: u64, priority: PriorityLevel, source: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            id,
            priority,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            source: source.into(),
            payload,
        }
    }

    /// Serializes message payload using compact Postcard binary encoding
    pub fn to_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    /// Deserializes binary stream back into a Message struct
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}