use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use sonic_rs::{JsonValueTrait, json};
use tracing::info;

use crate::helix_engine::graph_core::ops::tr_val::TraversalVal;
use crate::helix_engine::storage_core::storage_methods::StorageMethods;
use crate::helix_engine::types::GraphError;
use crate::helix_gateway::gateway::AppState;
use crate::helix_gateway::router::router::{Handler, HandlerInput, HandlerSubmission};
use crate::protocol::remapping::RemappingMap;
use crate::protocol::return_values::ReturnValue;
use crate::protocol::{self, request::RequestType};

// get node details by ID
// curl "http://localhost:PORT/node-details?id=YOUR_NODE_ID"

#[derive(Deserialize)]
pub struct NodeDetailsQuery {
    id: String,
}

#[derive(Serialize)]
pub struct NodeDetailsResponse {
    pub node: Option<ReturnValue>,
    pub found: bool,
}

pub async fn node_details_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<NodeDetailsQuery>,
) -> axum::http::Response<Body> {
    let mut req = protocol::request::Request {
        name: "node_details".to_string(),
        req_type: RequestType::Query,
        body: axum::body::Bytes::new(),
        in_fmt: protocol::Format::default(),
        out_fmt: protocol::Format::default(),
    };

    if let Ok(params_json) = sonic_rs::to_vec(&json!({
        "id": params.id
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

pub fn node_details_inner(input: &HandlerInput) -> Result<protocol::Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().map_err(GraphError::from)?;

    let node_id_str = if !input.request.body.is_empty() {
        match sonic_rs::from_slice::<sonic_rs::Value>(&input.request.body) {
            Ok(params) => params
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Err(_) => None,
        }
    } else {
        None
    };

    let node_id_str = node_id_str.ok_or_else(|| GraphError::New("id is required".to_string()))?;

    let node_id = match uuid::Uuid::parse_str(&node_id_str) {
        Ok(uuid) => uuid.as_u128(),
        Err(_) => return Err(GraphError::New("invalid UUID format".to_string())),
    };

    let remapping_vals = RemappingMap::new();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    match db.get_node(&txn, &node_id) {
        Ok(node) => {
            let traversal_val = TraversalVal::Node(node);
            return_vals.insert(
                "node".to_string(),
                ReturnValue::from_traversal_value_with_mixin(
                    traversal_val,
                    remapping_vals.borrow_mut(),
                ),
            );
            return_vals.insert("found".to_string(), ReturnValue::from(true));
        }
        Err(_) => {
            // Node not found
            return_vals.insert("node".to_string(), ReturnValue::Empty);
            return_vals.insert("found".to_string(), ReturnValue::from(false));
        }
    }

    Ok(protocol::Response {
        body: sonic_rs::to_vec(&return_vals).map_err(|e| GraphError::New(e.to_string()))?,
        fmt: Default::default(),
    })
}

inventory::submit! {
    HandlerSubmission(
        Handler::new("node_details", node_details_inner)
    )
}
