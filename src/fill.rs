use futures::{stream, StreamExt};

use crate::util::*;

pub async fn fill(force_delay: bool, gay: bool) {
    let results = get_all_images(gay);

    let (
        missing_full_images,
        missing_thumb_images,
    ) = missing_images(&results, gay);

    let missing_static_images = missing_static_images(gay);

    let missing_all_images_file = missing_all_images_file(gay);

    lines_splitter();

    println!("total images: {}", results.len());
    println!("missing thumb images: {}", missing_thumb_images.len());
    println!("missing full images: {}", missing_full_images.len());
    if let Some(missing_static_images) = &missing_static_images {
        println!("missing static images: {}", missing_static_images.len());
    }
    println!("missing all images file: {}", missing_all_images_file.len());

    lines_splitter();

    stream::iter(missing_full_images.into_iter().chain(missing_thumb_images))
    .map(|(missing_image,_)| async {
        let image_url = relative_to_link(&missing_image, gay);

        let new_image_path = mail_folder(gay).join(missing_image);
        if exists(&new_image_path) { return }

        let bytes = download_image(image_url).await;

        if let Some(bytes) = bytes {
            write_file(new_image_path, bytes);
        }

        force_sleep(force_delay);
    })
    .buffer_unordered(CONCURRENT_REQUESTS)
    .collect::<Vec<_>>()
    .await;

    if let Some(missing_static_images) = &missing_static_images {
        for (relative, image_path) in missing_static_images {
            if exists(image_path) { continue };
            let image_url = relative_to_link(relative, gay);
            if let Some(bytes) = download_image(image_url).await {
                write_file(image_path, bytes);
            }

            force_sleep(force_delay);
        }
    }

    for relative in missing_all_images_file {
        let image_path = mail_folder(gay).join(&relative);
        if exists(&image_path) { continue };
        let image_url = relative_to_link(&relative, gay);
        if let Some(bytes) = download_image(image_url).await {
            write_file(image_path, bytes);
        }

        force_sleep(force_delay);
    }

    crate::save::save(gay);
}