use std::fmt::Display;
use std::fs::File;
use std::path::{Path, PathBuf};
use sha1::{Digest, Sha1};
use crate::modules::changes::Changes;


#[derive(Debug, Clone)]
pub struct GameFile {
    pub hash: String,
    pub name: String,
}

impl GameFile {
    fn new(hash: String, name: String) -> Self {
        Self {
            hash,
            name
        }
    }
}


pub struct Manifest {
    files: Vec<GameFile>
}

impl Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for file in &self.files {
            writeln!(f, "{:<45} {}", file.hash, file.name)?;
        }
        Ok(())
    }
}

impl Manifest {
    fn new(files: Vec<GameFile>) -> Self {
        Self {
            files
        }
    }
    pub fn parse_manifest(path: &Option<PathBuf>) -> Option<Manifest> {
        let file = match &path {
            Some(file) => {
                file
            },
            None => {
                eprintln!("Please provide a manifest file (see the documentation).");
                return None;
            }
        };

        let manifest = match std::fs::read_to_string(file) {
            Ok(manifest) => manifest,
            Err(error) => {
                eprintln!("Error reading manifest file: {}", error);
                return None;
            }
        };

        if path.as_ref().unwrap().extension().unwrap().to_str().unwrap() == "txt" {
            let files = parse_depot_download_manifest(&manifest);
            files.map(Manifest::new)
        } else if path.as_ref().unwrap().extension().unwrap().to_str().unwrap() == "sha1" {
            let files = parse_manifest_viewer_manifest(&manifest);
            files.map(Manifest::new)
        } else {
            eprintln!("Unsupported manifest file type.");
            None
        }
    }

    pub fn validate_files(&self, directory: &Path, changes: Option<Changes>) -> Result<(), ()> {
        println!("Validating {}", directory.display());
        let mut hasher = Sha1::new();
        let mut bad_files = vec![];
        let mut mismatches = 0;
        let mut missing = 0;
        let mut successes = 0;
        let mut total = 0;

        let game_files = match changes {
            Some(mut changes) => {
                let mut new_files = vec![];
                new_files.append(&mut changes.added);
                new_files.append(&mut changes.modified);
                let files: Vec<GameFile> = self.files.iter()
                    .filter(|&game_file| new_files.contains(&game_file.name.replace('\\', "/")))
                    .cloned()
                    .collect();
                files
            },
            None => {
                self.files.clone()
            }
        };
        for game_file in &game_files {
            // let mut hasher = hasher.clone();
            print!("Validating {}...\t", game_file.name);
            let path = directory.join(&game_file.name);
            if path.is_dir() {
                continue;
            }
            total += 1;
            let mut file = match File::open(path) {
                Ok(file) => file,
                Err(error) => {
                    println!("Error: {}", error);
                    missing += 1;
                    bad_files.push(game_file.name.clone());
                    continue;
                }
            };

            let _ = std::io::copy(&mut file, &mut hasher);
            let hash = format!("{:x}", hasher.finalize_reset());
            if hash == game_file.hash.to_lowercase() {
                println!("Ok.");
                successes += 1;
            } else {
                println!("Hash mismatch.");
                bad_files.push(game_file.name.clone());
                mismatches += 1;
            }
        }

        println!("{} files checked, {} successes, {} mismatches, {} missing.", total, successes, mismatches, missing);
        if !bad_files.is_empty() {
            println!("Bad files:\n  {}", bad_files.join("\n  "));
            Err(())
        } else {
            Ok(())
        }
    }
}

fn parse_depot_download_manifest(manifest: &str) -> Option<Vec<GameFile>> {
    let mut lines = manifest.lines();
    let _ = lines.position(|line| line.contains("Name"));
    let mut game_files = vec![];
    for line in lines {
        let parts = line.split_whitespace().collect::<Vec<&str>>();
        if let Some(hash) = parts.get(2) {
            if let Some(name) = parts.get(4..) {
                let game_file = GameFile::new(hash.to_string(), name.join("").to_string());
                game_files.push(game_file);
            }
        }
    }
    Some(game_files)
}

fn parse_manifest_viewer_manifest(manifest: &str) -> Option<Vec<GameFile>> {
    let mut lines = manifest.lines();
    let _ = lines.position(|line| line.trim() == ";");
    let mut game_files = vec![];
    for line in lines {
        let parts = line.split_whitespace().collect::<Vec<&str>>();
        if let Some(hash) = parts.first() {
            if let Some(name) = parts.get(1..) {
                let game_file = GameFile::new(
                    hash.to_string(),
                    name.join(" ").replace('*', "").to_string());
                game_files.push(game_file);
            }
        }
    }
    Some(game_files)
}
