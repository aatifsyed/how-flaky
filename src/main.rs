use anyhow::Context as _;
use clap::{Parser, ValueEnum};
use colored::Colorize as _;
use std::{collections::HashMap, fmt, process::Output};

#[derive(ValueEnum, Clone, Copy)]
enum ShowOutput {
    OnSuccess,
    OnFailure,
    Always,
    Never,
}

#[derive(Parser)]
struct Args {
    /// The number of times to run the command
    #[arg(short, long, default_value_t = 100)]
    runs: usize,
    /// When to show the stdout and stderr of each run
    #[arg(short, long, default_value = "never")]
    show_output: ShowOutput,
    /// The command to run
    cmd: String,
    args: Vec<String>,
}

#[derive(Debug, Default)]
struct StatusSummary {
    successes: usize,
    killed: usize,
    failures: HashMap<i32, usize>,
}

fn main() -> anyhow::Result<()> {
    let Args {
        runs,
        show_output,
        cmd,
        args,
    } = Args::parse();

    let mut summary = StatusSummary::default();

    for _ in 0..runs {
        let Output {
            status,
            stdout,
            stderr,
        } = std::process::Command::new(&cmd)
            .args(&args)
            .output()
            .context("couldn't execute command")?;
        match status.code() {
            Some(0) => {
                summary.successes += 1;
                println!("{}\t{}", "ok".green(), summary)
            }
            Some(code) => {
                *summary.failures.entry(code).or_default() += 1;
                println!("{}\t{}", format!("failed: {code}").red(), summary);
            }
            None => {
                summary.killed += 1;
                println!("{}\t{}", "killed".red(), summary)
            }
        }
        match (show_output, status.success()) {
            (ShowOutput::OnSuccess, true)
            | (ShowOutput::OnFailure, false)
            | (ShowOutput::Always, _) => {
                if !stdout.is_empty() {
                    println!("{}", "stdout:".bold());
                    println!("{}", String::from_utf8_lossy(stdout.as_slice()).dimmed())
                }
                if !stderr.is_empty() {
                    println!("{}", "stderr:".bold());
                    println!("{}", String::from_utf8_lossy(stderr.as_slice()).dimmed())
                }
            }
            _ => {}
        }
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

impl fmt::Display for StatusSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let successes = self.successes;
        let failures = self.killed + self.failures.values().sum::<usize>();
        f.write_fmt(format_args!(
            "{}",
            format!("({successes} successes, {failures} failures)").dimmed()
        ))
    }
}
