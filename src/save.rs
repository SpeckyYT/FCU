use itertools::Itertools;
use pathdiff::diff_paths;

use crate::util::*;

use std::collections::HashSet;

pub fn save(gay: bool) {
    let mut all_images = HashSet::new();

    read_from_file(&mut all_images, gay);
    read_from_mail_folder(&mut all_images, gay);

    write(&all_images, gay);
}

#[allow(dead_code)]
pub fn push_string(content: &str, gay: bool) {
    let mut all_images = HashSet::new();

    read_from_file(&mut all_images, gay);

    content.lines().map(|line| all_images.insert(line.to_string())).for_each(drop);

    write(&all_images, gay);
}

pub fn read_from_file(all_images: &mut HashSet<String>, gay: bool) {
    std::fs::read_to_string(if gay { ALL_GAY_IMAGES_FILE } else { ALL_IMAGES_FILE })
        .unwrap_or_default()
        .lines()
        .map(|line| all_images.insert(standardify(line)))
        .for_each(drop);
}

pub fn read_from_mail_folder(all_images: &mut HashSet<String>, gay: bool) {
    get_all_images(gay)
    .into_iter()
    .map(|(p, _)| {
        let relative = diff_paths(p, mail_folder(gay).clone()).unwrap();
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

pub fn write(all_images: &HashSet<String>, gay: bool) {
    std::fs::write(if gay { ALL_GAY_IMAGES_FILE } else { ALL_IMAGES_FILE }, stringify(all_images)).unwrap();
}

pub fn standardify(string: &str) -> String {
    string
    .replace('\\', "/")
    .replace(".png", "")
    .trim()
    .to_string()
}
