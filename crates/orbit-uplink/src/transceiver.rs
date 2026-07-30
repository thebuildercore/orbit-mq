use orbit_core::{Message, OrbitError};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    Offline,
    Handshaking,
    Active,
}

/// A simulated hardware driver for an Optical Laser Transceiver.
pub struct OpticalTransceiver {
    pub state: LinkState,
}

impl OpticalTransceiver {
    pub fn new() -> Self {
        Self {
            state: LinkState::Offline,
        }
    }

    /// Simulates the satellite moving around the Earth. 
    /// In a production Unikernel, this function would actively poll 
    /// a hardware GPIO pin or network interface card (NIC) status.
    pub async fn wait_for_window(&mut self) {
        self.state = LinkState::Offline;
        println!("[Uplink] 🌑 Satellite is on the dark side of Earth. Link OFFLINE.");
        
        // Simulate a long 45-minute communication blackout (scaled down to 10 seconds for testing)
        sleep(Duration::from_secs(10)).await;
        
        self.state = LinkState::Handshaking;
        println!("[Uplink] 🛰️ Ground station acquired. Handshaking...");
        sleep(Duration::from_secs(1)).await; // Laser alignment time

        self.state = LinkState::Active;
        println!("[Uplink] 🟢 Optical Link ACTIVE. Burst transmission window open!");
    }

    /// Simulates the connection window closing as the satellite orbits out of view.
    pub async fn window_closes(&mut self) {
        // Simulate a short 5-minute transmission window (scaled down to 5 seconds)
        sleep(Duration::from_secs(5)).await;
        
        println!("[Uplink] 🔴 Satellite moving out of range. Optical link dropped.");
        self.state = LinkState::Offline;
    }

    /// Takes a message from the Priority Scheduler, converts it to a compact 
    /// binary format, and blasts it over the physical link.
    pub async fn transmit(&self, msg: &Message) -> Result<(), OrbitError> {
        if self.state != LinkState::Active {
            return Err(OrbitError::Io(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Attempted to transmit while laser is offline",
            )));
        }

        // Convert struct to ultra-compact binary using Postcard
        let bytes = msg.to_bytes()?;
        
        // Simulate the time it takes the laser hardware to pulse the data
        sleep(Duration::from_millis(50)).await;

        println!(
            "[Uplink -> Earth] Transmitted ID: {} | Tier: {:?} | Source: {} | {} bytes",
            msg.id, msg.priority, msg.source, bytes.len()
        );

        Ok(())
    }
}