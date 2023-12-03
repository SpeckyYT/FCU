pub const IMAGES_PATH: &str =       "Boombox Games/Fap CEO/mail";
pub const IMAGES_PATH_GAY: &str =   "Boombox Games/Fap CEO_ Men Stream/mail";
pub const IMAGES_URL: &str =        "https://cdn-fapceo.nutaku.net//db/art/mail/";
pub const IMAGES_URL_GAY: &str =    "https://cdn-fapceo.nutaku.net/gayceo/db/art/mail/";
pub const ALL_IMAGES_FILE: &str =   "all_images.txt";
pub const ALL_GAY_IMAGES_FILE: &str = "all_gay_images.txt";
pub const THUMB: &str = "_thumb";
pub const LINES_SPLITTER: &str = "====================";
pub const VERSION: u8 = 170;
pub const SLEEP_MIN: f64 = 0.5;
pub const SLEEP_DIFF: f64 = 0.5;
pub const SLEEP_TOO_MANY_REQUESTS: f64 = 180.0;
pub const ENABLE_PROXY: bool = cfg!(debug_assertions);

lazy_static::lazy_static!(
    pub static ref PROXIES_IPS: Vec<(String, String)> = {
        if ENABLE_PROXY {
            const PROXIES: &str = include_str!("../newer_proxies.txt");
            PROXIES.lines().filter_map(|line| {
                line.split_once(':').map(|(a,b)| (a.to_string(), b.to_string()))
            }).collect_vec()
        } else {
            vec![]
        }
    };
);

use std::{path::{PathBuf, Path}, fs, time::Duration};
use colored::Colorize;
use folder::scan;
use itertools::Itertools;
use pathdiff::diff_paths;
use lazy_static::lazy_static;
use reqwest::StatusCode;

type PathPair = Vec<(PathBuf, PathBuf)>;

lazy_static!{
    pub static ref ROAMING: PathBuf = PathBuf::from(std::env::var("APPDATA").unwrap());
    pub static ref LOCAL_LOW: PathBuf = ROAMING.join("../LocalLow");
}

pub fn mail_folder(gay: bool) -> PathBuf {
    LOCAL_LOW.join(if gay { IMAGES_PATH_GAY } else { IMAGES_PATH })
}

pub fn get_all_images(gay: bool) -> Vec<(PathBuf, Result<bool, std::io::Error>)> {
    let mail_folder = mail_folder(gay).clone();
    scan(
        mail_folder.clone(),
        |_| true,
        |path, _| Ok(path.exists()),
        (),
        1,
    )
    .collect()
}

pub fn missing_images(results: &Vec<(PathBuf, Result<bool, std::io::Error>)>, gay: bool) -> (PathPair, PathPair) {
    let mail_folder = mail_folder(gay).clone();

    let mut missing_full_images = vec![];
    let mut missing_thumbs_images = vec![];

    for (path, _) in results {
        let relative = diff_paths(path, &mail_folder).unwrap();
        let file_name = relative.file_name().unwrap().to_str().unwrap().to_string();

        let (pretty, exists) = match file_name.contains(THUMB) {
            true => {
                let full_image = thumb_to_full(&relative);
                let full_image_path = mail_folder.join(&full_image);
                let exists = full_image_path.exists();
                if !exists { missing_full_images.push((full_image.clone(), relative)) };
                (
                    prettify(&full_image),
                    full_image_path.exists(),
                )
            }
            false => {
                let thumb_image = full_to_thumb(&relative);
                let thumb_image_path = mail_folder.join(&thumb_image);
                let exists = thumb_image_path.exists();
                if !exists { missing_thumbs_images.push((thumb_image.clone(), relative)) };
                (
                    prettify(&thumb_image),
                    thumb_image_path.exists(),
                )
            }
        };

        let output = if exists {
            pretty.green()
        } else {
            pretty.red()
        };
        println!("{output}");
    }

    (missing_full_images, missing_thumbs_images)
}

pub fn missing_static_images(gay: bool) -> Option<PathPair> {
    match crate::images::IMAGES {
        Some(static_images) => {
            let mail_folder = mail_folder(gay).clone();

            let mut missing_static_images = vec![];

            for file_relative in static_images.lines() {
                let relative = PathBuf::from(format!("{file_relative}_{VERSION}.png"));
                let path = mail_folder.join(&relative);
                if exists(&path) { continue }
                missing_static_images.push((relative, path));
            }

            Some(missing_static_images)
        }
        None => None,
    }
}

pub fn missing_all_images_file(gay: bool) -> Vec<PathBuf> {
    let mut missing_all_images_file = vec![];

    if let Ok(content) = fs::read_to_string(if gay { ALL_GAY_IMAGES_FILE } else { ALL_IMAGES_FILE }) {
        missing_all_images_file.append(
            &mut content
            .lines()
            .map(|s| PathBuf::from(format!("{s}_{VERSION}.png")))
            .filter(|p| !exists(&mail_folder(gay).clone().join(p)))
            .collect()
        )
    }

    missing_all_images_file
}

pub fn random_proxy() -> reqwest::Proxy {
    let proxy = &PROXIES_IPS[(rand::random::<f64>() * PROXIES_IPS.len() as f64).floor() as usize];
    reqwest::Proxy::https(format!("http://{}:{}", proxy.0, proxy.1)).unwrap()
} 

pub fn download_image(url: &String) -> Option<Vec<u8>> {
    if url.contains("/0.png") { return None }

    loop {
        let client = if ENABLE_PROXY {
            reqwest::blocking::Client::builder().proxy(random_proxy())
        } else {
            reqwest::blocking::Client::builder()
        }
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36")
        .build().unwrap();

        let output = client.get(url).send();

        match output {
            Ok(res) if res.status().is_success() => {
                let bytes = res.bytes().unwrap().into_iter().collect::<Vec<u8>>();
                println!("{} '{}'", "[SUCCESS]".green(), url.red());
                break Some(bytes)
            },
            Ok(res) => {
                let code = match res.status() {
                    StatusCode::TOO_MANY_REQUESTS => {
                        println!("{} '{}'", "[TIMEOUT]".red(), url.red());
                        std::thread::sleep(Duration::from_secs_f64(SLEEP_TOO_MANY_REQUESTS));
                        continue
                    },
                    code => code,
                };
                println!("{} '{}' [{}]", "[FAILED]".red(), url.red(), code.as_u16());
                break None
            },
            Err(err) => {
                println!("{} '{}' [{}]", "[FAILED]".red(), url.red(), err);
                continue
            },
        }
    }
}

pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, data: C) {
    let path = path.as_ref();
    let parent = path.parent().unwrap();
    fs::create_dir_all(parent).unwrap();
    fs::write(path, data).unwrap();
}

pub fn force_sleep(enable: bool) {
    if enable {
        let duration = rand::random::<f64>() * SLEEP_DIFF + SLEEP_MIN;
        let duration = std::time::Duration::from_secs_f64(duration);
        std::thread::sleep(duration);
    }
}

pub fn thumb_to_full(relative: &Path) -> PathBuf {
    let mut full = relative.to_path_buf();
    full.set_file_name(relative.file_name().unwrap().to_str().unwrap().replace(THUMB, ""));
    full
}

pub fn full_to_thumb(relative: &Path) -> PathBuf {
    let mut thumb = relative.to_path_buf();
    let file_name = thumb.file_name().unwrap().to_str().unwrap().to_string();
    let mut segments = file_name.split('_').collect::<Vec<_>>();
    segments.insert(segments.len() - 1, "thumb");
    thumb.set_file_name(segments.join("_"));
    thumb
}

#[allow(dead_code)]
pub struct Relative {
    parent: String,
    stem: String,
    version: String,
    extension: String,
}

pub fn relative_split(relative: &Path) -> Relative {
    let parent = relative.parent().unwrap().to_str().unwrap().to_string();
    let extension = relative.extension().unwrap_or_default().to_str().unwrap().to_string();

    let stem = relative.file_stem().unwrap().to_str().unwrap().to_string();

    let mut segments = stem.split('_').collect::<Vec<&str>>();
    let version = segments.pop().unwrap().to_string();
    let stem = segments.join("_");

    Relative {
        parent,
        stem,
        version,
        extension,
    }
}

pub fn relative_to_link(relative: &Path, gay: bool) -> String {
    let mut relative = relative.to_path_buf();

    let relative_split = relative_split(&relative);

    relative.set_file_name(relative_split.stem);
    relative.set_extension(relative_split.extension);

    format!(
        "{}{}?v={}",
        if gay { IMAGES_URL_GAY } else { IMAGES_URL },
        prettify(&relative),
        relative_split.version,
    )
}

pub fn exists(path: &Path) -> bool {
    let parent = path.parent().unwrap();
    if !parent.exists() { fs::create_dir_all(parent).unwrap() };
    let parent_folder: Vec<String> = fs::read_dir(parent).unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_str().unwrap().to_string())
        .collect();

    let main = relative_split(path);

    parent_folder.into_iter().any(|relative| {
        relative_split(&PathBuf::from(relative)).stem == main.stem
    })
}

pub fn prettify(pathbuf: &Path) -> String {
    pathbuf.to_str().unwrap().replace('\\', "/")
}

pub fn lines_splitter() {
    println!("{}", LINES_SPLITTER.magenta());
}

#[macro_export]
macro_rules! images {
    [ $($main:tt $(/ $subfolder:ident)+)* ] => {
        pub const IMAGES: [&str; 0 $(+ [stringify!($main)].len())*] = [$(
            concat!(
                images!(@ $main),
                $(
                    "/", stringify!($subfolder),
                )+
            ),
        )*];
    };

    (@ amber) => { 0 };
    (# 0) => { "amber" };
    (@ sara) => { 1 };
    (# 1) => { "sara" };
    (@ chloe) => { 2 };
    (# 2) => { "chloe" };
    (@ lisa) => { 3 };
    (# 3) => { "lisa" };
    (@ alexis) => { 4 };
    (# 4) => { "alexis" };
    (@ mrs_rider) => { 5 };
    (# 5) => { "mrs_rider" };
    (@ ayumi) => { 6 };
    (# 6) => { "ayumi" };
    (@ lizzie) => { 7 };
    (# 7) => { "lizzie" };
    (@ amanda) => { 8 };
    (# 8) => { "amanda" };
    (@ nova) => { 9 };
    (# 9) => { "nova" };
    (@ amazonness) => { 10 }; // i forgor 💀
    (# 10) => { "amazonness" };
    (@ laura) => { 11 };
    (# 11) => { "laura" };
    (@ anika) => { 12 };
    (# 12) => { "anika" };
    (@ delliah) => { 13 }; // to check
    (# 13) => { "delliah" };
    (@ dominique) => { 14 }; // to check
    (# 14) => { "dominique" };
    (@ jane) => { 15 }; // to check
    (# 15) => { "jane" };
    (@ charlotte) => { 16 };
    (# 16) => { "charlotte" };
    (@ zoe) => { 17 };
    (# 17) => { "zoe" };
    (@ lana) => { 18 };
    (# 18) => { "lana" };
    (@ german) => { 19 }; // i forgor 💀
    (# 19) => { "german" };
    (@ mermaid) => { 20 }; // i forgor 💀
    (# 20) => { "mermaid" };
    (@ yrelianna) => { 21 }; // to check
    (# 21) => { "yrelianna" };
    (@ miriam) => { 22 }; // to check
    (# 22) => { "miriam" };
    (@ virgina) => { 23 }; // to check
    (# 23) => { "virgina" };
    (@ twins) => { 24 }; // i forgor 💀
    (# 24) => { "twins" };
    (@ holly) => { 25 };
    (# 25) => { "holly" };
    (@ mrs_santa) => { 26 };
    (# 26) => { "mrs_santa" };
    (@ eira) => { 27 }; // to check
    (# 27) => { "eira" };
    (@ dita) => { 28 };
    (# 28) => { "dita" };
    (@ tasha) => { 29 };
    (# 29) => { "tasha" };
    (@ rose) => { 30 };
    (# 30) => { "rose" };
    (@ princescumxxx) => { 31 };
    (# 31) => { "princescumxxx" };
    (@ group_chats) => { 32 };
    (# 32) => { "group_chats" };
    (@ summer_panorama) => { 33 };
    (# 33) => { "summer_panorama" };
    (@ maria) => { 34 };
    (# 34) => { "maria" };
    (@ dalla) => { 35 };
    (# 35) => { "dalla" };
    (@ saya) => { 36 };
    (# 36) => { "saya" };
    (@ winter_panorama) => { 40 };
    (# 40) => { "winter_panorama" };
    (@ credita) => { 41 }; // to check
    (# 41) => { "credita" };
    (@ olivia) => { 42 };
    (# 42) => { "olivia" };
    (@ sunset_panorama) => { 43 };
    (# 43) => { "sunset_panorama" };
    (@ midnight_panorama) => { 44 };
    (# 44) => { "midnight_panorama" };
    (@ solazola) => { 45 };
    (# 45) => { "solazola" };
    (@ roxanne) => { 46 };
    (# 46) => { "roxanne" };
    (@ bed_panorama) => { 47 };
    (# 47) => { "bed_panorama" };
    (@ blogoslavieni) => { 48 }; // to check
    (# 48) => { "blogoslavieni" };
    (@ faye) => { 49 };
    (# 49) => { "faye" };
    (@ dungeon_of_lust) => { 50 };
    (# 50) => { "dungeon_of_lust" };
    (@ mia20) => { 52 };
    (# 52) => { "mia20" };
    (@ christmas_panorama) => { 54 };
    (# 54) => { "christmas_panorama" };
    (@ jade) => { 56 }; // to check
    (# 56) => { "jade" };
    (@ easter_island) => { 57 };
    (# 57) => { "easter_island" };
    (@ star_whores) => { 59 };
    (# 59) => { "star_whores" };
    (@ iredana) => { 62 }; // to check
    (# 62) => { "iredana" };
    (@ super_whores) => { 63 }; // i forgor 💀
    (# 63) => { "super_whores" };
    (@ scarlett) => { 70 };
    (# 70) => { "scarlett" };
    (@ renegade) => { 71 }; // i forgor 💀
    (# 71) => { "renegade" };
    (@ skyler) => { 72 };
    (# 72) => { "skyler" };
    (@ aysha) => { 73 }; // to check
    (# 73) => { "aysha" };
    (@ mia10) => { 74 }; // to check
    (# 74) => { "mia10" };
    (@ spa_panorama) => { 75 }; // to check
    (# 75) => { "spa_panorama" };
    (@ rene) => { 76 };
    (# 76) => { "rene" };
    (@ oneira) => { 78 };
    (# 78) => { "oneira" };
    (@ horny_bunch) => { 80 }; // to check
    (# 80) => { "horny_bunch" };
    (@ hecate) => { 81 };
    (# 81) => { "hecate" };
    (@ retro) => { 83 };
    (# 83) => { "retro" };
    (@ country_girls) => { 84 }; // to check
    (# 84) => { "country_girls" };
    (@ kinky_picnic) => { 85 }; // to check
    (# 85) => { "kinky_picnic" };
    (@ new_year) => { 86 }; // to check
    (# 86) => { "new_year" };
    (@ emma) => { 87 };
    (# 87) => { "emma" };
    (@ lexee) => { 88 };
    (# 88) => { "lexee" };
    (@ dark_phantasies) => { 89 };
    (# 89) => { "dark_phantasies" };
    (@ pippa) => { 91 };
    (# 91) => { "pippa" };
    (@ wizard_of_oz) => { 92 };
    (# 92) => { "wizard_of_oz" };
    (@ desert_shit) => { 93 }; // to check
    (# 93) => { "desert_shit" };
    (@ georgia) => { 94 }; // to check
    (# 94) => { "georgia" };
    (@ archive) => { 1000 };
    (# 1000) => { "archive" };

    (@ $default:literal) => { $default };
}
