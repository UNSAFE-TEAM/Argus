use crate::output::event::ArgusEvent;

pub fn print(event: &ArgusEvent) {
    match serde_json::to_string(event) {
        Ok(line) => println!("{line}"),
        Err(err) => eprintln!("[jsonl serialize error] {err}"),
    }
}
