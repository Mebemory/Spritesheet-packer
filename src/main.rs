use std::{io::Write, num::ParseIntError};

use image::{DynamicImage, GenericImage};

fn main() {
    match try_create_spritesheet() {
        Ok(_) => (),
        Err(error) => {
            match error {
                SpritesheetErr::NoImagesFound => println!("Error: no images found"),
                SpritesheetErr::FilterImages => println!("Error: filter image error"),
                SpritesheetErr::ImageSaveError => println!("Error: save image error"),
                SpritesheetErr::ParseError => println!("Error: parse error"),
            };
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }
}

fn try_create_spritesheet() -> Result<(), SpritesheetErr> {
    let use_auto_row_count = get_settings();
    let path_to_images = find_images_path()?;
    let images = collect_images(path_to_images);
    let images = filter_images(images)?;
    let row_count = if use_auto_row_count {
        calculate_row_count(images.len())
    } else {
        println!("Image count: {}", images.len());
        get_input_row_count()?
    };
    let spritesheet = create_spritesheet(row_count, images);
    save_image(spritesheet)?;
    Ok(())
}

fn get_input_row_count() -> Result<u32, ParseIntError> {
    print!("Enter row count: ");
    _ = std::io::stdout().flush();
    let mut input_string: String = String::from("");
    std::io::stdin().read_line(&mut input_string).unwrap();
    input_string.trim().parse()
}

fn get_settings() -> bool {
    let args = std::env::args().collect::<Vec<String>>();
    args.get(1).map(|value| value == "auto").is_some()
}

fn find_images_path() -> Result<Vec<ImageData>, SpritesheetErr> {
    let mut images: Vec<ImageData> = Vec::new();

    let current_dir = std::env::current_dir().expect("Can't find current dir");
    let files_iter = std::fs::read_dir(current_dir).expect("Can't read dir");

    let files_iter = files_iter
        .filter(|file| file.is_ok())
        .map(|file| file.unwrap())
        .filter(|file| file.metadata().expect("Access to file denied").is_file());

    for file in files_iter {
        if !file.metadata().unwrap().is_file() {
            continue;
        }

        let file_name = file.file_name();
        let extension: Vec<&str> = file_name.to_str().unwrap().split(".").collect();

        if let Some(format) = get_image_format(extension[1]) {
            images.push(ImageData {
                path: file.path(),
                format: format,
            });
        }
    }

    if images.len() > 0 {
        Ok(images)
    } else {
        Err(SpritesheetErr::NoImagesFound)
    }
}

fn get_image_format(str: &str) -> Option<image::ImageFormat> {
    match str {
        "png" => Some(image::ImageFormat::Png),
        "jpeg" => Some(image::ImageFormat::Jpeg),
        "bmp" => Some(image::ImageFormat::Bmp),
        _ => None,
    }
}

fn collect_images(images_data: Vec<ImageData>) -> Vec<image::DynamicImage> {
    let mut images: Vec<image::DynamicImage> = Vec::new();
    for image_info in images_data {
        let file = std::fs::File::open(image_info.path).unwrap();
        let buffer = std::io::BufReader::new(file);
        let image = image::load(buffer, image_info.format).unwrap();
        images.push(image);
    }
    images
}

fn filter_images(images: Vec<DynamicImage>) -> Result<Vec<DynamicImage>, SpritesheetErr> {
    let mut resolution_map: std::collections::HashMap<(u32, u32), u32> =
        std::collections::HashMap::new();
    for image in images.iter() {
        let key = resolution_map
            .entry((image.height(), image.width()))
            .or_default();
        *key += 1;
    }

    let max_popular_value = resolution_map.values().max().unwrap();

    let mut popular_resolution: (u32, u32) = (0, 0);
    for entry in resolution_map.iter() {
        if entry.1 == max_popular_value {
            popular_resolution = entry.0.clone();
            break;
        }
    }

    if popular_resolution == (0, 0) {
        return Err(SpritesheetErr::FilterImages);
    }

    let mut filtered_images = Vec::new();

    for image in images {
        if image.height() == popular_resolution.0 && image.width() == popular_resolution.1 {
            filtered_images.push(image);
        }
    }

    Ok(filtered_images)
}

fn create_spritesheet(row_count: u32, images: Vec<image::DynamicImage>) -> image::DynamicImage {
    let image_res = (images[0].width(), images[0].height());
    let height = (images.len() as f32 / row_count as f32).ceil() as u32;
    let resolution = (row_count * images[0].width(), height * images[0].height());
    let mut spritesheet = image::DynamicImage::new_rgba8(resolution.0, resolution.1);

    let mut image_index = 0;
    for y in 0..height {
        for x in 0..row_count {
            if images.len() - 1 < image_index {
                break;
            }
            spritesheet
                .copy_from(&images[image_index], x * image_res.0, y * image_res.1)
                .unwrap();
            image_index += 1;
        }
    }

    spritesheet
}

fn calculate_row_count(images_count: usize) -> u32 {
    (images_count as f32).sqrt().floor() as u32
}

fn save_image(image: image::DynamicImage) -> Result<(), image::ImageError> {
    let mut path_to_save = std::path::PathBuf::new();
    path_to_save.push(std::env::current_dir().unwrap());
    path_to_save.push("spritesheet.png");

    image.save(path_to_save)
}

enum SpritesheetErr {
    NoImagesFound,
    FilterImages,
    ImageSaveError,
    ParseError,
}

impl From<image::ImageError> for SpritesheetErr {
    fn from(_: image::ImageError) -> Self {
        SpritesheetErr::ImageSaveError
    }
}

impl From<ParseIntError> for SpritesheetErr {
    fn from(_: ParseIntError) -> Self {
        SpritesheetErr::ParseError
    }
}

#[derive(Debug)]
struct ImageData {
    path: std::path::PathBuf,
    format: image::ImageFormat,
}
