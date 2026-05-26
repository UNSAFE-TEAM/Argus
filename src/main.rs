mod cli;
mod frida;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::parser();

    frida::control::run(args.target())?;

    Ok(())
}
