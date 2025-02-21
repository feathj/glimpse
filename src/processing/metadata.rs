use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;
use little_exif::u8conversion::*;

use serde::{Deserialize, Serialize};
use std::{error::Error, vec};

#[derive(Debug, Serialize, Deserialize)]
pub struct PhotoMeta {
    pub people: Vec<String>,
    pub description: String,
    pub description_embedding: Vec<f64>,
    pub tags: Vec<String>,
}

pub fn get_metadata(file: &str) -> Result<PhotoMeta, Box<dyn Error>> {
    let path = std::path::Path::new(file);
    let metadata = Metadata::new_from_path(path)?;

    let description_tag = metadata.get_tag(&ExifTag::ImageDescription(String::new())).next();
    if description_tag.is_some() {
        let description_buffer = &description_tag.unwrap().value_as_u8_vec(&metadata.get_endian());
        let description = String::from_u8_vec(
            &description_buffer,
            &metadata.get_endian()
        );
        return Ok(serde_json::from_str(&description)?); 
    }

    Ok(PhotoMeta {
        people: vec![],
        description: "".to_string(),
        description_embedding: vec![],
        tags: vec![],
    })
}

// Return a tuple of the metadata and the file path
pub fn get_metadata_list(files: &Vec<String>) -> Result<Vec<(String, PhotoMeta)>, Box<dyn Error>> {
    let mut metadata_list = vec![];
    for file in files {
        match get_metadata(&file) {
            Ok(metadata) => metadata_list.push((file.clone(), metadata)),
            // TODO: Verbosity
            Err(e) => println!("Failed to get metadata for {}: {:?}", file, e),
        }
    }
    Ok(metadata_list)
}

pub async fn write_metadata(file: &str, photo_metadata: PhotoMeta) -> Result<(), Box<dyn Error>> {
    let path = std::path::Path::new(file);
    let mut metadata = Metadata::new_from_path(path)?;

    metadata.set_tag(
        ExifTag::ImageDescription(serde_json::to_string(&photo_metadata)?)
    );
    metadata.write_to_file(path)?;

    Ok(())
}

// Implement print
impl std::fmt::Display for PhotoMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "People: {:?}, Description: {}, Tags: {:?}", self.people, self.description, self.tags)
    }
}