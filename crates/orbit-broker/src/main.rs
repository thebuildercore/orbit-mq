use orbit_core::Message;
use orbit_ingress::{BrokerServer, OrbitBrokerService};
use orbit_priority::PriorityScheduler;
use orbit_storage::MmapWal;
use orbit_uplink::{LinkState, OpticalTransceiver};

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting OrbitMQ Space Broker...");

    // 1. Initialize Zero-Copy Storage (Allocate 10MB on SSD for the WAL)
    let wal_path = "satellite_telemetry.wal";
    let wal = Arc::new(Mutex::new(MmapWal::new(wal_path, 10 * 1024 * 1024)?));
    println!("💾 Storage WAL initialized at {}", wal_path);

    // 2. Initialize the Priority Scheduler (Age threshold = 30 seconds for testing)
    let scheduler = Arc::new(Mutex::new(PriorityScheduler::new(Duration::from_secs(30))));
    println!("🧠 MLFQ Priority Scheduler online");

    // 3. Create the Internal Communication Channel (Ingress -> Storage/Queue)
    let (tx, mut rx) = mpsc::channel::<Message>(10_000);

    // 4. Start the gRPC Ingress Server (For internal satellite microservices)
    let grpc_addr = "0.0.0.0:50051".parse()?;
    let broker_service = OrbitBrokerService::new(tx);
    
    println!("📡 Internal API listening on {}", grpc_addr);
    tokio::spawn(async move {
        Server::builder()
            .add_service(BrokerServer::new(broker_service))
            .serve(grpc_addr)
            .await
            .unwrap();
    });

    // 5. Start the Queue Processor (Write to SSD, then push to Scheduler)
    let sched_clone = Arc::clone(&scheduler);
    let wal_clone = Arc::clone(&wal);
    
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = wal_clone.lock().await.append(&msg) {
                eprintln!("⚠️ WAL Write Failed: {:?}", e);
            }
            sched_clone.lock().await.enqueue(msg);
        }
    });

    // 6. Start the Deep Space Egress State Machine wrapped in an Arc<Mutex>
    let transceiver = Arc::new(Mutex::new(OpticalTransceiver::new()));
    
    loop {
        // Satellite is on the dark side of Earth; wait for connection
        {
            let mut tx_lock = transceiver.lock().await;
            tx_lock.wait_for_window().await;
        }
        
        // --- BURST TRANSMISSION WINDOW OPEN ---
        loop {
            // Check state
            {
                let tx_lock = transceiver.lock().await;
                if tx_lock.state != LinkState::Active {
                    break;
                }
            }

            let msg_opt = scheduler.lock().await.dequeue();
            let tx_clone = Arc::clone(&transceiver);

            tokio::select! {
                res = async {
                    if let Some(msg) = msg_opt {
                        let tx_lock = tx_clone.lock().await;
                        tx_lock.transmit(&msg).await
                    } else {
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        Ok(())
                    }
                } => {
                    if let Err(e) = res {
                        eprintln!("⚠️ Transmission failed: {:?}", e);
                    }
                },
                
                _ = async {
                    let mut tx_lock = transceiver.lock().await;
                    tx_lock.window_closes().await;
                } => {
                    break; 
                }
            }
        }
    }
}