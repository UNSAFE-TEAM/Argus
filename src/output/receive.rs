use tokio::sync::mpsc;

pub async fn receive(mut rx: mpsc::UnboundedReceiver<String>) {
    while let Some(msg) = rx.recv().await {
        println!("{msg}");
    }
}
