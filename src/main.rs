
#![feature(iter_next_chunk)]

use std::{net::TcpStream, io::{BufReader, BufRead, Write, Read, Bytes}};

use rand::Rng;

const CLIENT_TYPE: u16 = 1;
const TEAM_NAME: &str = "Rust_pirates";
//const SERVER_ADDRESS: &str = "10.10.10.32:5000";
const SERVER_ADDRESS: &str = "127.0.0.1:5000";

/// All coordinates of the home bases, coordinates for base 0 are in index 0.
const HOME_BASE_COORDINATES: [(u16, u16); 16] = [(100, 100), (300, 100), (500, 100),
    (700, 100), (900, 100), (900, 300), (900, 500), (900, 700), (900, 900), (700, 900),
    (500, 900), (300, 900), (100, 900), (100, 700), (100, 500), (100, 300)];

fn main() {
    println!("register: {:?}", &Register::new());
    match TcpStream::connect(&SERVER_ADDRESS) {
        Ok(mut tcp_stream) => {
            let mut br = BufReader::new(tcp_stream.try_clone().expect(""));
            tcp_stream.write_all(&Register::new().as_bytes());
            let mut turn_number = 1;
            loop {
                br = BufReader::new(tcp_stream.try_clone().unwrap());
                //let mut input_buffer = String::new();
                //br.read_line(&mut input_buffer);
                //println!("Out: {:?}", input_buffer);
                let turn = Turn::new(&mut br.bytes());
                //turn.print(turn_number);
                turn_number += 1;
                action(&mut tcp_stream, &turn);
            }
        }
        Err(e) => {
            println!("Error: {:?}", e)
        }
    }
}

fn action(stream: &mut TcpStream, turn: &Turn) {
    let mut actions: Vec<u8> = Vec::new();
    let ants = Ants::from_turn(&turn);
    ants.print_ants();
    for ant in &ants.ants {
        actions.push(ant.calc_move(&turn, &ants.ant_positions));
    }
    //for (i, ant) in ants.ants.iter().enumerate() {
    //    if i > 14 {
    //        actions.push(ant.calc_move(&turn, &Vec::new()));
    //    } else {
    //        actions.push(5);
    //    }
    //}
    match stream.write_all(&actions) {
        Err(e) => println!("Error, unable to send action: {}", e),
        Ok(_ok) => (),
    }
}

#[derive(Debug, Ord, PartialEq, PartialOrd, Eq)]
enum AntCargo {
    Sugar,
    ToxicWaste,
}

#[derive(Debug, Ord, Eq)]
struct Ant {
    /// Id of this ant
    id: u8,
    /// Current position on the board
    pos: (u16, u16),
    /// Current health
    health: u8,
    /// Stores what the ant is carrying
    cargo: Option<AntCargo>,
}

impl Ant {
    /// Creates a new ant
    fn new(id: u8, pos: (u16, u16), health: u8, cargo: Option<AntCargo>) -> Self {
        Self {
            id,
            pos,
            health,
            cargo,
        }
    }

    /// Decides in wich direction this ant will move in the next turn
    fn calc_move(&self, turn: &Turn, ant_positions: &Vec<(u16, u16)>) -> u8 {
        // Move to enemy base when carrying toxin (for now for exidential pickups when moving somewhere else)
        if self.cargo.is_some() && self.cargo.as_ref().unwrap() == &AntCargo::ToxicWaste {
            println!("leading team base coordinates: {:?}", turn.leading_team_base_coordinates());
            return self.get_direction(turn.leading_team_base_coordinates(), ant_positions, &turn);
        }
        // Move home when carrying sugar
        if self.cargo.is_some() && self.cargo.as_ref().unwrap() == &AntCargo::Sugar {
            return self.get_direction(HOME_BASE_COORDINATES[turn.team_id as usize], ant_positions, &turn);
        }
        // Search next piece of sugar
        match turn.nearest_sugar_coordinates(self.pos) {
            Some(pos) => self.get_direction(pos, ant_positions, &turn),
            None => 5,
        }
    }

    /// Returns the direction in wich the ant should go this turn.
    /// Takes into consideration if the most optimal path is blocked by another ant and changes direction accordingly.
    /// Ants that already carry things will not walk over sugar/toxins.
    fn get_direction(&self, target: (u16, u16), ant_positions: &Vec<(u16, u16)>, turn: &Turn) -> u8 {
        let mut direction = self.move_direction(target);
        for i in 0..9  {
            let next_pos = next_point(self.pos, direction);
            if !ant_positions.contains(&next_pos) {
                break;
            }
            if self.cargo.is_some() && !turn.object_positions().contains(&next_pos) {
                break;
            }
            direction = rand::thread_rng().gen_range(1..9);
        }
        direction
    }

    /// Returns the direction in which the ant should go to reach target.
    fn move_direction(&self, target: (u16, u16)) -> u8 {
        if self.pos.0 > target.0 && self.pos.1 > target.1 {
            return 1;
        }
        if self.pos.0 < target.0 && self.pos.1 > target.1 {
            return 3;
        }
        if self.pos.0 < target.0 && self.pos.1 < target.1 {
            return 9;
        }
        if self.pos.0 > target.0 && self.pos.1 < target.1 {
            return 7;
        }
        if self.pos.0 == target.0 && self.pos.1 < target.1 {
            return 8;
        }
        if self.pos.0 == target.0 && self.pos.1 > target.1 {
            return 2;
        }
        if self.pos.0 < target.0 && self.pos.1 == target.1 {
            return 6;
        }
        if self.pos.0 > target.0 && self.pos.1 == target.1 {
            return 4;
        }
        return 5
    }
}

impl PartialOrd for Ant {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl PartialEq for Ant {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }

    fn ne(&self, other: &Self) -> bool {
        self.id != other.id
    }
}

/// Returns the point that will be reached from origin by going in the direction
fn next_point(origin: (u16, u16), direction: u8) -> (u16, u16) {
    match direction {
        1 => (origin.0 - 1, origin.1 - 1),
        2 => (origin.0, origin.1 - 1),
        3 => (origin.0 + 1, origin.1 - 1),
        4 => (origin.0 - 1, origin.1),
        5 => origin,
        6 => (origin.0 + 1, origin.1),
        7 => (origin.0 - 1, origin.1 + 1),
        8 => (origin.0, origin.1 + 1),
        9 => (origin.0 + 1, origin.1 + 1),
        _ => panic!("Invalid direction value"),
    }
}

struct Ants {
    ants: Vec<Ant>,
    /// Stores all positions the ants are at the moment.
    /// Used to determine possible collisions when ants are moving.
    ant_positions: Vec<(u16, u16)>,
}

impl Ants {
    /// Creates ants from the turn.
    fn from_turn(turn: &Turn) -> Self {
        let team_id = turn.team_id;
        let mut ants = Vec::new();
        let mut ant_positions = Vec::new();
        for object in &turn.objects {
            // Check object team id
            if i16::from(object.b1.lower) != team_id {
                continue;
            }
            // Check if object is ant
            if !object.is_ant() {
                continue;
            }
            ants.push(Ant::new(object.b2.upper, object.pos,object.b2.lower, object.get_ant_cargo()));
            ant_positions.push(object.pos);
        }
        // Make sure that ants are sorted acending by id
        ants.sort();
        Self {
            ants,
            ant_positions,
        }
    }

    /// Prints the ants to the console
    fn print_ants(&self) {
        println!("Ants: ");
        for ant in &self.ants {
            println!("{:?}", ant);
        }
    }
}

#[derive(Debug)]
struct Turn {
    /// Team id of client
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
            let team = Team::new(input, i);
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

    fn print(&self, turn_number: i32) {
        println!("Turn {}:", turn_number);
        println!("Team id: {}", self.team_id);
        println!("Teams:");
        for team in &self.teams {
            println!("{:?}", team);
        }
        println!("Number of Objects: {}", self.nr_of_objects);
        for object in &self.objects {
            println!("Object: {:?}", object);
        }
        println!();
    }

    /// Returns the coordinates of the base for the enemy team with the currently most points.
    /// 
    /// Used to lead ants with toxins to enemy bases.
    fn leading_team_base_coordinates(&self) -> (u16, u16) {
        let mut coordinates = HOME_BASE_COORDINATES[15];
        let mut max_points = 0;
        for team in &self.teams {
            // Prevent own base form getting attacked.
            if team.team_name == TEAM_NAME {
                continue;
            }
            if max_points < team.points {
                max_points = team.points;
                coordinates = HOME_BASE_COORDINATES[team.id as usize];
            }
        }
        coordinates
    }

    /// Returns the coordinates for the nearest piece of sugar or `None` if no sugar is found.
    /// 
    /// `pos` - the current position
    fn nearest_sugar_coordinates(&self, pos: (u16, u16)) -> Option<(u16, u16)> {
        let mut nearest_sugar: Option<(u16, u16)> = None;
        let mut nearest_distance = u16::MAX;
        for object in &self.objects {
            let cargo = object.get_ant_cargo();
            if cargo.is_some() && cargo.as_ref().unwrap() == &AntCargo::Sugar && !object.is_ant() {
                let distance = get_distance(pos, object.pos);
                if nearest_distance > distance {
                    nearest_sugar = Some(object.pos);
                    nearest_distance = distance;
                }
            }
        }
        nearest_sugar
    }

    /// Returns a vector that contains all positions of objects. This includes ants.
    fn object_positions(&self) -> Vec<(u16, u16)> {
        let mut positions = Vec::new();
        for object in &self.objects {
            positions.push(object.pos);
        }
        positions
    }

}

/// Calculates the distance between two points
fn get_distance(pos1: (u16, u16), pos2: (u16, u16)) -> u16 {
    let x_diff = (pos1.0 as i32 - pos2.0 as i32).abs() as u32;
    let y_diff = (pos1.1 as i32 - pos2.1 as i32).abs() as u32;
    let distance_squared = x_diff * x_diff + y_diff * y_diff;
    (distance_squared as f64).sqrt() as u16
}

#[derive(Debug)]
struct Team {
    id: i16,
    points: u16,
    remaining_ants: u16,
    team_name: String, //16 bytes, if not exactly 16 this will brake
}

impl Team {
    fn new(bytes: &mut Bytes<BufReader<TcpStream>>, id: i16) -> Self {
        Self {
            id,
            points: u16::from_le_bytes(read_to_two_byte_array(bytes)),
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
    pos: (u16, u16),
}

impl Object {
    fn new(input: &mut Bytes<BufReader<TcpStream>>) -> Self {
        let b1 = Pair::new(input.next().unwrap().unwrap());
        let b2 = Pair::new(input.next().unwrap().unwrap());
        let x = u16::from_le_bytes(read_to_two_byte_array(input));
        let y = u16::from_le_bytes(read_to_two_byte_array(input));
        Self {
            b1,
            b2,
            pos: (x, y),
        }
    }

    /// Returns true if this object is an ant
    fn is_ant(&self) -> bool {
        (self.b1.upper & (1 << 1-1)) != 0
        //true
    }

    /// Returns the cargo the ant is currently carrying or none if no cargo is carried.
    fn get_ant_cargo(&self) -> Option<AntCargo> {
        // TODO Check if calculation is correct.
        // Make parsing of bits work properly
        // This is probably the cause for some problems
        // But the problem might also be thæts ants are blocking each other

        //if (self.b1.upper & (1 << 4-1)) != 0 {
        //    return Some(AntCargo::ToxicWaste);
        //}
        //if (self.b1.upper & (1 << 2-1)) != 0 {
        //    return Some(AntCargo::Sugar);
        //}
        if self.b1.upper == 2 || self.b1.upper == 3 {
            return Some(AntCargo::Sugar);
        }
        if self.b1.upper == 4 || self.b1.upper == 5 {
            return Some(AntCargo::ToxicWaste);
        }
        // Currently everything is interprted as sugar, probably serverside bug
        None
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
            upper: byte >> 4,
            lower: byte & 0xf,
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

#[cfg(test)]
mod tests {
    use crate::Ant;


    #[test]
    fn test_ant_movement() {
        let ant = Ant::new(0, (1, 1), 10, None);
        assert_eq!(ant.move_direction((0,0)), 1);
        assert_eq!(ant.move_direction((1,0)), 2);
        assert_eq!(ant.move_direction((2,0)), 3);
        assert_eq!(ant.move_direction((0,1)), 4);
        assert_eq!(ant.move_direction((1,1)), 5);
        assert_eq!(ant.move_direction((2,1)), 6);
        assert_eq!(ant.move_direction((0,2)), 7);
        assert_eq!(ant.move_direction((1,2)), 8);
        assert_eq!(ant.move_direction((2,2)), 9);
    }
}