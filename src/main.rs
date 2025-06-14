use std::{
    env, fs::{self, File}, io::{self, BufReader, Write}
};

use clap::{Arg, Command};
use colored::*;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct AppMainRequest {
    base_url: String,
    access_token_file: Option<String>,
    requests: Vec<RequestData>,
}

#[derive(Clone, Debug, Deserialize)]
struct RequestData {
    req_tag: String,
    req_title: String,
    req_type: String,
    req_end_point: String,
    req_params: Option<String>,
    req_token_path: Option<String>,
    req_token_type: Option<String>,
    req_token_save: Option<bool>,
    req_body: Option<RequestDataBody>,
}

#[derive(Clone, Debug, Deserialize)]
struct RequestDataBody {
    body_type: String,
    body_file: String,
}

fn expand_tilde(path: &str) -> String {
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

fn read_from_file(file_path: &str) -> io::Result<String> {
    let contents_path_buf = expand_tilde(&file_path);
    let contents = fs::read_to_string(contents_path_buf).unwrap();
    Ok(contents)
}

fn write_to_file(body_data: &Value, token_path: &str, token_structure: &str) -> io::Result<()> {
    let contents_path_buf = expand_tilde(&token_path);
    let token = get_nested_value(&body_data, token_structure);
    let mut file = File::create(contents_path_buf)?;
    file.write_all(token.unwrap_or_default().as_str().unwrap_or_default().as_bytes())?;
    Ok(())
}

fn get_nested_value(json: &Value, path: &str) -> Option<Value> {
    path.split('.')
        .fold(Some(json.clone()), |current_value, key| {
            current_value.and_then(|val| val.get(key).cloned())
        })
}

fn request_body_data(req_body: RequestDataBody) -> Value {
    let contents_path = expand_tilde(&req_body.body_file.clone());
    let body_file_path = contents_path;
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
            Arg::new("tag")
                .short('t')
                .long("tag")
                .value_name("TAG")
                .help("The API FILE TAG")
                .required(true),
        )
        .get_matches();

    let file = matches.get_one::<String>("file");
    let tag = matches.get_one::<String>("tag");

    let client = Client::new();

    if let Some(file_data) = file {

        let file_path_buf = expand_tilde(&file_data);
        let file = File::open(file_path_buf).expect("File should open read only");
        let reader = BufReader::new(file);
        let app_main_request: AppMainRequest =
            serde_json::from_reader(reader).expect("File should be proper JSON");

        let tag_value: Option<&RequestData> = app_main_request
            .requests
            .iter()
            .find(|&item| item.req_tag == *tag.unwrap());

        if let Some(request) = tag_value {
            let url = app_main_request.base_url + &request.req_end_point;
            let access_token_file = &app_main_request.access_token_file.unwrap_or_default();

            let method = &request.req_type;
            let title = &request.req_title;
            let params = request.req_params.clone().unwrap_or_default();
            let token_path = &request.req_token_path;
            let token_type = &request.req_token_type;
            let token_save = &request.req_token_save;
            let access_token = read_from_file(&access_token_file).unwrap();

            println!("");
            println!("{} {}", "TITLE    :".blue().bold(), title.green());
            println!("{} {}", "URL      :".blue().bold(), url.yellow());

            let token_data = format!(
                "{} {}",
                token_type.clone().unwrap_or_default(),
                access_token
            );

            // println!("{} {}", "TOKEN    :".blue().bold(), token_data.yellow());

            // Make the request
            let response = match method.as_str() {
                "GET" => client
                    .get(format!("{}{}", url, params))
                    .header("Authorization", token_data)
                    .header("Content-Type", "application/json")
                    .send(),
                "POST" => match &request.req_body {
                    Some(req_body) => {
                        let body_data = request_body_data(req_body.clone());

                        if req_body.body_type == "FORM_DATA" {
                            client
                                .post(format!("{}{}", url, params))
                                .form(&body_data)
                                .header("Authorization", token_data)
                                .header("Content-Type", "application/json")
                                .send()
                        } else {
                            let pretty_json_string = serde_json::to_string_pretty(&body_data)
                                .expect("Failed to convert to pretty JSON string");
                            client
                                .post(format!("{}{}", url, params))
                                .header("Authorization", token_data)
                                .header("Content-Type", "application/json")
                                .body(pretty_json_string)
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
                                .put(format!("{}{}", url, params))
                                .form(&body_data)
                                .header("Authorization", token_data)
                                .send()
                        } else {
                            let pretty_json_string = serde_json::to_string_pretty(&body_data)
                                .expect("Failed to convert to pretty JSON string");
                            client
                                .put(format!("{}{}", url, params))
                                .header("Authorization", token_data)
                                .header("Content-Type", "application/json")
                                .body(pretty_json_string)
                                .send()
                        }
                    }
                    None => client.post(url).header("Authorization", token_data).send(),
                },
                "DELETE" => client
                    .delete(format!("{}{}", url, params))
                    .header("Authorization", token_data)
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

                    if token_save.unwrap_or_default() {
                        let info = write_to_file(
                            &body,
                            &access_token_file,
                            &token_path.as_deref().unwrap_or_default(),
                        );

                        println!("Info : {:?}", info);
                    }
                    display_colored_json(&body, 0); // Display formatted and colored JSON
                    println!("");
                    println!("");
                }
                Err(err) => {
                    eprintln!("{}", "Error : ".red().bold());
                    eprintln!("{}", err.to_string().red());
                }
            }
        } else {
            println!("Item not found");
        }
    }
}
