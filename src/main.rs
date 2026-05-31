mod cli;
mod frida;
mod output;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::parser();
    let output = args.output();
    let path = args.save();
    let quiet = args.quiet();
    let preset = args.presets();
    let source = args.scripts_dir();

    // 消息隧道
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // 接收输出
    tokio::spawn(async move {
        output::receive(rx, output, path, quiet).await;
    });

    // frida
    let runner = frida::FridaRunner::new(args.target(), tx, preset, source);
    runner.run()?;

    Ok(())
}
