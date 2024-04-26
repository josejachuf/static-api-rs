use crate::error::AppResult;
use crate::utils::{
    add_item_to_json_file, convert_string_to_json, create_empty_json_file, delete_from_json_file,
    get_item_by_id, read_json_from_file, update_json_file,
};
use crate::AppConfig;
use salvo::http::StatusCode;
use salvo::prelude::*;

use serde::Serialize;

#[derive(Serialize)]
struct ApiResponse {
    data: Vec<serde_json::Value>,
    total: usize,
    limit: usize,
    skip: usize,
}

#[handler]
pub async fn get_all(req: &mut Request, depot: &mut Depot) -> AppResult<Json<ApiResponse>> {
    let app_config = depot.obtain::<AppConfig>().unwrap().clone();
    let data_dir = &app_config.data_dir;
    let file_path = req.param::<String>("f").unwrap();
    let limit = req.query::<usize>("limit").unwrap_or(30);
    let skip = req.query::<usize>("skip").unwrap_or(0);

    let json_string = match read_json_from_file(data_dir, &file_path).await {
        Ok(s) => s,
        Err(_) => {
            create_empty_json_file(data_dir, &file_path).await?;
            String::from("[]")
        }
    };

    let json_value = convert_string_to_json(&json_string)?;
    let total_records = json_value.as_array().unwrap().len();

    let limited_json_value: Vec<&serde_json::Value> = json_value
        .as_array()
        .unwrap()
        .iter()
        .skip(skip)
        .take(limit)
        .collect();

    let api_response = ApiResponse {
        data: limited_json_value.into_iter().cloned().collect(),
        total: total_records,
        limit,
        skip,
    };

    Ok(Json(api_response))
}

#[handler]
pub async fn get_one(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> AppResult<Json<serde_json::Value>> {
    let app_config = depot.obtain::<AppConfig>().unwrap().clone();
    let data_dir = &app_config.data_dir;
    let file_path = req.param::<String>("f").unwrap();
    let id = req.param::<u64>("id").unwrap();

    let result = get_item_by_id(data_dir, &file_path, id).await;

    match result {
        Ok(json_value) => Ok(Json(json_value)),
        Err(_) => {
            res.status_code(StatusCode::NOT_FOUND);
            Ok(Json(serde_json::json!({})))
        }
    }
}

#[handler]
pub async fn add_one(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> AppResult<Json<serde_json::Value>> {
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
) -> AppResult<Json<serde_json::Value>> {
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
) -> AppResult<Json<serde_json::Value>> {
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
