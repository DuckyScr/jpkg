mod cache;
mod lockfile;
mod logger;
mod manifest;
mod maven;
mod packager;
mod platform;
mod project;
mod resolver;
mod testing;
mod updater;
mod watcher;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use manifest::Manifest;
use maven::MavenClient;
use resolver::Resolver;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "jpkg")]
#[command(about = "A Java Package Manager written in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init {
        /// Project name
        name: String,
    },
    /// Find and add a dependency
    Find {
        /// Search query
        query: String,
    },
    /// Add a dependency
    Add {
        /// Dependency coordinate (group:artifact:version) or artifact name
        coordinate: String,
    },
    /// Install dependencies from jpkg.json
    Install {
        /// Verify checksums from lock file (fail if mismatch)
        #[arg(long)]
        frozen: bool,
        /// Use only cached JARs (no network)
        #[arg(long)]
        offline: bool,
    },
    /// Build the project
    Build {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Run the project
    Run {
        /// Main class to run (default: Main)
        #[arg(short, long)]
        main: Option<String>,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Remove a dependency
    Remove {
        /// Artifact name or coordinate to remove
        name: String,
    },
    /// Package project into executable JAR
    Package {
        /// Output JAR name (default: app.jar)
        #[arg(short, long)]
        output: Option<String>,
        /// Main class (default: Main)
        #[arg(short, long)]
        main: Option<String>,
    },
    /// Run tests
    Test {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Check for dependency updates
    Outdated,
    /// Update dependencies
    Update {
        /// Specific package to update
        package: Option<String>,
    },
    /// Watch for changes and auto-rebuild
    Watch {
        /// Run after each successful build
        #[arg(short, long)]
        run: bool,
        /// Main class to run (default: Main)
        #[arg(short, long)]
        main: Option<String>,
    },
    /// Manage cache
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
    /// Show last error log
    Log,
    /// Show version information
    Version,
}

#[derive(Subcommand)]
enum CacheCommands {
    /// List cached artifacts
    List,
    /// Clear cache
    Clean,
    /// Show cache size
    Size,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { name } => {
            project::init_project(name)?;
            println!(
                "{}",
                format!("✓ Initialized project '{}' in ./{}/", name, name)
                    .green()
                    .bold()
            );
            println!(
                "{}",
                format!("  Run 'cd {}' to enter the project directory", name).dimmed()
            );
        }
        Commands::Find { query } => {
            let client = MavenClient::new();
            println!("{}", format!("🔍 Searching for '{}'...", query).cyan());
            let results = client.search_artifact(query)?;

            if results.is_empty() {
                anyhow::bail!("{}", format!("No artifacts found for '{}'", query).red());
            }

            let items: Vec<String> = results
                .iter()
                .map(|r| format!("{}:{} ({})", r.g, r.a, r.latest_version))
                .collect();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select an artifact to add")
                .default(0)
                .items(&items)
                .interact()?;

            let selected = &results[selection];

            // Add to jpkg.json
            if !Path::new("jpkg.json").exists() {
                anyhow::bail!("{}", "jpkg.json not found. Run 'jpkg init' first.".red());
            }

            let content = fs::read_to_string("jpkg.json")?;
            let mut manifest: Manifest = serde_json::from_str(&content)?;

            manifest.dependencies.insert(
                format!("{}:{}", selected.g, selected.a),
                selected.latest_version.clone(),
            );

            let content = serde_json::to_string_pretty(&manifest)?;
            fs::write("jpkg.json", content)?;

            println!(
                "{}",
                format!(
                    "✓ Added {}:{}:{} to jpkg.json",
                    selected.g, selected.a, selected.latest_version
                )
                .green()
                .bold()
            );
        }
        Commands::Add { coordinate } => {
            let (group, artifact, version) = if coordinate.contains(':') {
                let parts: Vec<&str> = coordinate.split(':').collect();
                if parts.len() == 3 {
                    (
                        parts[0].to_string(),
                        parts[1].to_string(),
                        parts[2].to_string(),
                    )
                } else {
                    anyhow::bail!("{}", "Invalid coordinate format. Expected group:artifact:version or just artifact name".red());
                }
            } else {
                // Search mode
                let client = MavenClient::new();
                println!("{}", format!("🔍 Searching for '{}'...", coordinate).cyan());
                let results = client.search_artifact(coordinate)?;

                if results.is_empty() {
                    anyhow::bail!(
                        "{}",
                        format!("No artifacts found for '{}'", coordinate).red()
                    );
                }

                let selection = if results.len() == 1 {
                    0
                } else {
                    let items: Vec<String> = results
                        .iter()
                        .map(|r| format!("{}:{} ({})", r.g, r.a, r.latest_version))
                        .collect();

                    Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Select an artifact")
                        .default(0)
                        .items(&items)
                        .interact()?
                };

                let selected = &results[selection];
                (
                    selected.g.clone(),
                    selected.a.clone(),
                    selected.latest_version.clone(),
                )
            };

            let mut manifest: Manifest = if Path::new("jpkg.json").exists() {
                let content = fs::read_to_string("jpkg.json")?;
                serde_json::from_str(&content)?
            } else {
                anyhow::bail!("{}", "jpkg.json not found. Run 'jpkg init' first.".red());
            };

            manifest
                .dependencies
                .insert(format!("{}:{}", group, artifact), version.to_string());

            let content = serde_json::to_string_pretty(&manifest)?;
            fs::write("jpkg.json", content)?;
            println!(
                "{}",
                format!("✓ Added {}:{}:{} to jpkg.json", group, artifact, version)
                    .green()
                    .bold()
            );
        }
        Commands::Install { frozen, offline } => {
            if !Path::new("jpkg.json").exists() {
                anyhow::bail!("{}", "jpkg.json not found. Run 'jpkg init' first.".red());
            }
            let content = fs::read_to_string("jpkg.json")?;
            let manifest: Manifest = serde_json::from_str(&content)?;

            let client = MavenClient::new();
            let mut resolver = Resolver::new(&client, &manifest);

            println!("{}", "📦 Resolving dependencies...".cyan());
            let resolved = resolver.resolve()?;

            println!(
                "{}",
                format!("✓ Resolved {} packages:", resolved.len()).green()
            );
            for pkg in &resolved {
                println!("  {} {}", "•".blue(), pkg);
            }

            let lib_dir = Path::new("lib");
            if !lib_dir.exists() {
                fs::create_dir(lib_dir)?;
            }

            // Initialize cache
            cache::init_cache()?;

            // Create/load lock file
            let mut lockfile = if *frozen {
                lockfile::LockFile::load()?
            } else {
                lockfile::LockFile::new()
            };

            let pb = ProgressBar::new(resolved.len() as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("#>-"));

            for pkg in resolved {
                let parts: Vec<&str> = pkg.split(':').collect();
                let group = parts[0];
                let artifact = parts[1];
                let version = parts[2];

                let filename = format!("{}-{}.jar", artifact, version);
                let path = lib_dir.join(filename);

                // Verify checksum if frozen
                if *frozen {
                    if !lockfile.verify_package(&pkg, &path)? {
                        anyhow::bail!("{}", format!("Checksum mismatch for {}", pkg).red());
                    }
                }

                // Try cache first
                if let Some(cached_path) = cache::get_cached_jar(group, artifact, version)? {
                    if !path.exists() {
                        fs::copy(&cached_path, &path)?;
                    }
                } else if *offline {
                    // Offline mode: fail if not in cache
                    anyhow::bail!(
                        "{}",
                        format!("Artifact {} not in cache (offline mode)", pkg).red()
                    );
                } else {
                    // Download if not in cache
                    if !path.exists() {
                        client.download_jar(group, artifact, version, &path)?;
                        // Cache the downloaded JAR
                        cache::cache_jar(group, artifact, version, &path)?;
                    }
                }

                // Add to lock file
                lockfile.add_package(pkg.clone(), version.to_string(), &path, Vec::new())?;

                pb.inc(1);
            }
            pb.finish_with_message(format!("{}", "✓ Done!".green().bold()));

            // Save lock file
            lockfile.save()?;
            println!("{}", "✓ Saved jpkg.lock".green());
        }
        Commands::Build { verbose } => {
            project::build_project(*verbose)?;
        }
        Commands::Run { main, verbose } => {
            // Auto-build if needed
            if !Path::new("bin").exists() || fs::read_dir("bin")?.next().is_none() {
                println!("{}", "⚙️  Building project first...".yellow());
                project::build_project(false)?;
            }
            project::run_project(main.clone(), *verbose)?;
        }
        Commands::Remove { name } => {
            if !Path::new("jpkg.json").exists() {
                anyhow::bail!("{}", "jpkg.json not found".red());
            }
            let content = fs::read_to_string("jpkg.json")?;
            let mut manifest: Manifest = serde_json::from_str(&content)?;

            let mut key_to_remove = None;

            if manifest.dependencies.contains_key(name) {
                key_to_remove = Some(name.to_string());
            } else {
                for (key, _) in &manifest.dependencies {
                    if key.ends_with(&format!(":{}", name)) || key == name {
                        key_to_remove = Some(key.clone());
                        break;
                    }
                }
            }

            if let Some(key) = key_to_remove {
                manifest.dependencies.remove(&key);
                let content = serde_json::to_string_pretty(&manifest)?;
                fs::write("jpkg.json", content)?;
                println!(
                    "{}",
                    format!("✓ Removed {} from jpkg.json", key).green().bold()
                );
            } else {
                anyhow::bail!("{}", format!("Dependency '{}' not found", name).red());
            }
        }
        Commands::Package { output, main } => {
            packager::package_jar(output.clone(), main.clone())?;
        }
        Commands::Test { verbose } => {
            testing::run_tests(*verbose)?;
        }
        Commands::Outdated => {
            updater::check_updates()?;
        }
        Commands::Update { package } => {
            updater::update_dependencies(package.clone())?;
        }
        Commands::Watch { run, main } => {
            watcher::watch_and_build(*run, main.clone())?;
        }
        Commands::Log => match logger::get_last_error() {
            Ok(log) => {
                println!("{}", "📋 Last Error Log:".cyan().bold());
                println!();
                println!("{}", log);
            }
            Err(e) => {
                println!("{}", format!("Failed to read log: {}", e).red());
            }
        },
        Commands::Cache { command } => match command {
            CacheCommands::List => {
                let artifacts = cache::list_cached()?;
                if artifacts.is_empty() {
                    println!("{}", "Cache is empty".yellow());
                } else {
                    println!(
                        "{}",
                        format!("📦 Cached artifacts ({}):", artifacts.len()).cyan()
                    );
                    for artifact in artifacts {
                        println!("  {} {}", "•".blue(), artifact);
                    }
                }
            }
            CacheCommands::Clean => {
                cache::clear_cache()?;
                println!("{}", "✓ Cache cleared".green());
            }
            CacheCommands::Size => {
                let size = cache::cache_size()?;
                let size_mb = size as f64 / 1024.0 / 1024.0;
                println!("{}", format!("💾 Cache size: {:.2} MB", size_mb).cyan());
            }
        },
        Commands::Version => {
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            println!("{}", format!("jpkg v{}", VERSION).cyan().bold());
            println!(
                "{}",
                "A modern Java package manager written in Rust".dimmed()
            );
        }
    }

    Ok(())
}
