use image::{GenericImage, Rgba, RgbaImage};

pub fn resize_and_pad(image: RgbaImage, width: u32, height: u32) -> RgbaImage {
    let (mut current_w, mut current_h) = image.dimensions();

    while current_w < width && current_h < height {
        current_w *= 2;
        current_h *= 2;
    }

    current_w /= 2;
    current_h /= 2;

    let resized = image::imageops::resize(&image, current_w, current_h, image::imageops::FilterType::Lanczos3);

    pad(resized, width, height)
}

pub fn pad(image: RgbaImage, width: u32, height: u32) -> RgbaImage {
    let mut new_image = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0]));

    new_image.copy_from(&image, 0, 0).unwrap();
    new_image
}