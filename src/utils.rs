use std::{io::{Bytes, BufReader}, net::TcpStream};

/// Returns the point that will be reached from origin by going in the direction
pub fn next_point(origin: (u16, u16), direction: u8) -> (u16, u16) {
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

/// Calculates the distance between two points
pub fn get_distance(pos1: (u16, u16), pos2: (u16, u16)) -> u16 {
    let x_diff = (pos1.0 as i32 - pos2.0 as i32).abs() as u32;
    let y_diff = (pos1.1 as i32 - pos2.1 as i32).abs() as u32;
    let distance_squared = x_diff * x_diff + y_diff * y_diff;
    (distance_squared as f64).sqrt() as u16
}

/// Takes two bytes from the iterator and returns them as array.
/// 
/// # Panics
/// Panics when the iterator does not contains two elements or when the tcp stream contains errored elements.
pub fn read_to_two_byte_array(input: &mut Bytes<BufReader<TcpStream>>) -> Result<[u8; 2], String> {
    let mut bytes: [u8; 2] = [0u8; 2];
    for i in &mut bytes {
        let buf = input.next();
        if buf.is_none() {
            return Err(String::from("Too few elements!"));
        }
        let buf = buf.unwrap();
        if buf.is_err() {
            return Err(format!("{:?}", buf.err()));
        }
        *i = buf.unwrap();
    }
    Ok(bytes)
}

/// Takes 16 byte from the iterator and parses tham as a string.
pub fn bytes_to_string(input: &mut Bytes<BufReader<TcpStream>>) -> String {
    let mut s = String::new();
    for _i in 0..16 {
        s.push(input.next().unwrap().unwrap() as char);
    }
    s
}
