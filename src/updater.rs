use crate::maven::MavenClient;
use anyhow::Result;
use colored::Colorize;

pub fn check_updates() -> Result<()> {
    use crate::manifest::Manifest;
    use std::fs;
    use std::path::Path;

    if !Path::new("jpkg.json").exists() {
        anyhow::bail!("{}", "jpkg.json not found".red());
    }

    let content = fs::read_to_string("jpkg.json")?;
    let manifest: Manifest = serde_json::from_str(&content)?;

    if manifest.dependencies.is_empty() {
        println!("{}", "No dependencies to check".dimmed());
        return Ok(());
    }

    println!("{}", "🔍 Checking for updates...".cyan());

    let client = MavenClient::new();
    let mut updates_available = false;

    for (key, current_version) in &manifest.dependencies {
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() != 2 {
            continue;
        }

        let artifact = parts[1];

        // Search for latest version
        match client.search_artifact(artifact) {
            Ok(results) => {
                if let Some(result) = results.iter().find(|r| r.a == artifact && r.g == parts[0]) {
                    if &result.latest_version != current_version {
                        updates_available = true;
                        println!(
                            "  {} {} {} → {}",
                            "↑".yellow(),
                            key,
                            current_version.dimmed(),
                            result.latest_version.green()
                        );
                    } else {
                        println!("  {} {} {}", "✓".green(), key, current_version.dimmed());
                    }
                } else {
                    println!("  {} {} (not found)", "?".yellow(), key);
                }
            }
            Err(_) => {
                println!("  {} {} (search failed)", "✗".red(), key);
            }
        }
    }

    if !updates_available {
        println!();
        println!("{}", "✓ All dependencies are up to date".green().bold());
    } else {
        println!();
        println!("{}", "Run 'jpkg update' to update dependencies".dimmed());
    }

    Ok(())
}

pub fn update_dependencies(package: Option<String>) -> Result<()> {
    use crate::manifest::Manifest;
    use std::fs;
    use std::path::Path;

    if !Path::new("jpkg.json").exists() {
        anyhow::bail!("{}", "jpkg.json not found".red());
    }

    let content = fs::read_to_string("jpkg.json")?;
    let mut manifest: Manifest = serde_json::from_str(&content)?;

    let client = MavenClient::new();
    let mut updated = false;

    if let Some(pkg) = package {
        // Update specific package
        if let Some(current_version) = manifest.dependencies.get(&pkg) {
            let parts: Vec<&str> = pkg.split(':').collect();
            if parts.len() == 2 {
                match client.search_artifact(parts[1]) {
                    Ok(results) => {
                        if let Some(result) =
                            results.iter().find(|r| r.a == parts[1] && r.g == parts[0])
                        {
                            if &result.latest_version != current_version {
                                manifest
                                    .dependencies
                                    .insert(pkg.clone(), result.latest_version.clone());
                                println!(
                                    "{}",
                                    format!("✓ Updated {} to {}", pkg, result.latest_version)
                                        .green()
                                );
                                updated = true;
                            } else {
                                println!("{}", format!("{} is already up to date", pkg).dimmed());
                            }
                        }
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to check updates for {}: {}", pkg, e);
                    }
                }
            }
        } else {
            anyhow::bail!("{}", format!("Package '{}' not found", pkg).red());
        }
    } else {
        // Update all packages
        println!("{}", "🔄 Updating all dependencies...".cyan());

        let deps: Vec<_> = manifest.dependencies.keys().cloned().collect();

        for key in deps {
            let current_version = manifest.dependencies.get(&key).unwrap().clone();
            let parts: Vec<&str> = key.split(':').collect();

            if parts.len() != 2 {
                continue;
            }

            match client.search_artifact(parts[1]) {
                Ok(results) => {
                    if let Some(result) =
                        results.iter().find(|r| r.a == parts[1] && r.g == parts[0])
                    {
                        if result.latest_version != current_version {
                            manifest
                                .dependencies
                                .insert(key.clone(), result.latest_version.clone());
                            println!(
                                "  {} {} {} → {}",
                                "↑".yellow(),
                                key,
                                current_version.dimmed(),
                                result.latest_version.green()
                            );
                            updated = true;
                        }
                    }
                }
                Err(_) => {
                    println!("  {} {} (update failed)", "✗".red(), key);
                }
            }
        }
    }

    if updated {
        let content = serde_json::to_string_pretty(&manifest)?;
        fs::write("jpkg.json", content)?;
        println!();
        println!("{}", "✓ Dependencies updated in jpkg.json".green().bold());
        println!(
            "{}",
            "Run 'jpkg install' to download updated packages".dimmed()
        );
    } else {
        println!("{}", "No updates available".dimmed());
    }

    Ok(())
}
