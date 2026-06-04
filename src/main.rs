mod cli;
mod frida;
mod output;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_console_ansi();

    let args = cli::parser();
    let output = args.output();
    let path = args.save();
    let quiet = args.quiet();
    let preset = args.presets();
    let module = args.module();
    let source = args.scripts_dir();

    // 消息隧道
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // 接收输出
    tokio::spawn(async move {
        output::receive(rx, output, path, quiet).await;
    });

    // frida
    let runner = frida::FridaRunner::new(args.target(), tx, preset, module, source);
    runner.run()?;

    Ok(())
}

fn enable_console_ansi() {
    use std::ffi::c_void;

    type Handle = *mut c_void;

    const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    const STD_ERROR_HANDLE: u32 = -12i32 as u32;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    unsafe extern "system" {
        fn GetStdHandle(nStdHandle: u32) -> Handle;
        fn GetConsoleMode(hConsoleHandle: Handle, lpMode: *mut u32) -> i32;
        fn SetConsoleMode(hConsoleHandle: Handle, dwMode: u32) -> i32;
    }

    fn enable(handle_id: u32) {
        unsafe {
            let handle = GetStdHandle(handle_id);
            if handle.is_null() || handle as isize == -1 {
                return;
            }

            let mut mode = 0;
            if GetConsoleMode(handle, &mut mode) == 0 {
                return;
            }

            let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
    }

    enable(STD_OUTPUT_HANDLE);
    enable(STD_ERROR_HANDLE);
}
