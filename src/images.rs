pub const IMAGES: Option<(&str, &str)> =
    if cfg!(debug_assertions) {
        Some((include_str!("../all_images.txt"), include_str!("../all_gay_images.txt")))
    } else {
        None
    };
