use super::load::{self, ScriptLoadOptions};
use super::message::Handler;
use crate::cli::{Preset, Target};
use anyhow::Context;
use frida::{DeviceManager, Frida, ScriptOption, Session, SpawnOptions};
use std::{thread, time::Duration};
use tokio::sync::mpsc;

pub struct FridaRunner {
    command: Target,
    tx: mpsc::UnboundedSender<String>,
    preset: Option<Preset>,
}

impl FridaRunner {
    pub fn new(command: Target, tx: mpsc::UnboundedSender<String>, preset: Option<Preset>) -> Self {
        Self {
            command,
            tx,
            preset,
        }
    }

    pub fn run(self) -> anyhow::Result<()> {
        super::message::set_output_tx(self.tx)?;

        let options = ScriptLoadOptions {
            preset: self.preset,
        };

        let frida = unsafe { Frida::obtain() };
        let manager = DeviceManager::obtain(&frida);
        let mut device = manager.get_local_device().unwrap();

        match self.command {
            Target::Pid(pid) => {
                let session = device.attach(pid).with_context(|| "attach failed")?;
                let _script = load_script(&session, &options)?;
                wait_for_process_exit(&device, pid);
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
                let _script = load_script(&session, &options)?;

                device.resume(pid).with_context(|| "resume failed")?;
                wait_for_process_exit(&device, pid);
            }
        }

        Ok(())
    }
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

fn split_exec_command(command: &str) -> anyhow::Result<(String, Vec<String>)> {
    let mut parts = command.split_whitespace();

    let program = parts
        .next()
        .with_context(|| "exec command must contain a program path")?
        .to_string();

    let args = parts.map(str::to_string).collect();

    Ok((program, args))
}
