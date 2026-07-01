use crate::db::Database;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

static RUNNING: AtomicBool = AtomicBool::new(false);

/// Start the background daemon.
pub fn start(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    if RUNNING.load(Ordering::SeqCst) {
        return Err("Daemon is already running".into());
    }

    // Record PID for stop/status
    let pid = std::process::id();
    db.set_daemon_state("pid", &pid.to_string())?;
    db.set_daemon_state("status", "running")?;

    RUNNING.store(true, Ordering::SeqCst);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    thread::spawn(move || {
        // In a real daemon, this would watch shell history files
        // via inotify (Linux) or ReadDirectoryChanges (Windows).
        // For MVP, we poll every 60 seconds.
        while r.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(60));
            // Future: watch for new history entries and auto-suggest
        }
    });

    println!("Daemon started (pid: {})", pid);
    Ok(())
}

/// Stop the background daemon.
pub fn stop(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    if !RUNNING.load(Ordering::SeqCst) {
        return Err("Daemon is not running".into());
    }
    RUNNING.store(false, Ordering::SeqCst);
    db.set_daemon_state("status", "stopped")?;
    println!("Daemon stopped");
    Ok(())
}

/// Show daemon status.
pub fn status(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let status = db.get_daemon_state("status")?.unwrap_or_else(|| "stopped".into());
    let pid = db.get_daemon_state("pid")?;

    let is_running = RUNNING.load(Ordering::SeqCst);

    println!("Daemon status: {}", if is_running { "🟢 running" } else { "🔴 stopped" });
    if let Some(pid) = pid {
        println!("  PID: {}", pid);
    }
    println!("  Persisted state: {}", status);

    Ok(())
}
