// Uncomment this block to pass the first stage
use std::{io::{BufRead, BufReader, Read, Write}, net::TcpListener, thread};

mod multithreaded;
mod concurrent;

#[tokio::main]
async fn main() {
    concurrent::concurrent().await;
}
