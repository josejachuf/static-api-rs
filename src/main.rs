use dotenv;
use salvo::prelude::*;
use std::io;
use tokio::io::AsyncReadExt;
use std::path::PathBuf;

#[handler]
async fn index(res: &mut Response) -> Result<(), anyhow::Error> {
    let directorio = "data";

    let mut contenido = tokio::fs::read_dir(directorio).await?;
    let mut archivos: Vec<PathBuf> = Vec::new();

    while let Some(entrada) = contenido.next_entry().await? {
        let ruta = entrada.path();
        let nombre = entrada.file_name();
        if ruta.is_file() {
            archivos.push(nombre.into());
        }
    }

    let archivos: Vec<String> = archivos
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
                <ul>{}</ul>
            </body>
        </html>
        "#, archivos.join(""));

    res.render(Text::Html(html));
    Ok(())
}


async fn read_json_from_file(f: String) -> Result<String, io::Error> {
    let mut json_file = tokio::fs::File::open(format!("data/{f}.json")).await?;
    let mut json_string = String::new();
    json_file.read_to_string(&mut json_string).await?;
    Ok(json_string)
}

fn convert_string_to_json(json_string: String) -> Result<serde_json::Value, anyhow::Error> {
    let json_value: serde_json::Value = serde_json::from_str(&json_string)?;
    Ok(json_value)
}

#[handler]
async fn get_all(req: &mut Request) -> Result<Json<serde_json::Value>, anyhow::Error> {
    let file_path = req.param::<String>("f").unwrap();

    let json_string = read_json_from_file(file_path).await?;
    let json_value = convert_string_to_json(json_string)?;
    Ok(Json(json_value))
}

#[handler]
async fn get_one(req: &mut Request) -> Result<Json<serde_json::Value>, anyhow::Error> {
    let file_path = req.param::<String>("f").unwrap();
    let id = req.param::<u64>("id").unwrap();

    let json_string = read_json_from_file(file_path).await?;
    let json_value = convert_string_to_json(json_string)?;

    let filtered_item = json_value
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| {
            if let Some(item_id) = item["id"].as_u64() {
                item_id == id
            } else {
                false
            }
        })
        .cloned()
        .collect::<Vec<serde_json::Value>>();

    if filtered_item.len() > 0 {
        let filtered_item = &filtered_item[0];
        Ok(Json(filtered_item.clone()))
    } else {
        Ok(Json("Not found".into()))
    }
}


#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt().init();
    dotenv::dotenv().ok();

    let host = std::env::var("IPHOST").unwrap_or("127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or("5800".to_string());

    let router = Router::new().get(index)
        .push(Router::with_path("api/<f>")
              .get(get_all)
              // .post(post_one)
              )
        .push(Router::with_path("api/<f>/<id>")
              .get(get_one)
              // .put(put_one)
              // .delete(delete_one)
              )

        ;
    let acceptor = TcpListener::new(format!("{host}:{port}")).bind().await;
    Server::new(acceptor).serve(router).await;
}
