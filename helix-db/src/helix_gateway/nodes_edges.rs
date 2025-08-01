use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;
use sonic_rs::{json, JsonValueTrait};
use tracing::info;

use crate::helix_gateway::gateway::AppState;
use crate::helix_gateway::router::router::{Handler, HandlerSubmission, HandlerInput};
use crate::helix_engine::storage_core::graph_visualization::GraphVisualization;
use crate::helix_engine::types::GraphError;
use crate::protocol::{self, request::RequestType};

#[derive(Deserialize)]
pub struct NodesEdgesQuery {
    limit: Option<usize>,
    node_label: Option<String>,
}

pub async fn nodes_edges_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<NodesEdgesQuery>,
) -> axum::http::Response<Body> {
    let mut req = protocol::request::Request {
        name: "nodes_edges".to_string(),
        req_type: RequestType::Query,
        body: axum::body::Bytes::new(),
        in_fmt: protocol::Format::default(),
        out_fmt: protocol::Format::default(),
    };

    if let Ok(params_json) = sonic_rs::to_vec(&json!({
        "limit": params.limit,
        "node_label": params.node_label
    })) {
        req.body = axum::body::Bytes::from(params_json);
    }

    let res = state.worker_pool.process(req).await;

    match res {
        Ok(r) => r.into_response(),
        Err(e) => {
            info!(?e, "Got error");
            e.into_response()
        }
    }
}

pub fn nodes_edges_inner(input: &HandlerInput) -> Result<protocol::Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().map_err(GraphError::from)?;

    let (limit, node_label) = if !input.request.body.is_empty() {
        match sonic_rs::from_slice::<sonic_rs::Value>(&input.request.body) {
            Ok(params) => (
                params.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize),
                params.get("node_label").and_then(|v| v.as_str()).map(|s| s.to_string()),
            ),
            Err(_) => (None, None),
        }
    } else {
        (None, None)
    };

    let json_result = db.nodes_edges_to_json(&txn, limit, node_label)?;
    let db_stats = db.get_db_stats_json(&txn)?;

    let combined = format!(
        r#"{{"data": {}, "stats": {}}}"#,
        json_result,
        db_stats
    );

    Ok(protocol::Response {
        body: combined.into_bytes(),
        fmt: Default::default(),
    })
}

inventory::submit! {
    HandlerSubmission(
        Handler::new("nodes_edges", nodes_edges_inner)
    )
}