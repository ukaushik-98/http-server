use core::str;
use std::{io::Write, vec};

use flate2::{write::GzEncoder, Compression};
use itertools::Itertools;
use tokio::{
    fs::{self, read_to_string},
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};

pub async fn concurrent(directory: String) {
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        // let dir_clone = "/tmp/";
        let dir_clone = directory.clone();
        tokio::spawn(async move {
            let mut buf = Vec::with_capacity(1024);
            let mut buf_reader = BufReader::new(&mut stream);
            buf_reader.read_buf(&mut buf).await.unwrap();

            let incoming_request: Vec<&str> = str::from_utf8(&buf).unwrap().split("\r\n").collect();
            println!("INCOMING REQUEST: {:?}", incoming_request);
            let path: Vec<&str> = incoming_request[0].split_ascii_whitespace().collect();
            let request_accepted_encoding: &str = match incoming_request.iter().find(|x| x.starts_with("Accept-Encoding: ")) {
                Some(val) => val,
                None => "",
            };

            let encoding = if request_accepted_encoding.len() > 0 {
                let vec_of_encodings: Vec<&str> = request_accepted_encoding.split(": ").collect_vec();
                println!("REQ ENCODINGS: {:?}", request_accepted_encoding); 
                let encodings: Vec<&str> = vec_of_encodings[1].split(", ").collect();
                println!("VEC OF ENCODINGS: {:?}", encodings);
                let some_encoding = encodings.iter().find_position(|x| *x == &"gzip");
                match some_encoding {
                    Some(val) => val.1,
                    None => "",
                }
            } else {
                ""
            };

            match path[0] {
                "GET" => {
                    if path[1] == "/" && path[1].len() == 1 {
                        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await;
                    } else if path[1].len() > 6 && path[1][..6].to_string() == "/echo/" {
                        let echo_val = format!("{}\r\n", &path[1][6..]);
                        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                        encoder.write_all(echo_val.as_bytes());
                        let encoded_val = encoder.finish().unwrap();
                        let res_body = if encoding == "gzip" {
                            format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: {}\r\nContent-Length: {}\r\n\r\n", encoding, echo_val.len())
                        } else {
                            format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n", echo_val.len())
                        };
                        println!("RES BODY: {:?}", res_body);
                        stream.write_all(&res_body.into_bytes()).await;
                        stream.write_all(&encoded_val).await;
                    } else if path[1].len() == 11 && path[1][..11].to_string() == "/user-agent" {
                        let header_vec: Vec<&str> = incoming_request[2].split(" ").collect();
                        println!("HEADER VEC: {:?}", header_vec);
                        let header_val = header_vec[1];
                        let res_body = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n", header_val.len(), header_val);
                        match stream.write_all(res_body.as_bytes()).await {
                            Ok(_) => println!("SUCCESFULLY ECHOED: {}", header_val),
                            Err(_) => println!("FAILED TO WRITE RESPONSE!"),
                        }
                    } else if path[1].starts_with(&"/files/") {
                        let file_name = dir_clone.to_string() + &path[1][7..];
                        println!("FILE_NAME: {}", file_name);
                        let content = read_to_string(file_name).await;
                        let res_body = match content {
                            Ok(c) => {
                                format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}\r\n", c.len(), c)
                            }
                            Err(_) => {
                                format!("HTTP/1.1 404 Not Found\r\n\r\n")
                            }
                        };

                        match stream.write_all(res_body.as_bytes()).await {
                            Ok(_) => println!("SUCCESFULLY WROTE FILE"),
                            Err(_) => println!("FAILED TO WRITE RESPONSE!"),
                        }
                    } else {
                        stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await;
                    }
                }
                "POST" => {
                    if path[1].starts_with(&"/files/") {
                        let file_name = dir_clone.to_string() + &path[1][7..];
                        println!("FILE_NAME: {}", file_name);
                        let content = fs::write(file_name, incoming_request[5]).await;
                        let res_body = match content {
                            Ok(c) => {
                                format!("HTTP/1.1 201 Created\r\n\r\n")
                            }
                            Err(_) => {
                                format!("HTTP/1.1 500 Server Error\r\n\r\n")
                            }
                        };

                        match stream.write_all(res_body.as_bytes()).await {
                            Ok(_) => println!("SUCCESFULLY WROTE FILE"),
                            Err(_) => println!("FAILED TO WRITE RESPONSE!"),
                        }
                    } else {
                        stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await;
                    }
                }
                _ => {
                    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await;
                }
            }
        });
    }
}
