use std::{
    fs::{self, File},
    io::{self, BufReader, Read, Write},
};

use clap::{Arg, Command};
use colored::*;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct AppMainRequest {
    base_url: String,
    access_token_file: Option<String>,
    requests: Vec<RequestData>,
}

#[derive(Clone, Debug, Deserialize)]
struct RequestData {
    req_title: String,
    req_type: String,
    req_end_point: String,
    req_token_path: Option<String>,
    req_body: Option<RequestDataBody>,
}

#[derive(Clone, Debug, Deserialize)]
struct RequestDataBody {
    body_type: String,
    body_file: String,
}

fn read_from_file(file_path: &str) -> io::Result<String> {
    let contents = fs::read_to_string(file_path).unwrap();
    println!("file_path     : {}", file_path);
    println!("contents      : {}", contents);
    Ok(contents)
}

fn write_to_file(body_data: &Value, token_path: &str, token_structure: &str) -> io::Result<()> {
    let token = get_nested_value(&body_data, token_structure);
    let mut file = File::create(token_path)?; // Open the file in write mode (it will overwrite the file)
    file.write_all(
        token
            .unwrap_or_default()
            .as_str()
            .unwrap_or_default()
            .as_bytes(),
    )?;
    Ok(())
}

fn get_nested_value(json: &Value, path: &str) -> Option<Value> {
    path.split('.')
        .fold(Some(json.clone()), |current_value, key| {
            current_value.and_then(|val| val.get(key).cloned())
        })
}

fn request_body_data(req_body: RequestDataBody) -> Value {
    let body_file_path = req_body.body_file.clone();
    let body_file = File::open(body_file_path).expect("Failed to read file");
    let body_reader = BufReader::new(body_file);
    let body_data: Value =
        serde_json::from_reader(body_reader).expect("File should be proper JSON");
    println!("{}", "Request  :".blue().bold());
    println!("");
    display_colored_json(&body_data.clone(), 0); // Display formatted and colored JSON
    println!("");
    body_data
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
    // let upper_block = Block::default()
    //     .borders(Borders::NONE)
    //     .title(Title::from("My Title"));
    // let lower_block = Block::default()
    //     .borders(Borders::ALL)
    //     .title(Title::from("Second Title"));

    // let size = f.size();
    // let chunks = Layout::default()
    //     .direction(Direction::Vertical)
    //     .constraints([Constraint::Length(3), Constraint::Min(0)])
    //     .split(size);

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
        .arg(
            Arg::new("index")
                .short('i')
                .long("index")
                .value_name("INDEX")
                .help("The API FILE INDEX")
                .required(true),
        )
        .get_matches();

    let file = matches.get_one::<String>("file");
    let index_str = matches.get_one::<String>("index");

    if let Some(index_str) = index_str {
        match index_str.parse::<i8>() {
            Ok(number) => {
                let client = Client::new();

                if let Some(file_data) = file {
                    let file = File::open(file_data).expect("File should open read only");
                    let reader = BufReader::new(file);
                    let app_main_request: AppMainRequest =
                        serde_json::from_reader(reader).expect("File should be proper JSON");

                    let request = app_main_request.requests.get(number as usize).unwrap();
                    let url = app_main_request.base_url + &request.req_end_point;
                    let access_token_file = &app_main_request.access_token_file.unwrap_or_default();

                    let method = &request.req_type;
                    let title = &request.req_title;
                    let token_path = &request.req_token_path;
                    let access_token = read_from_file(&access_token_file).unwrap_or_default();

                    println!("");
                    println!("{} {}", "TITLE    :".blue().bold(), title.green());
                    println!("{} {}", "URL      :".blue().bold(), url.yellow());

                    // Make the request
                    let response = match method.as_str() {
                        "GET" => client.get(url).header("Authorization", access_token).send(),
                        "POST" => match &request.req_body {
                            Some(req_body) => {
                                let body_data = request_body_data(req_body.clone());
                                if req_body.body_type == "FORM_DATA" {
                                    client
                                        .post(url)
                                        .form(&body_data)
                                        .header("Authorization", access_token)
                                        .send()
                                } else {
                                    let pretty_json_string =
                                        serde_json::to_string_pretty(&body_data)
                                            .expect("Failed to convert to pretty JSON string");
                                    client
                                        .post(url)
                                        .body(pretty_json_string)
                                        .header("Authorization", access_token)
                                        .send()
                                }
                            }
                            None => client.post(url).send(),
                        },
                        "PUT" => match &request.req_body {
                            Some(req_body) => {
                                let body_data = request_body_data(req_body.clone());
                                if req_body.body_type == "FORM_DATA" {
                                    client
                                        .put(url)
                                        .form(&body_data)
                                        .header("Authorization", access_token)
                                        .send()
                                } else {
                                    let pretty_json_string =
                                        serde_json::to_string_pretty(&body_data)
                                            .expect("Failed to convert to pretty JSON string");
                                    client
                                        .put(url)
                                        .body(pretty_json_string)
                                        .header("Authorization", access_token)
                                        .send()
                                }
                            }
                            None => client
                                .post(url)
                                .header("Authorization", access_token)
                                .send(),
                        },
                        "DELETE" => client
                            .delete(url)
                            .header("Authorization", access_token)
                            .send(),
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
                            println!(
                                "{} {}",
                                "Status   :".blue().bold(),
                                status.to_string().green()
                            );
                            println!("{}", "Response :".blue().bold());
                            println!("");

                            let _ = write_to_file(
                                &body,
                                &access_token_file,
                                &token_path.as_deref().unwrap_or_default(),
                            );
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
            Err(e) => println!("Failed to parse the string: {}", e),
        }
    } else {
        eprintln!("No 'index' provided.");
    }
}
