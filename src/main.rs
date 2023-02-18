use std::{net::TcpStream, io::{BufReader, BufRead, Write}};

const CLIENT_TYPE: u16 = 1;
const TEAM_NAME: &str = "Rust_pirates";
const SERVER_ADDRESS: &str = "10.10.10.32:5000";

fn main() {
    println!("register: {:?}", &Register::new());
    match TcpStream::connect(&SERVER_ADDRESS) {
        Ok(mut tcp_stream) => {
            let mut br = BufReader::new(tcp_stream.try_clone().expect(""));
            tcp_stream.write_all(&Register::new().as_bytes());
            loop {
                let mut input_buffer = String::new();
                br.read_line(&mut input_buffer);
                println!("Out: {:?}", input_buffer);
            }
        }
        Err(e) => {
            println!("Error: {:?}", e)
        }
    }
}

#[derive(Debug)]
struct Register {
    client_type: u16,
    team_name: String,
}

impl Register {
    fn new() -> Self {
        if TEAM_NAME.len() > 16 {
            panic!("Teamname to long, should be <= 16 chars long!");
        }
        Self {
            client_type: CLIENT_TYPE, 
            team_name: TEAM_NAME.to_string(), 
        }
    }    
    
    fn as_bytes(&self) -> [u8; 18] {
        let mut out: Vec<u8> = Vec::new();
        out.push(self.client_type.try_into().unwrap());
        out.push(0);
        for b in self.team_name.as_bytes() {
            out.push(*b);
        }
        for i in self.team_name.len() .. 16 {
            out.push(0);
        }
        out.try_into().unwrap()
    }
}