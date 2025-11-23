use crate::manifest::Manifest;
use crate::platform;
use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn init_project(name: &str) -> Result<()> {
    if Path::new("jpkg.json").exists() {
        anyhow::bail!("{}", "Project already initialized (jpkg.json exists)".red());
    }

    // 1. Create jpkg.json
    let manifest = Manifest::new(name, "0.1.0");
    let content = serde_json::to_string_pretty(&manifest)?;
    fs::write("jpkg.json", content)?;

    // 2. Create directory structure
    fs::create_dir_all("src/main/java")?;
    fs::create_dir_all("lib")?;
    fs::create_dir_all("bin")?;

    // 3. Create Main.java if not exists
    let main_java = Path::new("src/main/java/Main.java");
    if !main_java.exists() {
        let java_content = r#"public class Main {
    public static void main(String[] args) {
        System.out.println("Hello from jpkg!");
    }
}
"#;
        fs::write(main_java, java_content)?;
    }

    // 4. Create .gitignore
    let gitignore = Path::new(".gitignore");
    if !gitignore.exists() {
        let gitignore_content = r#"/lib/
/bin/
/target/
.vscode/
.idea/
*.iml
"#;
        fs::write(gitignore, gitignore_content)?;
    }

    // 5. VS Code settings
    fs::create_dir_all(".vscode")?;
    let settings = r#"{
    "java.project.referencedLibraries": [
        "lib/**/*.jar"
    ],
    "java.project.sourcePaths": [
        "src/main/java"
    ],
    "java.project.outputPath": "bin"
}
"#;
    fs::write(".vscode/settings.json", settings)?;

    Ok(())
}

pub fn build_project(verbose: bool) -> Result<()> {
    if !Path::new("src/main/java").exists() {
        anyhow::bail!(
            "{}",
            "src/main/java not found. Run 'jpkg init' first.".red()
        );
    }
    fs::create_dir_all("bin")?;

    let mut java_files = Vec::new();
    visit_dirs(Path::new("src/main/java"), &mut java_files)?;

    if java_files.is_empty() {
        println!("{}", "⚠️  No Java source files found.".yellow());
        return Ok(());
    }

    if verbose {
        println!("{}", "🔨 Compiling Java files...".cyan());
        for file in &java_files {
            println!("  {} {}", "•".blue(), file.display());
        }
    }

    let mut cmd = Command::new("javac");
    cmd.arg("-d").arg("bin");
    let classpath = platform::build_classpath(&["lib/*"]);
    cmd.arg("-cp").arg(&classpath);
    cmd.arg("-sourcepath").arg("src/main/java");

    for file in java_files {
        cmd.arg(file);
    }

    if verbose {
        let output = cmd.output().context("Failed to run javac")?;
        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("{}", stderr);
            if !output.status.success() {
                let _ = crate::logger::log_error(&stderr);
            }
        }
        if output.status.success() {
            println!("{}", "✓ Build successful.".green().bold());
            Ok(())
        } else {
            anyhow::bail!("{}", "Build failed".red());
        }
    } else {
        let output = cmd.output().context("Failed to run javac")?;

        if output.status.success() {
            println!("{}", "✓ Build successful.".green().bold());
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Show first error line
            let lines: Vec<&str> = stderr.lines().collect();
            if let Some(first_error) = lines.iter().find(|l| l.contains("error:")) {
                eprintln!("{}", first_error.red());
            }

            eprintln!();
            eprintln!(
                "{}",
                "💡 Run with -v for full error output or use 'jpkg log' to see details".yellow()
            );

            // Log the full error
            let _ = crate::logger::log_error(&stderr);

            anyhow::bail!("{}", "Build failed".red());
        }
    }
}

pub fn run_project(main_class: Option<String>, verbose: bool) -> Result<()> {
    let main = main_class.unwrap_or_else(|| "Main".to_string());

    if verbose {
        println!("{}", format!("🚀 Running {}...", main).cyan());
        println!("{}", format!("   Classpath: bin:lib/*").dimmed());
    }

    let mut cmd = Command::new("java");
    let classpath = platform::build_classpath(&["bin", "lib/*"]);
    cmd.arg("-cp").arg(&classpath).arg(&main);

    let output = cmd.output().context("Failed to run java")?;

    // Always show stdout
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }

    // Handle errors
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        if verbose {
            // Show full stack trace
            eprintln!("{}", stderr);
        } else {
            // Show only error message, suggest -v
            let lines: Vec<&str> = stderr.lines().collect();
            if let Some(first_line) = lines.first() {
                eprintln!("{}", first_line.red());
            }
            eprintln!();
            eprintln!(
                "{}",
                "💡 Run with -v for full stack trace or use 'jpkg log' to see details".yellow()
            );
        }

        // Log the full error
        let _ = crate::logger::log_error(&stderr);

        anyhow::bail!("{}", "Program exited with error".red());
    }

    Ok(())
}

fn visit_dirs(dir: &Path, cb: &mut Vec<std::path::PathBuf>) -> Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else if let Some(ext) = path.extension() {
                if ext == "java" {
                    cb.push(path);
                }
            }
        }
    }
    Ok(())
}
