use core::str;

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};

pub async fn concurrent() {
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf = vec![];
            stream.read_buf(&mut buf).await.unwrap();

            let incoming_request: Vec<&str> = str::from_utf8(&buf).unwrap().split("\r\n").collect();
            let path: Vec<&str> = incoming_request[0].split_ascii_whitespace().collect();
            println!("INCOMING REQUEST: {:?}", incoming_request);
            
            match path[0] {
                "GET" => {
                    if path[1][..6].to_string() == "/echo/" {
                        let echo_val = path[1][6..].to_string();
                        let res_body = format!("HTTP/1.1 200 OK\\r\\nContent-Type: text/plain\\r\\nContent-Length: {}\\r\\n\\r\\{}\\r\\n", echo_val.len(), echo_val);
                        stream.write_all(res_body.as_bytes()).await
                    } else if path[1] == "/" {
                        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await
                    } else {
                        stream.write_all(b"").await
                    }
                    
                },
                _ => {
                    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await
                },
            }
        });        
    }
}