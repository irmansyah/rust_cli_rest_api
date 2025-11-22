use serde_json::Value;
use std::collections::HashMap;
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

/// Write to multiple files in a directory based on key mappings
fn write_multiple_files(body_data: &Value, dir_path: &Path, structure_str: &str) -> io::Result<()> {
    // Ensure the directory exists
    create_dir_all(dir_path)?;

    let mappings = parse_structure_mappings(structure_str);

    for mapping in mappings {
        let file_path = dir_path.join(&mapping.filename);

        let nested_value = get_nested_value(body_data, &mapping.json_path).unwrap();
        let variable_value = nested_value.as_str().unwrap_or_default();

        let mut file = File::create(&file_path)?;
        file.write_all(variable_value.as_bytes())?;

        println!(
            "Written to: {} (from path: {})",
            mapping.filename, mapping.json_path
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

/// Read multiple files from a directory and return as HashMap
pub fn read_files_from_dir(dir_path: &str) -> io::Result<HashMap<String, String>> {
    let full_path = expand_tilde(dir_path);
    let path = Path::new(&full_path);

    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Path is not a directory",
        ));
    }

    let mut files_content = HashMap::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.is_file() {
            if let Some(filename) = file_path.file_name().and_then(|s| s.to_str()) {
                let content = read_from_file(file_path.to_str().unwrap())?;
                files_content.insert(filename.to_string(), content);
            }
        }
    }

    Ok(files_content)
}

/// Write login variables to multiple files (convenience function)
pub fn write_login_variables(body_data: &Value, base_dir: &str) -> io::Result<()> {
    let variables = vec![
        ("access_token.txt", "token.access_token"),
        ("refresh_token.txt", "token.refresh_token"),
        ("email.txt", "user.email"),
        ("user_id.txt", "user.id"),
    ];

    for (filename, json_path) in variables {
        write_to_file(
            body_data,
            &Some(format!("{}/{}", base_dir, filename)),
            &Value::String(json_path.to_string()),
        )?;
    }

    Ok(())
}

/// Helper function to extract nested value from JSON (assuming this exists elsewhere)
/// This would typically be imported from another module
fn get_nested_value<'a>(data: &'a Value, path: &'a str) -> Option<&'a Value> {
    let mut current = data;
    for key in path.split('.') {
        current = current.get(key)?;
    }
    Some(current)
}
