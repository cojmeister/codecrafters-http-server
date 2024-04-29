use std::{env, ffi::OsString, fs, fs::DirEntry, io::Write, net::TcpListener, path::Path, sync::Arc, thread};
use std::fs::File;
use std::io::Read;
use std::net::TcpStream;
use std::str::FromStr;

use itertools::Itertools;

use http_request::{HttpMethod, HttpRequest};
use http_response::HttpResponse;

use crate::http_response::ContentType;

mod http_request;
mod http_response;


fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let args: Vec<String> = env::args().collect();

    let given_dir = Arc::new(parse_argline_args(args));

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                let given_dir = given_dir.clone();
                thread::spawn(move || {
                    handle_connection(_stream, given_dir);
                });
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
    return if server_directory.exists() && server_directory.is_dir() {
        Box::from(server_directory)
    } else {
        println!("Warning, {:?} isn't a directory, or doesn't exist", server_directory);
        Box::from(Path::new("."))
    }
}

fn handle_connection(mut stream: TcpStream, given_dir: Arc<Box<Path>>) {
    let files_in_dir = given_dir
        .read_dir()
        .expect("ReadDir Failed")
        .map(|entry| entry.expect("Failed to parse entry"))
        .collect_vec();
    let mut buffer = [0; 4096]; // Buffer size might affect server in the future.
    // Read data from the client stream into the buffer
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);
    let (_, http_request) = HttpRequest::parse_request(request.as_ref()).unwrap();

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

        let file_buffer: Vec<u8> = http_request.body.as_str().as_bytes().to_vec();

        file.write_all(&*file_buffer).expect("Error writing file");

        HttpResponse::new(201, "Created".to_string(), ContentType::None, "".to_string())
            .to_string()
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
        let content = request.path.trim_start_matches("/echo/").to_string();
        HttpResponse::new(200, "OK".to_string(), ContentType::TextPlain, content).to_string()
    } else if request.path.starts_with("/user-agent") {
        let content = request.headers["User-Agent"].clone();
        HttpResponse::new(200, "OK".to_string(), ContentType::TextPlain, content).to_string()
    } else if request.path.starts_with("/files/") {
        let filename = OsString::from_str(request.path.trim_start_matches("/files/"))
            .expect("Couldn't parse filename");
        return_file_request(filename, files_in_dir)
    } else {
        return HttpResponse::make_404().to_string();
    }
}


fn return_file_request(filename: OsString, files: Vec<DirEntry>) -> String {
    if files.iter().map(|x| x.file_name()).contains(&filename) {
        let filename = files.iter().filter(|&f| f.file_name() == filename).collect::<Vec<&DirEntry>>().get(0).unwrap().to_owned();
        let file_string = fs::read_to_string(filename.path()).unwrap();
        HttpResponse::new(
            200,
            "OK".to_string(),
            ContentType::ApplicationOctetStream,
            file_string).to_string()
    } else {
        HttpResponse::make_404().to_string()
    }
}
