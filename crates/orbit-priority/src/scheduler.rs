use orbit_core::{Message, PriorityLevel};
use std::collections::VecDeque;
use std::time::{Duration, SystemTime};

/// A Multi-Level Feedback Queue (MLFQ) that prevents data starvation.
pub struct PriorityScheduler {
    /// 4 tiers of queues matching our 4 PriorityLevels
    queues: [VecDeque<Message>; 4],
    /// How long a message sits before it is forcefully promoted to a higher tier
    aging_threshold: Duration,
}

impl PriorityScheduler {
    pub fn new(aging_threshold: Duration) -> Self {
        Self {
            queues: Default::default(), // Initializes 4 empty VecDeques
            aging_threshold,
        }
    }

    /// O(1) Push: Ingests a new message into its respective tier
    pub fn enqueue(&mut self, msg: Message) {
        let tier_index = msg.priority as usize;
        self.queues[tier_index].push_back(msg);
    }

    /// O(1) Pop: Extracts the highest priority message ready for transmission
    pub fn dequeue(&mut self) -> Option<Message> {
        // 1. First, forcefully promote any low-priority data that has been starving
        self.promote_starved_messages();
        
        // 2. Iterate from Tier 0 (Critical) down to Tier 3 (Media)
        for queue in self.queues.iter_mut() {
            if let Some(msg) = queue.pop_front() {
                return Some(msg); // Return the highest priority item found
            }
        }
        
        None // The entire broker is empty
    }

    /// The Starvation-Aging Engine
    fn promote_starved_messages(&mut self) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // Iterate upwards from Tier 3 (Lowest) to Tier 1 
        // (Tier 0 cannot be promoted, it is already max priority)
        for tier in (1..=3).rev() {
            // Because VecDeque is FIFO, the oldest message is always at the front.
            // We only need to check the front message. If it isn't starved, 
            // nothing behind it is starved either.
            while let Some(msg) = self.queues[tier].front() {
                let age = now.saturating_sub(msg.timestamp_ms);
                
                if age >= self.aging_threshold.as_millis() as u64 {
                    // Pop it out of the current low-priority tier
                    let mut starved_msg = self.queues[tier].pop_front().unwrap();
                    
                    // Push it to the back of the NEXT HIGHEST priority tier
                    self.queues[tier - 1].push_back(starved_msg);
                } else {
                    // Front message hasn't aged out yet; stop checking this tier.
                    break; 
                }
            }
        }
    }
}