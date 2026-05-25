use crate::error::{AppError, AppResult};
use crate::utils::{
    add_item_to_json_file, convert_string_to_json, create_empty_json_file, delete_from_json_file,
    get_item_by_id, patch_json_file, read_json_from_file, update_json_file,
};
use crate::AppConfig;
use salvo::http::StatusCode;
use salvo::prelude::*;
use std::cmp::Ordering;

use serde::Serialize;

#[derive(Serialize)]
struct ApiResponse {
    data: Vec<serde_json::Value>,
    total: usize,
    limit: usize,
    skip: usize,
}

fn compare_json_values(a: &serde_json::Value, b: &serde_json::Value) -> Ordering {
    match (a, b) {
        (serde_json::Value::Number(na), serde_json::Value::Number(nb)) => na
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&nb.as_f64().unwrap_or(0.0))
            .unwrap_or(Ordering::Equal),
        (serde_json::Value::String(sa), serde_json::Value::String(sb)) => sa.cmp(sb),
        _ => Ordering::Equal,
    }
}

#[handler]
pub async fn get_all(req: &mut Request, depot: &mut Depot) -> AppResult<Json<ApiResponse>> {
    let app_config = depot
        .obtain::<AppConfig>()
        .map_err(|_| AppError::Internal("missing app config".into()))?
        .clone();
    let data_dir = &app_config.data_dir;
    let file_path = req
        .param::<String>("f")
        .ok_or_else(|| AppError::Internal("missing param f".into()))?;
    let limit = req.query::<usize>("limit").unwrap_or(30);
    let skip = req.query::<usize>("skip").unwrap_or(0);
    let sort_field = req.query::<String>("_sort");
    let sort_order = req
        .query::<String>("_order")
        .unwrap_or_else(|| "asc".to_string());

    let json_string = match read_json_from_file(data_dir, &file_path).await {
        Ok(s) => s,
        Err(_) => {
            create_empty_json_file(data_dir, &file_path).await?;
            String::from("[]")
        }
    };

    let json_value = convert_string_to_json(&json_string)?;
    let arr = json_value
        .as_array()
        .ok_or_else(|| AppError::Internal("expected JSON array".into()))?;

    // Collect filter params from query string (exclude known params)
    let raw_query = req.uri().query().unwrap_or("");
    let filter_params: Vec<(String, String)> = raw_query
        .split('&')
        .filter_map(|pair| {
            if pair.is_empty() {
                return None;
            }
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.to_string();
            let val = parts.next().unwrap_or("").to_string();
            if matches!(key.as_str(), "limit" | "skip" | "_sort" | "_order") {
                None
            } else {
                Some((key, val))
            }
        })
        .collect();

    // Filter items
    let mut filtered: Vec<&serde_json::Value> = arr
        .iter()
        .filter(|item| {
            filter_params.iter().all(|(field, expected)| match &item[field] {
                serde_json::Value::String(s) => s == expected,
                serde_json::Value::Number(n) => n.to_string() == *expected,
                serde_json::Value::Bool(b) => b.to_string() == *expected,
                _ => false,
            })
        })
        .collect();

    // Sort items
    if let Some(ref field) = sort_field {
        filtered.sort_by(|a, b| {
            let ord = compare_json_values(&a[field], &b[field]);
            if sort_order == "desc" { ord.reverse() } else { ord }
        });
    }

    let total = filtered.len();
    let data: Vec<serde_json::Value> = filtered
        .into_iter()
        .skip(skip)
        .take(limit)
        .cloned()
        .collect();

    Ok(Json(ApiResponse {
        data,
        total,
        limit,
        skip,
    }))
}

#[handler]
pub async fn get_one(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> AppResult<Json<serde_json::Value>> {
    let app_config = depot
        .obtain::<AppConfig>()
        .map_err(|_| AppError::Internal("missing app config".into()))?
        .clone();
    let data_dir = &app_config.data_dir;
    let file_path = req
        .param::<String>("f")
        .ok_or_else(|| AppError::Internal("missing param f".into()))?;
    let id = req
        .param::<u64>("id")
        .ok_or_else(|| AppError::Internal("missing or invalid param id".into()))?;

    match get_item_by_id(data_dir, &file_path, id).await {
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
    let app_config = depot
        .obtain::<AppConfig>()
        .map_err(|_| AppError::Internal("missing app config".into()))?
        .clone();
    let data_dir = &app_config.data_dir;
    let file_path = req
        .param::<String>("f")
        .ok_or_else(|| AppError::Internal("missing param f".into()))?;

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
    let app_config = depot
        .obtain::<AppConfig>()
        .map_err(|_| AppError::Internal("missing app config".into()))?
        .clone();
    let data_dir = &app_config.data_dir;
    let file_path = req
        .param::<String>("f")
        .ok_or_else(|| AppError::Internal("missing param f".into()))?;
    let id = req
        .param::<u64>("id")
        .ok_or_else(|| AppError::Internal("missing or invalid param id".into()))?;

    let updated_item_json = req.parse_body::<serde_json::Value>().await?;
    let found = update_json_file(data_dir, &file_path, id, &updated_item_json).await?;

    if found {
        Ok(Json(updated_item_json))
    } else {
        res.status_code(StatusCode::NOT_FOUND);
        Ok(Json(serde_json::json!({})))
    }
}

#[handler]
pub async fn patch_one(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> AppResult<Json<serde_json::Value>> {
    let app_config = depot
        .obtain::<AppConfig>()
        .map_err(|_| AppError::Internal("missing app config".into()))?
        .clone();
    let data_dir = &app_config.data_dir;
    let file_path = req
        .param::<String>("f")
        .ok_or_else(|| AppError::Internal("missing param f".into()))?;
    let id = req
        .param::<u64>("id")
        .ok_or_else(|| AppError::Internal("missing or invalid param id".into()))?;

    let patch_json = req.parse_body::<serde_json::Value>().await?;

    match patch_json_file(data_dir, &file_path, id, &patch_json).await? {
        Some(updated) => Ok(Json(updated)),
        None => {
            res.status_code(StatusCode::NOT_FOUND);
            Ok(Json(serde_json::json!({})))
        }
    }
}

#[handler]
pub async fn delete_one(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> AppResult<Json<serde_json::Value>> {
    let app_config = depot
        .obtain::<AppConfig>()
        .map_err(|_| AppError::Internal("missing app config".into()))?
        .clone();
    let data_dir = &app_config.data_dir;
    let file_path = req
        .param::<String>("f")
        .ok_or_else(|| AppError::Internal("missing param f".into()))?;
    let id = req
        .param::<u64>("id")
        .ok_or_else(|| AppError::Internal("missing or invalid param id".into()))?;

    let found = delete_from_json_file(data_dir, &file_path, id).await?;
    if found {
        res.status_code(StatusCode::NO_CONTENT);
    } else {
        res.status_code(StatusCode::NOT_FOUND);
    }

    Ok(Json(serde_json::json!({})))
}
