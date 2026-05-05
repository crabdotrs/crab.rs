use crab_parser::{CBlockDecl, Expr, ImportStmt, Type};

pub struct CInterop;

impl CInterop {
    pub fn generate_cblock_wrapper(cblock: &CBlockDecl) -> String {
        let mut output = String::new();
        output.push_str("extern \"C\" {\n");
        output.push_str(&cblock.code);
        output.push_str("}\n");
        output
    }

    pub fn generate_header_bindings(header_path: &str) -> String {
        let mut output = String::new();
        let safe_name = header_path.replace('.', "_").replace('/', "_");
        output.push_str(&format!("mod {}_bindings {{\n", safe_name));
        output.push_str("    use std::os::raw::{c_int, c_double, c_char, c_void};\n");
        output.push_str("    use std::ffi::CString;\n\n");
        output.push_str("    extern \"C\" {\n");
        output.push_str("    }\n");
        output.push_str("}\n");
        output
    }

    pub fn map_c_type_to_rust(c_type: &str) -> String {
        match c_type {
            "int" | "c_int" => "c_int".to_string(),
            "double" | "c_double" => "c_double".to_string(),
            "float" | "c_float" => "c_float".to_string(),
            "char" | "c_char" => "c_char".to_string(),
            "bool" | "_Bool" => "bool".to_string(),
            "void" => "c_void".to_string(),
            "size_t" => "usize".to_string(),
            "ssize_t" => "isize".to_string(),
            _ if c_type.contains('*') => "*mut c_void".to_string(),
            _ => c_type.to_string(),
        }
    }

    pub fn map_crab_type_to_c(typ: &Type) -> String {
        match typ {
            Type::Int => "c_int".to_string(),
            Type::Double => "c_double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "*const c_char".to_string(),
            Type::Nullable(inner) => format!("Option<{}>", Self::map_crab_type_to_c(inner)),
            _ => "c_void".to_string(),
        }
    }

    pub fn generate_ffi_glue(
        function_name: &str,
        params: &[(String, Type)],
        return_type: &Type,
    ) -> String {
        let mut output = String::new();
        let c_return = Self::map_crab_type_to_c(return_type);
        output.push_str(&format!("pub extern \"C\" fn {}(", function_name));

        for (i, (name, typ)) in params.iter().enumerate() {
            let c_typ = Self::map_crab_type_to_c(typ);
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&format!("{}: {}", name, c_typ));
        }

        output.push_str(&format!(") -> {} {{\n", c_return));
        let default_return = match return_type {
            Type::Int | Type::Double => "0",
            Type::Bool => "false",
            Type::Void => "()",
            _ => "std::ptr::null_mut()",
        };
        
        if c_return != "c_void" {
            output.push_str(&format!("    {}\n", default_return));
        }
        output.push_str("}\n");
        output
    }

    pub fn is_c_header_import(import: &ImportStmt) -> bool {
        import.path.ends_with(".h")
    }

    pub fn wrap_c_call(func_name: &str, args: &[Expr]) -> String {
        let mut output = String::new();
        output.push_str(&format!("unsafe {{ {}(", func_name));

        for (i, _arg) in args.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&format!("arg{}", i));
        }

        output.push_str(") }");
        output
    }
}
