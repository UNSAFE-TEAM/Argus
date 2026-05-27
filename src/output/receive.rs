use tokio::sync::mpsc;

use crate::cli::Output;
use crate::output::event::FridaEnvelope;

pub async fn receive(mut rx: mpsc::UnboundedReceiver<String>, output: Output) {
    while let Some(raw) = rx.recv().await {
        match serde_json::from_str::<FridaEnvelope>(&raw) {
            Ok(envelope) => match output {
                Output::Console => crate::output::console::print(&envelope.payload),
                Output::Jsonl => crate::output::jsonl::print(&envelope.payload),
            },
            Err(err) => {
                eprintln!("[output parse error] {err}");
                eprintln!("{raw}");
            }
        }
    }
}
