use clap::{Arg, Command};
use salvo::prelude::*;
use std::path::{Path, PathBuf};
// use dirs;

mod handlers;
mod utils;

async fn init(data_dir: &str) {
    if !Path::new(data_dir).exists() {
        std::fs::create_dir(&data_dir).unwrap();
    }
}

#[handler]
async fn index(res: &mut Response) -> Result<(), anyhow::Error> {
    let data_dir = "data";

    let mut data_content = tokio::fs::read_dir(data_dir).await?;
    let mut data_files: Vec<PathBuf> = Vec::new();

    while let Some(data_input) = data_content.next_entry().await? {
        let ruta = data_input.path();
        let name = data_input.file_name();
        if ruta.is_file() {
            data_files.push(name.into());
        }
    }

    let data_files: Vec<String> = data_files
        .into_iter()
        .filter_map(|path| {
            path.file_stem().map(|stem| {
                format!(
                    r#"<li><a href="/api/{}">{}</a></li>"#,
                    stem.to_string_lossy().to_string(),
                    stem.to_string_lossy().to_string()
                )
            })
        })
        .collect();

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>Static API</title>
            </head>
            <body>
                <h1>Static API</h1>

                <p>This is a simple application simulating a basic REST API. It allows CRUD operations (Create, Read, Update, Delete) on different collections, where each collection is represented as a JSON file in the file system. If the collection does not exist, it is automatically created.</p>

                <h3>Collections</h3>

                <ul>{}</ul>

                <h3>Try something like</h3>

                <div style="background-color: #DEDEDE; padding: 10px;">

                    <pre>curl -X GET http://localhost:5800/api/&lt;collection&gt;</pre>

                    <pre>curl -X GET http://localhost:5800/api/&lt;collection&gt;/&lt;id&gt;</pre>

                    <pre>curl -X POST -H "Content-Type: application/json" -d '{{"field1":"value1", "field2":"value2"}}' http://localhost:5800/api/&lt;collection&gt;</pre>

                    <pre>curl -X PUT -H "Content-Type: application/json" -d '{{"field1":"new_value1", "new_field2":"value2"}}' http://localhost:5800/api/&lt;collection&gt;/&lt;id&gt;</pre>

                    <pre>curl -X DELETE http://localhost:5800/api/&lt;collection&gt;/&lt;id&gt;</pre>
                </div>

            </body>
        </html>
        "#,
        data_files.join("")
    );

    res.render(Text::Html(html));
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let matches = Command::new("static-api server")
        .arg(Arg::new("host")
            .short('i')
            .long("host")
            .value_name("HOST")
            .default_value("127.0.0.1")
            .help("IP address of the server")
            .required(false))
        .arg(Arg::new("port")
            .short('p')
            .long("port")
            .value_name("PORT")
            .default_value("5800")
            .help("Port that will listen to the server")
            .required(false))
        .get_matches();

    let host = matches.get_one::<String>("host").unwrap();
    let port = matches.get_one::<String>("port").unwrap();

    // if let Some(mut home_dir) = dirs::home_dir() {
    //     home_dir.push(".static-api");
    //
    //     if let Some(data_dir_str) = home_dir.to_str() {
    //         init(data_dir_str).await;
    //     } else {
    //         println!("Failed to convert the path to &str.");
    //     }
    // } else {
    //     println!("Unable to determine the user's directory.");
    // }

    init("data").await;

    let router = Router::new()
        .get(index)
        .push(
            Router::with_path("api/<f>")
                .get(handlers::get_all)
                .post(handlers::add_one),
        )
        .push(
            Router::with_path("api/<f>/<id>")
                .get(handlers::get_one)
                .put(handlers::update_one)
                .delete(handlers::delete_one),
        );
    let acceptor = TcpListener::new(format!("{host}:{port}")).bind().await;
    Server::new(acceptor).serve(router).await;
}
