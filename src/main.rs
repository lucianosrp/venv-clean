use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use num_format::Locale;
use num_format::ToFormattedString;
use venv_clean::VenvDir;
use venv_clean::{find_venv_dirs, VenvCollection};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Target directory
    dir: PathBuf,

    /// Run on interactive mode
    #[arg(short, long)]
    interactive: bool,

    /// Only do the scan (No files will be deleted)
    #[arg(short, long)]
    scan_only: bool,

    /// Dry run (No files will be deleted)
    #[arg(short, long)]
    dry: bool,

    /// Paths to ignore
    #[arg(long,num_args(1..))]
    ignore_paths: Option<Vec<String>>,
}

fn remove_dir_all_cond(venv_dir: VenvDir, size_deleted: &mut u64, cond: bool) {
    if cond {
        *size_deleted += &venv_dir.get_dir_size().unwrap_or_default();
        let _ = fs::remove_dir_all(venv_dir.path);
    }
}

fn main() -> io::Result<()> {
    let cli = Args::parse();
    let root_dir = cli.dir;
    let mut venv_dirs = VenvCollection::new();
    let delete_cond: bool = !cli.scan_only || !cli.dry;

    #[cfg(target_os = "linux")]
    let mut reserved_directories: Vec<&str> = vec![];

    #[cfg(target_os = "macos")]
    let mut reserved_directories: Vec<&str> = vec![];

    #[cfg(target_os = "windows")]
    let mut reserved_directories = vec!["My Music", "My Pictures", "My Videos"];

    let binding = cli.ignore_paths.unwrap_or(vec![]);
    let ignore_paths = binding.iter().map(|v| v.as_str());
    reserved_directories.extend(ignore_paths);

    if let Err(e) = find_venv_dirs(&root_dir, &mut venv_dirs, &reserved_directories) {
        eprintln!("Error searching for .venv directories");
        return Err(e);
    }

    println!(
        "Checked {} files.",
        venv_dirs.checked_files.to_formatted_string(&Locale::en)
    );
    println!(
        "Found {} .venv files.",
        venv_dirs.len().to_formatted_string(&Locale::en)
    );

    println!(
        "Total Size: {} MB",
        (venv_dirs.get_total_size() / 1024 / 1024).to_formatted_string(&Locale::en)
    );

    let mut size_deleted: u64 = 0;

    for venv_dir in venv_dirs.data {
        let venv_dir_path = &venv_dir.path;
        if cli.interactive {
            let mut user_option = String::new();

            print!("Do you wish to remove {:?} ? (y/N):", venv_dir_path);

            let _ = io::stdout().flush();
            io::stdin()
                .read_line(&mut user_option)
                .expect("Failed to read input");
            print!("");

            if user_option.trim().to_lowercase() == "y" {
                remove_dir_all_cond(venv_dir, &mut size_deleted, delete_cond)
            }
        } else {
            remove_dir_all_cond(venv_dir, &mut size_deleted, delete_cond)
        }
    }
    println!(
        "{} MB Reclaimed",
        (size_deleted / 1024 / 1024).to_formatted_string(&Locale::en)
    );
    Ok(())
}
