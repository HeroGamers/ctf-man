use clap::{Parser, Subcommand};

/// CTF Challenge Manager
///
/// Usage:
///   ctf                        - Launch TUI to select active CTF
///   ctf <category> <name>      - Create challenge folder in active CTF
#[derive(Parser, Debug)]
#[command(name = "ctf")]
#[command(about = "Manage CTF challenges", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {}

/// Parsed challenge creation arguments (for direct CLI usage)
#[allow(dead_code)]
pub struct ChallengeArgs {
    pub category: String,
    pub challenge: String,
}
