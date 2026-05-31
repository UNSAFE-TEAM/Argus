use frida::{Message, ScriptHandler};
use std::sync::OnceLock;
use tokio::sync::mpsc;

static OUTPUT_TX: OnceLock<mpsc::UnboundedSender<String>> = OnceLock::new();

pub fn set_output_tx(tx: mpsc::UnboundedSender<String>) -> anyhow::Result<()> {
    OUTPUT_TX
        .set(tx)
        .map_err(|_| anyhow::anyhow!("output channel already initialized"))?;
    Ok(())
}

pub struct Handler;

impl ScriptHandler for Handler {
    fn on_message(&mut self, message: Message, data: Option<Vec<u8>>) {
        if let Some(raw) = extract_argus_raw_message(&message) {
            if let Some(tx) = OUTPUT_TX.get() {
                let _ = tx.send(raw);
            }
            return;
        }

        eprintln!("[frida] {message:?}");

        if let Some(data) = data {
            eprintln!("[frida data] {data:?}");
        }
    }
}

fn extract_argus_raw_message(message: &Message) -> Option<String> {
    let Message::Other(value) = message else {
        return None;
    };

    value
        .get("data")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}
