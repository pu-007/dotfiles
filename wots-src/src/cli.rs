use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::types::PkgType;

#[derive(Parser)]
#[command(
    name = "wots",
    about = "WSL Dotfile Stow Tool — unified dotfile management.",
    version = env!("CARGO_PKG_VERSION"),
    disable_version_flag = true,
    disable_help_subcommand = true,
    disable_colored_help = false,
)]
pub struct Cli {
    #[arg(
        short = 'u',
        long = "win-user",
        global = true,
        help = "Windows username (required for Windows package sync). Also settable via WIN_USER env var."
    )]
    pub win_user: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Create a new package from existing files")]
    Create(CreateArgs),
    #[command(about = "Force-copy packages to their targets")]
    Sync(SyncArgs),
    #[command(about = "Show repository statistics")]
    Stats(StatsArgs),
    #[command(about = "List all packages with details")]
    List(ListArgs),
    #[command(about = "Show file differences between repo and targets")]
    Diff(DiffArgs),
    #[command(about = "Generate shell completion script", hide = true)]
    Completion(CompletionArgs),
}

#[derive(Args)]
pub struct CompletionArgs {
    #[arg(help = "Target shell")]
    pub shell: CompletionShell,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
}

#[derive(Args)]
pub struct CreateArgs {
    #[arg(help = "Source file(s) or dir(s).", required = true, num_args = 1..)]
    pub sources: Vec<String>,

    #[arg(short = 'a', long = "app-name", help = "Custom app name.")]
    pub app_name: Option<String>,

    #[arg(short = 't', long = "type", help = "Package type")]
    pub pkg_type: Option<PkgType>,

    #[arg(long = "no-stow", help = "Skip stow after creation.")]
    pub no_stow: bool,

    #[arg(long = "no-sync", help = "Skip Windows sync after creation.")]
    pub no_sync: bool,

    #[arg(short = 'n', long = "dry-run", help = "Preview only.")]
    pub dry_run: bool,

    #[arg(short = 'y', long = "yes", help = "Skip confirmation prompts.")]
    pub yes: bool,
}

#[derive(Args)]
pub struct SyncArgs {
    #[arg(short = 't', long = "type", help = "Only sync this type")]
    pub pkg_type: Option<PkgType>,

    #[arg(long = "app", help = "Sync only a specific package.")]
    pub app: Option<String>,

    #[arg(short = 'n', long = "dry-run", help = "Preview only.")]
    pub dry_run: bool,

    #[arg(long = "bypass", help = "Skip root confirmation.")]
    pub bypass: bool,

    #[arg(short = 'q', long = "quiet", help = "Minimal output.")]
    pub quiet: bool,
}

#[derive(Args)]
pub struct StatsArgs {
    #[arg(short = 'j', long = "json", help = "JSON output.")]
    pub json_output: bool,
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(short = 't', long = "type", help = "Filter by type")]
    pub pkg_type: Option<PkgType>,

    #[arg(short = 'j', long = "json", help = "JSON output.")]
    pub json_output: bool,
}

#[derive(Args)]
pub struct DiffArgs {
    #[arg(short = 't', long = "type", help = "Filter by type")]
    pub pkg_type: Option<PkgType>,

    #[arg(long = "app", help = "Show diff for a specific package.")]
    pub app: Option<String>,

    #[arg(short = 'j', long = "json", help = "JSON output.")]
    pub json_output: bool,
}
