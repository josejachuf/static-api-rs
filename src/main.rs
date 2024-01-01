use dotenv;
use salvo::prelude::*;
use std::io;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

use std::path::PathBuf;

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
                <ul>{}</ul>
            </body>
        </html>
        "#, data_files.join(""));

    res.render(Text::Html(html));
    Ok(())
}


async fn read_json_from_file(f: &str) -> Result<String, io::Error> {
    let mut json_file = tokio::fs::File::open(format!("data/{}.json", f)).await?;
    let mut json_string = String::new();
    json_file.read_to_string(&mut json_string).await?;
    Ok(json_string)
}

async fn create_empty_json_file(f: &str) -> Result<(), io::Error> {
    let file_path = format!("data/{}.json", f);
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file_path)
        .await?;
    file.write_all(b"[]").await?;
    Ok(())
}

async fn delete_from_json_file(f: &str, id: u64) -> Result<(), anyhow::Error> {
    let file_path = format!("data/{}.json", f);
    let json_string = match read_json_from_file(&f).await {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };

    let json_value = convert_string_to_json(&json_string)?;

    let filtered_items: Vec<serde_json::Value> = json_value
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| {
            if let Some(item_id) = item["id"].as_u64() {
                item_id != id
            } else {
                true
            }
        })
        .cloned()
        .collect();

    let json_string = serde_json::to_string_pretty(&filtered_items)?;
    tokio::fs::write(file_path, json_string).await?;

    Ok(())
}

fn convert_string_to_json(json_string: &str) -> Result<serde_json::Value, anyhow::Error> {
    let json_value: serde_json::Value = serde_json::from_str(json_string)?;
    Ok(json_value)
}

#[handler]
async fn get_all(req: &mut Request) -> Result<Json<serde_json::Value>, anyhow::Error> {
    let file_path = req.param::<String>("f").unwrap();

    let json_string = match read_json_from_file(&file_path).await {
        Ok(s) => s,
        Err(_) => {
            create_empty_json_file(&file_path).await?;
            String::from("[]")
        }
    };

    let json_value = convert_string_to_json(&json_string)?;
    Ok(Json(json_value))
}

#[handler]
async fn get_one(req: &mut Request) -> Result<Json<serde_json::Value>, anyhow::Error> {
    let file_path = req.param::<String>("f").unwrap();
    let id = req.param::<u64>("id").unwrap();

    let json_string = match read_json_from_file(&file_path).await {
        Ok(s) => s,
        Err(_) => {
            create_empty_json_file(&file_path).await?;
            String::from("[]")
        }
    };
    let json_value = convert_string_to_json(&json_string)?;

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

#[handler]
async fn add_one(req: &mut Request) -> Result<Json<serde_json::Value>, anyhow::Error> {
    let file_path = req.param::<String>("f").unwrap();

    let json_string = match read_json_from_file(&file_path).await {
        Ok(s) => s,
        Err(_) => {
            create_empty_json_file(&file_path).await?;
            String::from("[]")
        }
    };
    let mut json_value = convert_string_to_json(&json_string)?;

    let new_item_json = req.parse_body::<serde_json::Value>().await?;

    json_value.as_array_mut().unwrap().push(new_item_json);

    let json_string = serde_json::to_string_pretty(&json_value)?;
    let file_path = format!("data/{}.json", file_path);
    tokio::fs::write(file_path, json_string).await?;

    Ok(Json(json_value))
}

#[handler]
async fn delete_one(req: &mut Request) -> Result<Json<serde_json::Value>, anyhow::Error> {
    let file_path = req.param::<String>("f").unwrap();
    let id = req.param::<u64>("id").unwrap();

    delete_from_json_file(&file_path, id).await?;

    let json_string = read_json_from_file(&file_path).await?;
    let json_value = convert_string_to_json(&json_string)?;

    Ok(Json(json_value))
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
              .post(add_one)
              )
        .push(Router::with_path("api/<f>/<id>")
              .get(get_one)
              // .put(put_one)
              .delete(delete_one)
              )

        ;
    let acceptor = TcpListener::new(format!("{host}:{port}")).bind().await;
    Server::new(acceptor).serve(router).await;
}
