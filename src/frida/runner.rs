use super::load::{self, ScriptLoadOptions};
use super::message::Handler;
use crate::cli::{Module, Preset, Target};
use anyhow::{Context, bail};
use frida::{DeviceManager, Frida, ScriptOption, Session, SpawnOptions};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, io, thread};
use tokio::sync::mpsc;

const MESSAGE_DRAIN_DELAY: Duration = Duration::from_millis(250);
const PYTHON_HELPER_SOURCE: &str = include_str!("../../python/follow_children_runner.py");

pub struct FridaRunner {
    command: Target,
    tx: mpsc::UnboundedSender<String>,
    preset: Option<Preset>,
    module: Option<Module>,
    source: Option<PathBuf>,
    follow_children: bool,
}

impl FridaRunner {
    pub fn new(
        command: Target,
        tx: mpsc::UnboundedSender<String>,
        preset: Option<Preset>,
        module: Option<Module>,
        source: Option<PathBuf>,
        follow_children: bool,
    ) -> Self {
        Self {
            command,
            tx,
            preset,
            module,
            source,
            follow_children,
        }
    }

    pub fn run(self) -> anyhow::Result<()> {
        let output_tx = self.tx.clone();
        super::message::set_output_tx(output_tx.clone())?;

        let options = ScriptLoadOptions {
            preset: self.preset,
            module: self.module,
            scripts_dir: self.source,
        };

        if self.follow_children {
            run_with_python_helper(&self.command, &options, output_tx)
        } else {
            run_native(self.command, &options)
        }
    }
}

fn run_native(command: Target, options: &ScriptLoadOptions) -> anyhow::Result<()> {
    let frida = unsafe { Frida::obtain() };
    let manager = DeviceManager::obtain(&frida);
    let mut device = manager.get_local_device().unwrap();

    match command {
        Target::Pid(pid) => {
            let session = device.attach(pid).with_context(|| "attach failed")?;
            let _script = load_script(&session, options)?;
            wait_for_process_exit(&device, pid);
            drain_pending_messages();
        }

        Target::Exec(command) => {
            let (program, args) = split_exec_command(&command)?;
            let mut argv = Vec::with_capacity(args.len() + 1);
            argv.push(program.clone());
            argv.extend(args);

            let spawn_options = SpawnOptions::new().argv(argv);
            let pid = device
                .spawn(&program, &spawn_options)
                .with_context(|| "spawn failed")?;

            let session = device.attach(pid).with_context(|| "attach failed")?;
            let _script = load_script(&session, options)?;

            device.resume(pid).with_context(|| "resume failed")?;
            wait_for_process_exit(&device, pid);
            drain_pending_messages();
        }
    }

    Ok(())
}

fn run_with_python_helper(
    command: &Target,
    options: &ScriptLoadOptions,
    output_tx: mpsc::UnboundedSender<String>,
) -> anyhow::Result<()> {
    let script_source = load::scripts(options)?;
    let helper_file = TempFile::new("argus-follow-children", "py", PYTHON_HELPER_SOURCE)?;
    let source_file = TempFile::new("argus-script-source", "js", &script_source)?;

    let mut child = spawn_python_helper(helper_file.path(), source_file.path(), command)?;

    let stdout = child
        .stdout
        .take()
        .with_context(|| "python helper stdout unavailable")?;
    let stderr = child
        .stderr
        .take()
        .with_context(|| "python helper stderr unavailable")?;

    let stdout_thread = thread::spawn(move || -> io::Result<()> {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let _ = output_tx.send(trimmed.to_string());
        }
        Ok(())
    });

    let stderr_thread = thread::spawn(move || -> io::Result<()> {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            eprintln!("[python helper] {}", line?);
        }
        Ok(())
    });

    let status = child.wait().with_context(|| "python helper wait failed")?;

    stdout_thread
        .join()
        .map_err(|_| anyhow::anyhow!("python helper stdout thread panicked"))??;
    stderr_thread
        .join()
        .map_err(|_| anyhow::anyhow!("python helper stderr thread panicked"))??;

    if !status.success() {
        bail!("python helper failed with status {status}");
    }

    drain_pending_messages();
    Ok(())
}

fn spawn_python_helper(
    helper_path: &Path,
    script_source_path: &Path,
    command: &Target,
) -> anyhow::Result<Child> {
    let candidates = python_candidates();
    let mut last_error = None;

    for candidate in candidates {
        let mut cmd = Command::new(&candidate.program);
        cmd.args(&candidate.prefix_args)
            .arg(helper_path)
            .arg("--script-source-file")
            .arg(script_source_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        match command {
            Target::Pid(pid) => {
                cmd.arg("--pid").arg(pid.to_string());
            }
            Target::Exec(command) => {
                let (program, args) = split_exec_command(command)?;
                cmd.arg("--program").arg(program);
                for arg in args {
                    cmd.arg("--arg").arg(arg);
                }
            }
        }

        match cmd.spawn() {
            Ok(child) => return Ok(child),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                last_error = Some(anyhow::anyhow!(
                    "python interpreter not found: {}",
                    candidate.program
                ));
            }
            Err(err) => {
                return Err(err).with_context(|| {
                    format!("failed to spawn python helper with {}", candidate.program)
                });
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("no usable python interpreter found")))
        .context(
            "set ARGUS_PYTHON or install python with the frida package to use --follow-children",
        )
}

fn python_candidates() -> Vec<PythonCandidate> {
    if let Ok(program) = env::var("ARGUS_PYTHON") {
        if !program.trim().is_empty() {
            return vec![PythonCandidate::new(program, Vec::new())];
        }
    }

    vec![
        PythonCandidate::new("python".to_string(), Vec::new()),
        PythonCandidate::new("python3".to_string(), Vec::new()),
        PythonCandidate::new("py".to_string(), vec!["-3".to_string()]),
    ]
}

fn load_script<'a>(
    session: &'a Session<'a>,
    options: &ScriptLoadOptions,
) -> anyhow::Result<frida::Script<'a>> {
    let mut script_options = ScriptOption::default();
    let script_source = load::scripts(options)?;

    let mut script = session
        .create_script(&script_source, &mut script_options)
        .with_context(|| "create script failed")?;

    script
        .handle_message(Handler)
        .with_context(|| "handle message failed")?;

    script.load().with_context(|| "load script failed")?;

    Ok(script)
}

fn wait_for_process_exit(device: &frida::Device<'_>, pid: u32) {
    loop {
        let exists = device
            .enumerate_processes()
            .iter()
            .any(|process| process.get_pid() == pid);

        if !exists {
            break;
        }

        thread::sleep(Duration::from_secs(1));
    }
}

fn drain_pending_messages() {
    thread::sleep(MESSAGE_DRAIN_DELAY);
}

fn split_exec_command(command: &str) -> anyhow::Result<(String, Vec<String>)> {
    let mut parts = command.split_whitespace();

    let program = parts
        .next()
        .with_context(|| "exec command must contain a program path")?
        .to_string();

    let args = parts.map(str::to_string).collect();

    Ok((program, args))
}

struct PythonCandidate {
    program: String,
    prefix_args: Vec<String>,
}

impl PythonCandidate {
    fn new(program: String, prefix_args: Vec<String>) -> Self {
        Self {
            program,
            prefix_args,
        }
    }
}

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(prefix: &str, extension: &str, content: &str) -> anyhow::Result<Self> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = env::temp_dir().join(format!("{prefix}-{unique}.{extension}"));
        fs::write(&path, content)
            .with_context(|| format!("write temp file failed: {}", path.display()))?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
