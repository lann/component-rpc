use serde::{
    ser::{SerializeMap, SerializeSeq, SerializeTuple},
    Serialize, Serializer,
};
use wasmtime::component::Val;

pub trait ValSerialize {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

impl ValSerialize for Val {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ValWrapper::wrap_ref(self).serialize(serializer)
    }
}

#[repr(transparent)]
struct ValWrapper(Val);

impl ValWrapper {
    fn wrap_ref(val: &Val) -> &Self {
        // Sound because of #[repr(transparent)]
        unsafe { std::mem::transmute(val) }
    }
}

impl Serialize for ValWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            &Val::Bool(v) => serializer.serialize_bool(v),

            &Val::U8(v) => serializer.serialize_u8(v),
            &Val::U16(v) => serializer.serialize_u16(v),
            &Val::U32(v) => serializer.serialize_u32(v),
            &Val::U64(v) => serializer.serialize_u64(v),

            &Val::S8(v) => serializer.serialize_i8(v),
            &Val::S16(v) => serializer.serialize_i16(v),
            &Val::S32(v) => serializer.serialize_i32(v),
            &Val::S64(v) => serializer.serialize_i64(v),

            &Val::Float32(v) => {
                let f = f32::from_bits(v);
                if f.is_finite() {
                    serializer.serialize_f32(f)
                } else {
                    serializer.serialize_str(special_float_str(f as f64))
                }
            }
            &Val::Float64(v) => {
                let f = f64::from_bits(v);
                if f.is_finite() {
                    serializer.serialize_f64(f)
                } else {
                    serializer.serialize_str(special_float_str(f))
                }
            }

            &Val::Char(v) => serializer.serialize_char(v),
            Val::String(v) => serializer.serialize_str(v),

            Val::List(list) => {
                let mut seq = serializer.serialize_seq(Some(list.len()))?;
                for val in list.iter() {
                    seq.serialize_element(Self::wrap_ref(val))?;
                }
                seq.end()
            }

            Val::Record(record) => {
                let mut map = serializer.serialize_map(Some(record.ty().fields().len()))?;
                for (key, val) in record.fields() {
                    map.serialize_entry(key, Self::wrap_ref(val))?;
                }
                map.end()
            }

            Val::Tuple(tuple) => {
                let mut tup = serializer.serialize_tuple(tuple.values().len())?;
                for val in tuple.values() {
                    tup.serialize_element(Self::wrap_ref(val))?;
                }
                tup.end()
            }

            Val::Variant(variant) => {
                let val = variant.payload().map(Self::wrap_ref);
                serializer.collect_map([(variant.discriminant(), &val)])
            }

            Val::Enum(enum_) => serializer.serialize_str(enum_.discriminant()),

            Val::Union(union) => {
                serializer.collect_map([(union.discriminant(), Self::wrap_ref(union.payload()))])
            }

            Val::Option(option) => match option.value() {
                Some(val) => serializer.serialize_some(Self::wrap_ref(val)),
                None => serializer.serialize_none(),
            },

            Val::Result(result) => {
                let (variant, idx, opt_val) = match result.value() {
                    Ok(v) => ("result", 0, v),
                    Err(v) => ("error", 1, v),
                };
                let val = opt_val.map(Self::wrap_ref);
                serializer.serialize_newtype_variant("result", idx, variant, &val)
            }

            Val::Flags(flags) => serializer.collect_seq(flags.flags()),
        }
    }
}

fn special_float_str(f: f64) -> &'static str {
    debug_assert!(!f.is_finite());
    if f.is_nan() {
        "NaN"
    } else if f.is_sign_positive() {
        "Inf"
    } else {
        "-Inf"
    }
}
