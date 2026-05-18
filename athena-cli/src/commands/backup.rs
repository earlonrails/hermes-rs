use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use athena_core::paths::get_hermes_home;

pub fn run_backup() {
    println!("\nHermes Backup Utility");
    println!("═══════════════════════\n");

    let home_dir = get_hermes_home();
    if !home_dir.exists() {
        println!("No ~/.hermes configuration directory found at {}.", home_dir.display());
        return;
    }

    print!("Enter backup destination file path [default: ./hermes-backup.zip]: ");
    io::stdout().flush().ok();
    
    let mut dest_str = String::new();
    io::stdin().read_line(&mut dest_str).ok();
    let mut dest_str = dest_str.trim().to_string();
    if dest_str.is_empty() {
        dest_str = "./hermes-backup.zip".to_string();
    }

    let dest_path = PathBuf::from(dest_str);
    println!("Creating backup zip at {}...", dest_path.display());

    let file = match File::create(&dest_path) {
        Ok(f) => f,
        Err(e) => {
            println!("✗ Failed to create backup file: {}", e);
            return;
        }
    };

    let mut zip = zip::ZipWriter::new(file);

    println!("Scanning and compressing ~/.hermes files...");
    match add_directory_to_zip(&mut zip, &home_dir, &home_dir) {
        Ok(()) => {
            if let Err(e) = zip.finish() {
                println!("✗ Failed to finalize zip archive: {}", e);
            } else {
                println!("\n✓ Backup created successfully!");
                println!("Saved to: {}", dest_path.display());
            }
        }
        Err(e) => {
            println!("✗ Failed to write files to backup: {}", e);
        }
    }
}

fn add_directory_to_zip<W: Write + io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    base_path: &Path,
    current_dir: &Path,
) -> io::Result<()> {
    if !current_dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let relative_path = path.strip_prefix(base_path).unwrap();
            let relative_str = format!("{}/", relative_path.to_str().unwrap().replace('\\', "/"));
            
            // Skip logs directory entirely
            if relative_str.starts_with("logs/") {
                continue;
            }

            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            let _ = zip.add_directory(&relative_str, options);

            add_directory_to_zip(zip, base_path, &path)?;
        } else {
            let relative_path = path.strip_prefix(base_path).unwrap();
            let relative_str = relative_path.to_str().unwrap().replace('\\', "/");

            let mut f = File::open(&path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;

            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            zip.start_file(relative_str, options)?;
            zip.write_all(&buffer)?;
        }
    }
    Ok(())
}
