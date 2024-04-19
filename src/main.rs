use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::str;

fn main() {
    let mut client_socket: TcpStream = TcpStream::connect("91.236.168.95:16158").expect("Failed to connect");
    loop {
        let mut buffer: [u8; 1024] = [0; 1024];
        match client_socket.read(&mut buffer) {
            Ok(size) => {
                if size == 0 {
                    break;
                }
                let response: &str = str::from_utf8(&buffer[..size]).expect("Failed to read");
                let parts: Vec<&str> = response.split("$_$_$").collect();
                if parts.len() == 2 {
                    let message_type: &str = parts[0];
                    let message: &str = parts[1];
                    println!("{}", message);
                    if message_type == "1" {
                        let mut input_string: String = String::new();
                        io::stdin().read_line(&mut input_string).expect("Failed to read from stdin");
                        client_socket.write_all(input_string.as_bytes()).expect("Failed to send data");
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to receive data: {}", e);
                break;
            }
        }
    }
    println!("Client is exiting...");
    client_socket.shutdown(std::net::Shutdown::Both).expect("Shutdown failed");
}


