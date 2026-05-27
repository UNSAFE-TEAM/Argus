mod cli;
mod frida;
mod output;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::parser();
    let output = args.output();

    // 消息隧道
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // 接收输出
    tokio::spawn(async move {
        output::receive(rx, output).await;
    });

    // frida
    frida::control::run(args.target(), tx)?;

    Ok(())
}
