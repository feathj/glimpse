use std::error::Error;
use std::result::Result;
use glob::glob;

use crate::processing::metadata;
use crate::ai::vision;
use crate::ai::llm;
use crate::processing::args;

async fn tag_person(reference_file: &str, files: Vec<String>, person_name: &str, confidence: f32) -> Result<(), Box<dyn Error>> {
    let total = files.len();
    let mut count = 0;
    for file in files {
        count += 1;
        println!("{} / {}: {}", count, total, file);
        // check if file exists
        if !std::path::Path::new(&file).exists() {
            println!("File does not exist: {}", file);
            continue;
        }

        match metadata::get_metadata(&file) {
            Ok(mut metadata) => {
                if metadata.people.contains(&person_name.to_string()) {
                    println!("{} is already tagged in {}", person_name, file);
                } else {
                    match vision::compare_faces(reference_file, &file).await {
                        Ok(similarity) => {
                            if similarity >= confidence { // TODO: check if this is right threshold?
                                metadata.people.push(person_name.to_string());
                                if let Err(e) = metadata::write_metadata(&file, metadata).await {
                                    println!("Failed to write metadata for {}: {:?}", file, e);
                                } else {
                                    println!("Tagged {} in {}", person_name, file);
                                }
                            }
                        }
                        Err(e) => println!("Failed to compare faces for {}: {:?}", file, e),
                    }
                }
            }
            Err(e) => println!("Failed to get metadata for {}: {:?}", file, e),
        }
    }
    Ok(())
}

async fn show_metadata(files: Vec<String>) -> Result<(), Box<dyn Error>> {
    for file in files {
        match metadata::get_metadata(&file) {
            Ok(metadata) => println!("{}: {:?}", file, metadata),
            Err(e) => println!("Failed to get metadata for {}: {:?}", file, e),
        }
    }
    Ok(())
}

async fn clear_metadata(files: Vec<String>) -> Result<(), Box<dyn Error>> {
    for file in files {
        match metadata::write_metadata(&file, metadata::PhotoMeta {
            people: vec![],
            description: "".to_string(),
        }).await {
            Ok(_) => println!("Cleared metadata for {}", file),
            Err(e) => println!("Failed to clear metadata for {}: {:?}", file, e),
        }
    }
    Ok(())
}

async fn tag_description(files: Vec<String>) -> Result<(), Box<dyn Error>> {
    let total = files.len();
    let mut count = 0;
    for file in files {
        count += 1;
        println!("{} / {}: {}", count, total, file);
        // Load original metadata
        let mut metadata = metadata::get_metadata(&file)?;

        // get description from AI, including additional context (people in the photo)
        let description = llm::describe_image(&file, &metadata.description_context()).await?;
        metadata.description = description;

        match metadata::write_metadata(&file, metadata).await {
            Ok(_) => println!("Tagged description for {}", file),
            Err(e) => println!("Failed to tag description for {}: {:?}", file, e),
        }
    }
    Ok(())
}

pub async fn run(args: &args::Args) -> Result<(), Box<dyn Error>> {
    // expand glob pattern in files
    let files = glob(&args.files)?
        .filter_map(Result::ok)  // Handle errors for individual paths
        .filter_map(|path| path.to_str().map(String::from))  // Convert to strings
        .collect();


    if args.action == "tag-person" {
        return tag_person(&args.reference_file, files, &args.person_name, args.confidence).await;
    } else if args.action == "tag-description" {
        return tag_description(files).await;
    } else if args.action == "clear-metadata" {
        return clear_metadata(files).await;
    } else if args.action == "show-metadata" {
        return show_metadata(files).await;
    } else {
        println!("Unknown action: {}", args.action);
    }
    return Ok(());
}