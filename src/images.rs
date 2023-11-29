pub const IMAGES: Option<&str> =
    if cfg!(debug_assertions) {
        Some(include_str!("../all_images.txt"))
    } else {
        None
    };
