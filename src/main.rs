use std::{env, ffi::OsString, fs, fs::DirEntry, io::{BufRead, BufReader, Write}, net::TcpListener, path::Path, sync::Arc, thread};
use std::fs::File;
use std::net::TcpStream;
use std::ops::Add;
use std::str::FromStr;

use itertools::Itertools;

use http_request::{HttpMethod, HttpRequest};

use http_response::HttpResponse;

mod http_request;
mod http_response;


const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 Not Found\r\n\r\n";

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let args: Vec<String> = env::args().collect();

    let given_dir = Arc::new(parse_argline_args(args));

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let given_dir = given_dir.clone();
                thread::spawn(move || {
                    handle_connection(_stream, given_dir);
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

fn parse_argline_args(argline_args: Vec<String>) -> Box<Path> {
    let mut server_directory: &Path = Path::new(".");
    if argline_args.get(1).unwrap_or(&"empty".to_string()) == &("--directory".to_string()) {
        server_directory = match argline_args.get(2) {
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
        return Box::from(server_directory)
    } else {
        println!("Warning, {:?} isn't a directory, or doesn't exist", server_directory);
        return Box::from(Path::new("."))
    }
}

fn handle_connection(mut stream: TcpStream, given_dir: Arc<Box<Path>>) {
    let files_in_dir = given_dir
        .read_dir()
        .expect("ReadDir Failed")
        .map(|entry| entry.expect("Failed to parse entry"))
        .collect_vec();
    let buf_reader = BufReader::new(&mut stream);
    let http_request: String = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect::<Vec<String>>().join("\r\n").add("\r\n");
    let (_, http_request) = HttpRequest::parse_request(http_request.as_str()).unwrap();

    let response = match http_request.method {
        HttpMethod::Get => handle_get_request(&http_request, files_in_dir),
        HttpMethod::Post => handle_post_request(&http_request, given_dir)
    };

    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_post_request(http_request: &HttpRequest, given_dir: Arc<Box<Path>>) -> String {
    if http_request.path.starts_with("/files/") {
        let filename = http_request.path.trim_start_matches("/files/");
        let file_path = given_dir.join(Path::new(filename));
        let mut file = File::create(file_path).expect("Unable to create file");

        if http_request.body.len() == 0 {
            panic!("No body in request!")
        }

        let file_buffer: Vec<u8> = http_request.body.as_str().as_bytes().to_vec();

        file.write_all(&*file_buffer).expect("Error writing file");

        "Yikes".to_string()
    } else {
        HttpResponse::make_404().to_string()
    }
}

fn handle_get_request(
    request: &HttpRequest,
    files_in_dir: Vec<DirEntry>,
) -> String {
    if request.path.len() == 1 {
        HttpResponse::make_200().to_string()
    } else if request.path.starts_with("/echo/") {
        make_response_from_string(request.path.trim_start_matches("/echo/"))
    } else if request.path.starts_with("/user-agent") {
        make_response_from_string(request.headers["User-Agent"].as_str())
    } else if request.path.starts_with("/files/") {
        let filename = OsString::from_str(request.path.trim_start_matches("/files/"))
            .expect("Couldn't parse filename");
        return_file_request(filename, files_in_dir)
    } else {
        return HttpResponse::make_404().to_string();
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

fn return_file_request(filename: OsString, files: Vec<DirEntry>) -> String {

    if files.iter().map(|x| x.file_name()).contains(&filename) {
        let filename = files.iter().filter(|&f| f.file_name() == filename).collect::<Vec<&DirEntry>>().get(0).unwrap().to_owned();

        make_content_stream_from_file(fs::read_to_string(filename.path()).unwrap())
    } else {
        HttpResponse::make_404().to_string()
    }
}
