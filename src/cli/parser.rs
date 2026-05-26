use clap::{ArgGroup, Parser};

#[derive(Parser)]
#[command(name = "Argus", about = "Argus - Dynamic analysis tool based on Frida")]
#[command(
    version,
    arg_required_else_help = true,
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    infer_subcommands = true
)]
#[command(group(
    ArgGroup::new("target")
        .args(["exec", "pid"])
        .required(true)
        .multiple(false)
))]
pub struct Args {
    /// Designated program execution
    #[arg(short, long)]
    exec: Option<String>,

    // Execute with specified PID
    #[arg(short, long)]
    pid: Option<u32>,

    /// Display help information
    #[arg(short, long, action = clap::ArgAction::Help)]
    pub help: Option<bool>,

    /// Display version information
    #[arg(short = 'V', long, action = clap::ArgAction::Version)]
    pub version: Option<bool>,
}

#[derive(Debug)]
pub enum Target {
    Exec(String),
    Pid(u32),
}

impl Args {
    pub fn target(self) -> Target {
        if let Some(exec) = self.exec {
            return Target::Exec(exec);
        }

        if let Some(pid) = self.pid {
            return Target::Pid(pid);
        }

        unreachable!("clap requires either --exec or --pid");
    }
}
pub fn parser() -> Args {
    Args::parse()
}
