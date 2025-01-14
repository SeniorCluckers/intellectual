use std::process::exit;

use actix_files::Files;
use actix_web::{App, HttpServer};
use clap::{Arg, Command};
use log::{error, info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};

mod api;
mod artist;
mod genius;
mod home;
mod lyrics;
mod search;
mod templates;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = Command::new("intellectual")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Alternate front-end for Genius written in Rust")
        .arg(
            Arg::new("address")
                .short('a')
                .long("address")
                .value_name("ADDRESS")
                .help("Sets the address to listen on")
                .default_value("0.0.0.0")
                .num_args(1),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Sets the port to listen on")
                .default_value("8080")
                .num_args(1),
        )
        .arg(
            Arg::new("workers")
                .short('w')
                .long("workers")
                .value_name("WORKERS")
                .help("The amount of HTTP workers to use. 0 to equal physical CPU cores")
                .default_value("0")
                .num_args(1),
        )
        .get_matches();

    let address = matches.get_one::<String>("address").unwrap().as_str();
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| matches.get_one::<String>("port").unwrap().to_string())
        .parse::<u16>()
        .unwrap();
    let workers = matches
        .get_one::<String>("workers")
        .unwrap_or(&"0".to_string())
        .parse::<usize>()
        .unwrap();

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    if std::env::var("GENIUS_AUTH_TOKEN").is_err() {
        error!("GENIUS_AUTH_TOKEN environment variable not set!");
        exit(1);
    }

    info!(
        "Running Intellectual v{}, listening on {}:{}!",
        env!("CARGO_PKG_VERSION"),
        address,
        port
    );

    let mut server = HttpServer::new(|| {
        App::new()
            .service(api::api)
            .service(artist::artist)
            .service(home::home)
            .service(lyrics::lyrics)
            .service(search::search)
            .service(Files::new("/", "./static"))
    });

    if workers > 0 {
        server = server.workers(workers);
    }

    server.bind((address, port))?.run().await
}
