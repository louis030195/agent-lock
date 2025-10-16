mod auth;
mod locker;
mod sleep_prevention;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use sleep_prevention::SleepPreventer;

#[derive(Parser)]
#[command(name = "agent-lock")]
#[command(about = "Lock screen with PIN while keeping AI agents and background apps running", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Setup,
    Lock,
    Daemon,
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
                println!("No PIN configured. Run 'agent-lock setup' first.");
                return Ok(());
            }

            println!("Starting screen lock...");

            let mut sleep_preventer = SleepPreventer::new();
            sleep_preventer.start()?;

            locker::show_lock_screen()?;

            sleep_preventer.stop();
            println!("Screen unlocked. Sleep prevention disabled.");
        }
        Commands::Daemon => {
            if !config_exists()? {
                println!("No PIN configured. Run 'agent-lock setup' first.");
                return Ok(());
            }

            println!("Starting agent-lock daemon...");
            println!("Press Cmd+Shift+L to lock screen");
            println!("Press Ctrl+C to quit daemon");

            use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{Code, Modifiers, HotKey}};
            use std::sync::atomic::{AtomicBool, Ordering};
            use std::sync::Arc;

            let manager = GlobalHotKeyManager::new().context("Failed to create hotkey manager")?;
            let hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyL);
            manager.register(hotkey).context("Failed to register hotkey")?;

            let running = Arc::new(AtomicBool::new(true));
            let r = Arc::clone(&running);

            ctrlc::set_handler(move || {
                r.store(false, Ordering::SeqCst);
            })
            .context("Error setting Ctrl-C handler")?;

            let receiver = GlobalHotKeyEvent::receiver();

            while running.load(Ordering::SeqCst) {
                if let Ok(_event) = receiver.try_recv() {
                    println!("Hotkey triggered - locking screen...");
                    let mut sleep_preventer = SleepPreventer::new();
                    sleep_preventer.start()?;

                    if let Err(e) = locker::show_lock_screen() {
                        eprintln!("Error showing lock screen: {}", e);
                    }

                    sleep_preventer.stop();
                    println!("Screen unlocked");
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            manager.unregister(hotkey)?;
            println!("\nDaemon stopped");
        }
        Commands::Status => {
            if config_exists()? {
                println!("✓ PIN is configured");
                println!("✓ Ready to lock screen");
                println!("\nUsage:");
                println!("  agent-lock lock        - Lock screen immediately");
                println!("  agent-lock daemon      - Run in background (Cmd+Shift+L to lock)");
            } else {
                println!("✗ No PIN configured");
                println!("Run 'agent-lock setup' to configure a PIN");
            }
        }
    }

    Ok(())
}

fn config_exists() -> Result<bool> {
    let path = auth::get_config_path()?;
    Ok(path.exists())
}
