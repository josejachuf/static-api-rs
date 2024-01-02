use dotenv;
use salvo::prelude::*;
use std::path::{PathBuf, Path};

mod api;

async fn init(data_dir: &str) {
    if !Path::new(data_dir).exists() {
        std::fs::create_dir(&data_dir).unwrap();
    }
}

#[handler]
async fn index(res: &mut Response) -> Result<(), anyhow::Error> {
    let directorio = "data";

    let mut data_content = tokio::fs::read_dir(directorio).await?;
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
        .filter_map(|path| path.file_stem().map(|stem| format!(r#"<li><a href="/api/{}">{}</a></li>"#, stem.to_string_lossy().to_string(), stem.to_string_lossy().to_string())))
        .collect();

    let html = format!(r#"
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
        "#, data_files.join(""));

    res.render(Text::Html(html));
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    dotenv::dotenv().ok();
    init("data").await;

    let host = std::env::var("IPHOST").unwrap_or("127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or("5800".to_string());

    let router = Router::new().get(index)
        .push(Router::with_path("api/<f>")
              .get(api::get_all)
              .post(api::add_one)
              )
        .push(Router::with_path("api/<f>/<id>")
              .get(api::get_one)
              .put(api::update_one)
              .delete(api::delete_one)
              )

        ;
    let acceptor = TcpListener::new(format!("{host}:{port}")).bind().await;
    Server::new(acceptor).serve(router).await;
}
