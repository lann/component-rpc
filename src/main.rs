use std::any::{type_name, Any};

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use wasmtime::{
    component::{Component, Linker, Val},
    Config, Engine, Store,
};
use wit_component::DocumentPrinter;

const USAGE: &str = "component-rpc <path>";

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args();

    let _ = args.next().unwrap();
    let path = args.next().context(USAGE)?;
    let func_name = args.next().context(USAGE)?;

    let wasm =
        std::fs::read("examples/hello-rpc/target/wasm32-unknown-unknown/release/hello_rpc.wasm")
            .with_context(|| format!("Failed to read {path:?}"))?;

    let (doc, world_id) = wit_component::decode_world("hello-rpc", &wasm)
        .with_context(|| format!("Failed to decode World from {path:?}"))?;

    let world = doc
        .worlds
        .get(world_id)
        .expect("Document missing root World");

    let id = world.default.context("World has no default export")?;
    let interface = doc.interfaces.get(id).unwrap();

    let func_type = interface
        .functions
        .iter()
        .find(|f| f.name == func_name)
        .with_context(|| format!("Component has no default export function named {func_name:?}"))?;

    let doc_str = DocumentPrinter::default().print(&doc)?;
    println!("Parsed world:\n{doc_str}");

    use wit_parser::Type::*;

    let mut arg_vals: Vec<Val> = vec![];
    for (name, ty) in &func_type.params {
        let arg_str = args
            .next()
            .with_context(|| format!("Missing argument for parameter {name:?}"))?;

        arg_vals.push(match *ty {
            Bool => todo!(),
            U8 => todo!(),
            U16 => todo!(),
            U32 => Val::U32(deserialize_arg(&arg_str)?),
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

    let engine = Engine::new(Config::new().wasm_component_model(true))?;
    let component = Component::new(&engine, &wasm)?;
    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &component)?;
    let func = instance
        .get_func(&mut store, &func_name)
        .context("Instance missing function")?;

    println!("Calling with {arg_vals:?}");

    func.call(&mut store, &arg_vals, &mut result_vals)?;

    println!("Results: {result_vals:?}");

    Ok(())
}

fn deserialize_arg<T: Any + DeserializeOwned>(arg: &str) -> Result<T> {
    serde_json::from_str(arg).with_context(|| {
        let type_name = type_name::<T>();
        format!("Failed to parse {arg:?} as a {type_name:?}")
    })
}
