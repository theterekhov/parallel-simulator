use axum::{
    Json, Router,
    extract::Path,
    http::StatusCode,
    routing::{get, post},
};
use serde_json::Value;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, services::ServeDir};

const TASKS_DIR: &str = "tasks";

async fn list_tasks() -> Result<Json<Vec<String>>, StatusCode> {
    let mut entries = tokio::fs::read_dir(TASKS_DIR).await.map_err(|e| {
        tracing::error!("Не удалось прочитать папку tasks/: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut tasks_ids = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                tasks_ids.push(stem.to_string());
            }
        }
    }

    Ok(Json(tasks_ids))
}

async fn get_task(Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    if id.contains("..") || id.contains("/") {
        tracing::warn!("Подозрительный id: \"{id}\"");
        return Err(StatusCode::BAD_REQUEST);
    }

    let file_path = format!("{TASKS_DIR}/{id}.json");

    let content = tokio::fs::read_to_string(&file_path).await.map_err(|_| {
        tracing::warn!("Файл не найден: {file_path}");
        StatusCode::NOT_FOUND
    })?;

    let json = serde_json::from_str(&content).map_err(|e| {
        tracing::error!("Невалидный JSON: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(json))
}

async fn validate_task(Json(payload): Json<Value>) -> Result<StatusCode, StatusCode> {
    let schema_str = tokio::fs::read_to_string("task.schema.json")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let schema_json =
        serde_json::from_str(&schema_str).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let compiled_schema = jsonschema::JSONSchema::compile(&schema_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Err(errors) = compiled_schema.validate(&payload) {
        for error in errors {
            tracing::warn!("Ошибка валидации: {}", error);
        }

        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(StatusCode::OK)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/tasks", get(list_tasks))
        .route("/api/tasks/{id}", get(get_task))
        .route("/api/tasks/validate", post(validate_task))
        .layer(CorsLayer::permissive())
        .fallback_service(ServeDir::new("crates/frontend/dist"));

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Не удалось привязаться к порту 3000");

    tracing::info!("Сервер запущен: http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("Ошибка при работе сервера")
}
