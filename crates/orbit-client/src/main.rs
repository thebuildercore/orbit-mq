use orbit_ingress::pb::broker_client::BrokerClient;
use orbit_ingress::pb::PublishRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛰️ Connecting to OrbitMQ Broker on 127.0.0.1:50051...");
    
    // Connect to our broker's gRPC port
    let mut client = BrokerClient::connect("http://127.0.0.1:50051").await?;

    // Create a high-priority life support telemetry payload
    let request = tonic::Request::new(PublishRequest {
        priority: 0, // CriticalLifeSupport
        source: "life-support-module".into(),
        payload: b"Oxygen levels nominal, radiation shield holding.".to_vec(),
    });

    // Send the message
    let response = client.publish(request).await?;
    
    println!("Response from broker: {:?}", response.get_ref());

    Ok(())
}