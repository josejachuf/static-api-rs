use rand::Rng;
use salvo::prelude::*;
use std::io;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

use std::path::PathBuf;

fn generate_random_id() -> u64 {
    let mut rng = rand::thread_rng();
    // rng.gen_range(1..=u64::MAX)
    rng.gen_range(1..=100000)
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
pub async fn get_all(req: &mut Request) -> Result<Json<serde_json::Value>, anyhow::Error> {
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
pub async fn get_one(req: &mut Request) -> Result<Json<serde_json::Value>, anyhow::Error> {
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
pub async fn add_one(req: &mut Request) -> Result<Json<serde_json::Value>, anyhow::Error> {
    let file_path = req.param::<String>("f").unwrap();

    let json_string = match read_json_from_file(&file_path).await {
        Ok(s) => s,
        Err(_) => {
            create_empty_json_file(&file_path).await?;
            String::from("[]")
        }
    };
    let mut json_value = convert_string_to_json(&json_string)?;

    let mut new_item_json = req.parse_body::<serde_json::Value>().await?;

    if new_item_json.get("id").is_none() {
        new_item_json["id"] = serde_json::Value::from(generate_random_id());
    }

    json_value.as_array_mut().unwrap().push(new_item_json);

    let json_string = serde_json::to_string_pretty(&json_value)?;
    let file_path = format!("data/{}.json", file_path);
    tokio::fs::write(file_path, json_string).await?;

    Ok(Json(json_value))
}

#[handler]
pub async fn delete_one(req: &mut Request) -> Result<Json<serde_json::Value>, anyhow::Error> {
    let file_path = req.param::<String>("f").unwrap();
    let id = req.param::<u64>("id").unwrap();

    delete_from_json_file(&file_path, id).await?;

    let json_string = read_json_from_file(&file_path).await?;
    let json_value = convert_string_to_json(&json_string)?;

    Ok(Json(json_value))
}
