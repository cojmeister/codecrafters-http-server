// Uncomment this block to pass the first stage
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    thread,
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

// #[tokio::main]
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                thread::spawn(move || {
                    handle_connection(_stream);
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

fn handle_connection(mut stream: std::net::TcpStream) {
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
        .unwrap()
        .to_string();

    let mut parts = http_request[0].split_whitespace();

    let _method: HttpMethod = HttpMethod::from_str(parts.next().unwrap()).unwrap();
    let request_endpoint = parts.next().unwrap();
    let response = handle_request(request_endpoint, user_agent_header);

    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_request(request: &str, user_agent_header: String) -> String {
    println!("Header: {}", user_agent_header);
    if request.len() == 1 {
        OK_RESPONSE.to_string()
    } else if request.starts_with("/echo/") {
        make_response_from_string(request.trim_start_matches("/echo/"))
    } else if request.starts_with("/user-agent") {
        make_response_from_string(user_agent_header.as_str())
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
