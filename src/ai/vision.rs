use aws_config::meta::region::RegionProviderChain;

use aws_sdk_rekognition::config::Region;
use aws_sdk_rekognition::types::Image;
use aws_sdk_rekognition::primitives::Blob;
use aws_config::BehaviorVersion;
use std::error::Error;

use crate::graphics::images;

async fn rek_client() -> aws_sdk_rekognition::Client {
    let rek_region = std::env::var("AWS_REGION").ok();

    let rek_region_provider = RegionProviderChain::first_try(rek_region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    let rek_shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region(rek_region_provider)
        .load()
        .await;
    aws_sdk_rekognition::Client::new(&rek_shared_config)
}

pub async fn compare_faces(reference_file: &str, target_file: &str) -> Result<f32, Box<dyn Error>> {
    let rek_client = rek_client().await;

    let tmp_reference_file = images::resize_temp_image(reference_file, 1000)?; // TODO: make a more scientific decision on the resizes
    let source_image = Image::builder()
        .bytes(Blob::new(
            std::fs::read(&tmp_reference_file)?
        ))
        .build();
    images::clear_temp_file(&tmp_reference_file)?;

    let tmp_target_file = images::resize_temp_image(target_file, 1000)?;
    let target_image = Image::builder()
        .bytes(Blob::new(
            std::fs::read(&tmp_target_file)?
        ))
        .build();
    images::clear_temp_file(&tmp_target_file)?;


    let resp = rek_client.compare_faces()
        .source_image(source_image)
        .target_image(target_image)
        .send()
        .await?;

    // Grab first match if available
    if resp.face_matches.as_ref().map(|v| v.len()).unwrap_or(0) > 0 {
        let face_match = resp.face_matches.as_ref().unwrap().first().unwrap();
        return Ok(face_match.similarity.unwrap());
    }

    // No face match
    Ok(0.0)
}