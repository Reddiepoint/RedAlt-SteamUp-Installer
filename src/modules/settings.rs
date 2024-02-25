use std::env::current_dir;
use std::fmt::Display;
use std::fs::{create_dir, read_dir};
use std::io::ErrorKind;
use std::path::PathBuf;
use crate::get_input;
use crate::modules::changes::Changes;
use crate::modules::manifest::Manifest;

pub struct Settings {
    pub changes_file: Option<PathBuf>,
    game_directory: Option<PathBuf>,
    update_directory: Option<PathBuf>,
    backup_directory: Option<PathBuf>,
    manifest_file: Option<PathBuf>,
    validate_update: bool,
    validate_game: bool,
    create_backup: bool,
    copy_files: bool,
    remove_files: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            changes_file: {
                let mut files = read_dir(current_dir().unwrap()).unwrap();
                files
                    .find(|file| file.as_ref().unwrap().file_name().to_str().unwrap().contains("changes.json"))
                    .map(|file| file.unwrap().path())
            },
            game_directory: {
                let current_directory = current_dir().unwrap();
                let parent = current_directory.parent().unwrap();
                if let Some(grandparent) = parent.parent() {
                    Some(grandparent.to_path_buf())
                } else {
                    Some(parent.to_path_buf())
                }
            },
            update_directory: Some(current_dir().unwrap().parent().unwrap().to_path_buf()),
            backup_directory: None,
            manifest_file: {
                let mut files = read_dir(current_dir().unwrap()).unwrap();
                files
                    .find(|file| {
                        let file_name = file.as_ref().unwrap().file_name();
                        let file_name = file_name.to_str().unwrap();
                        file_name.contains("manifest") || file_name.contains("sha1")
                    })
                    .map(|file| file.unwrap().path())
            },
            validate_update: true,
            validate_game: true,
            create_backup: true,
            copy_files: true,
            remove_files: true,
        }
    }
}

impl Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let spacing = 45;
        writeln!(f, "{:spacing$} {}", "Using changes file (changes_file):", match &self.changes_file {
            Some(path) => path.to_str().unwrap(),
            None => "None",
        })?;
        writeln!(f, "{:spacing$} {}", "Using game directory (game_directory):", match &self.game_directory {
            Some(path) => path.to_str().unwrap(),
            None => "None",
        })?;
        writeln!(f, "{:spacing$} {}", "Using update directory (update_directory):", match &self.update_directory {
            Some(path) => path.to_str().unwrap(),
            None => "None",
        })?;
        writeln!(f, "{:spacing$} {}", "Using manifest file (manifest_file):", match &self.manifest_file {
            Some(path) => path.to_str().unwrap(),
            None => "None",
        })?;
        writeln!(f, "{:spacing$} {}", "Validate update files (validate_update):", match &self.manifest_file {
            Some(_) => self.validate_update.to_string(),
            None => "Disabled (requires manifest file)".to_string(),
        })?;
        writeln!(f, "{:spacing$} {}", "Validate game files (validate_game):", match &self.manifest_file {
            Some(_) => self.validate_game.to_string(),
            None => "Disabled (requires manifest file)".to_string(),
        })?;
        writeln!(f, "{:spacing$} {}", "Create backup (create_backup):", self.create_backup)?;
        writeln!(f, "{:spacing$} {}", "Copy files (copy_files):", self.copy_files)?;
        write!(f, "{:spacing$} {}", "Remove files (remove_files):", self.remove_files)?;
        Ok(())
    }
}

impl Settings {
    pub fn modify_fields(&mut self, input: String) {
        let input = input.split(' ').collect::<Vec<&str>>();
        let field = match input.get(1) {
            Some(field) => field.to_owned(),
            None => {
                eprintln!("Enter a field.");
                return;
            }
        };
        // Rest of input
        // let value = input[2..].join(" ").replace('"', "").trim().to_string();
        let value = match input.get(2) {
            Some(_) => { input[2..].join(" ").replace('"', "").trim().to_string() }
            None => {
                eprintln!("Enter a value.");
                return;
            }
        };
        let parse_bool = |value: &str| {
            let value = value.to_lowercase();
            if ["true", "t", "1"].contains(&value.as_str()) {
                Some(true)
            } else if ["false", "f", "0"].contains(&value.as_str()) {
                Some(false)
            } else {
                None
            }
        };
        match field {
            "changes_file" => self.changes_file = {
                let path = PathBuf::from(value);
                if path.is_file() {
                    Some(path)
                } else {
                    None
                }
            },
            "game_directory" => self.game_directory = {
                let path = PathBuf::from(value);
                if path.is_dir() {
                    Some(path)
                } else {
                    None
                }
            },
            "update_directory" => self.update_directory = {
                let path = PathBuf::from(value);
                if path.is_dir() {
                    Some(path)
                } else {
                    None
                }
            },
            "manifest_file" => self.manifest_file = {
                let path = PathBuf::from(value);
                if path.is_file() {
                    Some(path)
                } else {
                    None
                }
            },
            "validate_update_files" => match parse_bool(&value) {
                Some(value) => { self.validate_update = value },
                None => { eprintln!("Invalid value") }
            },
            "validate_game_files" => match parse_bool(&value) {
                Some(value) => { self.validate_game = value },
                None => { eprintln!("Invalid value") }
            }
            "create_backup" => match parse_bool(&value) {
                Some(value) => { self.create_backup = value },
                None => { eprintln!("Invalid value") }
            },
            "copy_files" => match parse_bool(&value) {
                Some(value) => { self.copy_files = value },
                None => { eprintln!("Invalid value") }
            },
            "remove_files" => match parse_bool(&value) {
                Some(value) => { self.remove_files = value },
                None => { eprintln!("Invalid value") }
            },
            _ => eprintln!("Field not found"),
        }

        println!("{}", self);
    }

    pub fn update_game(&mut self) {
        if self.game_directory.is_none() {
            eprintln!("Provide a game directory.");
            return;
        };

        let mut changes = match Changes::parse_changes(&self.changes_file) {
            Some(changes) => changes,
            None => return
        };

        println!("Updating {} with files in {} from {}.\n",
                 self.game_directory.as_ref().unwrap().file_name().unwrap().to_str().unwrap(),
                 self.update_directory.as_ref().unwrap().file_name().unwrap().to_str().unwrap(),
                 self.changes_file.as_ref().unwrap().file_name().unwrap().to_str().unwrap());
        println!("{}", self);

        let input = get_input("Continue? [y/N]: ");
        match input.to_lowercase().as_str() {
            "y" | "yes" => {
                if self.validate_update && self.manifest_file.is_some() {
                    let manifest = Manifest::parse_manifest(&self.manifest_file);
                    let manifest = match manifest {
                        Some(manifest) => manifest,
                        None => return,
                    };
                    let validation = manifest.validate_files(self.update_directory.as_ref().unwrap(), Some(changes.clone()));
                    if validation.is_err() {
                        let input = get_input("Continue? [y/N]: ");
                        match input.to_lowercase().as_str() {
                            "y" | "yes" => {},
                            _ => {
                                println!("Cancelled update.");
                                return;
                            }
                        }
                    }
                }

                if self.create_backup {
                    let backup_directory = self.game_directory.as_ref().unwrap().join(".Backup");
                    self.backup_directory = Some(backup_directory.into());
                    if let Err(error) = create_dir(self.backup_directory.as_ref().unwrap()) {
                        if error.kind() != ErrorKind::AlreadyExists {
                            eprintln!("Error creating backups directory: {}", error);
                            return;
                        }
                    };
                }
                if self.copy_files {
                    self.copy_files(&mut changes);
                }
                if self.remove_files {
                    self.remove_files(&mut changes);
                }

                if self.validate_game && self.manifest_file.is_some() {
                    let manifest = Manifest::parse_manifest(&self.manifest_file);
                    let manifest = match manifest {
                        Some(manifest) => manifest,
                        None => return,
                    };
                    let validation = manifest.validate_files(self.game_directory.as_ref().unwrap(), None);
                }
                println!("Finished updating. Type \"exit\" to close the program.");
            },
            _ => {
                println!("Cancelled update.");
            }
        };
    }

    fn copy_files(&self, changes: &mut Changes) {
        let mut new_files = vec![];
        new_files.append(&mut changes.added);
        new_files.append(&mut changes.modified);
        for path in &new_files {
            if path.contains(".RedAlt-Steam-Installer") {
                continue;
            }

            println!("Copying {} to {}", path, self.game_directory.as_ref().unwrap().file_name().unwrap().to_str().unwrap());
            let new_file = self.update_directory.as_ref().unwrap().join(path);
            let old_file = self.game_directory.as_ref().unwrap().join(path);
            if self.create_backup {
                let backup_file = self.backup_directory.as_ref().unwrap().join(path);
                let _ = std::fs::create_dir_all(backup_file.parent().unwrap());
                if let Err(error) = std::fs::copy(&old_file, backup_file) {
                    if error.kind() == ErrorKind::PermissionDenied {
                        eprintln!("Error copying to backup folder: {}", error);
                        return;
                    }
                }
            }

            let _ = std::fs::create_dir_all(old_file.parent().unwrap());
            if let Err(error) = std::fs::copy(&new_file, &old_file) {
                if error.kind() == ErrorKind::PermissionDenied {
                    eprintln!("Error copying to game folder: {}", error);
                    return;
                }
            }
        }
    }

    fn remove_files(&self, changes: &mut Changes) {
        for path in &changes.removed {
            println!("Removing {} from {}", path, self.game_directory.as_ref().unwrap().file_name().unwrap().to_str().unwrap());
            let old_file = self.game_directory.as_ref().unwrap().join(path);
            if self.create_backup {
                let backup_file = self.backup_directory.as_ref().unwrap().join(path);
                let _ = std::fs::create_dir_all(backup_file.parent().unwrap());
                if let Err(error) = std::fs::copy(&old_file, backup_file) {
                    if error.kind() == ErrorKind::PermissionDenied {
                        eprintln!("Error copying to backup folder: {}", error);
                        return;
                    }
                }
            }
            if let Err(error) = std::fs::remove_file(&old_file) {
                if error.kind() == ErrorKind::PermissionDenied {
                    eprintln!("Error removing file: {}", error);
                    return;
                }
            }
        }
    }

    pub fn show_changes(&self) {
        let mut changes = match Changes::parse_changes(&self.changes_file) {
            Some(changes) => changes,
            None => return
        };

        let spacing = 20;
        println!("Changes for {} ({}):", changes.name, changes.app);
        println!("{:spacing$} {}+", "Initial Build:", changes.initial_build);
        println!("{:spacing$} {}", "Final Build:", changes.final_build);
        println!("{:spacing$} {}", "Depot:", changes.depot);
        println!("{:spacing$} {}", "Manifest:", changes.manifest);
        let display_vec = |vec: &Vec<String>| {
            vec.iter().map(|value| format!("  {}", value)).collect::<Vec<String>>().join("\n")
        };

        if !changes.added.is_empty() {
            println!("Added:\n{}", display_vec(&changes.added));
        }
        if !changes.removed.is_empty() {
            println!("Removed:\n{}", display_vec(&changes.removed));
        }
        if !changes.modified.is_empty() {
            println!("Modified:\n{}", display_vec(&changes.modified));
        }
    }

    pub fn validate(&self, input: String) {
        let input = input.split(' ').collect::<Vec<&str>>();
        let directory = match input.get(1) {
            Some(directory) => directory.to_owned().trim(),
            None => {
                eprintln!("Enter the directory you want to validate.");
                return;
            }
        };

        let manifest = Manifest::parse_manifest(&self.manifest_file);
        let manifest = match manifest {
            Some(manifest) => manifest,
            None => {
                return;
            }
        };

        if directory == "update" {
            let _ = manifest.validate_files(self.update_directory.as_ref().unwrap(), Changes::parse_changes(&self.changes_file));
        } else if directory == "game" {
            let _ = manifest.validate_files(self.game_directory.as_ref().unwrap(), None);
        } else {
            eprintln!("Enter \"update\" or \"game\" to validate the files in that directory.");
        }
    }
}
