use colored::Colorize;
use serde_json::Value;
use std::env;
use std::fs::{create_dir_all, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

// File mapping structure for directory mode
#[derive(Debug)]
struct FileMapping {
    filename: String,
    json_path: String,
}

/// Expand tilde in paths to home directory
pub fn expand_tilde(path: &str) -> String {
    if let Ok(home) = env::var("HOME") {
        if path == "~" {
            home
        } else if let Some(rest) = path.strip_prefix("~/") {
            let mut home_path = PathBuf::from(home);
            home_path.push(rest);
            home_path.to_string_lossy().into_owned()
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    }
}

/// Write data to file(s) - handles both single file and directory modes
pub fn write_to_file(
    body_data: &Value,
    variable_path: &Option<String>,
    variable_structure: &Value,
) -> io::Result<()> {
    println!("body_data : {:?}", body_data);
    // If no path provided, skip writing
    let path_str = match variable_path {
        Some(path) => path,
        None => return Ok(()),
    };

    // Extract the structure string from the Value
    let structure_str = variable_structure.as_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "variable_structure is not a string",
        )
    })?;

    let full_path = expand_tilde(path_str);
    let path = Path::new(&full_path);

    // Check if path is a directory
    if path.is_dir() {
        // Directory mode: Read all files and extract values by key
        write_multiple_files(body_data, path, structure_str)
    } else {
        // File mode: Write single file (original behavior)
        write_single_file(body_data, path, structure_str)
    }
}

/// Write to a single file
fn write_single_file(body_data: &Value, path: &Path, structure_str: &str) -> io::Result<()> {
    let nested_value = get_nested_value(body_data, structure_str).unwrap();
    let variable_value = nested_value.as_str().unwrap_or_default();

    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    // Write to file
    let mut file = File::create(path)?;
    file.write_all(variable_value.as_bytes())?;
    Ok(())
}

/// Write to multiple files/folders in a directory based on key mappings
fn write_multiple_files(body_data: &Value, dir_path: &Path, structure_str: &str) -> io::Result<()> {
    // 1. Ensure the root directory exists
    create_dir_all(dir_path)?;

    let mappings = parse_structure_mappings(structure_str);

    for mapping in mappings {
        // 2. Resolve the full path for this specific file/mapping
        let file_path = dir_path.join(&mapping.filename);

        // 3. CRITICAL: Ensure the parent directory for this specific file exists
        // This allows mapping.filename to be "subdir/sub-subdir/file.txt"
        if let Some(parent) = file_path.parent() {
            create_dir_all(parent)?;
        }

        // 4. Extract data and write
        let nested_value = get_nested_value(body_data, &mapping.json_path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "JSON path not found"))?;
        
        let variable_value = nested_value.as_str().unwrap_or_default();

        let mut file = File::create(&file_path)?;
        file.write_all(variable_value.as_bytes())?;

        println!(
            "Written to: {} (from path: {})",
            file_path.display().to_string().yellow().bold(),
            mapping.json_path.to_string().yellow()
        );
    }

    Ok(())
}

/// Parse structure string to extract filename -> JSON path mappings
fn parse_structure_mappings(structure_str: &str) -> Vec<FileMapping> {
    let mut mappings = Vec::new();

    // Try JSON parsing first
    if let Ok(value) = serde_json::from_str::<Value>(structure_str) {
        if let Some(obj) = value.as_object() {
            for (filename, json_path_value) in obj {
                if let Some(json_path) = json_path_value.as_str() {
                    mappings.push(FileMapping {
                        filename: filename.clone(),
                        json_path: json_path.to_string(),
                    });
                }
            }
            return mappings;
        }
    }

    // Fallback to key:value format
    for pair in structure_str.split(',') {
        let parts: Vec<&str> = pair.splitn(2, ':').collect();
        if parts.len() == 2 {
            mappings.push(FileMapping {
                filename: parts[0].trim().to_string(),
                json_path: parts[1].trim().to_string(),
            });
        }
    }

    mappings
}

/// Read content from a file
pub fn read_from_file(file_path: &str) -> io::Result<String> {
    let full_path = expand_tilde(file_path);
    let mut file = File::open(full_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// This would typically be imported from another module
fn get_nested_value<'a>(data: &'a Value, path: &'a str) -> Option<&'a Value> {
    let mut current = data;
    for key in path.split('.') {
        current = current.get(key)?;
    }
    Some(current)
}
