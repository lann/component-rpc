use std::{
    collections::{BTreeMap, HashMap},
    num::TryFromIntError,
};

use serde::{
    de::{DeserializeSeed, Error as DeError, IgnoredAny, MapAccess, Unexpected, Visitor},
    Deserializer,
};
use wasmtime::component::{Type, Val};

use crate::TypeExt;

pub fn deserialize_val<'de, D: Deserializer<'de>>(
    ty: Type,
    deserializer: D,
) -> Result<Val, D::Error> {
    deserializer.deserialize_any(TypeWrapper(ty))
}

#[derive(Clone)]
pub struct TypeWrapper(Type);

impl TypeWrapper {
    fn is_option(&self) -> bool {
        matches!(self.0, Type::Option(_))
    }

    fn visit_option<V, E: DeError>(
        self,
        inner: fn(TypeWrapper, V) -> Result<Val, E>,
        v: V,
    ) -> Result<Val, E> {
        let option = self.0.unwrap_option();
        let visitor = TypeWrapper(option.ty());
        let val = inner(visitor, v)?;
        option.new_val(Some(val)).map_err(DeError::custom)
    }

    fn try_int_from_int<T, V, E>(&self, v: V, unexpected: Unexpected<'static>) -> Result<T, E>
    where
        T: TryFrom<V, Error = TryFromIntError>,
        E: DeError,
    {
        v.try_into()
            .map_err(|_| DeError::invalid_value(unexpected, self))
    }
}

impl<'de> DeserializeSeed<'de> for TypeWrapper {
    type Value = Val;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        match self.0 {
            Type::Bool => deserializer.deserialize_bool(self),
            Type::U8 => deserializer.deserialize_u8(self),
            Type::U16 => deserializer.deserialize_u16(self),
            Type::U32 => deserializer.deserialize_u32(self),
            Type::U64 => deserializer.deserialize_u64(self),
            Type::S8 => deserializer.deserialize_i8(self),
            Type::S16 => deserializer.deserialize_i16(self),
            Type::S32 => deserializer.deserialize_i32(self),
            Type::S64 => deserializer.deserialize_i64(self),
            Type::Float32 => deserializer.deserialize_f32(self),
            Type::Float64 => deserializer.deserialize_f64(self),
            Type::Char => deserializer.deserialize_char(self),
            Type::String => deserializer.deserialize_string(self),
            Type::List(_) => deserializer.deserialize_seq(self),
            Type::Record(_) => deserializer.deserialize_map(self),
            Type::Tuple(ref tuple) => {
                let len = tuple.types().len();
                deserializer.deserialize_tuple(len, self)
            }
            Type::Variant(_) => deserializer.deserialize_map(self),
            Type::Enum(_) => deserializer.deserialize_str(self),
            Type::Union(_) => deserializer.deserialize_map(self),
            Type::Option(_) => deserializer.deserialize_option(self),
            Type::Result(_) => deserializer.deserialize_enum("result", &["result", "error"], self),
            Type::Flags(_) => deserializer.deserialize_seq(self),
        }
    }
}

impl<'de> Visitor<'de> for TypeWrapper {
    type Value = Val;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a {}", self.0.desc())
    }

    fn visit_bool<E>(self, v: bool) -> Result<Val, E>
    where
        E: serde::de::Error,
    {
        if self.is_option() {
            return self.visit_option(Self::visit_bool, v);
        }
        match self.0 {
            Type::Bool => Ok(Val::Bool(v)),
            _ => Err(DeError::invalid_type(Unexpected::Bool(v), &self)),
        }
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if self.is_option() {
            return self.visit_option(Self::visit_i64, v);
        }
        let unexpected = Unexpected::Signed(v);
        Ok(match self.0 {
            Type::U8 => Val::U8(self.try_int_from_int(v, unexpected)?),
            Type::U16 => Val::U16(self.try_int_from_int(v, unexpected)?),
            Type::U32 => Val::U32(self.try_int_from_int(v, unexpected)?),
            Type::U64 => Val::U64(self.try_int_from_int(v, unexpected)?),
            Type::S8 => Val::S8(self.try_int_from_int(v, unexpected)?),
            Type::S16 => Val::S16(self.try_int_from_int(v, unexpected)?),
            Type::S32 => Val::S32(self.try_int_from_int(v, unexpected)?),
            Type::S64 => Val::S64(v),
            Type::Float32 => {
                let i: i16 = self.try_int_from_int(v, unexpected)?;
                let f: f32 = i.into();
                Val::Float32(f.to_bits())
            }
            Type::Float64 => {
                let i: i32 = self.try_int_from_int(v, unexpected)?;
                let f: f64 = i.into();
                Val::Float64(f.to_bits())
            }
            _ => return Err(DeError::invalid_type(unexpected, &self)),
        })
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if self.is_option() {
            return self.visit_option(Self::visit_u64, v);
        }
        let unexpected = Unexpected::Unsigned(v);
        Ok(match self.0 {
            Type::U8 => Val::U8(self.try_int_from_int(v, unexpected)?),
            Type::U16 => Val::U16(self.try_int_from_int(v, unexpected)?),
            Type::U32 => Val::U32(self.try_int_from_int(v, unexpected)?),
            Type::U64 => Val::U64(v),
            Type::S8 => Val::S8(self.try_int_from_int(v, unexpected)?),
            Type::S16 => Val::S16(self.try_int_from_int(v, unexpected)?),
            Type::S32 => Val::S32(self.try_int_from_int(v, unexpected)?),
            Type::S64 => Val::S64(self.try_int_from_int(v, unexpected)?),
            Type::Float32 => {
                let i: i16 = self.try_int_from_int(v, unexpected)?;
                let f: f32 = i.into();
                Val::Float32(f.to_bits())
            }
            Type::Float64 => {
                let i: i32 = self.try_int_from_int(v, unexpected)?;
                let f: f64 = i.into();
                Val::Float64(f.to_bits())
            }
            _ => return Err(DeError::invalid_type(unexpected, &self)),
        })
    }

    fn visit_f32<E>(self, mut v: f32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if self.is_option() {
            return self.visit_option(Self::visit_f32, v);
        }
        if v.is_nan() {
            v = f32::NAN;
        }
        Ok(match self.0 {
            Type::Float32 => Val::Float32(v.to_bits()),
            Type::Float64 => Val::Float64((v as f64).to_bits()),
            _ => return Err(DeError::invalid_type(Unexpected::Float(v as f64), &self)),
        })
    }

    fn visit_f64<E>(self, mut v: f64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if self.is_option() {
            return self.visit_option(Self::visit_f64, v);
        }
        if v.is_nan() {
            v = f64::NAN;
        }
        Ok(match self.0 {
            Type::Float64 => Val::Float64(v.to_bits()),
            _ => return Err(DeError::invalid_type(Unexpected::Float(v), &self)),
        })
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if self.is_option() {
            return self.visit_option(Self::visit_str, v);
        }
        Ok(match self.0 {
            Type::Char => {
                let mut chars = v.chars();
                let c = expect_exactly_one(
                    chars.next(),
                    chars.next(),
                    "expected exactly one char, got none",
                    "expected exactly one char, got more",
                )?;
                Val::Char(c)
            }
            Type::String => Val::String(v.to_string().into_boxed_str()),
            Type::Enum(enum_) => enum_.new_val(v).map_err(DeError::custom)?,
            Type::List(list) if list.ty() == Type::U8 => {
                let bytes = base64::decode(v).map_err(DeError::custom)?;
                list.new_val(bytes.into_iter().map(Val::U8).collect())
                    .map_err(DeError::custom)?
            }
            _ => return Err(DeError::invalid_type(Unexpected::Str(v), &self)),
        })
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if self.is_option() {
            return self.visit_option(Self::visit_string, v);
        }
        if self.0 == Type::String {
            Ok(Val::String(v.into_boxed_str()))
        } else {
            self.visit_str(&v)
        }
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if self.is_option() {
            return self.visit_option(Self::visit_bytes, v);
        }
        Ok(match self.0 {
            Type::List(list) if list.ty() == Type::U8 => list
                .new_val(v.iter().copied().map(Val::U8).collect())
                .map_err(DeError::custom)?,
            _ => return Err(DeError::invalid_type(Unexpected::Bytes(v), &self)),
        })
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        match self.0 {
            Type::Option(option) => option.new_val(None).map_err(DeError::custom),
            _ => Err(DeError::invalid_type(Unexpected::Option, &self)),
        }
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        match self.0 {
            Type::Option(option) => {
                let val = deserializer.deserialize_any(Self(option.ty()))?;
                option.new_val(Some(val)).map_err(DeError::custom)
            }
            _ => Err(DeError::invalid_type(Unexpected::Option, &self)),
        }
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_none()
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        if self.is_option() {
            return self.visit_option(Self::visit_seq, seq);
        }
        Ok(match &self.0 {
            Type::List(list) => {
                let ty = Self(list.ty());
                let mut vals = Vec::with_capacity(seq.size_hint().unwrap_or_default());
                while let Some(val) = seq.next_element_seed(ty.clone())? {
                    vals.push(val);
                }
                list.new_val(vals.into_boxed_slice())
                    .map_err(DeError::custom)?
            }

            Type::Tuple(tuple) => {
                let tuple_len = tuple.types().len();
                let mut vals = Vec::with_capacity(tuple_len);
                for (idx, ty) in tuple.types().enumerate() {
                    if let Some(val) = seq.next_element_seed(Self(ty))? {
                        vals.push(val);
                    } else {
                        return Err(DeError::invalid_length(idx, &self));
                    }
                }
                // Check for extra elements
                if seq.next_element::<IgnoredAny>()?.is_some() {
                    return Err(DeError::custom(format!(
                        "too many elements; expected {tuple_len}"
                    )));
                }
                tuple
                    .new_val(vals.into_boxed_slice())
                    .map_err(DeError::custom)?
            }

            Type::Flags(flags) => {
                let mut names = Vec::with_capacity(seq.size_hint().unwrap_or_default());
                while let Some(name) = seq.next_element::<&str>()? {
                    names.push(name);
                }
                flags.new_val(&names).map_err(DeError::custom)?
            }
            _ => return Err(DeError::invalid_type(Unexpected::Seq, &self)),
        })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if self.is_option() {
            return self.visit_option(Self::visit_map, map);
        }
        Ok(match &self.0 {
            Type::Record(record) => {
                let mut field_types = record
                    .fields()
                    .enumerate()
                    .map(|(idx, field)| (field.name, (idx, field.ty)))
                    .collect::<HashMap<_, _>>();

                let mut field_values = BTreeMap::new();
                while let Some(key) = map.next_key::<&str>()? {
                    let (idx, ty) = field_types.remove(key).ok_or_else(|| {
                        DeError::custom(format!("unknown or duplicate field {key:?}"))
                    })?;
                    let val = map.next_value_seed(TypeWrapper(ty))?;
                    field_values.insert(idx, (key, val));
                }

                record
                    .new_val(field_values.into_values())
                    .map_err(DeError::custom)?
            }

            Type::Variant(variant) => {
                let key = map
                    .next_key::<&str>()?
                    .ok_or_else(|| DeError::custom("empty map for variant"))?;

                let case = variant
                    .cases()
                    .find(|case| case.name == key)
                    .ok_or_else(|| DeError::custom(format!("unknown case {key:?} for variant")))?;

                let val = next_optional_value(&mut map, case.ty)?;

                if map.next_key::<IgnoredAny>()?.is_some() {
                    return Err(DeError::custom("too many elements; expected one"));
                }

                variant.new_val(key, val).map_err(DeError::custom)?
            }

            Type::Union(union) => {
                let discriminant: u32 = map
                    .next_key::<&str>()?
                    .ok_or_else(|| DeError::custom("empty map for union"))?
                    .parse()
                    .map_err(|_| DeError::custom("invalid key for union"))?;

                let ty = union.types().nth(discriminant as usize).ok_or_else(|| {
                    DeError::custom(format!("unknown case {discriminant} for union"))
                })?;

                let val = map.next_value_seed(Self(ty))?;

                if map.next_key::<IgnoredAny>()?.is_some() {
                    return Err(DeError::custom("too many elements; expected one"));
                }

                union.new_val(discriminant, val).map_err(DeError::custom)?
            }

            Type::Result(result) => {
                let key = map
                    .next_key()?
                    .ok_or_else(|| DeError::custom("empty map for result"))?;

                let val = match key {
                    "result" => Ok(next_optional_value(&mut map, result.ok())?),
                    "error" => Err(next_optional_value(&mut map, result.err())?),
                    other => {
                        return Err(DeError::custom(format!("unknown key {other:?} for result")))
                    }
                };

                if map.next_key::<IgnoredAny>()?.is_some() {
                    return Err(DeError::custom("too many elements; expected one"));
                }

                result.new_val(val).map_err(DeError::custom)?
            }

            _ => return Err(DeError::invalid_type(Unexpected::Map, &self)),
        })
    }
}

fn next_optional_value<'de, A: MapAccess<'de>>(
    map: &mut A,
    ty: Option<Type>,
) -> Result<Option<Val>, A::Error> {
    Ok(match ty {
        Some(ty) => Some(map.next_value_seed(TypeWrapper(ty))?),
        None => {
            map.next_value::<()>()?;
            None
        }
    })
}

fn expect_exactly_one<T, E: DeError>(
    first: Option<T>,
    second: Option<T>,
    too_few_msg: &str,
    too_many_msg: &str,
) -> Result<T, E> {
    match (first, second) {
        (Some(v), None) => Ok(v),
        (None, _) => Err(DeError::custom(too_few_msg)),
        (Some(_), Some(_)) => Err(DeError::custom(too_many_msg)),
    }
}
