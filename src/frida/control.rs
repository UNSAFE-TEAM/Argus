use crate::cli::Target;
use anyhow::Context;
use frida::{DeviceManager, Frida, Message, ScriptHandler, ScriptOption, Session, SpawnOptions};
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
struct Handler;

impl ScriptHandler for Handler {
    fn on_message(&mut self, message: Message, data: Option<Vec<u8>>) {
        println!("[frida] {message:?}");

        if let Some(data) = data {
            println!("[frida data] {data:?}");
        }
    }
}

pub fn run(command: Target) -> anyhow::Result<()> {
    let frida = unsafe { Frida::obtain() };
    let manager = DeviceManager::obtain(&frida);
    let mut device = manager.get_local_device().unwrap();

    println!("[*] device: {:?}", device.get_name());

    match command {
        Target::Pid(pid) => {
            println!("[*] attaching pid: {pid}");
            let session = device.attach(pid).with_context(|| "attach failed")?;
            load_script(session)?;
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
            load_script(session)?;

            device.resume(pid).with_context(|| "resume failed")?;
            println!("[*] process resumed");
        }
    }

    loop {
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

fn load_script(session: Session<'_>) -> anyhow::Result<()> {
    let mut script_options = ScriptOption::default();
    let script_source = load_demo_scripts()?;

    let mut script = session
        .create_script(&script_source, &mut script_options)
        .with_context(|| "create script failed")?;

    script
        .handle_message(Handler)
        .with_context(|| "handle message failed")?;

    script.load().with_context(|| "load script failed")?;

    println!("[*] script loaded");
    Ok(())
}

fn load_demo_scripts() -> anyhow::Result<String> {
    let scripts_dir = PathBuf::from("scripts");
    let bootstrap_path = scripts_dir.join("bootstrap.js");
    let mut script_paths = vec![bootstrap_path];

    script_paths.extend(collect_js_scripts(&scripts_dir.join("anti_debug"))?);
    script_paths.extend(collect_js_scripts(&scripts_dir.join("anti_sandbox"))?);

    let mut scripts = Vec::with_capacity(script_paths.len());
    for path in script_paths {
        let script = fs::read_to_string(&path)
            .with_context(|| format!("read script failed: {}", path.display()))?;
        scripts.push(script);
    }

    Ok(scripts.join("\n\n"))
}

fn collect_js_scripts(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("read scripts directory failed: {}", dir.display()))?;

    let mut scripts = Vec::new();
    for entry in entries {
        let path = entry
            .with_context(|| format!("read scripts directory entry failed: {}", dir.display()))?
            .path();

        if path.extension().is_some_and(|ext| ext == "js") {
            scripts.push(path);
        }
    }

    scripts.sort();
    Ok(scripts)
}
