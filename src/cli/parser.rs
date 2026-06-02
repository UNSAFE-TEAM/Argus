use std::path::PathBuf;

use clap::{ArgGroup, Parser, ValueEnum};

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

    /// Execute with specified PID
    #[arg(short = 'P', long)]
    pid: Option<u32>,

    /// Use a preset configuration
    #[arg(short = 'p', long = "preset", value_enum)]
    pub presets: Option<Preset>,

    /// Use a module configuration
    #[arg(short, long = "module", value_enum)]
    pub module: Option<Module>,

    /// Output mode selection
    #[arg(short, long, value_enum, default_value_t = Output::Console)]
    output: Output,

    /// Save the output to a file
    #[arg(short, long)]
    save: Option<PathBuf>,

    /// Disable console output
    #[arg(short, long)]
    quiet: bool,

    /// Load scripts from a local directory instead of embedded scripts
    #[arg(long = "scripts-dir")]
    scripts_dir: Option<PathBuf>,

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
#[derive(Debug, Clone, ValueEnum)]
pub enum Preset {
    Vmware,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Module {
    Behavior,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Output {
    Console,
    Jsonl,
}

impl Preset {
    pub fn dir_name(&self) -> &'static str {
        match self {
            Preset::Vmware => "vmware",
        }
    }
}

impl Module {
    pub fn dir_name(&self) -> &'static str {
        match self {
            Module::Behavior => "behavior",
        }
    }
}

impl Args {
    pub fn scripts_dir(&self) -> Option<PathBuf> {
        self.scripts_dir.clone()
    }
    pub fn presets(&self) -> Option<Preset> {
        self.presets.clone()
    }
    pub fn module(&self) -> Option<Module> {
        self.module.clone()
    }
    pub fn output(&self) -> Output {
        self.output
    }
    pub fn save(&self) -> Option<PathBuf> {
        self.save.clone()
    }

    pub fn quiet(&self) -> bool {
        self.quiet
    }
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
