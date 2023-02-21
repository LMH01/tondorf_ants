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
    pub print_ants: bool,
    #[arg(short, help = "Amount of gatherer ants, total amount of all ants needs to be 16",
        long_help = "Amount of  gatherer ants, total amount of all ants needs to be exact 16. Their top priority is to collect sugar.",
        required_unless_present_any = ["ant_help", "default_jobs", "random_jobs"])]
    pub gatherer_ants: Option<u8>,
    #[arg(short, help = "Amount of offensive ants",
        long_help = "Amount of offensive ants. Their top priority is to attack enemy ants.",
        required_unless_present_any = ["ant_help", "default_jobs", "random_jobs"])]
    pub offensive_ants: Option<u8>,
    #[arg(short, help = "Amount of waste mover ants",
        long_help = "Amount of waste mover ants. Their top priority is to move waste to enemy bases.",
        required_unless_present_any = ["ant_help", "default_jobs", "random_jobs"])]
    pub waste_mover_ants: Option<u8>,
    #[arg(short, long, help = "Print extended help regarding the different ant types.", exclusive = true)]
    pub ant_help: bool,
    #[arg(short, long, help = "Set the maximum amount of health enemy ants can have before they are attacked.", default_value = "10")]
    pub max_health: u8,
    #[arg(short, long, help = "If set the ant jobs will be set to a default value", default_value = "false",
        conflicts_with_all = ["gatherer_ants", "offensive_ants", "waste_mover_ants"])]
    pub default_jobs: bool,
    #[arg(short, long, help = "Set to make ant job selection random",
        conflicts_with_all = ["gatherer_ants", "offensive_ants", "waste_mover_ants", "default_jobs"])]
    pub random_jobs: bool,
}
