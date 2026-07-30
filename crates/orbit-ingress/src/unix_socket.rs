use orbit_core::Message;
use tokio::net::UnixListener;
use tokio::sync::mpsc;
use tokio::io::AsyncReadExt;
use std::path::Path;

/// Listens for local IPC messages from internal satellite daemons over a Unix Socket.
pub async fn start_unix_listener<P: AsRef<Path>>(socket_path: P, sender: mpsc::Sender<Message>) -> Result<(), std::io::Error> {
    if socket_path.as_ref().exists() {
        tokio::fs::remove_file(&socket_path).await?;
    }

    let listener = UnixListener::bind(socket_path)?;
    println!("[IPC] Unix Domain Socket listener active.");

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let tx = sender.clone();
            tokio::spawn(async move {
                let mut buf = Vec::new();
                if stream.read_to_end(&mut buf).await.is_ok() {
                    if let Ok(msg) = Message::from_bytes(&buf) {
                        let _ = tx.send(msg).await;
                    }
                }
            });
        }
    });

    Ok(())
}