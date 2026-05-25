use rand::RngExt;
use std::io;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::AppError;

pub fn generate_random_id() -> u64 {
    let mut rng = rand::rng();
    rng.random_range(1..=100000)
}

pub async fn read_json_from_file(data_dir: &str, f: &str) -> Result<String, io::Error> {
    let mut json_file = tokio::fs::File::open(format!("{}/{}.json", data_dir, f)).await?;
    let mut json_string = String::new();
    json_file.read_to_string(&mut json_string).await?;
    Ok(json_string)
}

pub async fn get_item_by_id(
    data_dir: &str,
    file_path: &str,
    id: u64,
) -> Result<serde_json::Value, AppError> {
    let json_string = match read_json_from_file(data_dir, file_path).await {
        Ok(s) => s,
        Err(_) => {
            create_empty_json_file(data_dir, file_path).await?;
            String::from("[]")
        }
    };
    let json_value = convert_string_to_json(&json_string)?;
    let arr = json_value
        .as_array()
        .ok_or_else(|| AppError::Internal("expected JSON array".into()))?;

    arr.iter()
        .find(|item| item["id"].as_u64() == Some(id))
        .cloned()
        .ok_or(AppError::ItemNotFound(id))
}

pub async fn create_empty_json_file(data_dir: &str, f: &str) -> Result<(), io::Error> {
    let file_path = format!("{}/{}.json", data_dir, f);
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file_path)
        .await?;
    file.write_all(b"[]").await?;
    Ok(())
}

pub async fn add_item_to_json_file(
    data_dir: &str,
    file_name: &str,
    mut new_item: serde_json::Value,
) -> Result<serde_json::Value, AppError> {
    let json_string = match read_json_from_file(data_dir, file_name).await {
        Ok(s) => s,
        Err(_) => {
            create_empty_json_file(data_dir, file_name).await?;
            String::from("[]")
        }
    };
    let mut json_value = convert_string_to_json(&json_string)?;
    let arr = json_value
        .as_array_mut()
        .ok_or_else(|| AppError::Internal("expected JSON array".into()))?;

    if new_item.get("id").is_none() {
        new_item["id"] = serde_json::Value::from(generate_random_id());
    }

    arr.push(new_item.clone());

    let json_string = serde_json::to_string_pretty(&json_value)?;
    let file_path = format!("{}/{}.json", data_dir, file_name);
    tokio::fs::write(file_path, json_string).await?;

    Ok(new_item)
}

pub async fn update_json_file(
    data_dir: &str,
    f: &str,
    id: u64,
    updated_item: &serde_json::Value,
) -> Result<bool, AppError> {
    let file_path = format!("{}/{}.json", data_dir, f);
    let json_string = match read_json_from_file(data_dir, f).await {
        Ok(s) => s,
        Err(_) => return Ok(false),
    };

    let mut json_value = convert_string_to_json(&json_string)?;
    let arr = json_value
        .as_array_mut()
        .ok_or_else(|| AppError::Internal("expected JSON array".into()))?;

    if let Some(index) = arr.iter().position(|item| item["id"].as_u64() == Some(id)) {
        arr[index] = updated_item.clone();
        let json_string = serde_json::to_string_pretty(&json_value)?;
        tokio::fs::write(file_path, json_string).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn patch_json_file(
    data_dir: &str,
    f: &str,
    id: u64,
    patch: &serde_json::Value,
) -> Result<Option<serde_json::Value>, AppError> {
    let file_path = format!("{}/{}.json", data_dir, f);
    let json_string = match read_json_from_file(data_dir, f).await {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };

    let mut json_value = convert_string_to_json(&json_string)?;
    let arr = json_value
        .as_array_mut()
        .ok_or_else(|| AppError::Internal("expected JSON array".into()))?;

    if let Some(index) = arr.iter().position(|item| item["id"].as_u64() == Some(id)) {
        if let (Some(existing), Some(patch_obj)) =
            (arr[index].as_object_mut(), patch.as_object())
        {
            for (k, v) in patch_obj {
                existing.insert(k.clone(), v.clone());
            }
        }
        let updated = arr[index].clone();
        let json_string = serde_json::to_string_pretty(&json_value)?;
        tokio::fs::write(file_path, json_string).await?;
        Ok(Some(updated))
    } else {
        Ok(None)
    }
}

pub async fn delete_from_json_file(data_dir: &str, f: &str, id: u64) -> Result<bool, AppError> {
    let file_path = format!("{}/{}.json", data_dir, f);
    let json_string = match read_json_from_file(data_dir, f).await {
        Ok(s) => s,
        Err(_) => return Ok(false),
    };

    let json_value = convert_string_to_json(&json_string)?;
    let arr = json_value
        .as_array()
        .ok_or_else(|| AppError::Internal("expected JSON array".into()))?;

    let mut found = false;
    let filtered: Vec<serde_json::Value> = arr
        .iter()
        .filter(|item| {
            if item["id"].as_u64() == Some(id) {
                found = true;
                false
            } else {
                true
            }
        })
        .cloned()
        .collect();

    let json_string = serde_json::to_string_pretty(&filtered)?;
    tokio::fs::write(file_path, json_string).await?;
    Ok(found)
}

pub fn convert_string_to_json(json_string: &str) -> Result<serde_json::Value, serde_json::Error> {
    let json_value: serde_json::Value = serde_json::from_str(json_string)?;
    Ok(json_value)
}

pub async fn delete_collection_sync(data_dir: &str, f: &str) -> Result<(), std::io::Error> {
    let file_path = format!("{}/{}.json", data_dir, f);
    tokio::fs::remove_file(file_path).await?;
    Ok(())
}
