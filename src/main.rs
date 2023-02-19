
#![feature(iter_next_chunk)]

use std::{net::TcpStream, io::{BufReader, BufRead, Write, Read, Bytes}};

const CLIENT_TYPE: u16 = 1;
const TEAM_NAME: &str = "Rust_pirates";
//const SERVER_ADDRESS: &str = "10.10.10.32:5000";
const SERVER_ADDRESS: &str = "127.0.0.1:5000";

fn main() {
    println!("register: {:?}", &Register::new());
    match TcpStream::connect(&SERVER_ADDRESS) {
        Ok(mut tcp_stream) => {
            let mut br = BufReader::new(tcp_stream.try_clone().expect(""));
            tcp_stream.write_all(&Register::new().as_bytes());
            let mut turn_number = 1;
            loop {
                //let mut input_buffer = String::new();
                //br.read_line(&mut input_buffer);
                //println!("Out: {:?}", input_buffer);
                let turn = Turn::new(&mut br.bytes());
                println!("Turn [{}]: {:?}", turn_number,turn);
                turn_number += 1;
                br = BufReader::new(tcp_stream.try_clone().unwrap());
            }
        }
        Err(e) => {
            println!("Error: {:?}", e)
        }
    }
}

#[derive(Debug)]
struct Turn {
    team_id: i16,
    teams: Vec<Team>,// 16 Teams are required
    nr_of_objects: u16,
    objects: Vec<Object>,
}

impl Turn {
    fn new(input: &mut Bytes<BufReader<TcpStream>>) -> Self {
        // Parse team id
        let team_id: i16 = i16::from_le_bytes(read_to_two_byte_array(input));// Frage: Welche Größenordnung? Muss hier little endian oder big endian benutzt werden?
        // Parse teams
        let mut teams: Vec<Team> = Vec::new();
        for i in 0..16 {
            let team = Team::new(input);
            teams.push(team);
        }
        // Parse number of objects
        let nr_of_objects = u16::from_le_bytes(read_to_two_byte_array(input));
        let mut objects: Vec<Object> = Vec::new();
        for _i in 0..nr_of_objects {
            objects.push(Object::new(input));
        }
        Self {
            team_id,
            teams,
            nr_of_objects,
            objects
        }
    }
}

#[derive(Debug)]
struct Team {
    points: u16,
    remaining_ants: u16,
    team_name: String, //16 bytes, if not exactly 16 this will brake
}

impl Team {
    fn new(bytes: &mut Bytes<BufReader<TcpStream>>) -> Self {
        Self {
            points: u16::from_le_bytes(read_to_two_byte_array(bytes)),// Frage: Welche Größenordnung? Muss hier little endian oder big endian benutzt werden?
            remaining_ants: u16::from_le_bytes(read_to_two_byte_array(bytes)),
            team_name: bytes_to_string(bytes),
        }
    }
}

fn read_to_two_byte_array(input: &mut Bytes<BufReader<TcpStream>>) -> [u8; 2] {
    let mut bytes: [u8; 2] = [0u8; 2];
    for i in 0..2 {
        bytes[i] = input.next().unwrap().unwrap();
    }
    bytes
}

fn bytes_to_string(input: &mut Bytes<BufReader<TcpStream>>) -> String {
    let mut s = String::new();
    for i in 0..16 {
        s.push(input.next().unwrap().unwrap() as char);
    }
    s
}

#[derive(Debug)]
struct Object {
    b1: Pair,// Contains object type and team id
    b2: Pair,// Contains ant ID and ant health
    x: u16,
    y: u16,
}

impl Object {
    fn new(input: &mut Bytes<BufReader<TcpStream>>) -> Self {
        Self {
            b1: Pair::new(input.next().unwrap().unwrap()),
            b2: Pair::new(input.next().unwrap().unwrap()),
            x: u16::from_le_bytes(read_to_two_byte_array(input)),
            y: u16::from_le_bytes(read_to_two_byte_array(input)),
        }
    }
}

/// Represents a data type that uses an u8 to store two 4 bit values.
#[derive(Debug)]
struct Pair {
    upper: u8,
    lower: u8,
}

impl Pair {
    fn new(byte: u8) -> Self {
        Self {
            upper: byte << 4,
            lower: byte >> 4,
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