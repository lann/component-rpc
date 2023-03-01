use anyhow::{Context, Result};
use serde_json::{json, Value};
use wit_parser::{Document, Function, Type};

pub fn build_openapi_doc(doc: &Document) -> Result<Value> {
    let world = &doc.worlds[doc.default_world()?];
    let iface = &doc.interfaces[world.default.context("no default interface")?];
    let paths = Value::Object(
        iface
            .functions
            .iter()
            .map(build_openapi_path)
            .collect::<Result<_>>()?,
    );

    let component_schemas = Value::Object(
        doc.types
            .iter()
            .map(|(id, def)| (format!("typedef-{}", id.index()), json!({})))
            .collect(),
    );

    Ok(json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Component RPC Server",
            "version": "0.0.0-dynamic",
        },
        "paths": paths,
        "components": {
            "schemas": component_schemas,
        },
    }))
}

fn build_openapi_path(func: &Function) -> Result<(String, Value)> {
    let request_params: Vec<_> = func.params.iter().map(|(_, ty)| ty).collect();

    let response_properties = match func.results.len() {
        0 => json!({}),
        1 => {
            let ty = func.results.iter_types().next().unwrap();

            json!({"result": 1})
        }
        _ => json!({"results": 2}),
    };

    let item = json!({
        "post": {
            "requestBody": {
                "content": {
                    "application/json": {
                        "schema": {
                            "type": "object",
                            "properties": {
                                "params": build_tuple_schema(&request_params),
                            }
                        }
                    }
                }
            },
            "responses": {
                "default": {
                    "content": {
                        "application/json": {
                            "schema": {
                                "type": "object",
                                "properties": response_properties,
                            }
                        }
                    }
                }
            }
        }
    });
    Ok((format!("/call/{}", func.name), item))
}

fn build_tuple_schema(types: &[&Type]) -> Value {
    let items: Vec<Value> = types.iter().map(|ty| build_type_schema(ty)).collect();
    json!({
        "type": "array",
        "minItems": items.len(),
        "maxItems": items.len(),
        "prefixItems": items,
    })
}

fn build_type_schema(ty: &Type) -> Value {
    json!({})
}
