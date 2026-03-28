use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Debug, Default, Serialize)]
struct Stats {
    dicom_ingested: u64,
    segmentations: u64,
    reconstructions: u64,
    windowing_ops: u64,
    total_requests: u64,
}

type AppState = Arc<Mutex<Stats>>;

// --- request / response types ---

#[derive(Debug, Deserialize)]
struct DicomRequest {
    series_uid: String,
    modality: String,
    slice_count: u32,
}

#[derive(Debug, Deserialize)]
struct SegmentRequest {
    series_uid: String,
    targets: Vec<String>,
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReconstructRequest {
    series_uid: String,
    iso_value: Option<f32>,
    output_format: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WindowingRequest {
    series_uid: String,
    preset: String,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T: Serialize> {
    ok: bool,
    request_id: String,
    data: T,
}

fn ok<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        ok: true,
        request_id: Uuid::new_v4().to_string(),
        data,
    })
}

// --- handlers ---

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "alice-medical-saas-core" }))
}

async fn medical_dicom(
    State(state): State<AppState>,
    Json(req): Json<DicomRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.dicom_ingested += 1;
    s.total_requests += 1;
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "series_uid": req.series_uid,
            "modality": req.modality,
            "slice_count": req.slice_count,
            "ingested": true,
            "voxel_spacing_mm": [0.75, 0.75, 1.5],
        })),
    )
}

async fn medical_segment(
    State(state): State<AppState>,
    Json(req): Json<SegmentRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.segmentations += 1;
    s.total_requests += 1;
    let model = req.model.unwrap_or_else(|| "totalsegmentator-v2".to_string());
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "series_uid": req.series_uid,
            "model": model,
            "targets": req.targets,
            "mask_id": Uuid::new_v4().to_string(),
            "dice_score": 0.92,
        })),
    )
}

async fn medical_reconstruct(
    State(state): State<AppState>,
    Json(req): Json<ReconstructRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.reconstructions += 1;
    s.total_requests += 1;
    let fmt = req.output_format.unwrap_or_else(|| "stl".to_string());
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "series_uid": req.series_uid,
            "iso_value": req.iso_value.unwrap_or(300.0),
            "output_format": fmt,
            "mesh_id": Uuid::new_v4().to_string(),
            "triangle_count": 84_200,
        })),
    )
}

async fn medical_windowing(
    State(state): State<AppState>,
    Json(req): Json<WindowingRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.windowing_ops += 1;
    s.total_requests += 1;
    let (wl, ww) = match req.preset.as_str() {
        "bone" => (300, 1500),
        "lung" => (-600, 1500),
        "brain" => (40, 80),
        _ => (60, 400),
    };
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "series_uid": req.series_uid,
            "preset": req.preset,
            "window_level": wl,
            "window_width": ww,
        })),
    )
}

async fn medical_stats(State(state): State<AppState>) -> Json<ApiResponse<serde_json::Value>> {
    let s = state.lock().unwrap();
    ok(serde_json::json!({
        "dicom_ingested": s.dicom_ingested,
        "segmentations": s.segmentations,
        "reconstructions": s.reconstructions,
        "windowing_ops": s.windowing_ops,
        "total_requests": s.total_requests,
    }))
}

// --- main ---

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let state: AppState = Arc::new(Mutex::new(Stats::default()));

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/medical/dicom", post(medical_dicom))
        .route("/api/v1/medical/segment", post(medical_segment))
        .route("/api/v1/medical/reconstruct", post(medical_reconstruct))
        .route("/api/v1/medical/windowing", post(medical_windowing))
        .route("/api/v1/medical/stats", get(medical_stats))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8118".to_string())
        .parse()
        .unwrap_or(8118);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("alice-medical-saas-core listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
