use orbit_core::{Message, PriorityLevel};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

// Include the generated Rust code from our .proto file
pub mod pb {
    tonic::include_proto!("orbit");
}

use pb::broker_server::Broker;
use pb::{PublishRequest, PublishResponse};

/// The gRPC server implementation
pub struct OrbitBrokerService {
    // Async channel sender to pass messages safely to the core Priority Scheduler
    sender: mpsc::Sender<Message>,
    // Thread-safe counter to assign unique IDs to incoming messages
    next_id: AtomicU64,
}

impl OrbitBrokerService {
    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self {
            sender,
            next_id: AtomicU64::new(1),
        }
    }
}

#[tonic::async_trait]
impl Broker for OrbitBrokerService {
    async fn publish(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishResponse>, Status> {
        let req = request.into_inner();

        // Safely map the incoming integer to our strict PriorityLevel enum
        let priority = match req.priority {
            0 => PriorityLevel::CriticalLifeSupport,
            1 => PriorityLevel::ThrusterMetrics,
            3 => PriorityLevel::HighResMedia,
            _ => PriorityLevel::GeneralTelemetry, // Default fallback
        };

        // Increment and fetch the ID atomically
        let msg_id = self.next_id.fetch_add(1, Ordering::SeqCst);

        // Construct the core Message payload
        let msg = Message::new(msg_id, priority, req.source, req.payload);

        // Dispatch the message into the async channel. 
        // If the channel is closed, the main broker daemon has crashed.
        if self.sender.send(msg).await.is_err() {
            return Err(Status::internal("Broker core is offline."));
        }

        // Return a 200 OK equivalent instantly to the microservice
        Ok(Response::new(PublishResponse {
            success: true,
            message: "Data safely buffered in orbit-mq".into(),
        }))
    }
}