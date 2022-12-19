use std::{
    any::{type_name, Any},
    sync::Arc,
};

use anyhow::Context;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value as JsonValue;
use wasmtime::{
    component::{Component, InstancePre, Linker, Val},
    Config, Engine, Store,
};
use wit_component::DocumentPrinter;
use wit_parser::{Document, InterfaceId};

const USAGE: &str = "component-rpc-server <path-to-component>";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    let _ = args.next().unwrap();
    let path = args.next().context(USAGE)?;

    let wasm = std::fs::read(&path).with_context(|| format!("Failed to read {path:?}"))?;

    let (doc, world_id) = wit_component::decode_world("hello-rpc", &wasm)
        .with_context(|| format!("Failed to decode World from {path:?}"))?;

    let doc_str = DocumentPrinter::default().print(&doc)?;
    println!("Parsed world:\n{doc_str}");

    let world = doc
        .worlds
        .get(world_id)
        .expect("Document missing root World");

    let iface_id = world.default.context("World has no default export")?;

    let engine = Engine::new(Config::new().wasm_component_model(true))?;
    let component = Component::new(&engine, &wasm)?;
    let linker = Linker::new(&engine);
    let instance_pre = linker.instantiate_pre(&component)?;

    let state = AppState {
        engine,
        instance_pre,
        doc,
        iface_id,
    };

    let app = Router::new()
        .route("/call", post(call))
        .with_state(Arc::new(state));

    let addr = "0.0.0.0:3456".parse().unwrap();

    println!("Serving on {addr:?}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

struct AppState {
    engine: Engine,
    instance_pre: InstancePre<()>,
    doc: Document,
    iface_id: InterfaceId,
}

#[derive(Deserialize)]
struct CallRequest {
    pub name: String,
    pub args: Vec<JsonValue>,
}

#[derive(Serialize)]
struct CallResponse {
    result: JsonValue,
}

async fn call(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CallRequest>,
) -> Result<Json<CallResponse>, AppError> {
    let interface = state.doc.interfaces.get(state.iface_id).unwrap();

    let func_name = req.name;

    let func_type = interface
        .functions
        .iter()
        .find(|f| f.name == func_name)
        .with_context(|| format!("Component has no default export function named {func_name:?}"))?;

    use wit_parser::Type::*;

    let mut args = req.args.into_iter();

    let mut arg_vals: Vec<Val> = vec![];
    for (name, ty) in &func_type.params {
        let arg_json = args
            .next()
            .with_context(|| format!("Missing argument for parameter {name:?}"))?;

        arg_vals.push(match ty {
            Bool => todo!(),
            U8 => todo!(),
            U16 => todo!(),
            U32 => Val::U32(deserialize_arg(arg_json)?),
            U64 => todo!(),
            S8 => todo!(),
            S16 => todo!(),
            S32 => todo!(),
            S64 => todo!(),
            Float32 => todo!(),
            Float64 => todo!(),
            Char => todo!(),
            String => todo!(),
            Id(_) => todo!(),
        });
    }

    let mut result_vals: Vec<Val> = vec![];
    match func_type.results {
        wit_parser::Results::Named(_) => todo!(),
        wit_parser::Results::Anon(ty) => result_vals.push(match ty {
            Bool => todo!(),
            U8 => todo!(),
            U16 => todo!(),
            U32 => Val::U32(0),
            U64 => todo!(),
            S8 => todo!(),
            S16 => todo!(),
            S32 => todo!(),
            S64 => todo!(),
            Float32 => todo!(),
            Float64 => todo!(),
            Char => todo!(),
            String => todo!(),
            Id(_) => todo!(),
        }),
    }

    let mut store = Store::new(&state.engine, ());
    let instance = state.instance_pre.instantiate(&mut store)?;
    let func = instance
        .get_func(&mut store, &func_name)
        .context("Instance missing function")?;

    func.call(&mut store, &arg_vals, &mut result_vals)?;

    let result = match result_vals[0] {
        Val::Bool(_) => todo!(),
        Val::S8(_) => todo!(),
        Val::U8(_) => todo!(),
        Val::S16(_) => todo!(),
        Val::U16(_) => todo!(),
        Val::S32(_) => todo!(),
        Val::U32(v) => v.into(),
        Val::S64(_) => todo!(),
        Val::U64(_) => todo!(),
        Val::Float32(_) => todo!(),
        Val::Float64(_) => todo!(),
        Val::Char(_) => todo!(),
        Val::String(_) => todo!(),
        Val::List(_) => todo!(),
        Val::Record(_) => todo!(),
        Val::Tuple(_) => todo!(),
        Val::Variant(_) => todo!(),
        Val::Enum(_) => todo!(),
        Val::Union(_) => todo!(),
        Val::Option(_) => todo!(),
        Val::Result(_) => todo!(),
        Val::Flags(_) => todo!(),
    };

    Ok(Json(CallResponse { result }))
}

fn deserialize_arg<T: Any + DeserializeOwned>(arg: JsonValue) -> anyhow::Result<T> {
    serde_json::from_value(arg).with_context(|| {
        let type_name = type_name::<T>();
        format!("Failed to parse arg as a {type_name:?}")
    })
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
