// Uncomment this block to pass the first stage
use std::{env::{self, Args}, io::{BufRead, BufReader, Read, Write}, net::TcpListener, thread};

mod multithreaded;
mod concurrent;


#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let directory = if args.len() > 3 {
        args[2].to_string()
    } else {
        "".to_string()
    };
    concurrent::concurrent(directory).await;
}
