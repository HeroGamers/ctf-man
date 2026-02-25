use anyhow::Context;

use ctf_man::config::Config;
use ctf_man::runner;
use ctf_man::ui::restore_terminal;

fn main() {
    setup_panic_hook();

    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    // Handle primary usage modes BEFORE clap parsing to avoid "unrecognized subcommand" errors
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 3 {
        // Primary CLI usage: ctf <category> <challenge>
        let config = Config::load().context("Failed to load configuration")?;
        let category = &args[1];
        let challenge = &args[2];
        runner::create_challenge(&config, category, challenge)?;
        return Ok(());
    } else if args.len() == 1 {
        // No arguments: Launch TUI
        let mut config = Config::load().context("Failed to load configuration")?;

        // Ensure templates directory exists if CTF directory is configured
        if let Some(ref ctf_dir) = config.default_ctf_directory {
            ctf_man::templates::ensure_templates_directory(ctf_dir)?;
        }

        runner::run_tui(&mut config)?;
        return Ok(());
    }

    // Invalid usage
    eprintln!("Usage:");
    eprintln!("  ctf                        - Launch TUI to select active CTF");
    eprintln!("  ctf <category> <name>      - Create challenge folder");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  ctf pwn simple_notes       - Creates ./pwn/simple_notes/");
    std::process::exit(1);
}

/// Setup panic hook to restore terminal before panicking
fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));
}
