use std::io;
use std::process;

use anyhow::Result;
use colored::Colorize;

fn main() {
    let result = run();

    match result {
        Ok(()) => process::exit(0),
        Err(e) => {
            if let Some(io_err) = e.downcast_ref::<io::Error>()
                && io_err.kind() == io::ErrorKind::BrokenPipe {
                    process::exit(0);
                }
            eprintln!("{} {}", "✗".red(), e);
            process::exit(1);
        }
    }
}

fn run() -> Result<()> {
    let cli = <wots::cli::Cli as clap::Parser>::parse();

    match cli.command {
        wots::cli::Command::Create(args) => wots::create::run(args),
        wots::cli::Command::Sync(args) => wots::sync::run(args),
        wots::cli::Command::Stats(args) => wots::commands::cmd_stats(&args),
        wots::cli::Command::List(args) => wots::commands::cmd_list(&args),
        wots::cli::Command::Diff(args) => wots::commands::cmd_diff(&args),
    }
}
