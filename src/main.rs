use clap::{Arg, Command};
use dirs;
use salvo::affix;
use salvo::cors::{self as cors, Cors};
use salvo::prelude::*;
use std::path::Path;

mod handlers;
mod html;
mod utils;

#[derive(Default, Clone, Debug)]
pub struct AppConfig {
    pub data_dir: String,
}

async fn init(data_dir: &str) {
    if !Path::new(data_dir).exists() {
        std::fs::create_dir(&data_dir).unwrap();
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let matches = Command::new("static-api server")
        .arg(
            Arg::new("host")
                .short('i')
                .long("host")
                .value_name("HOST")
                .default_value("127.0.0.1")
                .help("IP address of the server")
                .required(false),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .default_value("5800")
                .help("Port that will listen to the server")
                .required(false),
        )
        .get_matches();

    let host = matches.get_one::<String>("host").unwrap();
    let port = matches.get_one::<String>("port").unwrap();

    let mut data_dir = String::new();

    if let Some(mut home_dir) = dirs::home_dir() {
        home_dir.push(".static-api");

        if let Some(data_dir_str) = home_dir.to_str() {
            data_dir = data_dir_str.to_string();
        } else {
            data_dir = "data".to_string();
            println!("Failed to convert the path to &str.");
        }
    } else {
        println!("Unable to determine the user's directory.");
    }

    init(&data_dir).await;

    let app_config = AppConfig {
        data_dir: data_dir.clone(),
    };

    let cors_handler = Cors::new()
        .allow_origin(cors::Any)
        .allow_methods(cors::Any)
        .allow_headers(cors::Any)
        .into_handler();

    let router = Router::new()
        .hoop(affix::inject(app_config.clone()))
        .get(html::index)
        .push(
            Router::with_path("api/<f>")
                .hoop(cors_handler.clone())
                .options(handler::empty())
                .get(handlers::get_all)
                .post(handlers::add_one),
        )
        .push(
            Router::with_path("api/<f>/<id>")
                .hoop(cors_handler.clone())
                .options(handler::empty())
                .get(handlers::get_one)
                .put(handlers::update_one)
                .delete(handlers::delete_one),
        );
    let acceptor = TcpListener::new(format!("{host}:{port}")).bind().await;
    println!("Welcome to static-api!");
    println!("To get started, please visit the http://{host}:{port} in your browser:");
    Server::new(acceptor).serve(router).await;
}
