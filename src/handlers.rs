use crate::utils::{
    convert_string_to_json, create_empty_json_file, delete_from_json_file,
    read_json_from_file, add_item_to_json_file, update_json_file,
};
use crate::AppConfig;
use salvo::http::StatusCode;
use salvo::prelude::*;

#[handler]
pub async fn get_all(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, anyhow::Error> {
    let app_config = depot.obtain::<AppConfig>().unwrap().clone();
    let data_dir = &app_config.data_dir;
    let file_path = req.param::<String>("f").unwrap();

    let json_string = match read_json_from_file(&data_dir, &file_path).await {
        Ok(s) => s,
        Err(_) => {
            create_empty_json_file(&data_dir, &file_path).await?;
            String::from("[]")
        }
    };

    let json_value = convert_string_to_json(&json_string)?;
    Ok(Json(json_value))
}

#[handler]
pub async fn get_one(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, anyhow::Error> {
    let app_config = depot.obtain::<AppConfig>().unwrap().clone();
    let data_dir = &app_config.data_dir;

    let file_path = req.param::<String>("f").unwrap();
    let id = req.param::<u64>("id").unwrap();

    let json_string = match read_json_from_file(data_dir, &file_path).await {
        Ok(s) => s,
        Err(_) => {
            create_empty_json_file(data_dir, &file_path).await?;
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
        res.status_code(StatusCode::NOT_FOUND);
        Ok(Json(serde_json::json!({})))
    }
}

#[handler]
pub async fn add_one(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, anyhow::Error> {
    let app_config = depot.obtain::<AppConfig>().unwrap().clone();
    let data_dir = &app_config.data_dir;
    let file_path = req.param::<String>("f").unwrap();

    let new_item_json = req.parse_body::<serde_json::Value>().await?;

    let result = add_item_to_json_file(data_dir, &file_path, new_item_json).await?;
    res.status_code(StatusCode::CREATED);
    Ok(Json(result))
}

#[handler]
pub async fn update_one(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, anyhow::Error> {
    let app_config = depot.obtain::<AppConfig>().unwrap().clone();
    let data_dir = &app_config.data_dir;
    let file_path = req.param::<String>("f").unwrap();
    let id = req.param::<u64>("id").unwrap();

    let updated_item_json = req.parse_body::<serde_json::Value>().await?;

    let found_item = update_json_file(data_dir, &file_path, id, &updated_item_json).await?;

    if found_item {
        Ok(Json(updated_item_json))
    } else {
        res.status_code(StatusCode::NOT_FOUND);
        Ok(Json(serde_json::json!({})))
    }
}

#[handler]
pub async fn delete_one(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, anyhow::Error> {
    let app_config = depot.obtain::<AppConfig>().unwrap().clone();
    let data_dir = &app_config.data_dir;

    let file_path = req.param::<String>("f").unwrap();
    let id = req.param::<u64>("id").unwrap();

    let found_item = delete_from_json_file(data_dir, &file_path, id).await?;
    if found_item {
        res.status_code(StatusCode::NO_CONTENT);
    } else {
        res.status_code(StatusCode::NOT_FOUND);
    }

    Ok(Json(serde_json::json!({})))
}
