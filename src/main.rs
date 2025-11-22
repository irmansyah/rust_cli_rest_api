use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader},
};

use clap::{Arg, Command};
use colored::*;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use serde_json::Value;

mod file_ops;
use file_ops::{expand_tilde, read_from_file, write_to_file};

#[derive(Debug, Deserialize)]
struct AppMainRequest {
    base_url: String,
    headers: Option<HashMap<String, String>>,
    variable_dir: Option<String>,
    requests: Vec<RequestData>,
}

impl AppMainRequest {
    pub fn create_header_map(&self) -> HeaderMap {
        let mut header_map = HeaderMap::new();

        if let Some(headers) = &self.headers {
            for (key, value) in headers {
                if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
                    if let Ok(header_value) = HeaderValue::from_str(value) {
                        header_map.insert(header_name, header_value);
                    }
                }
            }
        }

        header_map
    }
}

#[derive(Clone, Debug, Deserialize)]
struct RequestData {
    req_tag: String,
    req_title: String,
    req_type: String,
    req_end_point: String,
    req_params: Option<String>,
    req_variable_type: Option<String>,
    req_variable_is_save: Option<bool>,
    req_variable_response_value: Option<Value>,
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

    if let Some(file_data) = file {
        let app_main_request = app_main_request(file_data).unwrap();

        let tag_value: Option<&RequestData> = app_main_request
            .requests
            .iter()
            .find(|&item| item.req_tag == *tag.unwrap());

        if let Some(request) = tag_value {
            let client = reqwest::blocking::Client::new();
            let main_headers = app_main_request.create_header_map(); // Create HeaderMap from the headers
            let main_variable_dir = app_main_request.variable_dir.clone().unwrap_or_default();

            let method = &request.req_type;
            let title = &request.req_title;
            let params = request.req_params.clone().unwrap_or_default();

            // let main_url = app_main_request.base_url.clone() + &request.req_end_point;
            let main_url = format!(
                "{}{}{}",
                app_main_request.base_url.clone(),
                &request.req_end_point,
                params
            );

            let variable_is_save = &request.req_variable_is_save;
            let variable_type = &request.req_variable_type;
            let variable_response_value = &request
                .req_variable_response_value
                .clone()
                .unwrap_or_default();

            // let access_token = read_from_file(&main_variable_dir).unwrap_or_default();

            println!("");
            println!("{} {}", "TITLE    :".blue().bold(), title.green());
            println!("{} {}", "URL      :".blue().bold(), main_url.yellow());

            let access_token = format!(
                "{} {}",
                variable_type.clone().unwrap_or_default(),
                read_from_file(format!("{}/{}", &main_variable_dir, "access_token.txt").as_str())
                    .unwrap_or_default()
            );
            println!("access_token : {:?}", access_token);

            let response = make_http_request(
                &client,
                &method,
                main_url,
                main_headers,
                access_token,
                request.req_body.clone(),
            );

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

                    if variable_is_save.unwrap_or_default() {
                        let variable_dir = &app_main_request.variable_dir;
                        let response_value = Value::String(variable_response_value.to_string());
                        println!("variable_response_value : {:?}", response_value);

                        let _ = write_to_file(&body, &variable_dir, &response_value);
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

    fn app_main_request(file_data: &str) -> Result<AppMainRequest, io::Error> {
        let file_path_buf = expand_tilde(&file_data);
        let file = File::open(file_path_buf).expect("File should open read only");
        let reader = BufReader::new(file);
        let app_main_request: AppMainRequest = serde_json::from_reader(reader)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(app_main_request)
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

    fn handle_get_request(
        client: &reqwest::blocking::Client,
        url: String,
        access_token: String,
        headers: HeaderMap,
    ) -> Result<reqwest::blocking::Response, Box<dyn std::error::Error>> {
        Ok(client.get(url).header("Authorization", access_token).headers(headers).send()?)
    }

    fn handle_post_request(
        client: &reqwest::blocking::Client,
        url: String,
        access_token: String,
        headers: HeaderMap,
        req_body: Option<RequestDataBody>,
    ) -> Result<reqwest::blocking::Response, Box<dyn std::error::Error>> {
        match req_body {
            Some(req_body) => {
                let body_data = request_body_data(req_body.clone());

                if req_body.body_type == "FORM_DATA" {
                    Ok(client.post(url).headers(headers).form(&body_data).send()?)
                } else {
                    let pretty_json_string = serde_json::to_string_pretty(&body_data)?;
                    Ok(client
                        .post(url)
                        .headers(headers)
                        .header("Authorization", access_token)
                        .body(pretty_json_string)
                        .send()?)
                }
            }
            None => Ok(client.post(url).headers(headers).send()?),
        }
    }

    fn handle_put_request(
        client: &reqwest::blocking::Client,
        url: String,
        access_token: String,
        headers: HeaderMap,
        req_body: Option<RequestDataBody>,
    ) -> Result<reqwest::blocking::Response, Box<dyn std::error::Error>> {
        match req_body {
            Some(req_body) => {
                let body_data = request_body_data(req_body.clone());

                if req_body.body_type == "FORM_DATA" {
                    Ok(client.put(url).headers(headers).form(&body_data).send()?)
                } else {
                    let pretty_json_string = serde_json::to_string_pretty(&body_data)?;
                    Ok(client
                        .put(url)
                        .headers(headers)
                        .header("Authorization", access_token)
                        .body(pretty_json_string)
                        .send()?)
                }
            }
            None => Ok(client.put(url).headers(headers).send()?),
        }
    }

    fn make_http_request(
        client: &reqwest::blocking::Client,
        method: &str,
        url: String,
        headers: HeaderMap,
        access_token: String,
        req_body: Option<RequestDataBody>,
    ) -> Result<reqwest::blocking::Response, Box<dyn std::error::Error>> {
        match method {
            "GET" => handle_get_request(client, url, access_token, headers),
            "POST" => handle_post_request(client, url, access_token, headers, req_body),
            "PUT" => handle_put_request(client, url, access_token, headers, req_body),
            _ => Err(format!("Unsupported HTTP method: {}", method).into()),
        }
    }
}
