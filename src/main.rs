use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::str;

fn main() {
    let mut client_socket: TcpStream =
        TcpStream::connect("localhost:12345").expect("Failed to connect to server");
    loop {
        let mut length_bytes: [u8; 4] = [0u8; 4];
        client_socket
            .read_exact(&mut length_bytes)
            .expect("Failed to read message length");
        let message_length: usize = u32::from_be_bytes(length_bytes) as usize;
        let mut response_bytes: Vec<u8> = vec![0u8; message_length];
        client_socket
            .read_exact(&mut response_bytes)
            .expect("Failed to read message");
        let response: &str = str::from_utf8(&response_bytes).expect("Failed to decode UTF-8");
        let parts: Vec<&str> = response.split("$_$_$").collect();
        let message_type: &str = parts[0];
        let message: &str = parts[1];
        println!("{}", message);
        if message_type == "1" {
            loop {
                let mut response: String = String::new();
                if message.contains("Choose your name") {
                    break;
                }
                io::stdin()
                    .read_line(&mut response)
                    .expect("Failed to read from stdin");
                match response.trim().parse::<i32>() {
                    Ok(_) => break,
                    Err(_) => println!("Invalid. Try again"),
                }
            }
            client_socket
                .write_all(response.as_bytes())
                .expect("Failed to send response");
        }
    }
}
