mod redlock;

use redlock::Redlock;

use colored::Colorize;
use log::{debug, LevelFilter};
use structopt::clap::AppSettings;
use structopt::StructOpt;

use std::ffi::OsString;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Execute a command when holding a distributed lock
#[derive(Debug, StructOpt)]
#[structopt(settings = &[AppSettings::TrailingVarArg])]
struct Args {
    /// List of redis servers used for the distributed lock
    #[structopt(long, short, required = true)]
    server: Vec<String>,
    /// Name of the lock to acquire
    #[structopt(long, short)]
    lock_name: String,
    /// Time in seconds after which the lock will expire
    #[structopt(long, short, default_value = "60")]
    ttl: u64,
    /// Time in seconds to try to acquire the lock
    #[structopt(long, default_value = "60")]
    timeout: u64,
    /// Command with arguments to execute
    #[structopt(parse(from_os_str))]
    cmd: Vec<OsString>,
    /// Verbose output
    #[structopt(long, short)]
    verbose: bool,
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();

    let mut builder = env_logger::builder();
    if args.verbose {
        builder.filter_module("redlock", LevelFilter::Debug);
    }
    builder.init();

    if args.cmd.is_empty() {
        return Ok(());
    }

    let servers: Vec<String> = args
        .server
        .iter()
        .map(|s| format!("redis://{}", s))
        .collect();

    let start = Instant::now();
    let timeout = Duration::from_secs(args.timeout as u64);
    while start.elapsed() <= timeout {
        debug!("trying to lock");
        if let Some(lock) =
            Redlock::try_lock(&servers, &args.lock_name, Duration::from_secs(args.ttl))
        {
            let lock_name = args.lock_name.clone();
            ctrlc::set_handler(move || {
                // move in the copy of the lock
                let _lock = &lock;
                // unlock on drop
                debug!("unlocked {}", lock_name);
            })?;

            if let Some(cmd) = args.cmd.first() {
                let _ = Command::new(cmd)
                    .args(args.cmd.iter().skip(1))
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output();
            }
            debug!("command finished");
            return Ok(());
        }
        thread::sleep(Duration::from_millis(100));
    }

    Err(format!(
        "failed to acquire lock {} in {} sec",
        args.lock_name, args.ttl
    )
    .into())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "error:".red(), e);
        std::process::exit(1);
    }
}
