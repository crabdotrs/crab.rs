use crab_parser::Type;

pub struct NullSafety;

impl NullSafety {
    pub fn wrap_option(typ: &Type) -> String {
        match typ {
            Type::Nullable(inner) => format!("Option<{}>", Self::type_to_rust(inner)),
            _ => Self::type_to_rust(typ),
        }
    }

    pub fn type_to_rust(typ: &Type) -> String {
        match typ {
            Type::Int => "i64".to_string(),
            Type::Double => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "String".to_string(),
            Type::Void => "()".to_string(),
            Type::Nullable(inner) => format!("Option<{}>", Self::type_to_rust(inner)),
            Type::List(elem) => format!("Vec<{}>", Self::type_to_rust(elem)),
            Type::Map(key, val) => format!(
                "std::collections::HashMap<{}, {}>",
                Self::type_to_rust(key),
                Self::type_to_rust(val)
            ),
            Type::Set(elem) => format!("std::collections::HashSet<{}>", Self::type_to_rust(elem)),
            Type::Custom(name) => name.clone(),
            Type::Generic(name, params) => {
                let params_str = params
                    .iter()
                    .map(|p| Self::type_to_rust(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, params_str)
            }
            Type::Future(inner) => format!("impl std::future::Future<Output = {}>", Self::type_to_rust(inner)),
            Type::Stream(inner) => format!("impl futures::Stream<Item = {}>", Self::type_to_rust(inner)),
            Type::Result(ok, err) => format!(
                "Result<{}, {}>",
                Self::type_to_rust(ok),
                Self::type_to_rust(err)
            ),
            Type::OptionT(inner) => format!("Option<{}>", Self::type_to_rust(inner)),
            Type::Dynamic => "dyn std::any::Any".to_string(),
            Type::Never => "!".to_string(),
            _ => "()".to_string(),
        }
    }

    pub fn generate_null_check(var_name: &str, typ: &Type) -> String {
        match typ {
            Type::Nullable(_) => format!("{}.is_some()", var_name),
            _ => format!("{}.is_some()", var_name),
        }
    }

    pub fn generate_unwrap(var_name: &str, default: Option<&str>) -> String {
        match default {
            Some(d) => format!("{}.unwrap_or({})", var_name, d),
            None => format!("{}.unwrap()", var_name),
        }
    }

    pub fn generate_null_coalesce(left: &str, right: &str) -> String {
        format!("{}.unwrap_or({})", left, right)
    }

    pub fn is_nullable(typ: &Type) -> bool {
        matches!(typ, Type::Nullable(_))
    }

    pub fn unwrap_nullable(typ: &Type) -> Option<&Type> {
        match typ {
            Type::Nullable(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn generate_safe_call(obj: &str, method: &str, args: &[String]) -> String {
        let args_str = args.join(", ");
        format!("{}.map(|v| v.{}({}))", obj, method, args_str)
    }

    pub fn generate_null_assertion(expr: &str) -> String {
        format!("{}.unwrap()", expr)
    }
}
