// Uncomment this block to pass the first stage
use std::{env, ffi::OsString, fs, fs::DirEntry, io::{BufRead, BufReader, Write}, net::TcpListener, path::Path, sync::Arc, thread};
use std::ops::Deref;

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";

const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 Not Found\r\n\r\n";
use std::str::FromStr;

use itertools::Itertools;

#[derive(Debug, PartialEq)]
enum HttpMethod {
    Get,
    Post,
}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(input: &str) -> Result<HttpMethod, Self::Err> {
        match input {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            _ => Err(()),
        }
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let args: Vec<String> = env::args().collect();

    let files_in_server_dir = Arc::new(parse_argline_args(args));

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let files_in_server_dir = files_in_server_dir.clone();
                thread::spawn(move || {
                    handle_connection(_stream, files_in_server_dir);
                });
                // tokio::spawn(async move {
                //     println!("New spawn");
                //     handle_connection(_stream).await;
                // });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn parse_argline_args(arline_args: Vec<String>) -> Vec<DirEntry> {
    let mut server_directory: &Path = Path::new(".");
    let mut files_in_server_dir: Vec<DirEntry> = vec![];
    if arline_args.get(1).unwrap_or(&"empty".to_string()) == &("--directory".to_string()) {
        server_directory = match arline_args.get(2) {
            Some(x) => Path::new(x),
            None => Path::new("."),
        };
        println!("Given dir: {}", server_directory.to_str().unwrap());
    } else {
        println!(
            "No given dir, using default: {}",
            server_directory.to_str().unwrap()
        );
    }
    if server_directory.exists() && server_directory.is_dir() {
        files_in_server_dir = server_directory
            .read_dir()
            .expect("ReadDir Failed")
            .map(|entry| entry.expect("Failed to parse entry"))
            .collect_vec();
    }
    files_in_server_dir
}

fn handle_connection(mut stream: std::net::TcpStream, files_in_dir: Arc<Vec<DirEntry>>) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let user_agent_header: String = http_request
        .iter()
        .filter(|s| s.starts_with("User-Agent:"))
        .map(|s| s.split_whitespace().nth(1).unwrap())
        .collect::<Vec<_>>()
        .first()
        .unwrap_or(&"")
        .to_string();

    let mut parts = http_request[0].split_whitespace();

    let _method: HttpMethod = HttpMethod::from_str(parts.next().unwrap()).unwrap();
    let request_endpoint = parts.next().unwrap();
    let response = handle_request(request_endpoint, user_agent_header, files_in_dir);

    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_request(
    request: &str,
    user_agent_header: String,
    files_in_dir: Arc<Vec<DirEntry>>,
) -> String {
    println!("Header: {}", user_agent_header);
    if request.len() == 1 {
        OK_RESPONSE.to_string()
    } else if request.starts_with("/echo/") {
        make_response_from_string(request.trim_start_matches("/echo/"))
    } else if request.starts_with("/user-agent") {
        make_response_from_string(user_agent_header.as_str())
    } else if request.starts_with("/files/") {
        let filename = OsString::from_str(request.trim_start_matches("/files/"))
            .expect("Couldn't parse filename");
        return_file_request(filename, files_in_dir)
    } else {
        return NOT_FOUND_RESPONSE.to_string();
    }
}

fn make_response_from_string(text_for_response: &str) -> String {
    let base_text = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: ".to_string();
    let content_length = text_for_response.len();
    format!(
        "{} {}\r\n\r\n{}",
        base_text, content_length, text_for_response
    )
}
fn make_content_stream_from_file(file_in_string: String) -> String {
    let base_text = "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: ".to_string();
    let content_length = file_in_string.len();
    format!(
        "{} {}\r\n\r\n{}",
        base_text, content_length, file_in_string
    )
}

fn return_file_request(filename: OsString, files: Arc<Vec<DirEntry>>) -> String {

    if files.iter().map(|x| x.file_name()).contains(&filename) {
        let filename = files.iter().filter(|&f| f.file_name() == filename).collect::<Vec<&DirEntry>>().get(0).unwrap().to_owned();

        make_content_stream_from_file(fs::read_to_string(filename.path()).unwrap())
    } else {
        NOT_FOUND_RESPONSE.to_string()
    }
}
