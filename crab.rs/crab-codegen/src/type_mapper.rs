use crab_parser::Type;
pub struct TypeMapper;
impl TypeMapper {
    pub fn dart_to_rust(typ: &Type) -> String {
        match typ {
            Type::Int => "i64".to_string(),
            Type::Double => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "String".to_string(),
            Type::Void => "()".to_string(),
            Type::Nullable(inner) => {
                format!("Option<{}>", Self::dart_to_rust(inner))
            }
            Type::List(elem) => {
                format!("Vec<{}>", Self::dart_to_rust(elem))
            }
            Type::Map(key, val) => {
                format!(
                    "std::collections::HashMap<{}, {}>",
                    Self::dart_to_rust(key),
                    Self::dart_to_rust(val)
                )
            }
            Type::Set(elem) => {
                format!("std::collections::HashSet<{}>", Self::dart_to_rust(elem))
            }
            Type::Custom(name) => {
                if name == "Err" {
                    "String".to_string()
                } else if name == "Option" || name == "Result" {
                    name.clone()
                } else if name.starts_with("Result<") || name.starts_with("Option<") {
                    name.clone()
                } else if name == "Future" {
                    "impl std::future::Future".to_string()
                } else if name == "Request" {
                    "HttpRequest".to_string()
                } else if name == "Response" {
                    "HttpResponse".to_string()
                } else if name == "DateTime" {
                    "chrono::DateTime<chrono::Utc>".to_string()
                } else if name == "List" {
                    "Vec<serde_json::Value>".to_string()
                } else if name == "Map" {
                    "std::collections::HashMap<String, serde_json::Value>".to_string()
                } else if name == "Set" {
                    "std::collections::HashSet".to_string()
                } else {
                    name.clone()
                }
            }
            Type::Generic(name, params) => {
                let param_str = params
                    .iter()
                    .map(|p| Self::dart_to_rust(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                match name.as_str() {
                    "Result" => format!("Result<{}>", param_str),
                    "Option" => format!("Option<{}>", param_str),
                    "Future" => {
                        if params.len() == 1 {
                            format!(
                                "impl std::future::Future<Output = {}>",
                                Self::dart_to_rust(&params[0])
                            )
                        } else {
                            format!("{}<{}>", name, param_str)
                        }
                    }
                    "Stream" => {
                        if params.len() == 1 {
                            format!(
                                "impl futures::Stream<Item = {}>",
                                Self::dart_to_rust(&params[0])
                            )
                        } else {
                            format!("{}<{}>", name, param_str)
                        }
                    }
                    "List" => {
                        if params.len() == 1 {
                            format!("Vec<{}>", Self::dart_to_rust(&params[0]))
                        } else {
                            format!("Vec<{}>", param_str)
                        }
                    }
                    "Map" => {
                        if params.len() == 2 {
                            format!(
                                "std::collections::HashMap<{}, {}>",
                                Self::dart_to_rust(&params[0]),
                                Self::dart_to_rust(&params[1])
                            )
                        } else {
                            format!("{}<{}>", name, param_str)
                        }
                    }
                    "Set" => {
                        if params.len() == 1 {
                            format!(
                                "std::collections::HashSet<{}>",
                                Self::dart_to_rust(&params[0])
                            )
                        } else {
                            format!("{}<{}>", name, param_str)
                        }
                    }
                    _ => format!("{}<{}>", name, param_str),
                }
            }
            Type::Function {
                params,
                return_type,
            } => {
                let param_str = params
                    .iter()
                    .map(|p| Self::dart_to_rust(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({}) -> {}", param_str, Self::dart_to_rust(return_type))
            }
            Type::Tuple(elems) => {
                let elem_str = elems
                    .iter()
                    .map(|e| Self::dart_to_rust(e))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", elem_str)
            }
            Type::Record(fields) => {
                let field_str = fields
                    .iter()
                    .map(|(name, typ)| {
                        if let Some(n) = name {
                            format!("{}: {}", n, Self::dart_to_rust(typ))
                        } else {
                            Self::dart_to_rust(typ)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", field_str)
            }
            Type::Future(inner) => {
                format!(
                    "impl std::future::Future<Output = {}>",
                    Self::dart_to_rust(inner)
                )
            }
            Type::Stream(inner) => {
                format!("impl futures::Stream<Item = {}>", Self::dart_to_rust(inner))
            }
            Type::Result(ok, err) => {
                format!(
                    "Result<{}, {}>",
                    Self::dart_to_rust(ok),
                    Self::dart_to_rust(err)
                )
            }
            Type::OptionT(inner) => {
                format!("Option<{}>", Self::dart_to_rust(inner))
            }
            Type::Dynamic => "dyn std::any::Any".to_string(),
            Type::Never => "!".to_string(),
        }
    }
    pub fn is_nullable(typ: &Type) -> bool {
        matches!(typ, Type::Nullable(_))
    }
    pub fn is_collection(typ: &Type) -> bool {
        matches!(typ, Type::List(_) | Type::Map(_, _) | Type::Set(_))
    }
    pub fn collection_element_type(typ: &Type) -> Option<Type> {
        match typ {
            Type::List(elem) => Some((**elem).clone()),
            Type::Set(elem) => Some((**elem).clone()),
            Type::Map(key, val) => Some(Type::Tuple(vec![(**key).clone(), (**val).clone()])),
            _ => None,
        }
    }
}
