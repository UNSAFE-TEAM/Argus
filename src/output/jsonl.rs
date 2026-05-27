use crate::output::event::ArgusEvent;

// pub fn print(event: &ArgusEvent) {
//     match format(event) {
//         Ok(line) => println!("{line}"),
//         Err(err) => eprintln!("[jsonl serialize error] {err}"),
//     }
// }

pub fn format(event: &ArgusEvent) -> Result<String, serde_json::Error> {
    serde_json::to_string(event)
}
