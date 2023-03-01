use anyhow::Result;
use wasmtime::component::{Type, Val};

pub trait TypeExt {
    fn default_val(&self) -> Result<Val>;

    fn desc(&self) -> &'static str;
}

impl TypeExt for Type {
    fn default_val(&self) -> Result<Val> {
        Ok(match self {
            Type::Bool => Val::Bool(false),
            Type::U8 => Val::U8(0),
            Type::U16 => Val::U16(0),
            Type::U32 => Val::U32(0),
            Type::U64 => Val::U64(0),
            Type::S8 => Val::S8(0),
            Type::S16 => Val::S16(0),
            Type::S32 => Val::S32(0),
            Type::S64 => Val::S64(0),
            Type::Float32 => Val::Float32(0),
            Type::Float64 => Val::Float64(0),
            Type::Char => Val::Char('\x00'),
            Type::String => Val::String("".into()),

            Type::List(list) => list.new_val(Default::default())?,
            Type::Record(record) => {
                let values = record
                    .fields()
                    .map(|field| Ok((field.name, field.ty.default_val()?)))
                    .collect::<Result<Vec<_>>>()?;
                record.new_val(values)?
            }
            Type::Tuple(tuple) => {
                let values = tuple
                    .types()
                    .map(|ty| ty.default_val())
                    .collect::<Result<_>>()?;
                tuple.new_val(values)?
            }
            Type::Variant(variant) => {
                let case = variant.cases().next().expect("variant should have a case");
                let value = case.ty.map(|ty| ty.default_val()).transpose()?;
                variant.new_val(case.name, value)?
            }
            Type::Enum(enum_) => {
                let name = enum_.names().next().expect("enum should have a case");
                enum_.new_val(name)?
            }
            Type::Union(union) => {
                let ty = union.types().next().expect("union should have a case");
                union.new_val(0, ty.default_val()?)?
            }
            Type::Option(option) => option.new_val(None)?,
            Type::Result(result) => {
                let value = result.ok().map(|ty| ty.default_val()).transpose()?;
                result.new_val(Ok(value))?
            }
            Type::Flags(flags) => flags.new_val(&[])?,
        })
    }

    fn desc(&self) -> &'static str {
        match self {
            Type::Bool => "bool",
            Type::S8 => "s8",
            Type::U8 => "u8",
            Type::S16 => "s16",
            Type::U16 => "u16",
            Type::S32 => "s32",
            Type::U32 => "u32",
            Type::S64 => "s64",
            Type::U64 => "u64",
            Type::Float32 => "float32",
            Type::Float64 => "float64",
            Type::Char => "char",
            Type::String => "string",
            Type::List(_) => "list",
            Type::Record(_) => "record",
            Type::Tuple(_) => "tuple",
            Type::Variant(_) => "variant",
            Type::Enum(_) => "enum",
            Type::Union(_) => "union",
            Type::Option(_) => "option",
            Type::Result(_) => "result",
            Type::Flags(_) => "flags",
        }
    }
}
