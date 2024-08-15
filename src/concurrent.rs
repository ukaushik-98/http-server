use core::str;

use tokio::{io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader}, net::TcpListener};

pub async fn concurrent() {
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf = Vec::with_capacity(1024);
            let mut buf_reader = BufReader::new(&mut stream);
            buf_reader.read_buf(&mut buf).await.unwrap();
                

            let incoming_request: Vec<&str> = str::from_utf8(&buf).unwrap().split("\r\n").collect();
            println!("INCOMING REQUEST: {:?}", incoming_request);
            let path: Vec<&str> = incoming_request[0].split_ascii_whitespace().collect();
            
            match path[0] {
                "GET" => {
                    if path[1] == "/" && path[1].len() == 1 {
                        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await;
                    } else if path[1].len() > 6 && path[1][..6].to_string() == "/echo/" {
                        let echo_val = path[1][6..].to_string();
                        let res_body = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n", echo_val.len(), echo_val);
                        match stream.write_all(res_body.as_bytes()).await {
                            Ok(_) => println!("SUCCESFULLY ECHOED: {}", echo_val),
                            Err(_) => println!("FAILED TO WRITE RESPONSE!"),
                        }
                    } else if path[1].len() == 11 && path[1][..11].to_string() == "/user-agent" {
                        let header_vec: Vec<&str> = incoming_request[2].split(" ").collect(); 
                        println!("HEADER VEC: {:?}", header_vec);
                        let header_val = header_vec[1];
                        let res_body = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n", header_val.len(), header_val);
                        match stream.write_all(res_body.as_bytes()).await {
                            Ok(_) => println!("SUCCESFULLY ECHOED: {}", header_val),
                            Err(_) => println!("FAILED TO WRITE RESPONSE!"),
                        } 
                    } else {
                        stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await;
                    }
                },
                _ => {
                    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await;
                },
            }
        });        
    }
}