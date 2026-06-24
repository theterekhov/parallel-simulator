use axum::{
    Json, Router,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::Serialize;
use serde_json::Value;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, services::ServeDir};

const TASKS_DIR: &str = "tasks";

#[derive(Serialize)]
struct ValidationError {
    errors: Vec<String>,
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

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

        if path.extension().and_then(|ext| ext.to_str()) == Some("json")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            tasks_ids.push(stem.to_string());
        }
    }

    Ok(Json(tasks_ids))
}

async fn get_task(Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    let id = id.replace('\\', "/");
    if id.contains("..") || id.contains('/') || id.contains('\0') {
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

async fn validate_task(
    Json(payload): Json<Value>,
) -> Result<Json<Value>, axum::response::Response> {
    let schema_str = tokio::fs::read_to_string("task.schema.json")
        .await
        .map_err(|e| {
            tracing::error!("Не удалось прочитать task.schema.json: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

    let schema_json = serde_json::from_str(&schema_str).map_err(|e| {
        tracing::error!("Не удалось распарсить schema JSON: {e}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })?;

    let compiled_schema = jsonschema::JSONSchema::compile(&schema_json).map_err(|e| {
        tracing::error!("Не удалось скомпилировать JSON Schema: {e}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })?;

    if let Err(errors) = compiled_schema.validate(&payload) {
        let details: Vec<String> = errors.map(|e| e.to_string()).collect();
        for error in &details {
            tracing::warn!("Ошибка валидации: {error}");
        }

        return Err((
            StatusCode::BAD_REQUEST,
            Json(ValidationError { errors: details }),
        )
            .into_response());
    }

    Ok(Json(serde_json::json!({"status": "ok"})))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/tasks", get(list_tasks))
        .route("/api/tasks/validate", post(validate_task))
        .route("/api/tasks/{id}", get(get_task))
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
