use crate::util::*;
use colored::Colorize;
use folder::scan;
use pretty_bytes::converter::convert;

pub fn stats(gay: bool) {
    let mut fulls: u16 = 0;
    let mut fulls_bytes: u128 = 0;
    let mut thumbs: u16 = 0;
    let mut thumbs_bytes: u128 = 0;
    
    scan(
        &mail_folder(gay).clone(),
        |_| true,
        |path, _| Ok(path.exists() && path.metadata().unwrap().is_file()),
        (),
        1,
    ).for_each(|(path, _)| {
        let is_thumb = path.file_name().unwrap().to_str().unwrap().contains("_thumb_");
        if is_thumb {
            thumbs += 1;
            if let Ok(meta) = path.metadata() {
                thumbs_bytes += meta.len() as u128;
            }
        } else {
            fulls += 1;
            if let Ok(meta) = path.metadata() {
                fulls_bytes += meta.len() as u128;
            }
        }
    });

    println!("{}", LINES_SPLITTER.magenta());
    println!("Full images: {}", fulls.to_string().green());
    println!("Full images size: {}", convert(fulls_bytes as f64).green());
    println!("Thumbnail images: {}", thumbs.to_string().green());
    println!("Thumbnail images size: {}", convert(thumbs_bytes as f64).green());
    println!("{}", LINES_SPLITTER.magenta());
}