use image::{imageops::FilterType, GenericImageView};
use anyhow::Result;
use std::path::PathBuf;
use std::path::Path;

use aws_sdk_bedrockruntime::{
    types::{ImageBlock, ImageFormat, ImageSource},
    primitives::Blob,
};

pub fn resize_image(input_path: &str, output_path: &str, max_size: u32) -> Result<()> {
    // Open the image
    let img = image::open(&input_path)?;
    
    // Calculate new dimensions while maintaining aspect ratio
    let (width, height) = img.dimensions();
    let ratio = width as f32 / height as f32;
    
    let (new_width, new_height) = if width > height {
        let new_width = max_size;
        let new_height = (max_size as f32 / ratio) as u32;
        (new_width, new_height)
    } else {
        let new_height = max_size;
        let new_width = (max_size as f32 * ratio) as u32;
        (new_width, new_height)
    };
    
    // Resize the image
    let resized = img.resize(new_width, new_height, FilterType::Lanczos3);

    // Save to output path
    resized.save(output_path)?;
    
    Ok(())
}

pub fn resize_temp_image(input_path: &str, max_size: u32) -> Result<String> {
    let path = Path::new(input_path);
    let suffix = path.extension().unwrap().to_str().unwrap();

    let temp_file = tempfile::Builder::new()
        .prefix("gimpse_resized_")
        .suffix(&format!(".{}", suffix))
        .keep(true)
        .tempfile()?;
    let output_path = temp_file.path().to_str().unwrap().to_string();

    resize_image(input_path, &output_path, max_size)?;
    Ok(output_path)
}

pub fn clear_temp_file(file_path: &str) -> Result<()> {
    let path = PathBuf::from(file_path);
    std::fs::remove_file(path)?;
    Ok(())
}

fn path_to_image_format(file_path: &str) -> Result<ImageFormat> {
    let path = Path::new(file_path);
    let extension = path.extension().unwrap().to_str().unwrap();
    let format = match extension {
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "png" => ImageFormat::Png,
        "gif" => ImageFormat::Gif,
        "webp" => ImageFormat::Webp,
        _ => return Err(anyhow::anyhow!("Unsupported image format")),
    };
    Ok(format)
}

pub fn path_to_bedrock_image_block(file_path: &str) -> Result<ImageBlock> {
    let bytes = std::fs::read(file_path)?;
    let image = ImageBlock::builder()
        .format(path_to_image_format(file_path)?)
        .source(ImageSource::Bytes(Blob::new(bytes)))
        .build()?;
    Ok(image)
}