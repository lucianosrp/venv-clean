use clap::builder::OsStr;
use fs_extra::dir::get_size;
use std::fs;
use std::io;
use std::path::PathBuf;

pub struct VenvDir {
    pub path: PathBuf,
}
impl VenvDir {
    pub fn get_dir_size(&self) -> Result<u64, fs_extra::error::Error> {
        get_size(&self.path)
    }
}

pub struct VenvCollection {
    pub checked_files: usize,
    pub data: Vec<VenvDir>,
}
impl VenvCollection {
    pub fn new() -> Self {
        Self {
            checked_files: 0,
            data: vec![],
        }
    }
    pub fn get_total_size(&self) -> u64 {
        self.data
            .iter()
            .map(|x| x.get_dir_size().unwrap_or(0))
            .sum()
    }

    pub fn push(&mut self, item: PathBuf) {
        self.data.push(VenvDir { path: item })
    }

    pub fn len(&mut self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&mut self) -> bool {
        self.data.len() == 0
    }
}

impl Default for VenvCollection {
    fn default() -> Self {
        Self::new()
    }
}
pub fn find_venv_dirs(
    dir: &PathBuf,
    venv_dirs: &mut VenvCollection,
    reserved_directories: &Vec<&str>,
) -> io::Result<()> {
    match fs::read_dir(dir) {
        Ok(dir) => {
            for entry in dir {
                venv_dirs.checked_files += 1;

                let entry = entry.unwrap();
                let path = entry.path();
                let filename = &path
                    .file_name()
                    .and_then(|x| x.to_str())
                    .expect("Unable to get filename");

                if !reserved_directories.contains(filename) {
                    if path.is_dir() && path.file_name() == Some(&OsStr::from(".venv")) {
                        venv_dirs.push(path);
                    } else if path.is_dir() {
                        find_venv_dirs(&path, venv_dirs, reserved_directories)?;
                    }
                }
            }
        }
        Err(_) => println!(
            "Error while opening {:?} - Will be skipped.",
            dir.file_name().unwrap()
        ),
    }

    Ok(())
}
