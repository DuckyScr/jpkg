use anyhow::Result;
use colored::Colorize;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn watch_and_build(run_after_build: bool, main_class: Option<String>) -> Result<()> {
    println!("{}", "👀 Watching for changes...".cyan());
    println!("{}", "  Press Ctrl+C to stop".dimmed());
    println!();

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(_event) = res {
                let _ = tx.send(());
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(1)),
    )?;

    watcher.watch(Path::new("src"), RecursiveMode::Recursive)?;

    // Initial build
    build_and_run(run_after_build, main_class.clone())?;

    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(_) => {
                // Debounce - wait a bit for more changes
                std::thread::sleep(Duration::from_millis(200));

                // Drain any pending events
                while rx.try_recv().is_ok() {}

                println!();
                println!("{}", "🔄 Changes detected, rebuilding...".yellow());
                println!();

                match build_and_run(run_after_build, main_class.clone()) {
                    Ok(_) => {
                        println!();
                        println!("{}", "👀 Watching for changes...".cyan());
                    }
                    Err(e) => {
                        println!();
                        println!("{}", format!("❌ Error: {}", e).red());
                        println!("{}", "👀 Watching for changes...".cyan());
                    }
                }
            }
            Err(_) => {
                // Timeout, continue watching
            }
        }
    }
}

fn build_and_run(run_after_build: bool, main_class: Option<String>) -> Result<()> {
    crate::project::build_project(false)?;

    if run_after_build {
        println!();
        println!("{}", "▶️  Running...".cyan());
        match crate::project::run_project(main_class, false) {
            Ok(_) => {
                println!("{}", "✓ Program finished successfully".green());
            }
            Err(e) => {
                println!("{}", format!("❌ Program crashed: {}", e).red());
                println!("{}", "💡 Use 'jpkg log' to see full error details".yellow());
            }
        }
    }

    Ok(())
}
