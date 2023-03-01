use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use wasmtime::{
    component::{Component, InstancePre, Linker, Val},
    Config, Engine, Store,
};
use wit_parser::Document;

use component_rpc::{json_to_val, openapi::build_openapi_doc, val_to_json, TypeExt};

const USAGE: &str = "component-rpc-server <path-to-component>";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    let _ = args.next().unwrap();
    let path = args.next().context(USAGE)?;

    let wasm = std::fs::read(&path).with_context(|| format!("Error reading {path:?}"))?;

    let (doc, _) = wit_component::decode_world("", &wasm)?;

    let engine = Engine::new(Config::new().wasm_component_model(true))?;
    let component = Component::new(&engine, &wasm)
        .with_context(|| format!("Error loading component from {path:?}"))?;
    let linker = Linker::new(&engine);
    let instance_pre = linker.instantiate_pre(&component)?;

    let state = AppState {
        doc,
        engine,
        instance_pre,
    };

    let app = Router::new()
        .route("/call", post(call))
        .route("/openapi.json", get(openapi))
        .with_state(Arc::new(state));

    let addr = "0.0.0.0:3456".parse().unwrap();

    println!("Serving on {addr:?}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

struct AppState {
    doc: Document,
    engine: Engine,
    instance_pre: InstancePre<()>,
}

#[derive(Deserialize)]
struct CallRequest {
    pub name: String,
    pub args: Vec<JsonValue>,
}

async fn call(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CallRequest>,
) -> Result<Json<JsonValue>, AppError> {
    let func_name = req.name;

    let mut store = Store::new(&state.engine, ());
    let instance = state.instance_pre.instantiate(&mut store)?;

    let func = instance
        .exports(&mut store)
        .root()
        .func(&func_name)
        .with_context(|| format!("No such export {func_name:?}"))?;

    let mut args = req.args.into_iter();

    let mut arg_vals: Vec<Val> = vec![];
    for (idx, param_type) in func.params(&store).iter().enumerate() {
        let arg_json = args
            .next()
            .with_context(|| format!("Missing argument for parameter {idx}"))?;

        arg_vals.push(json_to_val(param_type, arg_json)?);
    }

    let mut result_vals = func
        .results(&store)
        .into_vec()
        .into_iter()
        .map(|ty| ty.default_val())
        .collect::<anyhow::Result<Vec<_>>>()?;

    let func = instance
        .get_func(&mut store, &func_name)
        .context("Instance missing function")?;

    func.call(&mut store, &arg_vals, &mut result_vals)?;

    Ok(Json(results_to_json(&result_vals)))
}

async fn openapi(State(state): State<Arc<AppState>>) -> Result<Json<JsonValue>, AppError> {
    let openapi = build_openapi_doc(&state.doc)?;
    Ok(Json(openapi))
}

fn results_to_json(result_vals: &[Val]) -> JsonValue {
    if result_vals.is_empty() {
        json!({})
    } else if result_vals.len() == 1 {
        let val = &result_vals[0];
        let json = val_to_json(val);
        if let Val::Result(_) = val {
            return json;
        }
        json!({ "result": json })
    } else {
        let results: Vec<_> = result_vals.iter().map(val_to_json).collect();
        json!({ "results": results })
    }
}

struct AppError(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        println!("Error: {:?}", self.0);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
