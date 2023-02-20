use std::{net::TcpStream, io::Write};

use rand::Rng;

use crate::{Ant, Ants, Turn, AntCargo, HOME_BASE_COORDINATES, TEAM_NAME, utils::{get_distance, next_point}, Position};

/// Analyzes the current game state and makes an approprate turn by moving each ant one tile.
pub fn turn(stream: &mut TcpStream, turn: &Turn) {
    let mut actions: Vec<u8> = Vec::new();
    let ants = Ants::from_turn(&turn, None);
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

impl Ant {
    /// Decides in wich direction this ant will move in the next turn
    fn calc_move(&self, turn: &Turn, ant_positions: &Vec<(u16, u16)>) -> u8 {
        // Do nothing when dead
        if self.health == 0 {
            return 5;
        }
        // Move home when lifes <= 3
        if self.health <= 3 {
            return self.get_direction(HOME_BASE_COORDINATES[turn.team_id as usize], ant_positions, &turn);
        }
        // Attack clostest enemy ant when ant is below 5 health
        if true {
            let nearest_enemy = turn.nearest(self.pos, &turn.enemy_ants(Some(5)));
            if nearest_enemy.is_some() {
                return self.get_direction(nearest_enemy.unwrap(), ant_positions, turn);
            }
        }

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

}

impl Turn {

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
        let mut sugar_pieces = Vec::new();
        for object in &self.objects {
            let cargo = object.get_ant_cargo();
            if cargo.is_some() && cargo.as_ref().unwrap() == &AntCargo::Sugar && !object.is_ant() {
                sugar_pieces.push(object);
            }
        }
        self.nearest(pos, &sugar_pieces)
    }

    pub fn nearest<T: Position>(&self, pos: (u16, u16), input: &Vec<T>) -> Option<(u16, u16)> {
        let mut nearest: Option<(u16, u16)> = None;
        let mut nearest_distance = u16::MAX;
        for object in input {
            let distance = get_distance(pos, object.pos());
            if nearest_distance > distance {
                nearest = Some(object.pos());
                nearest_distance = distance;
            }
        }
        nearest
    }
    
}
