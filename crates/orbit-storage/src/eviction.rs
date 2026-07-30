use orbit_core::PriorityLevel;

/// Decides which message can be safely dropped if the SSD fills up.
pub struct EvictionPolicy;

impl EvictionPolicy {
    /// Returns true if message A has lower priority than message B 
    /// (meaning A should be evicted first).
    pub fn should_evict(current_priority: PriorityLevel, incoming_priority: PriorityLevel) -> bool {
        // Lower numeric value = higher importance. 
        // If incoming is higher importance (lower number) than existing, drop low priority data.
        current_priority > incoming_priority
    }
}