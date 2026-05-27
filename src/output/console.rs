use crate::output::event::{ArgusEvent, ArgusEventKind};

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const MAGENTA: &str = "\x1b[35m";

// pub fn print(event: &ArgusEvent) {
//     println!("{}", format(event));
// }

pub fn format(event: &ArgusEvent) -> String {
    let event_name = event_name(&event.event);
    let event_color = event_color(&event.event);
    let address = event.subject.address.as_deref().unwrap_or("-");

    match event.event {
        ArgusEventKind::Register => {
            format!(
                "{}[{}]{} [{}] {}",
                event_color, event_name, RESET, event.tag, event.subject.name
            )
        }
        _ => {
            format!(
                "{}[{}]{} [{}] {}{} @ {}{} {}",
                event_color,
                event_name,
                RESET,
                event.tag,
                event.subject.name,
                DIM,
                address,
                RESET,
                event.data
            )
        }
    }
}

fn event_name(event: &ArgusEventKind) -> &'static str {
    match event {
        ArgusEventKind::Init => "init",
        ArgusEventKind::Register => "register",
        ArgusEventKind::Collect => "collect",
        ArgusEventKind::Triggered => "triggered",
        ArgusEventKind::Skip => "skip",
        ArgusEventKind::Error => "error",
        ArgusEventKind::Other => "other",
    }
}

fn event_color(event: &ArgusEventKind) -> &'static str {
    match event {
        ArgusEventKind::Init => DIM,
        ArgusEventKind::Register => GREEN,
        ArgusEventKind::Collect => CYAN,
        ArgusEventKind::Triggered => YELLOW,
        ArgusEventKind::Skip => MAGENTA,
        ArgusEventKind::Error => RED,
        ArgusEventKind::Other => RESET,
    }
}
