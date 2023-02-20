use std::{io::{Bytes, BufReader}, net::TcpStream};

use crate::{TEAM_NAME, Turn, utils::{read_to_two_byte_array, bytes_to_string, u8_vec_to_to_string}, Team, Object, Pair};

const CLIENT_TYPE: u16 = 1;

#[derive(Debug)]
pub struct Register {
    client_type: u16,
    team_name: String,
}

impl Register {
    pub fn new() -> Self {
        if TEAM_NAME.len() > 16 {
            panic!("Teamname to long, should be <= 16 chars long!");
        }
        Self {
            client_type: CLIENT_TYPE, 
            team_name: TEAM_NAME.to_string(), 
        }
    }    
    
    pub fn as_bytes(&self) -> [u8; 18] {
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

impl Turn {

    /// Creates a new `Turn` object by parsing the bytes of the tcp stream
    pub fn new(input: &mut Bytes<BufReader<TcpStream>>, b: &mut TcpStream) -> Self {
        // Parse team id
        let team_id: i16 = i16::from_le_bytes(read_to_two_byte_array(input).unwrap());// Frage: Welche Größenordnung? Muss hier little endian oder big endian benutzt werden?
        // Parse teams
        let mut teams: Vec<Team> = Vec::new();
        for i in 0..16 {
            let team = Team::new(input, i);
            //let team = Team::new_new(b, i);
            //println!("Team {}: {:?}", i, team);
            teams.push(team);
        }
        // Parse number of objects
        let nr_of_objects = u16::from_le_bytes(read_to_two_byte_array(input).unwrap());
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

impl Team {

    /// Creates a new team by parsing the bytes in the tcp stream
    fn new(bytes: &mut Bytes<BufReader<TcpStream>>, id: i16) -> Self {
        Self {
            id,
            points: u16::from_le_bytes(read_to_two_byte_array(bytes).unwrap()),
            remaining_ants: u16::from_le_bytes(read_to_two_byte_array(bytes).unwrap()),
            team_name: bytes_to_string(bytes),
        }
    }

}

impl Object {

    /// Creates a new object by parsing the bytes in the tcp stream
    fn new(input: &mut Bytes<BufReader<TcpStream>>) -> Self {
        let b1 = Pair::new(input.next().unwrap().unwrap());
        let b2 = Pair::new(input.next().unwrap().unwrap());
        let x = u16::from_le_bytes(read_to_two_byte_array(input).unwrap());
        let y = u16::from_le_bytes(read_to_two_byte_array(input).unwrap());
        Self {
            b1,
            b2,
            pos: (x, y),
        }
    }
}

impl Pair {

    /// Creates a new pair by parsing the upper 4 and lower 4 bits of the input byte
    fn new(byte: u8) -> Self {
        Self {
            upper: byte >> 4,
            lower: byte & 0xf,
        }
    }
}

