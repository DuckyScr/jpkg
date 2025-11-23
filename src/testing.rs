use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn run_tests(verbose: bool) -> Result<()> {
    // Check for test directory
    let test_dir = Path::new("src/test/java");
    if !test_dir.exists() {
        println!("{}", "⚠️  No test directory found (src/test/java)".yellow());
        println!("{}", "  Create tests in src/test/java to run them".dimmed());
        return Ok(());
    }

    // Compile tests
    println!("{}", "🧪 Compiling tests...".cyan());

    let mut test_files = Vec::new();
    visit_dirs(test_dir, &mut test_files)?;

    if test_files.is_empty() {
        println!("{}", "⚠️  No test files found".yellow());
        return Ok(());
    }

    fs::create_dir_all("bin/test")?;

    let mut cmd = Command::new("javac");
    cmd.arg("-d").arg("bin/test");
    cmd.arg("-cp").arg("lib/*:bin"); // Include main classes and libs
    cmd.arg("-sourcepath").arg("src/test/java");

    for file in &test_files {
        cmd.arg(file);
    }

    if verbose {
        println!("{}", "  Test files:".dimmed());
        for file in &test_files {
            println!("    {} {}", "•".blue(), file.display());
        }
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("{}", "Test compilation failed".red());
    }

    println!("{}", "✓ Tests compiled".green());

    // Run tests (basic - just execute test classes)
    println!("{}", "🧪 Running tests...".cyan());

    // Find test classes
    let mut test_count = 0;
    let mut passed = 0;

    for file in test_files {
        if let Some(class_name) = extract_class_name(&file) {
            test_count += 1;

            if verbose {
                println!("{}", format!("  Running {}...", class_name).dimmed());
            }

            let result = Command::new("java")
                .arg("-cp")
                .arg("bin/test:bin:lib/*")
                .arg(&class_name)
                .output()?;

            if result.status.success() {
                passed += 1;
                if verbose {
                    println!("{}", format!("    ✓ {}", class_name).green());
                }
            } else {
                println!("{}", format!("    ✗ {}", class_name).red());
                if verbose && !result.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&result.stderr));
                }
            }
        }
    }

    println!();
    if passed == test_count {
        println!(
            "{}",
            format!("✓ All {} tests passed", test_count).green().bold()
        );
    } else {
        println!(
            "{}",
            format!("✗ {} of {} tests passed", passed, test_count).yellow()
        );
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

fn extract_class_name(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}
