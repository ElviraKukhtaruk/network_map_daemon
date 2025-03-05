use clap::Parser;
use crate::config::parse_cli;
use super::parse_config::Server;

pub fn get_parameters_from_cli() -> Server {
    let cli = parse_cli::Cli::parse();

    Server {
        server_id: cli.server_id,
        label: cli.label,
        interface_filter: cli.interface_filter,
        lat: cli.lat,
        lng: cli.lng,
        hostname: cli.hostname,
        city: cli.city,
        country: cli.country
    }

}
