use std::collections::HashMap;
use std::str::FromStr;
use itertools::Itertools;

use nom::{bytes::complete, character::complete::char, IResult};
use nom::character::complete::crlf;
use nom::multi::{fold_many0, separated_list1};

#[derive(Debug, PartialEq, Eq)]
pub struct HttpRequest {
    method: HttpMethod,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpRequest {
    fn parse_method(input: &str) -> IResult<&str, HttpMethod> {
        let (i, o) = complete::take_until(" ")(input)?;
        Ok((i, HttpMethod::from_str(o).unwrap()))
    }

    fn parse_path(input: &str) -> IResult<&str, &str> {
        let (remaining, _) = char(' ')(input)?;
        complete::take_till(|b| b == ' ' || b == '\r' || b == '\n')(remaining)
    }

    fn parse_headers(input: &str) -> IResult<&str, HashMap<String, String>> {
        fold_many0(separated_list1(crlf, HttpRequest::parse_header_line), HashMap::new, |mut map, headers| {
            for (key, value) in headers {
                map.insert(key, value);
            }
            map
        })(input)
    }

    fn parse_header_line(input: &str) -> IResult<&str, (String, String)> {
        let (remaining, key) = complete::take_until(":")(input)?;
        let (remaining, _) = char(':')(remaining)?;
        let (remaining, _) = char(' ')(remaining)?;
        let (remaining, value) = complete::take_till(|b| b == '\r' || b == '\n')(remaining)?;
        Ok((remaining, (key.trim().to_string(), value.trim().to_string())))
    }

    fn parse_body(input: &str, body_length: usize) -> IResult<&str, &str> {
        let (remaining, body) = complete::take_till(|b| b != '\r' && b != '\n')(input)?;
        complete::take(body_length)(remaining)
    }

    pub fn parse_request(input: &str) -> IResult<&str, HttpRequest> {
        let (remaining, method) = HttpRequest::parse_method(input)?;
        let (remaining, path) = HttpRequest::parse_path(remaining)?;
        // let (remaining, _) = complete::take_until1("\r\n")(remaining)?;
        let (remaining, _) = complete::take_till(|b| b == '\r' || b == '\n')(remaining)?;
        let (remaining, headers) = HttpRequest::parse_headers(remaining)?;
        // Optionally parse body based on content type and length
        let mut body;
        if headers.contains_key("Content-Length")
            && headers["Content-Length"].parse().unwrap_or(0) > 0 {
            (_, body) = HttpRequest::parse_body(remaining, headers["Content-Length"].parse().unwrap())?;
        } else {
            body = "";
        }

        Ok((remaining, HttpRequest {
            method,
            path: path.to_string(),
            headers: headers.into_iter().sorted().collect(),
            body: body.to_string(),
        }))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum HttpMethod {
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


#[cfg(test)]
mod test_http_method {
    use super::*;

    #[test]
    fn test_http_method_from_string() {
        assert_eq!(HttpMethod::from_str("POST"), Ok(HttpMethod::Post));
        assert_eq!(HttpMethod::from_str("GET"), Ok(HttpMethod::Get));
    }

    #[test]
    fn test_http_method_from_string_err() {
        assert_eq!(HttpMethod::from_str("HiMom"), Err(()));
        assert_eq!(HttpMethod::from_str("post"), Err(()));
        assert_eq!(HttpMethod::from_str("Get"), Err(()));
    }
}

#[cfg(test)]
mod test_http_request {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_parse_method_get() {
        let actual = HttpRequest::parse_method("GET /files/CACHEDIR.TAG HTTP/1.1");
        let expected = Ok((" /files/CACHEDIR.TAG HTTP/1.1", HttpMethod::Get));
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_parse_method_post() {
        let actual = HttpRequest::parse_method("POST /files/CACHEDIR.TAG HTTP/1.1");
        let expected = Ok((" /files/CACHEDIR.TAG HTTP/1.1", HttpMethod::Post));
        assert_eq!(actual, expected)
    }

    #[test]
    #[should_panic]
    fn test_parse_method_err() {
        let input = "ASASA /files/CACHEDIR.TAG HTTP/1.1";
        let actual = HttpRequest::parse_method(input);
    }

    #[test]
    fn test_parse_path() {
        let input = " /files/CACHEDIR.TAG HTTP/1.1";
        let expected = Ok((" HTTP/1.1", "/files/CACHEDIR.TAG"));
        let actual = HttpRequest::parse_path(input);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_parse_path_2() {
        let input = " /echo HTTP/1.1";
        let expected = Ok((" HTTP/1.1", "/echo"));
        let actual = HttpRequest::parse_path(input);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_parse_path_3() {
        let input = " /files/target/debug HTTP/1.1";
        let expected = Ok((" HTTP/1.1", "/files/target/debug"));
        let actual = HttpRequest::parse_path(input);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_parse_headers() {
        let input = "Host: localhost:4221
User-Agent: curl/8.4.0
Accept: */*
";
        let expected_map: HashMap<String, String> = HashMap::from([
            ("Accept".to_string(), "*/*".to_string()),
            ("Host".to_string(), "localhost:4221".to_string()),
            ("User-Agent".to_string(), "curl/8.4.0".to_string()),
        ]).into_iter().sorted().collect();
        ;
        let expected_remainder = "\n";
        let (actual_remainder, actual_map) = HttpRequest::parse_headers(input).unwrap();
        assert_eq!(actual_remainder, expected_remainder);
        assert_eq!(actual_map.into_iter().sorted().collect::<HashMap<String, String>>(), expected_map);
    }

    #[test]
    fn test_parse_header_line() {
        let input = "Host: localhost:4221\r\n";
        let expected = Ok(("\r\n", ("Host".to_string(), "localhost:4221".to_string())));
        let actual = HttpRequest::parse_header_line(input);
        assert_eq!(actual, expected);
        let input = "User-Agent: curl/8.4.0\r\n";
        let expected = Ok(("\r\n", ("User-Agent".to_string(), "curl/8.4.0".to_string())));
        let actual = HttpRequest::parse_header_line(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_request_no_body() {
        let input = "GET /files/CACHEDIR.TAG HTTP/1.1
Host: localhost:4221
User-Agent: curl/8.4.0
Accept: */*
";
        let expected = HttpRequest {
            method: HttpMethod::Get,
            path: "/files/CACHEDIR.TAG".to_string(),
            headers: HashMap::from([
                ("Accept".to_string(), "*/*".to_string()),
                ("Host".to_string(), "localhost:4221".to_string()),
                ("User-Agent".to_string(), "curl/8.4.0".to_string()),
            ]).into_iter().sorted().collect(),
            body: "".to_string(),
        };
        let (_, actual) = HttpRequest::parse_request(input).unwrap();
        assert_eq!(actual, expected)
    }


    #[test]
    fn test_parse_request_with_body() {
        let input = "POST /files/CACHEDIR.TAG HTTP/1.1
Host: localhost:4221
User-Agent: curl/8.4.0
Content-Length: 10
Accept: */*

DonkeyKong
";
        let expected = HttpRequest {
            method: HttpMethod::Post,
            path: "/files/CACHEDIR.TAG".to_string(),
            headers: HashMap::from([
                ("Accept".to_string(), "*/*".to_string()),
                ("Host".to_string(), "localhost:4221".to_string()),
                ("User-Agent".to_string(), "curl/8.4.0".to_string()),
                ("Content-Length".to_string(), "10".to_string()),
            ]).into_iter().sorted().collect(),
            body: "DonkeyKong".to_string(),
        };
        let (_, actual) = HttpRequest::parse_request(input).unwrap();
        assert_eq!(actual, expected)
    }
}