use std::collections::HashMap;

use clap::{Arg, Command};
use colored::*;
use reqwest::blocking::Client;
use serde_json::Value;

fn display_colored_json(value: &Value, indent_level: usize) {
    match value {
        Value::Object(map) => {
            print!("{}", "{".blue());
            let len = map.len();
            let iter = map.iter().enumerate();
            for (i, (key, val)) in iter {
                print!("\n{}", " ".repeat(indent_level + 4));
                print!("\"{}\": ", key.blue()); // Key in blue with quotes
                display_colored_json(val, indent_level + 4);
                if i != len - 1 {
                    print!(",");
                }
            }
            print!("\n{}", " ".repeat(indent_level));
            print!("{}", "}".blue());
        }
        Value::Array(arr) => {
            print!("{}", "[".blue());
            let len = arr.len();
            let iter = arr.iter().enumerate();
            for (i, val) in iter {
                print!("\n{}", " ".repeat(indent_level + 4));
                display_colored_json(val, indent_level + 4);
                if i != len - 1 {
                    print!(",");
                }
            }
            print!("\n{}", " ".repeat(indent_level));
            print!("{}", "]".blue());
        }
        Value::String(s) => print!("\"{}\"", s.yellow()),
        Value::Number(num) => print!("{}", num.to_string().green()),
        Value::Bool(b) => print!("{}", b.to_string().purple()),
        Value::Null => print!("{}", "null".red()),
    }
}

fn main() {
    let matches = Command::new("API CLI")
        .version("1.0")
        .author("Your Name <you@example.com>")
        .about("CLI app for making API requests")
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .value_name("URL")
                .help("The API endpoint URL")
                .required(true),
        )
        .arg(
            Arg::new("method")
                .short('m')
                .long("method")
                .value_name("METHOD")
                .help("HTTP method (GET, POST, etc.)")
                .default_value("GET"),
        )
        .arg(
            Arg::new("data")
                .short('d')
                .long("data")
                .value_name("DATA")
                .help("JSON payload for POST/PUT requests"),
        )
        .get_matches();

    // Extract arguments
    let url = matches.get_one::<String>("url").unwrap();
    let method = matches.get_one::<String>("method").unwrap().to_uppercase();
    let data = matches.get_one::<String>("data");

    // Initialize HTTP client
    let client = Client::new();

    // Make the request
    let response = match method.as_str() {
        "GET" => client.get(url).send(),
        "POST" => {
            // Check if the data is form data or raw data
            if let Some(form_data) = data {
                // If the data looks like form data (contains '&' and '='), treat it as form data
                if form_data.contains('&') && form_data.contains('=') {
                    let mut form = HashMap::new();
                    for pair in form_data.split('&') {
                        let mut parts = pair.splitn(2, '=');
                        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                            form.insert(key.to_string(), value.to_string());
                        }
                    }
                    client.post(url).form(&form).send()
                } else {
                    // Otherwise treat it as raw data (likely JSON or plain text)
                    client.post(url).body(form_data.to_string()).send()
                }
            } else {
                // If no data is provided, just send an empty POST request
                client.post(url).send()
            }
        }
        "PUT" => client
            .put(url)
            .body(data.map_or("", |v| v).to_string())
            .send(),
        "DELETE" => client.delete(url).send(),
        _ => {
            eprintln!("{}", "Unsupported HTTP method".red());
            return;
        }
    };

    // Handle response
    match response {
        Ok(resp) => {
            let status = resp.status();
            let body: Value = resp.json().unwrap_or_else(
                |_| serde_json::json!({"error": "Failed to parse response as JSON"}),
            );
            println!("");
            println!("");
            println!(
                "{} {}",
                "Status   :".blue().bold(),
                status.to_string().green()
            );
            println!("{}", "Response :".blue().bold());
            println!("");
            display_colored_json(&body, 0); // Display formatted and colored JSON
            println!("");
            println!("");
        }
        Err(err) => {
            eprintln!("{}", "Error:".red().bold());
            eprintln!("{}", err.to_string().red());
        }
    }
}
