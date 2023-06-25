use salvo::prelude::*;
use std::fs;
use std::error::Error;
use std::io::{self, Read};


#[handler]
async fn hello() -> &'static str {
    let directorio = "json";

    // Lee el contenido del directorio
    let contenido = fs::read_dir(directorio)
        .expect("Error al leer el directorio");

    // Itera sobre los elementos del directorio
    for elemento in contenido {
        if let Ok(entrada) = elemento {
            let ruta = entrada.path();
            let nombre = entrada.file_name();

            // Verifica si es un archivo
            if ruta.is_file() {
                println!("Archivo: {:?}", nombre);
            }
        }
    }
    "Hello World"
}


async fn read_json_from_file(f: String) -> Result<String, io::Error> {
    let mut json_file = fs::File::open(format!("json/{f}.json"))?;
    let mut json_string = String::new();
    json_file.read_to_string(&mut json_string)?;
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
async fn get_one(req: &mut Request, res: &mut Response) {
    let file_path = req.param::<String>("f").unwrap();
    let id = req.param::<u64>("id").unwrap();

    let mut file = std::fs::File::open(format!("json/{file_path}.json")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let json_value: serde_json::Value = serde_json::from_str(&contents).unwrap();

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
        res.render(Json(&filtered_item[0]));
    } else {
        // StatusCode::NO_CONTENT;
        res.render(Json("Not found"));
    }
}


#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt().init();

    let router = Router::new().get(hello)
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
    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}
