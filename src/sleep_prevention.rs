use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct SleepPreventer {
    running: Arc<AtomicBool>,
    #[cfg(target_os = "macos")]
    handle: Option<thread::JoinHandle<()>>,
}

impl SleepPreventer {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            #[cfg(target_os = "macos")]
            handle: None,
        }
    }

    #[cfg(target_os = "macos")]
    pub fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.running);

        let handle = thread::spawn(move || {
            use std::process::Command;

            let child = Command::new("caffeinate")
                .arg("-d")
                .arg("-i")
                .arg("-s")
                .spawn();

            match child {
                Ok(mut process) => {
                    while running.load(Ordering::SeqCst) {
                        thread::sleep(Duration::from_secs(1));
                    }
                    let _ = process.kill();
                }
                Err(e) => {
                    eprintln!("Failed to start caffeinate: {}", e);
                }
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn start(&mut self) -> Result<()> {
        use windows::Win32::System::Power::{
            SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
        };

        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
        }

        self.running.store(true, Ordering::SeqCst);
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    #[cfg(target_os = "windows")]
    pub fn stop(&mut self) {
        use windows::Win32::System::Power::{SetThreadExecutionState, ES_CONTINUOUS};

        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS);
        }

        self.running.store(false, Ordering::SeqCst);
    }
}

impl Drop for SleepPreventer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sleep_preventer_lifecycle() {
        let mut preventer = SleepPreventer::new();
        assert!(!preventer.running.load(Ordering::SeqCst));

        preventer.start().unwrap();
        assert!(preventer.running.load(Ordering::SeqCst));

        preventer.stop();
        assert!(!preventer.running.load(Ordering::SeqCst));
    }
}
