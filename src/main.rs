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

            #[cfg(target_os = "macos")]
            {
                use std::sync::atomic::{AtomicBool, Ordering};
                use std::sync::Arc;

                let locked = Arc::new(AtomicBool::new(true));
                let locked_clone = Arc::clone(&locked);

                ctrlc::set_handler(move || {
                    if !locked_clone.load(Ordering::SeqCst) {
                        std::process::exit(0);
                    }
                }).ok();

                locker::show_lock_screen()?;
                locked.store(false, Ordering::SeqCst);
            }

            #[cfg(not(target_os = "macos"))]
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
            println!();
            println!("Note: If hotkey doesn't work, grant Accessibility permissions:");
            println!("  System Settings → Privacy & Security → Accessibility");
            println!();

            use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{Code, Modifiers, HotKey}};
            use std::sync::atomic::{AtomicBool, Ordering};
            use std::sync::Arc;
            use std::process::Command;

            let manager = GlobalHotKeyManager::new().context("Failed to create hotkey manager - may need Accessibility permissions")?;

            // Try Cmd+Shift+L first, fallback to Cmd+Option+L
            let hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyL);
            let registered_hotkey = match manager.register(hotkey) {
                Ok(_) => {
                    println!("✓ Hotkey registered: Cmd+Shift+L");
                    hotkey
                }
                Err(_) => {
                    // Fallback to Cmd+Option+L
                    let alt_hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyL);
                    match manager.register(alt_hotkey) {
                        Ok(_) => {
                            println!("✓ Hotkey registered: Cmd+Option+L (fallback)");
                            alt_hotkey
                        }
                        Err(e) => {
                            println!("✗ Failed to register hotkey: {}", e);
                            println!("\nTroubleshooting:");
                            println!("1. Grant Accessibility permissions:");
                            println!("   System Settings → Privacy & Security → Accessibility");
                            println!("2. Add Terminal.app (or your terminal) to allowed apps");
                            println!("3. Restart terminal and try again");
                            println!("\nNote: Some apps may conflict with global hotkeys");
                            return Err(e.into());
                        }
                    }
                }
            };

            let running = Arc::new(AtomicBool::new(true));
            let r = Arc::clone(&running);

            ctrlc::set_handler(move || {
                r.store(false, Ordering::SeqCst);
            })
            .context("Error setting Ctrl-C handler")?;

            let receiver = GlobalHotKeyEvent::receiver();
            let exe_path = std::env::current_exe()?;

            while running.load(Ordering::SeqCst) {
                if let Ok(_event) = receiver.try_recv() {
                    println!("Hotkey triggered - locking screen...");

                    let child = Command::new(&exe_path)
                        .arg("lock")
                        .spawn();

                    match child {
                        Ok(mut process) => {
                            let _ = process.wait();
                            println!("Screen unlocked");
                        }
                        Err(e) => {
                            eprintln!("Failed to spawn lock process: {}", e);
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            manager.unregister(registered_hotkey)?;
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
