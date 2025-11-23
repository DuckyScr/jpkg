use anyhow::Result;
use colored::Colorize;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub fn package_jar(output_name: Option<String>, main_class: Option<String>) -> Result<()> {
    let jar_name = output_name.unwrap_or_else(|| "app.jar".to_string());
    let output_path = Path::new("target").join(&jar_name);

    // Create target directory
    fs::create_dir_all("target")?;

    println!("{}", "📦 Creating JAR package...".cyan());

    let file = File::create(&output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Add compiled classes from bin/
    if Path::new("bin").exists() {
        add_directory_to_zip(&mut zip, Path::new("bin"), "bin", options)?;
    } else {
        anyhow::bail!(
            "{}",
            "bin/ directory not found. Run 'jpkg build' first.".red()
        );
    }

    // Add dependencies from lib/
    if Path::new("lib").exists() {
        for entry in fs::read_dir("lib")? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                extract_jar_to_zip(&mut zip, &path, options)?;
            }
        }
    }

    // Create MANIFEST.MF
    let manifest = if let Some(main) = main_class {
        format!(
            "Manifest-Version: 1.0\nMain-Class: {}\nCreated-By: jpkg\n",
            main
        )
    } else {
        "Manifest-Version: 1.0\nMain-Class: Main\nCreated-By: jpkg\n".to_string()
    };

    zip.start_file("META-INF/MANIFEST.MF", options)?;
    zip.write_all(manifest.as_bytes())?;

    zip.finish()?;

    println!(
        "{}",
        format!("✓ Created JAR: {}", output_path.display())
            .green()
            .bold()
    );
    println!(
        "{}",
        format!("  Run with: java -jar {}", output_path.display()).dimmed()
    );

    Ok(())
}

fn add_directory_to_zip(
    zip: &mut ZipWriter<File>,
    dir: &Path,
    prefix: &str,
    options: SimpleFileOptions,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(prefix)?.to_str().unwrap();

        if path.is_file() {
            zip.start_file(name, options)?;
            let bytes = fs::read(&path)?;
            zip.write_all(&bytes)?;
        } else if path.is_dir() {
            add_directory_to_zip(zip, &path, prefix, options)?;
        }
    }
    Ok(())
}

fn extract_jar_to_zip(
    zip: &mut ZipWriter<File>,
    jar_path: &Path,
    options: SimpleFileOptions,
) -> Result<()> {
    let file = File::open(jar_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // Skip META-INF files from dependencies
        if name.starts_with("META-INF/") {
            continue;
        }

        if !file.is_dir() {
            zip.start_file(&name, options)?;
            io::copy(&mut file, zip)?;
        }
    }

    Ok(())
}
