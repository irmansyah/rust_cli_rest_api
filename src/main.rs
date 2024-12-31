use std::{collections::HashMap, fs::File, io::BufReader};

use clap::{Arg, Command};
use colored::*;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct AppMainRequest {
    app_title: String,
    base_url: String,
    requests: Vec<RequestData>,
}

#[derive(Clone, Debug, Deserialize)]
struct RequestData {
    req_title: String,
    req_type: String,
    req_end_point: String,
    req_body: Option<RequestDataBody>,
}

#[derive(Clone, Debug, Deserialize)]
struct RequestDataBody {
    body_type: String,
    body_file: String,
}

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
        .author("Irmansyah <irmansyahproject@gmail.com>")
        .about("CLI app for making API requests")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("The API FILE")
                .required(true),
        )
        .get_matches();

    let file = matches.get_one::<String>("file");

    // Initialize HTTP client
    let client = Client::new();

    if let Some(file_data) = file {
        let file = File::open(file_data).expect("File should open read only");
        let reader = BufReader::new(file);
        let app_main_request: AppMainRequest =
            serde_json::from_reader(reader).expect("File should be proper JSON");

        let request = app_main_request.requests.get(1).unwrap();

        let url = app_main_request.base_url + &request.req_end_point;
        println!("url : {}", url.to_string());
        let method = &request.req_type;

        // Make the request
        let response = match method.as_str() {
            "GET" => client.get(url).send(),
            "POST" => match &request.req_body {
                Some(req_body) => {
                    let body_file_path = req_body.body_file.clone();
                    println!("body_file_path : {}", body_file_path);
                    let body_file = File::open(body_file_path).expect("Failed to read file");
                    let body_reader = BufReader::new(body_file);
                    let body_data: HashMap<String, Value> =
                        serde_json::from_reader(body_reader).expect("File should be proper JSON");
                    let pretty_json_string = serde_json::to_string_pretty(&body_data)
                        .expect("Failed to convert to pretty JSON string");
                    println!("request.req_type : {:?}", request.req_type);
                    if req_body.body_type == "FORM_DATA" {
                        client.post(url).form(&body_data).send()
                    } else {
                        client.post(url).body(pretty_json_string).send()
                    }
                }
                None => client.post(url).send(),
            },
            "PUT" => match &request.req_body {
                Some(req_body) => {
                    let body_file_path = req_body.body_file.clone();
                    println!("body_file_path : {}", body_file_path);
                    let body_file = File::open(body_file_path).expect("Failed to read file");
                    let body_reader = BufReader::new(body_file);
                    let body_data: HashMap<String, Value> =
                        serde_json::from_reader(body_reader).expect("File should be proper JSON");
                    let pretty_json_string = serde_json::to_string_pretty(&body_data)
                        .expect("Failed to convert to pretty JSON string");
                    println!("request.req_type : {:?}", request.req_type);
                    if req_body.body_type == "FORM_DATA" {
                        client.put(url).form(&body_data).send()
                    } else {
                        client.put(url).body(pretty_json_string).send()
                    }
                }
                None => client.post(url).send(),
            },
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
}
