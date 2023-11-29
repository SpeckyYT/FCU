use itertools::Itertools;
use pathdiff::diff_paths;

use crate::util::*;

use std::collections::HashSet;

pub fn save() {
    let mut all_images = HashSet::new();

    read_from_file(&mut all_images);
    read_from_mail_folder(&mut all_images);

    write(&all_images);
}

#[allow(dead_code)]
pub fn push_string(content: &str) {
    let mut all_images = HashSet::new();

    read_from_file(&mut all_images);

    content.lines().map(|line| all_images.insert(line.to_string())).for_each(drop);

    write(&all_images);
}

pub fn read_from_file(all_images: &mut HashSet<String>) {
    std::fs::read_to_string(ALL_IMAGES_FILE)
        .unwrap_or_default()
        .lines()
        .map(|line| all_images.insert(standardify(line)))
        .for_each(drop);
}

pub fn read_from_mail_folder(all_images: &mut HashSet<String>) {
    get_all_images()
    .into_iter()
    .map(|(p, _)| {
        let relative = diff_paths(p, MAIL_FOLDER.clone()).unwrap();
        let segments = relative.to_str().unwrap()
        .split('_')
        .collect::<Vec<&str>>();
        let relative = segments[0..(segments.len()-1)].join("_");
        all_images.insert(standardify(&relative));
    })
    .for_each(drop);
}

pub fn stringify(all_images: &HashSet<String>) -> String {
    let mut all_images = all_images
        .iter()
        .filter_map(|s| {
            let string = standardify(s);
            if string.is_empty() {
                None
            } else {
                Some(string)
            }
        })
        .unique()
        .collect::<Vec<String>>();
    all_images.sort_unstable();

    all_images.join("\n")
}

pub fn write(all_images: &HashSet<String>) {
    std::fs::write(ALL_IMAGES_FILE, stringify(all_images)).unwrap();
}

pub fn standardify(string: &str) -> String {
    string
    .replace('\\', "/")
    .replace(".png", "")
    .trim()
    .to_string()
}
