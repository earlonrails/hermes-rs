use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use hermes_core::paths::get_hermes_home;

pub fn run_import() {
    println!("\nHermes Restore & Import Utility");
    println!("═════════════════════════════════\n");

    print!("Enter backup file path to restore [default: ./hermes-backup.zip]: ");
    io::stdout().flush().ok();
    
    let mut backup_str = String::new();
    io::stdin().read_line(&mut backup_str).ok();
    let mut backup_str = backup_str.trim().to_string();
    if backup_str.is_empty() {
        backup_str = "./hermes-backup.zip".to_string();
    }

    let backup_path = PathBuf::from(backup_str);
    if !backup_path.exists() {
        println!("✗ Backup file {} does not exist.", backup_path.display());
        return;
    }

    let home_dir = get_hermes_home();
    print!("Are you sure you want to restore? This will overwrite existing files in {}! [y/N]: ", home_dir.display());
    io::stdout().flush().ok();
    
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).ok();
    if confirm.trim().to_lowercase() != "y" {
        println!("Restore cancelled.");
        return;
    }

    println!("Extracting backup into {}...", home_dir.display());
    
    let file = match File::open(&backup_path) {
        Ok(f) => f,
        Err(e) => {
            println!("✗ Failed to open backup file: {}", e);
            return;
        }
    };

    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => {
            println!("✗ Failed to parse zip archive: {}", e);
            return;
        }
    };

    if !home_dir.exists() {
        let _ = fs::create_dir_all(&home_dir);
    }

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(e) => {
                println!("✗ Error reading file index {}: {}", i, e);
                continue;
            }
        };

        let outpath = match file.enclosed_name() {
            Some(path) => home_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            let _ = fs::create_dir_all(&outpath);
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    let _ = fs::create_dir_all(p);
                }
            }
            
            let mut outfile = match File::create(&outpath) {
                Ok(f) => f,
                Err(e) => {
                    println!("✗ Failed to create file {}: {}", outpath.display(), e);
                    continue;
                }
            };

            if let Err(e) = std::io::copy(&mut file, &mut outfile) {
                println!("✗ Failed to write file {}: {}", outpath.display(), e);
            }
        }
    }

    println!("\n✓ Restore completed successfully!");
}
