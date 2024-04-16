// Uncomment this block to pass the first stage
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";

const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 Not Found\r\n\r\n";
use std::str::FromStr;

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

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");

                handle_connection(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: std::net::TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let mut parts = http_request[0].split_whitespace();

    let _method: HttpMethod = HttpMethod::from_str(parts.next().unwrap()).unwrap();
    let request_endpoint = parts.next().unwrap();
    let response = handle_request(request_endpoint);

    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_request(request: &str) -> String {
    if request.len() == 1 {
        OK_RESPONSE.to_string()
    } else if request.starts_with("/echo/") {
        make_response_from_string(request.trim_start_matches("/echo/"))
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
