use std::io;
use std::process;

use anyhow::Result;
use clap::CommandFactory;
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

    // Apply --win-user override via WIN_USER env var before any LazyLock is evaluated.
    // SAFETY: called at program start before any threads are spawned.
    if let Some(ref user) = cli.win_user {
        unsafe { std::env::set_var("WIN_USER", user) };
    }

    match cli.command {
        wots::cli::Command::Create(args) => wots::create::run(args),
        wots::cli::Command::Sync(args) => wots::sync::run(args),
        wots::cli::Command::Stats(args) => wots::commands::cmd_stats(&args),
        wots::cli::Command::List(args) => wots::commands::cmd_list(&args),
        wots::cli::Command::Diff(args) => wots::commands::cmd_diff(&args),
        wots::cli::Command::Completion(args) => {
            use clap_complete::{generate, Shell};
            use wots::cli::CompletionShell;

            let mut cmd = wots::cli::Cli::command();
            let shell = match args.shell {
                CompletionShell::Bash => Shell::Bash,
                CompletionShell::Zsh => Shell::Zsh,
                CompletionShell::Fish => Shell::Fish,
            };
            generate(shell, &mut cmd, "wots", &mut io::stdout());
            Ok(())
        }
    }
}
