use std::{net::TcpStream, io::{BufReader, BufRead, Write, Read, Bytes}, panic::UnwindSafe, collections::HashSet};

use utils::{read_to_two_byte_array, bytes_to_string};

use crate::{network::{Register}, ai::{turn, no_move}};

/// Some utility functions to calculate some things
mod utils;
/// Functionality used to connect to the server
mod network;
/// Ant controll
mod ai;

pub const TEAM_NAME: &str = "Rust_pirates";
const SERVER_ADDRESS: &str = "127.0.0.1:5000";

/// All coordinates of the home bases, coordinates for base 0 are in index 0.
const HOME_BASE_COORDINATES: [(u16, u16); 16] = [(100, 100), (300, 100), (500, 100),
    (700, 100), (900, 100), (900, 300), (900, 500), (900, 700), (900, 900), (700, 900),
    (500, 900), (300, 900), (100, 900), (100, 700), (100, 500), (100, 300)];
/// Points where ants will navigate to to lead them to their homebase  without clashing into an enemy homebase.
const HOME_BASE_BEACONS: [(u16, u16); 16] = [(110, 110), (300, 110), (500, 110), 
    (700, 110), (890, 110), (890, 300), (890, 500), (890, 700), (890, 890), (700, 890),
    (500, 890), (300, 890), (100, 890), (110, 700), (110, 500), (110, 300)];

/// The configgured ant jobs. Ant with id 0 will have the job at index 0 and so forth.
const ANT_JOBS: [AntJob; 16] = [
    AntJob::Gatherer,
    AntJob::Gatherer,
    AntJob::Gatherer,
    AntJob::Gatherer,
    AntJob::Gatherer,
    AntJob::Gatherer,
    AntJob::Gatherer,
    AntJob::Offensive,
    AntJob::Offensive,
    AntJob::Offensive,
    AntJob::Offensive,
    AntJob::Offensive,
    AntJob::Offensive,
    AntJob::Offensive,
    AntJob::WasteMover,
    AntJob::WasteMover,
];

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
                let t = Turn::new(&mut br.bytes(), &mut tcp_stream);
                //t.print(turn_number);
                turn_number += 1;
                turn(&mut tcp_stream, &t);
            }
        }
        Err(e) => {
            println!("Error: {:?}", e)
        }
    }
}

/// Different types of ants
#[derive(Debug, Ord, PartialEq, PartialOrd, Eq, Clone, Copy)]
enum AntJob {
    /// These ants will focus on gathering sugar back to the base
    Gatherer,
    /// These ants will seek to attack enemy ants, prioritiesed as followed: toxin > sugar > none.
    Offensive,
    /// These ants will bring toxic waste into the enemy base that is currently leading the game.
    /// If no more toxic waste is found they will performe the Offensive ants job.
    WasteMover,
}

#[derive(Debug, Ord, PartialEq, PartialOrd, Eq)]
enum AntCargo {
    Sugar,
    ToxicWaste,
}

#[derive(Debug, Ord, Eq)]
pub struct Ant {
    /// Id of this ant
    id: u8,
    /// Current position on the board
    pos: (u16, u16),
    /// Current health
    health: u8,
    /// Stores what the ant is carrying
    cargo: Option<AntCargo>,
    /// The job this ant is directed to do
    job: Option<AntJob>,
}

impl Ant {
    /// Creates a new ant
    fn new(id: u8, pos: (u16, u16), health: u8, cargo: Option<AntCargo>, job: Option<AntJob>) -> Self {
        Self {
            id,
            pos,
            health,
            cargo,
            job,
        }
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

impl Position for Ant {
    fn pos(&self) -> (u16, u16) {
        self.pos
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
    /// 
    /// `team_id` - determines for which team the ants should be build. If `None` ants will be build for own team.
    fn from_turn(turn: &Turn, team_id: Option<i16>) -> Self {
        let team_id = match team_id {
            None => turn.team_id,
            Some(id) => id,
        };
        let mut ants = Vec::new();
        let mut ant_positions = Vec::new();
        let mut missing_ants:HashSet<u8> = (0..16).collect(); // Stores ids of ants that are not yet added to the ants vec
        for object in &turn.objects {
            // Check object team id
            if i16::from(object.b1.lower) != team_id {
                continue;
            }
            // Check if object is ant
            if !object.is_ant() {
                continue;
            }
            if team_id == turn.team_id {
                ants.push(Ant::new(object.b2.upper, object.pos,object.b2.lower, object.get_ant_cargo(), Some(ANT_JOBS[object.b2.upper as usize])));
            } else {
                ants.push(Ant::new(object.b2.upper, object.pos,object.b2.lower, object.get_ant_cargo(), None));
            }
            ant_positions.push(object.pos);
            missing_ants.remove(&object.b2.upper);
        }
        // Add dead ants to vec
        // This is done to make sure that an action for each ant is submitted to the server even when ants are dead
        for id in &missing_ants {
            ants.push(Ant::new(*id, (0, 0), 0, None, None));
        }
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
pub struct Turn {
    /// Team id of client
    team_id: i16,
    teams: Vec<Team>,// 16 Teams are required
    nr_of_objects: u16,
    objects: Vec<Object>,
}

impl Turn {
    
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

    /// Returns a vector that contains all positions of objects. This includes ants.
    fn object_positions(&self) -> Vec<(u16, u16)> {
        let mut positions = Vec::new();
        for object in &self.objects {
            positions.push(object.pos);
        }
        positions
    }

    /// Builds ants for all enemy teams.
    /// Only includes ants that are alive.
    /// 
    /// - `live_threshold` can be set to limit the ants that are shown to only ants with less or equal amount of health.
    fn enemy_ants(&self, live_threshold: Option<u8>) -> Vec<Ant> {
        let mut ants = Vec::new();
        for i in 0..15 {
            if i == self.team_id {
                continue;
            }
            for ant in Ants::from_turn(&self, Some(i)).ants {
                if ant.health <= 0 {
                    continue;
                }
                if live_threshold.is_some() && ant.health > live_threshold.unwrap() {
                    continue;
                }
                ants.push(ant);
            }
        }
        ants
    }

}

#[derive(Debug)]
struct Team {
    id: i16,
    points: u16,
    remaining_ants: u16,
    team_name: String, //16 bytes, if not exactly 16 this will brake
}

#[derive(Debug)]
struct Object {
    b1: Pair,// Contains object type and team id
    b2: Pair,// Contains ant ID and ant health
    pos: (u16, u16),
}

impl Object {

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

impl Position for Object {
    fn pos(&self) -> (u16, u16) {
        self.pos
    }
}

impl Position for &Object {
    fn pos(&self) -> (u16, u16) {
        self.pos
    }
}

/// Represents a data type that uses an u8 to store two 4 bit values.
#[derive(Debug)]
struct Pair {
    upper: u8,
    lower: u8,
}

/// Trait to get position of objects
pub trait Position {
    /// Returns the position
    fn pos(&self) -> (u16, u16);
}

#[cfg(test)]
mod tests {
    use crate::Ant;

    #[test]
    fn test_ant_movement() {
        let ant = Ant::new(0, (1, 1), 10, None, None);
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