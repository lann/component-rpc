#![cfg(test)]

use anyhow::Result;
use serde_json::json;
use wasmtime::component::Val;

use super::ValSerialize;

#[test]
fn simple_types_to_json() -> Result<()> {
    use Val::*;
    for (val, want) in [
        (Bool(true), json!(true)),
        (U8(200), json!(200)),
        (U16(0xffff), json!(0xffff)),
        (U32(12345678), json!(12345678)),
        (U64(0xffffffff), json!(4294967295u64)),
        (S8(-100), json!(-100)),
        (S16(-0x7ffe), json!(-0x7ffe)),
        (S32(12345678), json!(12345678)),
        (S64(-0x7ffffffe), json!(-0x7ffffffe)),
        (Char('☃'), json!("☃")),
        (String("abracadabra".into()), json!("abracadabra")),
    ] {
        let got = val.serialize(serde_json::value::Serializer)?;
        assert_eq!(got, want, "val: {val:?}");
    }
    Ok(())
}
