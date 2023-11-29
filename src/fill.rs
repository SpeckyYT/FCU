use crate::util::*;

pub fn fill(force_delay: bool) {
    let results = get_all_images();

    let (
        missing_full_images,
        missing_thumb_images,
    ) = missing_images(&results);

    let missing_static_images = missing_static_images();

    let missing_all_images_file = missing_all_images_file();

    lines_splitter();

    println!("total images: {}", results.len());
    println!("missing thumb images: {}", missing_thumb_images.len());
    println!("missing full images: {}", missing_full_images.len());
    if let Some(missing_static_images) = &missing_static_images {
        println!("missing static images: {}", missing_static_images.len());
    }
    println!("missing all images file: {}", missing_all_images_file.len());

    lines_splitter();

    for (_, (missing_image, _)) in missing_full_images.into_iter().chain(missing_thumb_images).enumerate() {
        let image_url = relative_to_link(&missing_image);

        let new_image_path = MAIL_FOLDER.join(missing_image);
        if exists(&new_image_path) { continue }

        let bytes = download_image(&image_url);
        if let Some(bytes) = bytes {
            write_file(new_image_path, bytes);
        }

        force_sleep(force_delay);
    }

    if let Some(missing_static_images) = &missing_static_images {
        for (relative, image_path) in missing_static_images {
            if exists(image_path) { continue };
            let image_url = relative_to_link(relative);
            if let Some(bytes) = download_image(&image_url) {
                write_file(image_path, bytes);
            }

            force_sleep(force_delay);
        }
    }

    for relative in missing_all_images_file {
        let image_path = MAIL_FOLDER.join(&relative);
        if exists(&image_path) { continue };
        let image_url = relative_to_link(&relative);
        if let Some(bytes) = download_image(&image_url) {
            write_file(image_path, bytes);
        }

        force_sleep(force_delay);
    }

    crate::save::save();
}