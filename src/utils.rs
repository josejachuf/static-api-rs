use rand::Rng;
use std::io;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub fn generate_random_id() -> u64 {
    let mut rng = rand::thread_rng();
    // rng.gen_range(1..=u64::MAX)
    rng.gen_range(1..=100000)
}

pub async fn read_json_from_file(f: &str) -> Result<String, io::Error> {
    let mut json_file = tokio::fs::File::open(format!("data/{}.json", f)).await?;
    let mut json_string = String::new();
    json_file.read_to_string(&mut json_string).await?;
    Ok(json_string)
}

pub async fn create_empty_json_file(f: &str) -> Result<(), io::Error> {
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

pub async fn update_json_file(
    f: &str,
    id: u64,
    updated_item: &serde_json::Value,
) -> Result<(), anyhow::Error> {
    let file_path = format!("data/{}.json", f);
    let json_string = match read_json_from_file(&f).await {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };

    let mut json_value = convert_string_to_json(&json_string)?;

    if let Some(index) = json_value
        .as_array()
        .unwrap()
        .iter()
        .position(|item| item["id"].as_u64() == Some(id))
    {
        json_value.as_array_mut().unwrap()[index] = updated_item.clone();

        let json_string = serde_json::to_string_pretty(&json_value)?;
        tokio::fs::write(file_path, json_string).await?;
    }

    Ok(())
}

pub async fn delete_from_json_file(f: &str, id: u64) -> Result<(), anyhow::Error> {
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

pub fn convert_string_to_json(json_string: &str) -> Result<serde_json::Value, anyhow::Error> {
    let json_value: serde_json::Value = serde_json::from_str(json_string)?;
    Ok(json_value)
}