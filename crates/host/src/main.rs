use std::io::stdout;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::Duration;

use anyhow::{bail, Result};
use clap::Parser;
use hyperlight_host::func::HostFunction0 as _;
use hyperlight_host::func::HostFunction1 as _;
use hyperlight_host::func::ParameterValue;
use hyperlight_host::func::ReturnValue;
use hyperlight_host::sandbox::SandboxConfiguration;
use hyperlight_host::GuestBinary;
use hyperlight_host::HyperlightError;

use hyperlight_host::func::ReturnType;
use hyperlight_host::sandbox_state::sandbox::EvolvableSandbox;
use hyperlight_host::sandbox_state::transition::Noop;
use hyperlight_host::{MultiUseSandbox, UninitializedSandbox};

mod stdin;

#[derive(Parser, Debug)]
struct Args {
    /// Guest binary to execute
    guest: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;

    let writer = move |msg: String| -> Result<i32, HyperlightError> {
        print!("{msg}");
        let _ = stdout().flush();
        Ok(msg.len() as i32)
    };
    let writer = Arc::new(StdMutex::new(writer));

    let stdin = Arc::new(stdin::BlockingStdin::new());
    stdin.spawn();

    let stdin_clone = stdin.clone();
    let reader = move |count: u64| -> Result<Vec<u8>, HyperlightError> {
        Ok(stdin_clone.read(count as usize))
    };
    let reader = Arc::new(StdMutex::new(reader));

    let stdin_clone = stdin.clone();
    let try_read = move |count: u64| -> Result<Vec<u8>, HyperlightError> {
        Ok(stdin_clone.try_read(count as usize))
    };
    let try_read = Arc::new(StdMutex::new(try_read));

    let stdin_clone = stdin.clone();
    let poll_read = move |timeout: u64| -> Result<bool, HyperlightError> {
        Ok(stdin_clone.poll_data(Duration::from_micros(timeout)))
    };
    let poll_read = Arc::new(StdMutex::new(poll_read));

    let time = move || -> Result<u64, HyperlightError> {
        let now = std::time::SystemTime::now();
        let now = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        Ok(now.as_micros() as u64)
    };
    let time = Arc::new(StdMutex::new(time));

    let sleep = |duration: u64| -> Result<(), HyperlightError> {
        std::thread::sleep(Duration::from_micros(duration));
        Ok(())
    };
    let sleep = Arc::new(StdMutex::new(sleep));

    let mut cfg = SandboxConfiguration::default();
    cfg.set_kernel_stack_size(2 * 1024 * 1024);
    cfg.set_heap_size(32 * 1024 * 1024);
    cfg.set_output_data_size(4 * 1024 * 1024);
    cfg.set_max_execution_time(Duration::from_secs(100000));

    // Create an uninitialized sandbox with a guest binary
    let mut sandbox = UninitializedSandbox::new(
        GuestBinary::FilePath(args.guest.to_string_lossy().to_string()),
        Some(cfg),
        None,
        Some(&writer),
    )?;

    reader.register(&mut sandbox, "HostInput")?;
    try_read.register(&mut sandbox, "TryInput")?;
    poll_read.register(&mut sandbox, "PollInput")?;
    time.register(&mut sandbox, "GetTime")?;
    sleep.register(&mut sandbox, "Sleep")?;

    let mut sandbox: MultiUseSandbox = sandbox.evolve(Noop::default())?;

    // Call guest function
    let result = sandbox.call_guest_function_by_name(
        "Main", // function must be defined in the guest binary
        ReturnType::Int,
        Some(vec![ParameterValue::String("my friend".to_string())]),
    )?;

    let ReturnValue::Int(result) = result else {
        bail!("Expected an integer return value");
    };

    if result != 0 {
        std::process::exit(result);
    }

    Ok(())
}
