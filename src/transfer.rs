use crate::util::*;
use std::{fs, path::PathBuf};
use folder::scan;

pub fn transfer(full_folder: Option<PathBuf>, thumb_folder: Option<PathBuf>) {
    let scanner = |thumb: bool, folder: PathBuf| {
        fs::create_dir_all(&folder).unwrap();
        scan(
            &MAIL_FOLDER.clone(),
            |_| true,
            |path, _| Ok(path.exists() && path.metadata().unwrap().is_file()),
            (),
            1,
        )
        .filter(|(path, _)| path.file_name().unwrap().to_str().unwrap().contains("_thumb_") == thumb)
        .for_each(|(path, _)| {
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            let new_path = folder.join(filename);
            if new_path.exists() { return }
            fs::write(new_path, fs::read(path).unwrap()).unwrap();
        })
    };
    if let Some(full_folder) = full_folder {
        scanner(false, full_folder)
    }
    if let Some(thumb_folder) = thumb_folder {
        scanner(true, thumb_folder)
    }
}