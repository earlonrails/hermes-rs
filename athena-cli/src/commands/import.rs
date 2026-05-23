use std::fs::{self, File};
use std::path::PathBuf;
use athena_core::paths::get_athena_home;
use cliclack::{intro, input, confirm, outro_cancel, spinner};
use anyhow::Result;

pub fn run_import() -> Result<()> {
    intro("Athena Restore & Import Utility")?;

    let backup_str: String = input("Enter backup file path to restore")
        .default_input("./athena-backup.zip")
        .interact()?;
    let backup_str = backup_str.trim().to_string();

    let backup_path = PathBuf::from(backup_str);
    if !backup_path.exists() {
        outro_cancel(format!("Backup file {} does not exist.", backup_path.display()))?;
        return Ok(());
    }

    let home_dir = get_athena_home();
    let confirm_restore: bool = confirm(format!("Are you sure you want to restore? This will overwrite existing files in {}!", home_dir.display()))
        .interact()?;

    if !confirm_restore {
        outro_cancel("Restore cancelled.")?;
        return Ok(());
    }

    let s = spinner();
    s.start(format!("Extracting backup into {}...", home_dir.display()));

    let file = match File::open(&backup_path) {
        Ok(f) => f,
        Err(e) => {
            s.error(format!("Failed to open backup file: {}", e));
            return Ok(());
        }
    };

    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => {
            s.error(format!("Failed to parse zip archive: {}", e));
            return Ok(());
        }
    };

    if !home_dir.exists() {
        let _ = fs::create_dir_all(&home_dir);
    }

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
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
                Err(_) => continue,
            };

            let _ = std::io::copy(&mut file, &mut outfile);
        }
    }

    s.stop("Restore completed successfully!");
    
    Ok(())
}
