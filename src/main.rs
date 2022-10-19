use std::{collections::HashMap, io};

use anyhow::Context;
use clap::Parser;
use tracing::{debug, info};

#[derive(Debug, clap::Parser)]
struct Args {
    /// The number of times to run the comand
    #[arg(short, long, default_value_t = 100)]
    runs: usize,
    /// The command to run
    #[arg(last(true), num_args(1..), required(true))]
    cmd: Vec<String>,
}

#[derive(Debug, Default)]
struct StatusSummary {
    successes: usize,
    killed: usize,
    failures: HashMap<i32, usize>,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .pretty()
        .with_file(false)
        .without_time()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let mut args = Args::parse();
    debug!(?args);

    let head = args.cmd.remove(0);
    let tail = args.cmd;

    info!("running `{head}` with args {tail:?}");

    let mut summary = StatusSummary::default();

    for _ in 0..args.runs {
        let output = std::process::Command::new(&head)
            .args(&tail)
            .output()
            .context("couldn't execute command")?;
        match output.status.code() {
            Some(0) => summary.successes += 1,
            Some(code) => *summary.failures.entry(code).or_default() += 1,
            None => summary.killed += 1,
        }
        info!(?output.status, ?summary);
        debug!(?output);
    }

    println!("successes: {}", summary.successes);
    println!(
        "failures: {}",
        summary.failures.values().sum::<usize>() + summary.killed
    );
    if summary.killed != 0 {
        println!("(killed): {}", summary.killed)
    }
    for (code, count) in summary.failures {
        println!("(exit code {code}): {count}")
    }

    Ok(())
}
