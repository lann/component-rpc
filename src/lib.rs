pub mod openapi;
pub mod serde;
mod type_ext;

use anyhow::{bail, ensure, Context, Result};
use serde_json::Value as JsonValue;
use wasmtime::component::{Type, Val};

pub use type_ext::TypeExt;

pub fn json_to_val(ty: &Type, json: JsonValue) -> Result<Val> {
    use serde_json::from_value;
    Ok(match ty {
        Type::Bool => Val::Bool(from_value(json)?),
        Type::U8 => Val::U8(from_value(json)?),
        Type::U16 => Val::U16(from_value(json)?),
        Type::U32 => Val::U32(from_value(json)?),
        Type::U64 => Val::U64(from_value(json)?),
        Type::S8 => Val::S8(from_value(json)?),
        Type::S16 => Val::S16(from_value(json)?),
        Type::S32 => Val::S32(from_value(json)?),
        Type::S64 => Val::S64(from_value(json)?),
        Type::Float32 => {
            let value = match json.as_str() {
                Some("NaN") => f32::NAN,
                Some("Infinity") => f32::INFINITY,
                Some("-Infinity") => f32::NEG_INFINITY,
                _ => from_value(json)?,
            };
            Val::Float32(value.to_bits())
        }
        Type::Float64 => {
            let value = match json.as_str() {
                Some("NaN") => f64::NAN,
                Some("Infinity") => f64::INFINITY,
                Some("-Infinity") => f64::NEG_INFINITY,
                _ => from_value(json)?,
            };
            Val::Float64(value.to_bits())
        }
        Type::Char => Val::Char(from_value(json)?),
        Type::String => Val::String(from_value(json)?),

        Type::List(list) => {
            let JsonValue::Array(json_array) = json else {
                bail!("Cannot deserialize {json:?} into list");
            };
            let values = json_array
                .into_iter()
                .map(|item| json_to_val(&list.ty(), item))
                .collect::<Result<_>>()?;
            list.new_val(values)?
        }
        Type::Record(record) => {
            let JsonValue::Object(mut json_object) = json else {
                bail!("Cannot deserialize {json:?} into record");
            };
            let values = record
                .fields()
                .map(|field| {
                    let field_value = json_object.remove(field.name).unwrap_or(JsonValue::Null);
                    Ok((field.name, json_to_val(&field.ty, field_value)?))
                })
                .collect::<Result<Vec<_>>>()?;
            record.new_val(values)?
        }
        Type::Tuple(tuple) => {
            let JsonValue::Array(json_array) = json else {
                bail!("Cannot deserialize {json:?} into tuple");
            };
            let json_len = json_array.len();
            let tuple_len = tuple.types().len();
            ensure!(
                json_len == tuple_len,
                "Cannot deserialize list of len {tuple_len} into tuple of len {tuple_len}"
            );

            let values = tuple
                .types()
                .zip(json_array)
                .map(|(ty, json_value)| json_to_val(&ty, json_value))
                .collect::<Result<_>>()?;
            tuple.new_val(values)?
        }
        Type::Variant(variant) => {
            let (key, json_value) =
                get_single_entry_json(json).context("Couldn't deserialize into variant")?;
            let case = variant
                .cases()
                .find(|case| case.name == key)
                .with_context(|| format!("No variant case named {key:?}"))?;
            let value = case.ty.map(|ty| json_to_val(&ty, json_value)).transpose()?;
            variant.new_val(case.name, value)?
        }
        Type::Enum(enum_) => {
            let JsonValue::String(json_string) = json else {
                bail!("Cannot deserialize {json:?} into enum");
            };
            enum_.new_val(&json_string)?
        }
        Type::Union(union) => {
            let (key, json_value) =
                get_single_entry_json(json).context("Couldn't deserialize into union")?;
            let idx: u32 = key
                .parse()
                .with_context(|| format!("Invalid key {key:?} for union"))?;
            let ty = union
                .types()
                .nth(idx as usize)
                .with_context(|| format!("No such union case {idx}"))?;
            let value = json_to_val(&ty, json_value)?;
            union.new_val(idx, value)?
        }
        Type::Option(option) => {
            let value = if json.is_null() {
                None
            } else {
                Some(json_to_val(&option.ty(), json)?)
            };
            option.new_val(value)?
        }
        Type::Result(result) => {
            let (key, json_value) =
                get_single_entry_json(json).context("Couldn't deserialize into result")?;
            let value = match key.as_str() {
                "result" => Ok(result
                    .ok()
                    .map(|ty| json_to_val(&ty, json_value))
                    .transpose()?),
                "error" => Err(result
                    .err()
                    .map(|ty| json_to_val(&ty, json_value))
                    .transpose()?),
                _ => bail!("Invalid key {key:?} for result"),
            };
            result.new_val(value)?
        }
        Type::Flags(flags) => {
            let values: Vec<String> = from_value(json)?;
            let names: Vec<&str> = values.iter().map(|v| v.as_str()).collect();
            flags.new_val(&names)?
        }
    })
}

pub fn val_to_json(val: &Val) -> JsonValue {
    match val {
        &Val::Bool(v) => v.into(),
        &Val::U8(v) => v.into(),
        &Val::U16(v) => v.into(),
        &Val::U32(v) => v.into(),
        &Val::U64(v) => v.into(),
        &Val::S8(v) => v.into(),
        &Val::S16(v) => v.into(),
        &Val::S32(v) => v.into(),
        &Val::S64(v) => v.into(),
        &Val::Float32(v) => {
            let f = f32::from_bits(v);
            if f.is_nan() {
                "NaN".to_string().into()
            } else if f.is_infinite() {
                if f.is_sign_negative() {
                    "Infinity".to_string().into()
                } else {
                    "-Infinity".to_string().into()
                }
            } else {
                f.into()
            }
        }
        &Val::Float64(v) => {
            let f = f64::from_bits(v);
            if f.is_nan() {
                "NaN".to_string().into()
            } else if f.is_infinite() {
                if f.is_sign_negative() {
                    "Infinity".to_string().into()
                } else {
                    "-Infinity".to_string().into()
                }
            } else {
                f.into()
            }
        }
        &Val::Char(v) => v.to_string().into(),
        Val::String(v) => v.to_string().into(),

        Val::List(list) => {
            let vec = list.iter().map(val_to_json).collect();
            JsonValue::Array(vec)
        }
        Val::Record(record) => {
            let map = record
                .fields()
                .map(|(key, val)| (key.to_string(), val_to_json(val)))
                .collect();
            JsonValue::Object(map)
        }
        Val::Tuple(tuple) => {
            let vec = tuple.values().iter().map(val_to_json).collect();
            JsonValue::Array(vec)
        }
        Val::Variant(variant) => {
            let key = variant.discriminant().to_string();
            let value = option_val_to_json(variant.payload());
            JsonValue::Object([(key, value)].into_iter().collect())
        }
        Val::Enum(enum_) => enum_.discriminant().to_string().into(),
        Val::Union(union) => {
            let key = union.discriminant().to_string();
            let value = val_to_json(union.payload());
            JsonValue::Object([(key, value)].into_iter().collect())
        }
        Val::Option(option) => option_val_to_json(option.value()),
        Val::Result(result) => {
            let (key, v) = match result.value() {
                Ok(v) => ("result", v),
                Err(v) => ("error", v),
            };
            let value = option_val_to_json(v);
            JsonValue::Object([(key.to_string(), value)].into_iter().collect())
        }
        Val::Flags(flags) => {
            let values = flags.flags().map(|name| name.to_string().into()).collect();
            JsonValue::Array(values)
        }
    }
}

fn get_single_entry_json(json: JsonValue) -> Result<(String, JsonValue)> {
    let JsonValue::Object(object) = json else {
        bail!("expected object, got {json:?}");
    };
    let len = object.len();
    ensure!(len == 1, "expected one entry, got {len}");
    Ok(object.into_iter().next().unwrap())
}

fn option_val_to_json(val: Option<&Val>) -> JsonValue {
    match val {
        Some(v) => val_to_json(v),
        None => JsonValue::Null,
    }
}
