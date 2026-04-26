use std::path::PathBuf;
use std::fs::{File, rename, read_dir};
use std::io::Write;
use std::path::Path;
use image::{DynamicImage, GenericImageView, ImageResult, GenericImage, ImageOutputFormat,ImageError};
use std::vec::Vec;
use std::error::Error;
use std::fmt;
use std::env;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args: Vec<String> = env::args().collect();

    // Check if there is an optional command line argument
    let path_buf: Option<PathBuf> = if args.len() > 1 {
        // Convert the argument to a PathBuf and wrap it in Some()
        Some(PathBuf::from(&args[1]))
    } else {
        // If there's no argument, return None
        eprintln!("Usage: {} <folder_path> [-pk]", args[0]);
        None
    };

    let process_keyframes = args.iter().any(|arg| arg == "-pk");

    match path_buf {
        Some(path) => {
            let folder_path = path.clone();
            
            if process_keyframes {
                rename_parser(&path)
            }

            create_tile_map(&folder_path)?;
        }
        None => {
           println!("no folder path found");
        }
    }
   
    
    Ok(())
}

fn rename_parser(folder_path: &PathBuf)
{

    let translations = vec![
        (40, "00"),
        (1,  "01"),
        (10, "02"),
        (15, "03"),
        (30, "04"),
        (25, "05"),
        (20, "06"),    
        (45, "07"),
    ];

    let mut translation_map = HashMap::new();

    for (old, new) in translations {
        translation_map.insert(old.to_string(), new.to_string());
    }

    if let Ok(entries) = read_dir(folder_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(extension) = entry.path().extension() {
                    if extension == "png" {
                        if let Some(filename) = entry.path().file_stem() {
                            if let Some(filename_str) = filename.to_str() {
                                let parts: Vec<&str> = filename_str.split('_').collect();
                                if let Ok(number) = parts[0].parse::<u32>() {
                                    if let Some(new_prefix) = translation_map.get(&number.to_string()) {
                                        let mut new_filename = String::from(new_prefix);
                                        for part in parts.iter().skip(1) {
                                            new_filename.push_str("_");
                                            new_filename.push_str(part);
                                        }
                                        new_filename.push_str(".png");
                                        let new_path = entry.path().with_file_name(new_filename);
                                        if let Err(e) = rename(entry.path(), &new_path) {
                                            eprintln!("Failed to rename {:?} to {:?}: {}", entry.path(), new_path, e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TileMapError {
    details: String,
}

impl TileMapError {
    fn new(msg: &str) -> TileMapError {
        TileMapError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for TileMapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for TileMapError {
    fn description(&self) -> &str {
        &self.details
    }
}

fn create_tile_map(folder_path: &PathBuf) -> ImageResult<()> {
    // Load all PNG files in the folder
    let png_files: Vec<_> = std::fs::read_dir(folder_path)?
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                if let Some("png") = entry.path().extension().and_then(|e| e.to_str()) {
                    Some(entry.path())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    if png_files.is_empty() {
        return Err(ImageError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No PNG files found in the folder.",
        )));

    }
    for file in &png_files{
        println!("name:{:?}",file)
    }
    // Get the dimensions of the first image
    let img_path = &png_files[0];
    let img = image::open(img_path)?;
    let (w, h) = img.dimensions();

    // Calculate the number of rows and columns
    let num_images = png_files.len();
    let mut num_cols = (num_images as f64).sqrt().ceil() as u32;
    let mut num_rows = (num_images as f64 / num_cols as f64).ceil() as u32;

    // Adjust the number of rows and columns to make the map more square
    let aspect_ratio = w as f64 / h as f64;
    if aspect_ratio > 1.0 {
        num_cols = (num_cols as f64 * aspect_ratio).ceil() as u32;
    } else {
        num_rows = (num_rows as f64 / aspect_ratio).ceil() as u32;
    }

    // Create the tile map image
    let mut tile_map = DynamicImage::new_rgba8(w * num_cols, h * num_rows);
    for (i, png_file) in png_files.iter().enumerate() {
        let img = image::open(png_file)?;
        let row = i as u32 / num_cols;
        let col = i as u32 % num_cols;
        tile_map.copy_from(&img, w * col, h * row)?;
    }

    // Save the tile map image
    let parent_folder_name = folder_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|f| f.to_str())
        .unwrap_or("unknown_parent");
    let current_folder_name = folder_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("unknown_folder");
    let output_filename = format!(
        "tile_map_{}_{}.png",
        parent_folder_name, current_folder_name
    );
    let output_path = Path::new(folder_path).join(output_filename);
    let mut output_file = File::create(output_path)?;
    tile_map.write_to(&mut output_file, ImageOutputFormat::Png)?;

    Ok(())
}