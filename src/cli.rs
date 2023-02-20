use std::net::Ipv4Addr;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author = "LMH01", version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, long_help = "The team name under wich the client should register at the server", default_value = Some("Rust_pirates"))]
    pub team_name: String,
    #[arg(short, long, help = "The ip address of the server", default_value = "127.0.0.1")]
    pub ip: Ipv4Addr,
    #[arg(long, help = "The port of the server", default_value = "5000")]
    pub port: u16,
    #[arg(short, long, help = "Submit to print the players ants into console")]
    pub print_ants: bool
}
