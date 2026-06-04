use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
};

use tokio::sync::mpsc;

use crate::{
    cli::Output,
    output::{console, event::FridaEnvelope, jsonl},
};

pub async fn receive(
    mut rx: mpsc::UnboundedReceiver<String>,
    output: Output,
    path: Option<PathBuf>,
    quiet: bool,
) {
    let mut writer = path.and_then(|p| {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&p)
            .map(BufWriter::new)
            .map_err(|err| eprintln!("[output file error] {}: {err}", p.display()))
            .ok()
    });

    while let Some(raw) = rx.recv().await {
        let envelope = match serde_json::from_str::<FridaEnvelope>(&raw) {
            Ok(envelope) => envelope,
            Err(err) => {
                eprintln!("[output parse error] {err}");
                eprintln!("{raw}");
                continue;
            }
        };

        let line = match output {
            Output::Console => console::format(&envelope.payload),
            Output::Jsonl => match jsonl::format(&envelope.payload) {
                Ok(line) => line,
                Err(err) => {
                    eprintln!("[jsonl serialize error] {err}");
                    continue;
                }
            },
        };

        if !quiet {
            println!("{line}");
        }

        if let Some(writer) = writer.as_mut() {
            if let Err(err) = writeln!(writer, "{line}") {
                eprintln!("[output file write error] {err}");
            }

            if let Err(err) = writer.flush() {
                eprintln!("[output file flush error] {err}");
            }
        }
    }

    if let Some(mut writer) = writer {
        let _ = writer.flush();
    }
}
