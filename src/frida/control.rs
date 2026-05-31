use crate::cli::{Preset, Target};
use anyhow::Context;
use frida::{DeviceManager, Frida, Message, ScriptHandler, ScriptOption, Session, SpawnOptions};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
    thread,
    time::Duration,
};

use tokio::sync::mpsc;

static OUTPUT_TX: OnceLock<mpsc::UnboundedSender<String>> = OnceLock::new();

struct Handler;

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

pub fn run(
    command: Target,
    tx: mpsc::UnboundedSender<String>,
    preset: Option<Preset>,
) -> anyhow::Result<()> {
    let _ = OUTPUT_TX.set(tx);

    let frida = unsafe { Frida::obtain() };
    let manager = DeviceManager::obtain(&frida);
    let mut device = manager.get_local_device().unwrap();
    println!("[*] device: {:?}", device.get_name());

    match command {
        Target::Pid(pid) => {
            println!("[*] attaching pid: {pid}");
            let session = device.attach(pid).with_context(|| "attach failed")?;
            let _script = load_script(&session, preset)?;
            wait_for_process_exit(&device, pid);
        }
        Target::Exec(command) => {
            let (program, args) = split_exec_command(&command)?;
            let mut argv = Vec::with_capacity(args.len() + 1);
            argv.push(program.clone());
            argv.extend(args);

            println!("[*] spawning: {}", argv.join(" "));

            let spawn_options = SpawnOptions::new().argv(argv);
            let pid = device
                .spawn(&program, &spawn_options)
                .with_context(|| "spawn failed")?;

            println!("[*] spawned pid: {pid}");
            println!("[*] attaching pid: {pid}");

            let session = device.attach(pid).with_context(|| "attach failed")?;
            let _script = load_script(&session, preset)?;

            device.resume(pid).with_context(|| "resume failed")?;
            println!("[*] process resumed");

            wait_for_process_exit(&device, pid);
        }
    }

    Ok(())
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

fn split_exec_command(command: &str) -> anyhow::Result<(String, Vec<String>)> {
    let mut parts = command.split_whitespace();
    let program = parts
        .next()
        .with_context(|| "exec command must contain a program path")?
        .to_string();
    let args = parts.map(str::to_string).collect();

    Ok((program, args))
}

fn load_script<'a>(
    session: &'a Session<'a>,
    preset: Option<Preset>,
) -> anyhow::Result<frida::Script<'a>> {
    let mut script_options = ScriptOption::default();
    let script_source = load_demo_scripts(preset)?;

    let mut script = session
        .create_script(&script_source, &mut script_options)
        .with_context(|| "create script failed")?;

    script
        .handle_message(Handler)
        .with_context(|| "handle message failed")?;

    script.load().with_context(|| "load script failed")?;

    println!("[*] script loaded");
    Ok(script)
}
fn load_demo_scripts(preset: Option<Preset>) -> anyhow::Result<String> {
    let scripts_dir = PathBuf::from("scripts");

    let mut scripts = Vec::new();

    scripts.push(include_str!("../../runtime/bootstrap/agent.v1.js").to_string());
    scripts.push(include_str!("../../runtime/sensors/sensors.v1.js").to_string());

    let mut script_paths = Vec::new();

    script_paths.extend(collect_js_scripts(&scripts_dir.join("sensors"))?);
    script_paths.extend(collect_js_scripts(&scripts_dir.join("anti_injection"))?);
    script_paths.extend(collect_js_scripts(&scripts_dir.join("anti_debug"))?);
    script_paths.extend(collect_js_scripts(&scripts_dir.join("anti_sandbox"))?);
    script_paths.extend(load_preset_scripts(&scripts_dir, preset)?);

    for path in script_paths {
        let script = fs::read_to_string(&path)
            .with_context(|| format!("read script failed: {}", path.display()))?;
        scripts.push(script);
    }

    Ok(scripts.join("\n\n"))
}

fn load_preset_scripts(scripts_dir: &Path, preset: Option<Preset>) -> anyhow::Result<Vec<PathBuf>> {
    let Some(preset) = preset else {
        return Ok(Vec::new());
    };

    let preset_dir = scripts_dir.join("presets").join(preset.dir_name());

    collect_js_scripts(&preset_dir)
}

fn collect_js_scripts(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("read scripts directory failed: {}", dir.display()))?;

    let mut scripts = Vec::new();
    for entry in entries {
        let path = entry
            .with_context(|| format!("read scripts directory entry failed: {}", dir.display()))?
            .path();

        if path.is_dir() {
            scripts.extend(collect_js_scripts(&path)?);
        } else if path.extension().is_some_and(|ext| ext == "js") {
            scripts.push(path);
        }
    }

    scripts.sort();
    Ok(scripts)
}
