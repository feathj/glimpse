use std::error::Error;
use std::result::Result;
use glob::glob;

mod ai;
mod metadata;
mod imageproc;

async fn tag_person(reference_file: &str, files: Vec<String>, person_name: &str) -> Result<(), Box<dyn Error>> {
    let total = files.len() + 1;
    let mut count = 0;
    for file in files {
        count += 1;
        println!("{} / {}: {}", count, total, file);
        // check if file exists
        if !std::path::Path::new(&file).exists() {
            println!("File does not exist: {}", file);
            continue;
        }

        match metadata::get_metadata(&file).await {
            Ok(mut metadata) => {
                if metadata.people.contains(&person_name.to_string()) {
                    println!("{} is already tagged in {}", person_name, file);
                } else {
                    match ai::compare_faces(reference_file, &file).await {
                        Ok(similarity) => {
                            if similarity > 0.9 { // TODO: check if this is right threshold?
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

async fn describe(files: Vec<String>) -> Result<(), Box<dyn Error>> {
    for file in files {
        let metadata = metadata::get_metadata(&file).await?;
        println!("{}: {:?}", file, metadata);
    }
    return Ok(());
} 

pub async fn run(args: &super::Args) -> Result<(), Box<dyn Error>> {
    // expand glob pattern in files
    let files = glob(&args.files)?
        .filter_map(Result::ok)  // Handle errors for individual paths
        .filter_map(|path| path.to_str().map(String::from))  // Convert to strings
        .collect();


    if args.action == "tag-person" {
        return tag_person(&args.reference_file, files, &args.person_name).await;
    } else if args.action == "describe" {
        return describe(files).await;
    } else {
        println!("Unknown action: {}", args.action);
    }
    return Ok(());
}