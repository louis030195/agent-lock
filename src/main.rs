mod auth;
mod locker;
mod sleep_prevention;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use sleep_prevention::SleepPreventer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "screen-locker")]
#[command(about = "Lock your screen while keeping apps running and preventing sleep", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Setup,
    Lock,
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup => {
            auth::setup_pin()?;
        }
        Commands::Lock => {
            if !config_exists()? {
                println!("No PIN configured. Run 'screen-locker setup' first.");
                return Ok(());
            }

            println!("Starting screen lock...");
            println!("Sleep prevention: enabled");
            println!("Background apps: will continue running");

            let mut sleep_preventer = SleepPreventer::new();
            sleep_preventer.start()?;

            let running = Arc::new(AtomicBool::new(true));
            let r = Arc::clone(&running);

            ctrlc::set_handler(move || {
                r.store(false, Ordering::SeqCst);
            })
            .context("Error setting Ctrl-C handler")?;

            println!("\n🔒 Screen locked. Enter your PIN to unlock.\n");

            while running.load(Ordering::SeqCst) {
                if locker::prompt_for_unlock()? {
                    break;
                }
            }

            sleep_preventer.stop();
            println!("\n✓ Screen unlocked. Sleep prevention disabled.");
        }
        Commands::Status => {
            if config_exists()? {
                println!("✓ PIN is configured");
                println!("✓ Ready to lock screen");
            } else {
                println!("✗ No PIN configured");
                println!("Run 'screen-locker setup' to configure a PIN");
            }
        }
    }

    Ok(())
}

fn config_exists() -> Result<bool> {
    let path = auth::get_config_path()?;
    Ok(path.exists())
}
