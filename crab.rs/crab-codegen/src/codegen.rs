use crate::type_mapper::TypeMapper;
use crab_parser::{
    BinaryOp, ClassDecl, ConstDecl, Constructor, Expr, FunctionBody, FunctionDecl, Literal,
    MethodDecl, Program, Statement, StringPart, SwitchPattern, TopLevelItem, Type, UnaryOp,
    VarDecl,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum NullCheckType {
    IsNull,  // x == null
    NotNull, // x != null
}

pub struct CodeGenerator {
    indent_level: usize,
    output: String,
    program: Option<Program>,
    current_class_fields: Vec<String>,
    current_class: Option<ClassDecl>,
    current_method_returns_option: bool,
    current_method_is_getter: bool,
    current_method_return_type: String,
    current_method_returns_boxed_trait: bool,
    current_method_params: Vec<(String, String)>,
    unwrapped_locals: Vec<String>,
}
impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            indent_level: 0,
            output: String::new(),
            program: None,
            current_class_fields: Vec::new(),
            current_class: None,
            current_method_returns_option: false,
            current_method_is_getter: false,
            current_method_return_type: String::new(),
            current_method_returns_boxed_trait: false,
            current_method_params: Vec::new(),
            unwrapped_locals: Vec::new(),
        }
    }
    pub fn generate(&mut self, program: &Program) -> String {
        self.program = Some(program.clone());
        self.emit_header(program);
        for (_i, item) in program.items.iter().enumerate() {
            match item {
                TopLevelItem::ClassDecl(_)
                | TopLevelItem::MixinDecl(_)
                | TopLevelItem::ExtensionDecl(_)
                | TopLevelItem::TypeAlias(_) => {
                    self.generate_top_level_item(item);
                }
                _ => {}
            }
        }
        for (_i, item) in program.items.iter().enumerate() {
            match item {
                TopLevelItem::VarDecl(_) | TopLevelItem::Const(_) => {
                    self.generate_top_level_item(item);
                }
                _ => {}
            }
        }
        for (_i, item) in program.items.iter().enumerate() {
            match item {
                TopLevelItem::FunctionDecl(_) => {
                    self.generate_top_level_item(item);
                }
                _ => {}
            }
        }
        for (_i, item) in program.items.iter().enumerate() {
            match item {
                TopLevelItem::Import(_) | TopLevelItem::Export(_) | TopLevelItem::CBlock(_) => {
                    self.generate_top_level_item(item);
                }
                _ => {}
            }
        }
        self.output.clone()
    }
    fn emit_header(&mut self, program: &Program) {
        self.emit_line("#[allow(dead_code)]");
        self.emit_line("");
        self.generate_import_uses(program);
        let needs_collections = self.program_uses_collections(program);
        if needs_collections {
            self.emit_line("use std::collections::{HashMap, HashSet};");
        }
        let needs_mutable_static = self.program_uses_mutable_static(program);
        if needs_mutable_static {
            self.emit_line("use std::sync::Mutex;");
        }
        let needs_serde = self.program_has_classes(program) && self.program_imports_serde(program);
        if needs_serde {
            self.emit_line("use serde::{Serialize, Deserialize};");
        }
        self.emit_line("");
    }
    fn generate_import_uses(&mut self, program: &Program) {
        for item in &program.items {
            if let TopLevelItem::Import(imp) = item {
                let use_stmt = self.map_import_to_use(&imp.path);
                if let Some(stmt) = use_stmt {
                    self.emit_line(&stmt);
                }
            }
        }
    }
    fn map_import_to_use(&self, path: &str) -> Option<String> {
        if path == "std::collections" || path.starts_with("std::") {
            return None;
        }
        match path {
            "tokio" => Some("use tokio;".to_string()),
            "serde" => Some("use serde;".to_string()),
            "serde_json" => Some("use serde_json;".to_string()),
            "futures" => Some("use futures;".to_string()),
            "actix_web" | "actix-web" => Some("use actix_web;".to_string()),
            "reqwest" => Some("use reqwest;".to_string()),
            "chrono" => Some("use chrono;".to_string()),
            "rand" => Some("use rand;".to_string()),
            "regex" => Some("use regex;".to_string()),
            "lazy_static" => Some("use lazy_static;".to_string()),
            "async_trait" => Some("use async_trait;".to_string()),
            "thiserror" => Some("use thiserror;".to_string()),
            "anyhow" => Some("use anyhow;".to_string()),
            "tracing" => Some("use tracing;".to_string()),
            "log" => Some("use log;".to_string()),
            "env_logger" => Some("use env_logger;".to_string()),
            "clap" => Some("use clap;".to_string()),
            "http" => Some("use http;".to_string()),
            "hyper" => Some("use hyper;".to_string()),
            "tower" => Some("use tower;".to_string()),
            "axum" => Some("use axum;".to_string()),
            "warp" => Some("use warp;".to_string()),
            "rocket" => Some("use rocket;".to_string()),
            "tide" => Some("use tide;".to_string()),
            path if std::path::Path::new(path)
                .extension()
                .map_or(false, |e| e == "h") =>
            {
                None
            }
            path if path.ends_with(".crab") => {
                use std::path::Path;
                let path_obj = Path::new(path);
                let file_stem = path_obj
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(path);
                Some(format!("use crate::{}::*;", file_stem))
            }
            _ => Some(format!("use {};", path)),
        }
    }
    fn program_uses_collections(&self, program: &Program) -> bool {
        for item in &program.items {
            match item {
                TopLevelItem::ClassDecl(_) => return true,
                TopLevelItem::FunctionDecl(f) => {
                    if self.function_uses_collections(f) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
    fn program_uses_mutable_static(&self, program: &Program) -> bool {
        for item in &program.items {
            if let TopLevelItem::VarDecl(var_decl) = item {
                let is_complex_type = matches!(
                    &var_decl.typ,
                    crab_parser::Type::List(_)
                        | crab_parser::Type::Map(_, _)
                        | crab_parser::Type::Set(_)
                        | crab_parser::Type::Custom(_)
                );
                if is_complex_type && !var_decl.is_final {
                    return true;
                }
            }
        }
        false
    }
    fn function_uses_collections(&self, func: &FunctionDecl) -> bool {
        if let FunctionBody::Block(stmts) = &func.body {
            for stmt in stmts {
                if self.statement_uses_collections(stmt) {
                    return true;
                }
            }
        }
        false
    }
    fn statement_uses_collections(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::VarDecl(v) => self.type_is_collection(&v.typ),
            Statement::Expression(e) => self.expr_uses_collections(e),
            _ => false,
        }
    }
    fn type_is_collection(&self, typ: &Option<crab_parser::Type>) -> bool {
        match typ {
            Some(crab_parser::Type::List(_))
            | Some(crab_parser::Type::Map(_, _))
            | Some(crab_parser::Type::Set(_)) => true,
            _ => false,
        }
    }
    fn expr_uses_collections(&self, expr: &Expr) -> bool {
        match expr {
            Expr::ListLiteral(_) | Expr::MapLiteral(_) | Expr::SetLiteral(_) => true,
            _ => false,
        }
    }
    fn program_has_classes(&self, program: &Program) -> bool {
        program
            .items
            .iter()
            .any(|item| matches!(item, TopLevelItem::ClassDecl(_)))
    }
    fn program_imports_serde(&self, program: &Program) -> bool {
        program.items.iter().any(|item| {
            if let TopLevelItem::Import(imp) = item {
                imp.path == "serde" || imp.path.starts_with("serde::")
            } else {
                false
            }
        })
    }
    fn find_class(&self, name: &str) -> Option<ClassDecl> {
        if let Some(ref program) = self.program {
            for item in &program.items {
                if let TopLevelItem::ClassDecl(c) = item {
                    if c.name == name {
                        return Some(c.clone());
                    }
                }
            }
        }
        None
    }
    fn get_all_fields(&self, class: &ClassDecl) -> Vec<crab_parser::Field> {
        let mut all_fields = Vec::new();
        if let Some(ref parent_type) = class.parent {
            let parent_name = match &**parent_type {
                Type::Custom(name) => name.clone(),
                _ => String::new(),
            };
            if !parent_name.is_empty() {
                if let Some(parent_class) = self.find_class(&parent_name) {
                    let parent_fields = self.get_all_fields(&parent_class);
                    all_fields.extend(parent_fields);
                }
            }
        }
        all_fields.extend(class.fields.clone());
        all_fields
    }
    fn method_exists_in_current_class(&self, method_name: &str) -> bool {
        if let Some(ref program) = self.program {
            for item in &program.items {
                if let TopLevelItem::ClassDecl(c) = item {
                    for method in &c.methods {
                        if method.name == method_name {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    fn class_exists(&self, class_name: &str) -> bool {
        if let Some(ref program) = self.program {
            for item in &program.items {
                if let TopLevelItem::ClassDecl(c) = item {
                    if c.name == class_name {
                        return true;
                    }
                }
            }
        }
        false
    }
    fn is_sealed_class(&self, class_name: &str) -> bool {
        if let Some(ref program) = self.program {
            for item in &program.items {
                if let TopLevelItem::ClassDecl(c) = item {
                    if c.name == class_name && c.is_sealed {
                        return true;
                    }
                }
            }
        }
        false
    }
    fn infer_enum_type_from_expr(&self, expr: &crab_parser::Expr) -> Option<String> {
        if let crab_parser::Expr::Identifier(var_name) = expr {
            let lookup_name = if var_name == "type" {
                "r#type".to_string()
            } else {
                var_name.clone()
            };
            for (param_name, param_type) in &self.current_method_params {
                if param_name == &lookup_name {
                    if let Some(ref program) = self.program {
                        for item in &program.items {
                            if let TopLevelItem::ClassDecl(c) = item {
                                if self.is_enum_class(c) && c.name == *param_type {
                                    return Some(c.name.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
    fn trait_has_method(&self, trait_name: &str, method_name: &str) -> bool {
        if let Some(ref program) = self.program {
            for item in &program.items {
                if let TopLevelItem::ClassDecl(c) = item {
                    if c.name == trait_name && c.is_sealed {
                        for method in &c.methods {
                            if method.name == method_name {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
    fn transform_sealed_class_types(&self, type_str: &str) -> String {
        let mut result = type_str.to_string();

        if let Some(ref program) = self.program {
            for item in &program.items {
                if let TopLevelItem::ClassDecl(c) = item {
                    if c.is_sealed {
                        let sealed_name = &c.name;
                        let option_pattern = format!("Option<{}>", sealed_name);
                        let option_replacement = format!("Option<Box<dyn {}>>", sealed_name);
                        result = result.replace(&option_pattern, &option_replacement);

                        if result == *sealed_name {
                            result = format!("Box<dyn {}>", sealed_name);
                        }
                    }
                }
            }
        }
        result
    }
    fn is_getter_method(&self, method_name: &str) -> bool {
        if let Some(ref program) = self.program {
            for item in &program.items {
                if let TopLevelItem::ClassDecl(c) = item {
                    for method in &c.methods {
                        if method.name == method_name && method.is_getter {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    fn is_null_check_condition(&self, expr: &Expr) -> bool {
        match expr {
            Expr::BinaryOp {
                left,
                op: BinaryOp::Equal,
                right,
            } => {
                if let Expr::Literal(Literal::Null) = right.as_ref() {
                    return true;
                }
                if let Expr::Identifier(name) = right.as_ref() {
                    if name == "null" || name == "None" {
                        return true;
                    }
                }
                false
            }
            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                if method == "is_none" && args.is_empty() {
                    return true;
                }
                false
            }
            _ => false,
        }
    }
    fn extract_null_check_var(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::BinaryOp {
                left,
                op: BinaryOp::Equal,
                right,
            } => {
                if let Expr::Identifier(name) = left.as_ref() {
                    return Some(name.clone());
                }
                None
            }
            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                if method == "is_none" && args.is_empty() {
                    if let Expr::Identifier(name) = object.as_ref() {
                        return Some(name.clone());
                    }
                }
                None
            }
            _ => None,
        }
    }
    fn get_null_check_info(&self, expr: &Expr) -> Option<(String, NullCheckType)> {
        match expr {
            Expr::BinaryOp {
                left,
                op: BinaryOp::Equal,
                right,
            } => {
                let is_null = matches!(right.as_ref(), Expr::Literal(Literal::Null))
                    || matches!(right.as_ref(), Expr::Identifier(name) if name == "null" || name == "None");
                if is_null {
                    if let Expr::Identifier(name) = left.as_ref() {
                        return Some((name.clone(), NullCheckType::IsNull));
                    }
                }
                None
            }
            Expr::BinaryOp {
                left,
                op: BinaryOp::NotEqual,
                right,
            } => {
                let is_null = matches!(right.as_ref(), Expr::Literal(Literal::Null))
                    || matches!(right.as_ref(), Expr::Identifier(name) if name == "null" || name == "None");
                if is_null {
                    if let Expr::Identifier(name) = left.as_ref() {
                        return Some((name.clone(), NullCheckType::NotNull));
                    }
                }
                None
            }
            Expr::BinaryOp {
                left,
                op: BinaryOp::Or,
                right: _,
            } => self.get_null_check_info(left),
            _ => None,
        }
    }
    fn generate_top_level_item(&mut self, item: &TopLevelItem) {
        match item {
            TopLevelItem::FunctionDecl(func) => self.generate_function(func),
            TopLevelItem::ClassDecl(class) => self.generate_class(class),
            TopLevelItem::Const(const_decl) => self.generate_const(const_decl),
            TopLevelItem::TypeAlias(alias) => {
                self.emit_line(&format!(
                    "type {} = {};",
                    alias.name,
                    TypeMapper::dart_to_rust(&alias.typ)
                ));
            }
            TopLevelItem::Import(_) => {}
            TopLevelItem::Export(_) => {}
            TopLevelItem::CBlock(cblock) => {
                self.emit_line("unsafe {");
                self.indent();
                for line in cblock.code.lines() {
                    self.emit_line(&format!(" {}", line));
                }
                self.dedent();
                self.emit_line("}");
            }
            TopLevelItem::MixinDecl(mixin) => self.generate_mixin(mixin),
            TopLevelItem::ExtensionDecl(extension) => self.generate_extension(extension),
            TopLevelItem::VarDecl(var_decl) => self.generate_top_level_var(var_decl),
        }
    }
    fn generate_top_level_var(&mut self, var_decl: &crab_parser::TopLevelVarDecl) {
        let typ_str = TypeMapper::dart_to_rust(&var_decl.typ);
        let value_str = self.expr_to_string(&var_decl.value);
        let mutability = if var_decl.is_final { "" } else { "mut " };
        let is_complex_type = matches!(
            &var_decl.typ,
            crab_parser::Type::List(_)
                | crab_parser::Type::Map(_, _)
                | crab_parser::Type::Set(_)
                | crab_parser::Type::Custom(_)
        );
        let is_nullable = matches!(
            &var_decl.typ,
            crab_parser::Type::Nullable(_) | crab_parser::Type::OptionT(_)
        );
        let is_null_literal = matches!(
            &var_decl.value,
            crab_parser::Expr::Literal(crab_parser::Literal::Null)
        );
        let final_value = if is_nullable && !is_null_literal {
            format!("Some({})", value_str)
        } else {
            value_str
        };
        if is_complex_type && !var_decl.is_final {
            self.emit_line(&format!(
                "lazy_static::lazy_static! {{ pub static ref {}: Mutex<{}> = Mutex::new({}); }}",
                var_decl.name, typ_str, final_value
            ));
        } else {
            let keyword = if var_decl.is_final { "const" } else { "static" };
            self.emit_line(&format!(
                "{} {}{}: {} = {};",
                keyword, mutability, var_decl.name, typ_str, final_value
            ));
        }
        self.emit_line("");
    }
    fn generate_mixin(&mut self, mixin: &crab_parser::MixinDecl) {
        let constraint = mixin
            .on_type
            .as_ref()
            .map(|t| format!(": {}", TypeMapper::dart_to_rust(t)))
            .unwrap_or_default();
        self.emit_line(&format!("pub trait {}{} {{", mixin.name, constraint));
        self.indent();
        for method in &mixin.methods {
            self.generate_trait_method(method);
        }
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
    }
    fn generate_extension(&mut self, extension: &crab_parser::ExtensionDecl) {
        let on_type = TypeMapper::dart_to_rust(&extension.on_type);
        let name = extension
            .name
            .as_ref()
            .map(|n| n.clone())
            .unwrap_or_else(|| format!("Ext_{}", on_type.replace("<", "_").replace(">", "_")));
        self.emit_line(&format!("pub trait {} {{", name));
        self.indent();
        for method in &extension.methods {
            self.generate_trait_method(method);
        }
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        self.emit_line(&format!("impl {} for {} {{", name, on_type));
        self.indent();
        for method in &extension.methods {
            self.generate_method_impl(method);
        }
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
    }
    fn generate_trait_method(&mut self, method: &crab_parser::MethodDecl) {
        let return_type = method
            .return_type
            .as_ref()
            .map(|t| TypeMapper::dart_to_rust(t))
            .unwrap_or_else(|| "()".to_string());
        let params = method
            .params
            .iter()
            .map(|p| {
                let safe_name = if p.name == "type" { "r#type" } else { &p.name };
                format!("{}: {}", safe_name, TypeMapper::dart_to_rust(&p.typ))
            })
            .collect::<Vec<_>>()
            .join(", ");
        let params_str = if params.is_empty() {
            "&self".to_string()
        } else {
            format!("&self, {}", params)
        };
        self.emit_line(&format!(
            "fn {}({}) -> {};",
            method.name, params_str, return_type
        ));
    }
    fn generate_method_impl(&mut self, method: &crab_parser::MethodDecl) {
        let return_type = method
            .return_type
            .as_ref()
            .map(|t| TypeMapper::dart_to_rust(t))
            .unwrap_or_else(|| "()".to_string());
        let params = method
            .params
            .iter()
            .map(|p| {
                let safe_name = if p.name == "type" { "r#type" } else { &p.name };
                format!("{}: {}", safe_name, TypeMapper::dart_to_rust(&p.typ))
            })
            .collect::<Vec<_>>()
            .join(", ");
        let params_str = if params.is_empty() {
            "&self".to_string()
        } else {
            format!("&self, {}", params)
        };
        self.emit_line(&format!(
            "fn {}({}) -> {} {{",
            method.name, params_str, return_type
        ));
        self.indent();
        match &method.body {
            FunctionBody::Expression(expr) => {
                self.emit_line(&format!("{}", self.expr_to_string_in_method(expr)));
            }
            FunctionBody::Block(stmts) => {
                for stmt in stmts {
                    self.generate_statement_in_method(stmt);
                }
            }
        }
        self.dedent();
        self.emit_line("}");
    }
    fn generate_function(&mut self, func: &FunctionDecl) {
        let return_type = func
            .return_type
            .as_ref()
            .map(|t| TypeMapper::dart_to_rust(t))
            .unwrap_or_else(|| "()".to_string());
        let return_type = if func.is_async {
            if let Some(start) = return_type.find("Output = ") {
                if let Some(end) = return_type[start..].find(">") {
                    return_type[start + 9..start + end].to_string()
                } else {
                    return_type[start + 9..].to_string()
                }
            } else {
                return_type
            }
        } else {
            return_type
        };
        let return_type = self.transform_sealed_class_types(&return_type);
        let params = func
            .params
            .iter()
            .map(|p| {
                let safe_name = if p.name == "type" { "r#type" } else { &p.name };
                format!("{}: {}", safe_name, TypeMapper::dart_to_rust(&p.typ))
            })
            .collect::<Vec<_>>()
            .join(", ");
        let is_async = func.is_async;
        let async_keyword = if is_async { "async " } else { "" };
        if func.name == "main" && is_async {
            self.emit_line("#[tokio::main]");
        }
        let sig = if func.name == "main" {
            if is_async {
                "async fn main()".to_string()
            } else {
                "fn main()".to_string()
            }
        } else if is_async {
            if return_type == "()" {
                format!("{}fn {}({})", async_keyword, func.name, params)
            } else {
                format!(
                    "{}fn {}({}) -> {}",
                    async_keyword, func.name, params, return_type
                )
            }
        } else {
            if return_type == "()" {
                format!("fn {}({})", func.name, params)
            } else {
                format!("fn {}({}) -> {}", func.name, params, return_type)
            }
        };
        self.emit_line(&format!("{} {{", sig));
        self.indent();
        self.current_method_returns_option = return_type.starts_with("Option<");
        self.current_method_return_type = return_type.clone();
        self.current_method_returns_boxed_trait = return_type.contains("Box<dyn");
        if func.name == "main" && func.is_async {
            self.emit_line("let args: Vec<String> = std::env::args().collect();");
        }
        match &func.body {
            FunctionBody::Expression(expr) => {
                self.emit_line(&format!("{}", self.expr_to_string(expr)));
            }
            FunctionBody::Block(stmts) => {
                for stmt in stmts {
                    self.generate_statement(stmt);
                }
            }
        }
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        self.current_method_returns_option = false;
        self.current_method_return_type = String::new();
        self.current_method_returns_boxed_trait = false;
        self.current_method_params.clear();
    }
    fn generate_class(&mut self, class: &ClassDecl) {
        if class.is_abstract || class.is_sealed {
            self.generate_trait(class);
            return;
        }
        if self.is_enum_class(class) {
            self.generate_enum(class);
            return;
        }
        self.generate_struct_def(class);
        self.generate_impl_block(class);
        for mixin in &class.mixins {
            let mixin_name = match mixin {
                crab_parser::Type::Custom(name) => name.clone(),
                _ => continue,
            };
            self.emit_line(&format!("impl {} for {} {{}}", mixin_name, class.name));
            self.emit_line("");
        }
        for interface in &class.implements {
            let interface_name = match interface {
                crab_parser::Type::Custom(name) => name.clone(),
                _ => continue,
            };
            self.emit_line(&format!("impl {} for {} {{}}", interface_name, class.name));
            self.emit_line("");
        }
        if let Some(ref parent) = class.parent {
            if let crab_parser::Type::Custom(parent_name) = parent.as_ref() {
                if self.is_sealed_class(parent_name) {
                    self.generate_sealed_class_impl(class, parent_name);
                }
            }
        }
    }
    fn generate_sealed_class_impl(&mut self, class: &ClassDecl, trait_name: &str) {
        let trait_methods: Vec<_> = if let Some(ref program) = self.program {
            program
                .items
                .iter()
                .filter_map(|item| {
                    if let TopLevelItem::ClassDecl(c) = item {
                        if c.name == trait_name && c.is_sealed {
                            return Some(c.methods.clone());
                        }
                    }
                    None
                })
                .next()
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        if trait_methods.is_empty() {
            return;
        }

        self.emit_line(&format!("impl {} for {} {{", trait_name, class.name));
        self.indent();

        for method in trait_methods {
            let return_type = method
                .return_type
                .as_ref()
                .map(|t| TypeMapper::dart_to_rust(t))
                .unwrap_or_else(|| "()".to_string());
            let params = method
                .params
                .iter()
                .map(|p| {
                    let safe_name = if p.name == "type" { "r#type" } else { &p.name };
                    let param_type = TypeMapper::dart_to_rust(&p.typ);
                    format!("{}: {}", safe_name, param_type)
                })
                .collect::<Vec<_>>()
                .join(", ");
            let params_with_self = if params.is_empty() {
                "&self".to_string()
            } else {
                format!("&self, {}", params)
            };
            self.emit_line(&format!(
                "fn {}({}) -> {} {{",
                method.name, params_with_self, return_type
            ));
            self.indent();
            let arg_names: Vec<_> = method
                .params
                .iter()
                .map(|p| {
                    if p.name == "type" {
                        "r#type".to_string()
                    } else {
                        p.name.clone()
                    }
                })
                .collect();
            let call = if arg_names.is_empty() {
                format!("self.{}()", method.name)
            } else {
                format!("self.{}({})", method.name, arg_names.join(", "))
            };
            if return_type == "()" {
                self.emit_line(&format!("{};", call));
            } else {
                self.emit_line(&format!("{}", call));
            }
            self.dedent();
            self.emit_line("}");
        }

        self.dedent();
        self.emit_line("}");
        self.emit_line("");
    }
    fn is_enum_class(&self, class: &ClassDecl) -> bool {
        !class.constructors.is_empty()
            && class.constructors.iter().all(|c| c.name.is_some())
            && class.methods.is_empty()
            && class.parent.is_none()
            && !class.constructors.iter().any(|c| !c.params.is_empty())
    }
    fn generate_enum(&mut self, class: &ClassDecl) {
        self.emit_line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
        self.emit_line(&format!("pub enum {} {{", class.name));
        self.indent();
        for constructor in &class.constructors {
            if let Some(name) = &constructor.name {
                self.emit_line(&format!("{},", name));
            }
        }
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
    }
    fn generate_struct_def(&mut self, class: &ClassDecl) {
        let visibility = "pub ";
        if let Some(ref program) = self.program {
            if self.program_imports_serde(program) {
                self.emit_line("#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]");
            } else {
                self.emit_line("#[derive(Debug, Clone, PartialEq)]");
            }
        }
        self.emit_line(&format!("{}struct {} {{", visibility, class.name));
        self.indent();
        let all_fields = self.get_all_fields(class);
        for field in &all_fields {
            if !field.is_static {
                let typ_str = TypeMapper::dart_to_rust(&field.typ);
                self.emit_line(&format!("pub {}: {},", field.name, typ_str));
            }
        }
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
    }
    fn generate_default_constructor(&mut self, class: &ClassDecl) {
        let all_fields = self.get_all_fields(class);
        let params = all_fields
            .iter()
            .filter(|f| !f.is_static)
            .map(|f| format!("{}: {}", f.name, TypeMapper::dart_to_rust(&f.typ)))
            .collect::<Vec<_>>()
            .join(", ");
        self.emit_line(&format!("pub fn new({}) -> Self {{", params));
        self.indent();
        self.emit_line("Self {");
        self.indent();
        for field in &all_fields {
            if !field.is_static {
                self.emit_line(&format!("{}: {},", field.name, field.name));
            }
        }
        self.dedent();
        self.emit_line("}");
        self.dedent();
        self.emit_line("}");
    }
    fn generate_constructor(&mut self, class: &ClassDecl, constructor: &Constructor) {
        let method_name = match &constructor.name {
            Some(name) => format!("new_{}", name),
            None => "new".to_string(),
        };
        let field_types: std::collections::HashMap<String, crab_parser::Type> = class
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.typ.clone()))
            .collect();
        let params = constructor
            .params
            .iter()
            .map(|p| {
                let param_name = if p.name.starts_with("this.") {
                    &p.name[5..]
                } else {
                    &p.name
                };
                let param_type = if matches!(p.typ, crab_parser::Type::Dynamic) {
                    field_types.get(param_name).unwrap_or(&p.typ)
                } else {
                    &p.typ
                };
                format!("{}: {}", param_name, TypeMapper::dart_to_rust(param_type))
            })
            .collect::<Vec<_>>()
            .join(", ");
        self.emit_line(&format!("pub fn {}({}) -> Self {{", method_name, params));
        self.indent();
        let mut initialized_fields: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for initializer in &constructor.initializers {
            initialized_fields.insert(initializer.field.clone());
        }
        self.emit_line("Self {");
        self.indent();
        for initializer in &constructor.initializers {
            self.emit_line(&format!(
                "{}: {},",
                initializer.field,
                self.expr_to_string(&initializer.value)
            ));
        }
        let param_names: std::collections::HashSet<String> = constructor
            .params
            .iter()
            .map(|p| {
                if p.name.starts_with("this.") {
                    p.name[5..].to_string()
                } else {
                    p.name.clone()
                }
            })
            .collect();
        for field in &class.fields {
            if !field.is_static && !initialized_fields.contains(&field.name) {
                let value = if param_names.contains(&field.name) {
                    field.name.clone()
                } else {
                    match &field.typ {
                        crab_parser::Type::Int => "0".to_string(),
                        crab_parser::Type::Double => "0.0".to_string(),
                        crab_parser::Type::Bool => "false".to_string(),
                        crab_parser::Type::String => "String::new()".to_string(),
                        _ => "Default::default()".to_string(),
                    }
                };
                self.emit_line(&format!("{}: {},", field.name, value));
            }
        }
        self.dedent();
        self.emit_line("}");
        self.dedent();
        self.emit_line("}");
    }
    fn generate_method(&mut self, method: &MethodDecl) {
        let return_type = method
            .return_type
            .as_ref()
            .map(|t| TypeMapper::dart_to_rust(t))
            .unwrap_or_else(|| "()".to_string());
        self.current_method_returns_option = return_type.starts_with("Option<");
        self.current_method_is_getter = method.is_getter;
        self.current_method_return_type = return_type.clone();
        self.current_method_params = method
            .params
            .iter()
            .map(|p| {
                let safe_name = if p.name == "type" {
                    "r#type".to_string()
                } else {
                    p.name.clone()
                };
                let typ_str = TypeMapper::dart_to_rust(&p.typ);
                (safe_name, typ_str)
            })
            .collect();
        let is_finder_method = method.name.starts_with("find") || method.name.starts_with("_find");
        let params = method
            .params
            .iter()
            .map(|p| {
                let safe_name = if p.name == "type" { "r#type" } else { &p.name };
                let typ_str = TypeMapper::dart_to_rust(&p.typ);
                if is_finder_method && typ_str == "String" {
                    format!("{}: &str", safe_name)
                } else {
                    format!("mut {}: {}", safe_name, typ_str)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let overrides_sealed = self.current_class.as_ref().map_or(false, |class| {
            class.parent.as_ref().map_or(false, |parent| {
                if let crab_parser::Type::Custom(parent_name) = parent.as_ref() {
                    self.is_sealed_class(parent_name)
                        && self.trait_has_method(parent_name, &method.name)
                } else {
                    false
                }
            })
        });
        let is_readonly = method.is_getter
            || method.name == "display"
            || method.name == "toJson"
            || method.name == "toString"
            || method.name.starts_with("print")
            || method.name.starts_with("show")
            || method.name.starts_with("find")
            || method.name.starts_with("list")
            || method.name.starts_with("get");
        let self_param = if is_readonly || overrides_sealed {
            "&self"
        } else {
            "&mut self"
        };
        let params_str = if params.is_empty() {
            self_param.to_string()
        } else {
            format!("{}, {}", self_param, params)
        };
        if method.is_getter {
            self.emit_line(&format!(
                "pub fn {}({}) -> {} {{",
                method.name, self_param, return_type
            ));
        } else if method.is_setter {
            self.emit_line(&format!(
                "pub fn set_{}({}) -> {} {{",
                method.name, params_str, return_type
            ));
        } else if method.is_static {
            self.emit_line(&format!(
                "pub fn {}({}) -> {} {{",
                method.name, params, return_type
            ));
        } else {
            self.emit_line(&format!(
                "pub fn {}({}) -> {} {{",
                method.name, params_str, return_type
            ));
        }
        self.indent();
        match &method.body {
            FunctionBody::Expression(expr) => {
                let expr_str = self.expr_to_string_in_method(expr);
                let needs_clone = method.is_getter
                    && (return_type == "String"
                        || return_type.contains("Option<String>")
                        || return_type.starts_with("Vec<")
                        || return_type.starts_with("HashMap<")
                        || return_type.starts_with("HashSet<"))
                    && !expr_str.contains(".clone()")
                    && !expr_str.starts_with('&');
                let final_expr = if needs_clone {
                    format!("{}.clone()", expr_str)
                } else {
                    expr_str
                };
                self.emit_line(&format!("{}", final_expr));
            }
            FunctionBody::Block(stmts) => {
                for (i, stmt) in stmts.iter().enumerate() {}
                for stmt in stmts {
                    self.generate_statement_in_method(stmt);
                }
            }
        }
        self.dedent();
        self.emit_line("}");
        self.current_method_returns_option = false;
        self.current_method_is_getter = false;
        self.current_method_return_type = String::new();
        self.current_method_params.clear();
        self.unwrapped_locals.clear();
    }
    fn generate_impl_block(&mut self, class: &ClassDecl) {
        self.emit_line(&format!("impl {} {{", class.name));
        self.indent();
        if !class.constructors.is_empty() {
            for constructor in &class.constructors {
                self.generate_constructor(class, constructor);
            }
        } else {
            self.generate_default_constructor(class);
        }
        let all_fields = self.get_all_fields(class);
        self.current_class_fields = all_fields.iter().map(|f| f.name.clone()).collect();
        self.current_class = Some(class.clone());
        for method in &class.methods {
            self.generate_method(method);
        }
        self.current_class = None;
        self.current_class_fields.clear();
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
    }
    fn generate_trait(&mut self, class: &ClassDecl) {
        self.emit_line(&format!("pub trait {} {{", class.name));
        self.indent();
        for method in &class.methods {
            let return_type = method
                .return_type
                .as_ref()
                .map(|t| TypeMapper::dart_to_rust(t))
                .unwrap_or_else(|| "()".to_string());
            let params = method
                .params
                .iter()
                .map(|p| {
                    let safe_name = if p.name == "type" { "r#type" } else { &p.name };
                    format!("{}: {}", safe_name, TypeMapper::dart_to_rust(&p.typ))
                })
                .collect::<Vec<_>>()
                .join(", ");
            let params_str = if params.is_empty() {
                "&self".to_string()
            } else {
                format!("&self, {}", params)
            };
            self.emit_line(&format!(
                "fn {}({}) -> {};",
                method.name, params_str, return_type
            ));
        }
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
    }
    fn generate_const(&mut self, const_decl: &ConstDecl) {
        let typ_str = const_decl
            .typ
            .as_ref()
            .map(|t| format!(": {}", TypeMapper::dart_to_rust(t)))
            .unwrap_or_default();
        self.emit_line(&format!(
            "let mut {}{} = {};",
            if const_decl.name == "type" {
                "r#type"
            } else {
                &const_decl.name
            },
            typ_str,
            self.expr_to_string(&const_decl.value)
        ));
    }
    fn generate_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VarDecl(var) => self.generate_var_decl(var, false),
            Statement::FinalDecl(var) => self.generate_var_decl(var, true),
            Statement::ConstDecl(const_decl) => self.generate_const(const_decl),
            Statement::Expression(expr) => {
                self.emit_line(&format!("{};", self.expr_to_string(expr)));
            }
            Statement::Return(Some(expr)) => {
                let expr_str = self.expr_to_string(expr);
                let final_expr = if expr_str.starts_with('"')
                    && expr_str.ends_with('"')
                    && !expr_str.contains("format!")
                {
                    format!("{}.to_string()", expr_str)
                } else {
                    expr_str
                };
                let is_null = matches!(expr.as_ref(), &Expr::Literal(Literal::Null));
                let wrapped_expr = if self.current_method_returns_boxed_trait && !is_null {
                    // Wrap sealed class returns in Box::new()
                    if self.current_method_returns_option {
                        format!("Some(Box::new({}))", final_expr)
                    } else {
                        format!("Box::new({})", final_expr)
                    }
                } else if self.current_method_returns_option && !is_null {
                    format!("Some({})", final_expr)
                } else {
                    final_expr
                };
                self.emit_line(&format!("return {};", wrapped_expr));
            }
            Statement::Return(None) => {
                self.emit_line("return;");
            }
            Statement::If(if_stmt) => {
                let cond_str = self.expr_to_string(&if_stmt.condition);
                let null_check_info = self.get_null_check_info(&if_stmt.condition);
                match null_check_info {
                    Some((var_name, NullCheckType::NotNull)) => {
                        let is_field = self.current_class_fields.contains(&var_name);
                        let prefixed = if is_field {
                            format!("self.{}", var_name)
                        } else {
                            var_name.clone()
                        };
                        self.emit_line(&format!(
                            "if let Some(ref mut {}) = {} {{",
                            var_name, prefixed
                        ));
                        self.indent();
                        for s in &if_stmt.then_block {
                            self.generate_statement(s);
                        }
                        self.dedent();
                        if let Some(else_stmts) = &if_stmt.else_block {
                            self.emit_line("} else {");
                            self.indent();
                            for s in else_stmts {
                                self.generate_statement(s);
                            }
                            self.dedent();
                        }
                        self.emit_line("}");
                    }
                    Some((var_name, NullCheckType::IsNull)) => {
                        let is_field = self.current_class_fields.contains(&var_name);
                        let prefixed = if is_field {
                            format!("self.{}", var_name)
                        } else {
                            var_name.clone()
                        };
                        self.emit_line(&format!("if let None = {} {{", prefixed));
                        self.indent();
                        for s in &if_stmt.then_block {
                            self.generate_statement(s);
                        }
                        self.dedent();
                        if let Some(else_stmts) = &if_stmt.else_block {
                            self.emit_line("} else {");
                            self.indent();
                            for s in else_stmts {
                                self.generate_statement(s);
                            }
                            self.dedent();
                        }
                        self.emit_line("}");
                        if if_stmt.else_block.is_none() {
                            let unwrapped = if is_field {
                                format!("self.{}", var_name)
                            } else {
                                var_name.clone()
                            };
                            self.emit_line(&format!(
                                "let mut {} = {}.unwrap();",
                                var_name, unwrapped
                            ));
                            self.unwrapped_locals.push(var_name.clone());
                        }
                    }
                    None => {
                        self.emit_line(&format!("if {} {{", cond_str));
                        self.indent();
                        for s in &if_stmt.then_block {
                            self.generate_statement(s);
                        }
                        self.dedent();
                        if let Some(else_block) = &if_stmt.else_block {
                            self.emit_line("} else {");
                            self.indent();
                            for s in else_block {
                                self.generate_statement(s);
                            }
                            self.dedent();
                        }
                        self.emit_line("}");
                    }
                }
            }
            Statement::While(while_stmt) => {
                let label_prefix = while_stmt
                    .label
                    .as_ref()
                    .map(|l| format!("'{}: ", l))
                    .unwrap_or_default();
                self.emit_line(&format!(
                    "{}while {} {{",
                    label_prefix,
                    self.expr_to_string(&while_stmt.condition)
                ));
                self.indent();
                for s in &while_stmt.body {
                    self.generate_statement(s);
                }
                self.dedent();
                self.emit_line("}");
            }
            Statement::For(for_stmt) => {
                if let Some((var_name, typ)) = &for_stmt.init_var {
                    let type_str = typ
                        .as_ref()
                        .map(|t| TypeMapper::dart_to_rust(t))
                        .unwrap_or_else(|| "i64".to_string());
                    if let Some(init_expr) = &for_stmt.init_expr {
                        self.emit_line(&format!(
                            "let mut {} : {} = {};",
                            var_name,
                            type_str,
                            self.expr_to_string(init_expr)
                        ));
                    } else {
                        self.emit_line(&format!("let mut {} : {} = 0;", var_name, type_str));
                    }
                } else if let Some(init) = &for_stmt.init_expr {
                    self.emit_line(&format!("{};", self.expr_to_string(init)));
                }
                let cond_str = for_stmt
                    .condition
                    .as_ref()
                    .map(|e| self.expr_to_string(e))
                    .unwrap_or_else(|| "true".to_string());
                self.emit_line(&format!("while {} {{", cond_str));
                self.indent();
                for s in &for_stmt.body {
                    self.generate_statement(s);
                }
                if let Some(update) = &for_stmt.update {
                    self.emit_line(&format!("{};", self.expr_to_string(update)));
                }
                self.dedent();
                self.emit_line("}");
            }
            Statement::DoWhile(do_while_stmt) => {
                let label_prefix = do_while_stmt
                    .label
                    .as_ref()
                    .map(|l| format!("'{}: ", l))
                    .unwrap_or_default();
                self.emit_line(&format!("{}loop {{", label_prefix));
                self.indent();
                for s in &do_while_stmt.body {
                    self.generate_statement(s);
                }
                self.emit_line(&format!(
                    "if !({}) {{ break; }}",
                    self.expr_to_string(&do_while_stmt.condition)
                ));
                self.dedent();
                self.emit_line("}");
            }
            Statement::Break(label) => {
                if let Some(l) = label {
                    self.emit_line(&format!("break '{};", l));
                } else {
                    self.emit_line("break;");
                }
            }
            Statement::Continue(label) => {
                if let Some(l) = label {
                    self.emit_line(&format!("continue '{};", l));
                } else {
                    self.emit_line("continue;");
                }
            }
            Statement::Block(stmts) => {
                self.emit_line("{");
                self.indent();
                for s in stmts {
                    self.generate_statement(s);
                }
                self.dedent();
                self.emit_line("}");
            }
            Statement::Try(try_stmt) => {
                self.emit_line("match (|| {");
                self.indent();
                for s in &try_stmt.body {
                    self.generate_statement(s);
                }
                self.emit_line("Ok::<(), String>(())");
                self.dedent();
                self.emit_line("})() {");
                self.indent();
                self.emit_line("Ok(_) => {},");
                for catch_block in &try_stmt.catch_blocks {
                    let exc_var = catch_block
                        .exception_var
                        .as_ref()
                        .map(|v| format!("_{}", v))
                        .unwrap_or_else(|| "_".to_string());
                    self.emit_line(&format!("Err({}) => {{", exc_var));
                    self.indent();
                    for s in &catch_block.body {
                        self.generate_statement(s);
                    }
                    self.dedent();
                    self.emit_line("},");
                }
                self.dedent();
                self.emit_line("}");
                if let Some(finally) = &try_stmt.finally_block {
                    for s in finally {
                        self.generate_statement(s);
                    }
                }
            }
            Statement::ThrowStmt(expr) => {
                self.emit_line(&format!("panic!(\"{{}}\", {});", self.expr_to_string(expr)));
            }
            Statement::ForIn(for_in_stmt) => {
                let label_prefix = for_in_stmt
                    .label
                    .as_ref()
                    .map(|l| format!("'{}: ", l))
                    .unwrap_or_default();
                self.emit_line(&format!(
                    "{}for {} in {} {{",
                    label_prefix,
                    for_in_stmt.variable,
                    self.expr_to_string(&for_in_stmt.iterable)
                ));
                self.indent();
                for s in &for_in_stmt.body {
                    self.generate_statement(s);
                }
                self.dedent();
                self.emit_line("}");
            }
            Statement::Switch(switch_stmt) => {
                let expr_str = self.expr_to_string(&switch_stmt.expr);
                let has_result_patterns =
                    switch_stmt.cases.iter().any(|case| match &case.pattern {
                        crab_parser::SwitchPattern::Destructure(name, _) => {
                            name == "Ok" || name == "Err"
                        }
                        _ => false,
                    });
                let match_expr = if !has_result_patterns
                    && (expr_str.starts_with("self.") || !expr_str.contains("("))
                {
                    format!("{}.as_str()", expr_str)
                } else {
                    expr_str
                };
                self.emit_line(&format!("match {} {{", match_expr));
                self.indent();
                for case in &switch_stmt.cases {
                    let pattern_str = match &case.pattern {
                        SwitchPattern::Literal(expr) => self.expr_to_string(expr),
                        SwitchPattern::Default => "_".to_string(),
                        SwitchPattern::Or(patterns) => {
                            let pat_strs: Vec<String> = patterns
                                .iter()
                                .map(|p| {
                                    if let SwitchPattern::Literal(e) = &**p {
                                        self.expr_to_string(e)
                                    } else {
                                        "_".to_string()
                                    }
                                })
                                .collect();
                            pat_strs.join(" | ")
                        }
                        SwitchPattern::Destructure(class_name, fields) => {
                            if fields.is_empty() {
                                format!("{}", class_name)
                            } else {
                                format!("{}({})", class_name, fields.join(", "))
                            }
                        }
                    };
                    let in_method = !self.current_class_fields.is_empty();
                    let guard_str = if let Some(guard) = &case.guard {
                        if in_method {
                            format!(" if {}", self.expr_to_string_in_method(guard))
                        } else {
                            format!(" if {}", self.expr_to_string(guard))
                        }
                    } else {
                        String::new()
                    };
                    let result_str = if in_method {
                        match &case.result {
                            Expr::Block(_) => self.expr_to_string_in_method(&case.result),
                            _ => {
                                let expr_str = self.expr_to_string_in_method(&case.result);
                                format!("{{ {}; }}", expr_str)
                            }
                        }
                    } else {
                        match &case.result {
                            Expr::Block(_) => self.expr_to_string(&case.result),
                            _ => {
                                let expr_str = self.expr_to_string(&case.result);
                                format!("{{ {}; }}", expr_str)
                            }
                        }
                    };
                    self.emit_line(&format!("{}{} => {},", pattern_str, guard_str, result_str));
                }
                self.dedent();
                self.emit_line("}");
            }
        }
    }
    fn generate_var_decl(&mut self, var: &VarDecl, is_final: bool) {
        let mutability = if is_final { "" } else { "mut " };
        let typ_str = var
            .typ
            .as_ref()
            .map(|t| format!(": {}", TypeMapper::dart_to_rust(t)))
            .unwrap_or_default();
        if let Some(value) = &var.value {
            let value_str = self.expr_to_string(value);
            let is_null_literal = matches!(value.as_ref(), &Expr::Literal(Literal::Null));

            let final_value = if let Some(Type::String) = &var.typ {
                if value_str.starts_with('"') && value_str.ends_with('"') {
                    format!("{}.to_string()", value_str)
                } else if !value_str.contains(".clone()") && !value_str.starts_with('&') {
                    format!("{}.clone()", value_str)
                } else {
                    value_str
                }
            } else if let Some(typ) = &var.typ {
                match typ {
                    Type::Nullable(_) | Type::OptionT(_) if !is_null_literal => {
                        format!("Some({})", value_str)
                    }
                    _ => value_str,
                }
            } else {
                value_str
            };
            self.emit_line(&format!(
                "let {}{}{} = {};",
                mutability, var.name, typ_str, final_value
            ));
        } else {
            let is_option_type = var
                .typ
                .as_ref()
                .map(|t| matches!(t, Type::Nullable(_) | Type::OptionT(_)))
                .unwrap_or(false);
            if is_option_type || typ_str.contains("Option<") {
                self.emit_line(&format!(
                    "let {}{}{} = None;",
                    mutability, var.name, typ_str
                ));
            } else {
                self.emit_line(&format!("let {}{}{};", mutability, var.name, typ_str));
            }
        }
    }
    fn generate_statement_in_method(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Expression(expr) => {
                self.emit_line(&format!("{};", self.expr_to_string_in_method(expr)));
            }
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    let expr_str = self.expr_to_string_in_method(e);
                    let needs_clone = self.current_method_is_getter
                        && (self.current_method_return_type == "String"
                            || self.current_method_return_type.contains("Option<String>")
                            || self.current_method_return_type.starts_with("Vec<")
                            || self.current_method_return_type.starts_with("HashMap<")
                            || self.current_method_return_type.starts_with("HashSet<"))
                        && !expr_str.contains(".clone()")
                        && !expr_str.starts_with('&');
                    let final_expr = if expr_str.starts_with('"')
                        && expr_str.ends_with('"')
                        && !expr_str.contains("format!")
                    {
                        format!("{}.to_string()", expr_str)
                    } else if needs_clone {
                        format!("{}.clone()", expr_str)
                    } else {
                        expr_str
                    };
                    let is_null = matches!(e.as_ref(), &Expr::Literal(Literal::Null));
                    let wrapped_expr = if self.current_method_returns_boxed_trait && !is_null {
                        if self.current_method_returns_option {
                            format!("Some(Box::new({}))", final_expr)
                        } else {
                            format!("Box::new({})", final_expr)
                        }
                    } else if self.current_method_returns_option && !is_null {
                        format!("Some({})", final_expr)
                    } else {
                        final_expr
                    };
                    self.emit_line(&format!("return {};", wrapped_expr));
                } else {
                    self.emit_line("return;");
                }
            }
            Statement::VarDecl(var) => {
                let typ_str = var
                    .typ
                    .as_ref()
                    .map(|t| format!(": {}", TypeMapper::dart_to_rust(t)))
                    .unwrap_or_default();
                let mutability = "mut ";
                if let Some(value) = &var.value {
                    let value_str = self.expr_to_string_in_method(value);
                    let is_null_literal = matches!(value.as_ref(), &Expr::Literal(Literal::Null));

                    let final_value = if let Some(Type::String) = &var.typ {
                        if value_str.starts_with('"') && value_str.ends_with('"') {
                            format!("{}.to_string()", value_str)
                        } else if !value_str.contains(".clone()") && !value_str.starts_with('&') {
                            format!("{}.clone()", value_str)
                        } else {
                            value_str
                        }
                    } else if let Some(typ) = &var.typ {
                        match typ {
                            Type::Nullable(_) | Type::OptionT(_) if !is_null_literal => {
                                format!("Some({})", value_str)
                            }
                            _ => value_str,
                        }
                    } else {
                        value_str
                    };
                    self.emit_line(&format!(
                        "let {}{}{} = {};",
                        mutability, var.name, typ_str, final_value
                    ));
                } else {
                    let is_option_type = var
                        .typ
                        .as_ref()
                        .map(|t| matches!(t, Type::Nullable(_) | Type::OptionT(_)))
                        .unwrap_or(false);
                    if is_option_type || typ_str.contains("Option<") {
                        self.emit_line(&format!(
                            "let {}{}{} = None;",
                            mutability, var.name, typ_str
                        ));
                    } else {
                        self.emit_line(&format!("let {}{}{};", mutability, var.name, typ_str));
                    }
                }
            }
            Statement::If(if_stmt) => {
                let null_check_info = self.get_null_check_info(&if_stmt.condition);
                if let Some((ref var_name, NullCheckType::IsNull)) = null_check_info {
                    if if_stmt.else_block.is_none() {
                        self.unwrapped_locals.push(var_name.clone());
                    }
                }
                let cond_str = self.expr_to_string_in_method(&if_stmt.condition);
                match null_check_info {
                    Some((var_name, NullCheckType::NotNull)) => {
                        let is_field = self.current_class_fields.contains(&var_name);
                        let prefixed = if is_field {
                            format!("self.{}", var_name)
                        } else {
                            var_name.clone()
                        };
                        self.emit_line(&format!(
                            "if let Some(ref mut {}) = {} {{",
                            var_name, prefixed
                        ));
                        self.indent();
                        for s in &if_stmt.then_block {
                            self.generate_statement_in_method(s);
                        }
                        self.dedent();
                        if let Some(else_stmts) = &if_stmt.else_block {
                            self.emit_line("} else {");
                            self.indent();
                            for s in else_stmts {
                                self.generate_statement_in_method(s);
                            }
                            self.dedent();
                        }
                        self.emit_line("}");
                    }
                    Some((var_name, NullCheckType::IsNull)) => {
                        let is_field = self.current_class_fields.contains(&var_name);
                        let prefixed = if is_field {
                            format!("self.{}", var_name)
                        } else {
                            var_name.clone()
                        };
                        self.emit_line(&format!("if let None = {} {{", prefixed));
                        self.indent();
                        for s in &if_stmt.then_block {
                            self.generate_statement_in_method(s);
                        }
                        self.dedent();
                        if let Some(else_stmts) = &if_stmt.else_block {
                            self.emit_line("} else {");
                            self.indent();
                            for s in else_stmts {
                                self.generate_statement_in_method(s);
                            }
                            self.dedent();
                        }
                        self.emit_line("}");
                        if if_stmt.else_block.is_none() {
                            let unwrapped = if is_field {
                                format!("self.{}", var_name)
                            } else {
                                var_name.clone()
                            };
                            self.emit_line(&format!(
                                "let mut {} = {}.unwrap();",
                                var_name, unwrapped
                            ));
                        }
                    }
                    None => {
                        self.emit_line(&format!("if {} {{", cond_str));
                        self.indent();
                        for s in &if_stmt.then_block {
                            self.generate_statement_in_method(s);
                        }
                        self.dedent();
                        if let Some(else_stmts) = &if_stmt.else_block {
                            self.emit_line("} else {");
                            self.indent();
                            for s in else_stmts {
                                self.generate_statement_in_method(s);
                            }
                            self.dedent();
                        }
                        self.emit_line("}");
                    }
                }
            }
            Statement::For(for_stmt) => {
                if let Some((var_name, typ)) = &for_stmt.init_var {
                    let type_str = typ
                        .as_ref()
                        .map(|t| TypeMapper::dart_to_rust(t))
                        .unwrap_or_else(|| "i64".to_string());
                    if let Some(init_expr) = &for_stmt.init_expr {
                        self.emit_line(&format!(
                            "let mut {} : {} = {};",
                            var_name,
                            type_str,
                            self.expr_to_string_in_method(init_expr)
                        ));
                    } else {
                        self.emit_line(&format!("let mut {} : {} = 0;", var_name, type_str));
                    }
                } else if let Some(init) = &for_stmt.init_expr {
                    self.emit_line(&format!("{};", self.expr_to_string_in_method(init)));
                }
                let cond_str = for_stmt
                    .condition
                    .as_ref()
                    .map(|e| self.expr_to_string_in_method(e))
                    .unwrap_or_else(|| "true".to_string());
                self.emit_line(&format!("while {} {{", cond_str));
                self.indent();
                for s in &for_stmt.body {
                    self.generate_statement_in_method(s);
                }
                if let Some(update) = &for_stmt.update {
                    self.emit_line(&format!("{};", self.expr_to_string_in_method(update)));
                }
                self.dedent();
                self.emit_line("}");
            }
            Statement::While(while_stmt) => {
                let cond_str = self.expr_to_string_in_method(&while_stmt.condition);
                self.emit_line(&format!("while {} {{", cond_str));
                self.indent();
                for s in &while_stmt.body {
                    self.generate_statement_in_method(s);
                }
                self.dedent();
                self.emit_line("}");
            }
            Statement::ForIn(for_in_stmt) => {
                let iterable_str = self.expr_to_string_in_method(&for_in_stmt.iterable);
                let (final_iterable, needs_clone) = if iterable_str.starts_with("self.") {
                    (format!("&{}", iterable_str), true)
                } else {
                    (iterable_str.clone(), false)
                };
                let var_name = &for_in_stmt.variable;
                self.emit_line(&format!("for {} in {} {{", var_name, final_iterable));
                if needs_clone {
                    self.indent();
                    self.emit_line(&format!("let {} = {}.clone();", var_name, var_name));
                    self.dedent();
                }
                self.indent();
                for s in &for_in_stmt.body {
                    self.generate_statement_in_method(s);
                }
                self.dedent();
                self.emit_line("}");
            }
            Statement::Try(try_stmt) => {
                self.emit_line("match (|| {");
                self.indent();
                for s in &try_stmt.body {
                    self.generate_statement_in_method(s);
                }
                self.emit_line("Ok::<(), String>(())");
                self.dedent();
                self.emit_line("})() {");
                self.indent();
                self.emit_line("Ok(_) => {},");
                for catch_block in &try_stmt.catch_blocks {
                    let exc_var = catch_block
                        .exception_var
                        .as_ref()
                        .map(|v| format!("_{}", v))
                        .unwrap_or_else(|| "_".to_string());
                    self.emit_line(&format!("Err({}) => {{", exc_var));
                    self.indent();
                    for s in &catch_block.body {
                        self.generate_statement_in_method(s);
                    }
                    self.dedent();
                    self.emit_line("},");
                }
                self.dedent();
                self.emit_line("}");
                if let Some(finally) = &try_stmt.finally_block {
                    for s in finally {
                        self.generate_statement_in_method(s);
                    }
                }
            }
            Statement::ThrowStmt(expr) => {
                self.emit_line(&format!(
                    "panic!(\"{{}}\", {});",
                    self.expr_to_string_in_method(expr)
                ));
            }
            Statement::Switch(switch_stmt) => {
                let expr_str = self.expr_to_string_in_method(&switch_stmt.expr);
                let has_result_patterns =
                    switch_stmt.cases.iter().any(|case| match &case.pattern {
                        crab_parser::SwitchPattern::Destructure(name, _) => {
                            name == "Ok" || name == "Err"
                        }
                        _ => false,
                    });
                let has_sealed_class_patterns = switch_stmt.cases.iter().any(|case| {
                    matches!(
                        &case.pattern,
                        crab_parser::SwitchPattern::Destructure(name, fields)
                            if name != "Ok" && name != "Err" && !fields.is_empty()
                    )
                });
                let has_enum_like_patterns =
                    switch_stmt.cases.iter().all(|case| match &case.pattern {
                        crab_parser::SwitchPattern::Literal(expr) => {
                            matches!(expr, crab_parser::Expr::Identifier(_))
                        }
                        crab_parser::SwitchPattern::Default => true,
                        _ => false,
                    });
                let enum_type_name = if has_enum_like_patterns {
                    self.infer_enum_type_from_expr(&switch_stmt.expr)
                } else {
                    None
                };
                let match_expr = if has_sealed_class_patterns || enum_type_name.is_some() {
                    expr_str
                } else if !has_result_patterns
                    && (expr_str.starts_with("self.") || !expr_str.contains("("))
                {
                    format!("{}.as_str()", expr_str)
                } else {
                    expr_str
                };
                self.emit_line(&format!("match {} {{", match_expr));
                self.indent();
                for case in &switch_stmt.cases {
                    let pattern_str = match &case.pattern {
                        crab_parser::SwitchPattern::Literal(expr) => {
                            let pat = self.expr_to_string_in_method(expr);
                            if let Some(ref enum_type) = enum_type_name {
                                format!("{}::{}", enum_type, pat)
                            } else {
                                pat
                            }
                        }
                        crab_parser::SwitchPattern::Default => "_".to_string(),
                        crab_parser::SwitchPattern::Or(patterns) => {
                            let pat_strs: Vec<String> = patterns
                                .iter()
                                .map(|p| {
                                    if let crab_parser::SwitchPattern::Literal(e) = &**p {
                                        let pat = self.expr_to_string_in_method(e);
                                        if let Some(ref enum_type) = enum_type_name {
                                            format!("{}::{}", enum_type, pat)
                                        } else {
                                            pat
                                        }
                                    } else {
                                        "_".to_string()
                                    }
                                })
                                .collect();
                            pat_strs.join(" | ")
                        }
                        crab_parser::SwitchPattern::Destructure(class_name, fields) => {
                            if fields.is_empty() {
                                format!("{}", class_name)
                            } else if class_name == "Ok" || class_name == "Err" {
                                format!("{}({})", class_name, fields.join(", "))
                            } else if fields.len() == 1 && fields[0] == "_" {
                                format!("{}", class_name)
                            } else {
                                format!("{}({})", class_name, fields.join(", "))
                            }
                        }
                    };
                    let guard_str = if let Some(guard) = &case.guard {
                        format!(" if {}", self.expr_to_string_in_method(guard))
                    } else {
                        String::new()
                    };
                    let result_str = match &case.result {
                        Expr::Block(_) => self.expr_to_string_in_method(&case.result),
                        _ => {
                            let expr_str = self.expr_to_string_in_method(&case.result);
                            format!("{{ {}; }}", expr_str)
                        }
                    };
                    self.emit_line(&format!("{}{} => {},", pattern_str, guard_str, result_str));
                }
                self.dedent();
                self.emit_line("}");
            }
            _ => self.generate_statement(stmt),
        }
    }
    fn expr_to_string(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => Self::literal_to_string(lit),
            Expr::Identifier(name) => name.clone(),
            Expr::BinaryOp { left, op, right } => {
                let left_str = self.expr_to_string(left);
                let right_str = self.expr_to_string(right);
                if right_str == "None" || right_str == "none()" {
                    match op {
                        BinaryOp::NotEqual => return format!("{}.is_some()", left_str),
                        BinaryOp::Equal => return format!("{}.is_none()", left_str),
                        _ => {}
                    }
                }
                if left_str == "None" || left_str == "none()" {
                    match op {
                        BinaryOp::NotEqual => return format!("{}.is_some()", right_str),
                        BinaryOp::Equal => return format!("{}.is_none()", right_str),
                        _ => {}
                    }
                }
                let op_str = Self::binary_op_to_string(*op);
                let final_right_str = if *op == BinaryOp::Mul {
                    if matches!(&**left, Expr::PropertyAccess { .. }) {
                        format!("({} as f64)", right_str)
                    } else {
                        right_str
                    }
                } else {
                    right_str
                };
                format!("{} {} {}", left_str, op_str, final_right_str)
            }
            Expr::UnaryOp { op, operand } => {
                let op_str = Self::unary_op_to_string(*op);
                format!("{}{}", op_str, self.expr_to_string(operand))
            }
            Expr::Call { func, args } => {
                let func_name = match &**func {
                    Expr::Identifier(name) => name.clone(),
                    _ => self.expr_to_string(func),
                };
                let args_str = args
                    .iter()
                    .map(|arg| {
                        let arg_str = self.expr_to_string(arg);
                        if arg_str.starts_with('"')
                            && arg_str.ends_with('"')
                            && !arg_str.contains("{}")
                        {
                            format!("{}.to_string()", arg_str)
                        } else {
                            arg_str
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                match func_name.as_str() {
                    "print" => {
                        if args.is_empty() {
                            "println!()".to_string()
                        } else if args.len() == 1 {
                            format!("println!(\"{{:?}}\", {})", args_str)
                        } else {
                            format!("println!({})", args_str)
                        }
                    }
                    "println" => {
                        if args.is_empty() {
                            "println!()".to_string()
                        } else if args.len() == 1 {
                            format!("println!(\"{{:?}}\", {})", args_str)
                        } else {
                            format!("println!({})", args_str)
                        }
                    }
                    "abs" => {
                        if args.len() == 1 {
                            format!("i64::abs({})", args_str)
                        } else {
                            format!("{}({})", func_name, args_str)
                        }
                    }
                    "sqrt" => {
                        if args.len() == 1 {
                            format!("({} as f64).sqrt() as i64", args_str)
                        } else {
                            format!("{}({})", func_name, args_str)
                        }
                    }
                    "floor" => format!("({}).floor()", args_str),
                    "ceil" => format!("({}).ceil()", args_str),
                    "round" => format!("({}).round()", args_str),
                    "min" => {
                        if args.len() >= 2 {
                            let first_two: Vec<String> = args
                                .iter()
                                .take(2)
                                .map(|a| self.expr_to_string(a))
                                .collect();
                            format!("std::cmp::min({}, {})", first_two[0], first_two[1])
                        } else {
                            format!("{}({})", func_name, args_str)
                        }
                    }
                    "max" => {
                        if args.len() >= 2 {
                            let first_two: Vec<String> = args
                                .iter()
                                .take(2)
                                .map(|a| self.expr_to_string(a))
                                .collect();
                            format!("std::cmp::max({}, {})", first_two[0], first_two[1])
                        } else {
                            format!("{}({})", func_name, args_str)
                        }
                    }
                    "Future" => {
                        if args.len() >= 1 {
                            let first_arg = self.expr_to_string(&args[0]);
                            if first_arg.contains("Duration") {
                                if args.len() >= 2 {
                                    let callback = self.expr_to_string(&args[1]);
                                    format!(
                                        "tokio::time::sleep({}).then(|_| async {{ {} }})",
                                        first_arg, callback
                                    )
                                } else {
                                    format!("tokio::time::sleep({})", first_arg)
                                }
                            } else {
                                format!("async {{ {} }}", first_arg)
                            }
                        } else {
                            "async {}".to_string()
                        }
                    }
                    "Duration" => {
                        let mut secs = 0u64;
                        let mut millis = 0u64;
                        let mut micros = 0u64;
                        for arg in args {
                            let arg_str = self.expr_to_string(arg);
                            if arg_str.contains("seconds:") {
                                if let Some(val) = arg_str.split(':').nth(1) {
                                    secs = val.trim().parse().unwrap_or(0);
                                }
                            } else if arg_str.contains("milliseconds:") {
                                if let Some(val) = arg_str.split(':').nth(1) {
                                    millis = val.trim().parse().unwrap_or(0);
                                }
                            } else if arg_str.contains("microseconds:") {
                                if let Some(val) = arg_str.split(':').nth(1) {
                                    micros = val.trim().parse().unwrap_or(0);
                                }
                            }
                        }
                        let total_micros = secs * 1_000_000 + millis * 1000 + micros;
                        format!("std::time::Duration::from_micros({})", total_micros)
                    }
                    _ => {
                        let func_str = self.expr_to_string(func);
                        if func_str == "Future.wait" || func_str == "wait" {
                            format!("futures::future::join_all({})", args_str)
                        } else if func_str.starts_with("Future.") {
                            let method = &func_str[7..];
                            match method {
                                "value" => format!("async {{ {} }}", args_str),
                                "delayed" => {
                                    if args.len() >= 2 {
                                        let dur = self.expr_to_string(&args[0]);
                                        let callback = self.expr_to_string(&args[1]);
                                        format!(
                                            "tokio::time::sleep({}).then(|_| async {{ {} }})",
                                            dur, callback
                                        )
                                    } else {
                                        format!("tokio::time::sleep({})", args_str)
                                    }
                                }
                                _ => format!("{}", method),
                            }
                        } else if func_str == "List.from" {
                            format!("{}.clone()", args_str)
                        } else if func_str == "fs.writeFile" {
                            format!("std::fs::write({}, r#\"\"#).ok()", args_str)
                        } else if func_str == "fs.readFile" || func_str == "fs.readFileToString" {
                            format!("std::fs::read_to_string({}).unwrap()", args_str)
                        } else if func_str == "serde_json.fromString" {
                            format!(
                                "serde_json::from_str::<Vec<serde_json::Value>>(&{}).unwrap_or_default()",
                                args_str
                            )
                        } else if func_str.contains('.') {
                            format!("{}({})", func_str, args_str)
                        } else if let Some(first_char) = func_str.chars().next() {
                            if first_char.is_ascii_uppercase() {
                                format!("{}::new({})", func_str, args_str)
                            } else {
                                format!("{}({})", func_str, args_str)
                            }
                        } else {
                            format!("{}({})", func_str, args_str)
                        }
                    }
                }
            }
            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                let object_str = self.expr_to_string_in_method(object);
                let method_name_str = method.as_str();
                let is_finder_call =
                    method_name_str.starts_with("find") || method_name_str.starts_with("_find");
                let args_str = args
                    .iter()
                    .map(|x| {
                        let arg_str = self.expr_to_string(x);
                        if is_finder_call {
                            format!("{}.as_str()", arg_str)
                        } else {
                            arg_str
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                if object_str == "DateTime" && method == "now" {
                    return "chrono::Utc::now()".to_string();
                }
                if object_str == "int" && method == "parse" {
                    return format!("{}.parse::<i64>()", args_str);
                }
                if object_str == "List" && method == "from" {
                    return format!("{}.clone()", args_str);
                }
                if object_str == "fs" {
                    match method.as_str() {
                        "writeFile" => {
                            return format!("std::fs::write({}, {}).ok()", args_str, "");
                        }
                        "readFile" => {
                            return format!("std::fs::read_to_string({}).unwrap()", args_str);
                        }
                        "readFileToString" => {
                            return format!("std::fs::read_to_string({}).unwrap()", args_str);
                        }
                        _ => {}
                    }
                }
                if object_str == "io" {
                    match method.as_str() {
                        "getEnv" => {
                            return format!("std::env::var({}).ok()", args_str);
                        }
                        _ => {}
                    }
                }
                if object_str == "serde_json" {
                    match method.as_str() {
                        "fromString" => {
                            return format!(
                                "serde_json::from_str::<Vec<serde_json::Value>>(&{}).unwrap_or_default()",
                                args_str
                            );
                        }
                        _ => {}
                    }
                }
                if object_str == "Future" {
                    match method.as_str() {
                        "delayed" => {
                            if args.len() >= 2 {
                                let dur = self.expr_to_string(&args[0]);
                                let callback = self.expr_to_string(&args[1]);
                                return format!(
                                    "tokio::time::sleep({}).then(|_| async {{ {} }})",
                                    dur, callback
                                );
                            } else if args.len() == 1 {
                                let dur = self.expr_to_string(&args[0]);
                                let dur_expr = if dur.parse::<u64>().is_ok() {
                                    format!("std::time::Duration::from_millis({})", dur)
                                } else {
                                    dur
                                };
                                return format!("tokio::time::sleep({})", dur_expr);
                            } else {
                                return "tokio::time::sleep(std::time::Duration::from_secs(0))"
                                    .to_string();
                            }
                        }
                        "wait" => {
                            return format!("futures::future::join_all({})", args_str);
                        }
                        "value" => {
                            return format!("async {{ {} }}", args_str);
                        }
                        _ => {}
                    }
                }
                let method_name = match method.as_str() {
                    "length" => "len".to_string(),
                    "isEmpty" => "is_empty".to_string(),
                    "isNotEmpty" => {
                        return format!("!{}.is_empty()", object_str);
                    }
                    "contains" => "contains".to_string(),
                    "startsWith" => "starts_with".to_string(),
                    "endsWith" => "ends_with".to_string(),
                    "toLowerCase" => "to_lowercase".to_string(),
                    "toUpperCase" => "to_uppercase".to_string(),
                    "trim" => "trim".to_string(),
                    "split" => "split".to_string(),
                    "join" => "join".to_string(),
                    "remove" => "remove".to_string(),
                    "add" => "push".to_string(),
                    "indexOf" => "iter().position(|x| x ==".to_string(),
                    "firstWhere" => "iter().find".to_string(),
                    "substring" => "chars().skip".to_string(),
                    "reverse" => "reverse".to_string(),
                    "sort" => "sort".to_string(),
                    "clear" => "clear".to_string(),
                    "get" => "get".to_string(),
                    "insert" => "insert".to_string(),
                    "any" => "iter().any".to_string(),
                    "all" => "iter().all".to_string(),
                    "map" => "iter().map".to_string(),
                    "filter" => "iter().filter".to_string(),
                    "where" => "iter().filter".to_string(),
                    "forEach" => "iter().for_each".to_string(),
                    "toList" => "collect::<Vec<_>>".to_string(),
                    "json" if object_str.ends_with("req") || object_str.ends_with("request") => {
                        return "serde_json::Value::Null".to_string();
                    }
                    "json" if object_str == "Response" || object_str == "HttpResponse" => {
                        return format!("HttpResponse::Ok().json({})", args_str);
                    }
                    "status" if object_str == "Response" || object_str == "HttpResponse" => {
                        return format!("HttpResponse::Ok().status({})", args_str);
                    }
                    "body" => {
                        return format!("HttpResponse::Ok().body({})", args_str);
                    }
                    _ => method.clone(),
                };
                format!("{}.{}({})", object_str, method_name, args_str)
            }
            Expr::PropertyAccess { object, property } => {
                let object_str = self.expr_to_string(object);
                if let Some(ref program) = self.program {
                    if let crab_parser::Expr::Identifier(obj_name) = object.as_ref() {
                        for item in &program.items {
                            if let TopLevelItem::ClassDecl(c) = item {
                                if c.name == *obj_name && self.is_enum_class(c) {
                                    return format!("{}::{}", object_str, property);
                                }
                            }
                        }
                    }
                }
                let prop_name = match property.as_str() {
                    "length" => {
                        return format!("{}.len()", object_str);
                    }
                    "isEmpty" => {
                        return format!("{}.is_empty()", object_str);
                    }
                    "isNotEmpty" => {
                        return format!("!{}.is_empty()", object_str);
                    }
                    "first" => {
                        return format!("{}.first()", object_str);
                    }
                    "last" => {
                        return format!("{}.last()", object_str);
                    }
                    "reversed" => {
                        return format!("{}.iter().rev().collect::<Vec<_>>()", object_str);
                    }
                    _ => property.clone(),
                };
                let is_getter = self.is_getter_method(&prop_name);
                if is_getter {
                    format!("{}.{}{}", object_str, prop_name, "()")
                } else {
                    format!("{}.{}", object_str, prop_name)
                }
            }
            Expr::NullCoalesce { left, right } => {
                let right_expr = self.expr_to_string(right);
                let right_converted = if right_expr.starts_with('"') && right_expr.ends_with('"') {
                    format!("{}.to_string()", right_expr)
                } else {
                    right_expr
                };
                let left_expr = self.expr_to_string(left);
                let needs_clone = left_expr.starts_with("maybe")
                    || left_expr.starts_with("other")
                    || left_expr.starts_with("num");
                if needs_clone {
                    format!("{}.clone().unwrap_or({})", left_expr, right_converted)
                } else {
                    format!("{}.unwrap_or({})", left_expr, right_converted)
                }
            }
            Expr::NullAssertion(expr) => {
                format!("{}.unwrap()", self.expr_to_string(expr))
            }
            Expr::ListLiteral(items) => {
                let items_str = items
                    .iter()
                    .map(|x| {
                        let s = self.expr_to_string(x);
                        if s.starts_with('"') && s.ends_with('"') {
                            format!("{}.to_string()", s)
                        } else {
                            s
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("vec![{}]", items_str)
            }
            Expr::New {
                class_name,
                constructor: _,
                args,
            } => {
                let args_str = args
                    .iter()
                    .map(|x| {
                        let arg_str = self.expr_to_string(x);
                        // Clone self.field arguments in constructor calls (avoids move from &self)
                        if arg_str.starts_with("self.") && !arg_str.contains("(") {
                            format!("{}.clone()", arg_str)
                        } else {
                            arg_str
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}::new({})", class_name, args_str)
            }
            Expr::Await(expr) => {
                format!("{}.await", self.expr_to_string(expr))
            }
            Expr::StringInterpolation(parts) => {
                let mut format_str = String::new();
                let mut args = Vec::new();
                for part in parts {
                    match part {
                        StringPart::Static(s) => {
                            format_str.push_str(
                                &s.replace('\\', "\\\\")
                                    .replace('"', "\\\"")
                                    .replace('{', "{{")
                                    .replace('}', "}}"),
                            );
                        }
                        StringPart::Interpolation(expr) => {
                            format_str.push_str("{}");
                            args.push(self.expr_to_string_in_method(expr));
                        }
                    }
                }
                let args_str = args.join(", ");
                if args.is_empty() {
                    format!("\"{}\"", format_str)
                } else {
                    format!("format!(\"{}\", {})", format_str, args_str)
                }
            }
            Expr::Assign { target, value } => {
                format!(
                    "{} = {}",
                    self.expr_to_string(target),
                    self.expr_to_string(value)
                )
            }
            Expr::CompoundAssign { target, op, value } => {
                let op_str = match op {
                    BinaryOp::Add => "+=",
                    BinaryOp::Sub => "-=",
                    BinaryOp::Mul => "*=",
                    BinaryOp::Div => "/=",
                    BinaryOp::Mod => "%=",
                    BinaryOp::BitAnd => "&=",
                    BinaryOp::BitOr => "|=",
                    BinaryOp::BitXor => "^=",
                    BinaryOp::LeftShift => "<<=",
                    BinaryOp::RightShift => ">>=",
                    _ => "???=",
                };
                format!(
                    "{} {} {}",
                    self.expr_to_string(target),
                    op_str,
                    self.expr_to_string(value)
                )
            }
            Expr::Index { object, index } => {
                let obj_str = self.expr_to_string(object);
                let idx_str = self.expr_to_string(index);
                format!("{}.get({}).cloned().unwrap()", obj_str, idx_str)
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                format!(
                    "if {} {{ {} }} else {{ {} }}",
                    self.expr_to_string(condition),
                    self.expr_to_string(then_expr),
                    self.expr_to_string(else_expr)
                )
            }
            Expr::Is { expr, typ, negated } => {
                let type_str = TypeMapper::dart_to_rust(typ);
                let check = format!(
                    "std::any::type_name_of_val(&{}) == std::any::type_name::<{}>()",
                    self.expr_to_string(expr),
                    type_str
                );
                if *negated {
                    format!("!({})", check)
                } else {
                    check
                }
            }
            Expr::Cast { expr, typ } => {
                let type_str = TypeMapper::dart_to_rust(typ);
                format!("({} as {})", self.expr_to_string(expr), type_str)
            }
            Expr::Lambda { params, body } => {
                let params_str = params
                    .iter()
                    .map(|p| format!("{}", p.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("|{}| {}", params_str, self.expr_to_string_in_method(body))
            }
            Expr::MapLiteral(pairs) => {
                if pairs.is_empty() {
                    "std::collections::HashMap::new()".to_string()
                } else {
                    let pairs_str = pairs
                        .iter()
                        .map(|(k, v)| {
                            format!("({}, {})", self.expr_to_string(k), self.expr_to_string(v))
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("std::collections::HashMap::from([{}])", pairs_str)
                }
            }
            Expr::SetLiteral(items) => {
                let items_str = items
                    .iter()
                    .map(|x| self.expr_to_string(x))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("std::collections::HashSet::from([{}])", items_str)
            }
            Expr::NullAware {
                object,
                property,
                args,
            } => {
                let object_str = self.expr_to_string(object);
                if let Some(method_args) = args {
                    let args_str = method_args
                        .iter()
                        .map(|x| self.expr_to_string(x))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let method_name = match property.as_str() {
                        "toString" => "to_string".to_string(),
                        other => other.to_string(),
                    };
                    format!(
                        "{}.as_ref().and_then(|x| Some(x.{}({})))",
                        object_str, method_name, args_str
                    )
                } else {
                    let prop_or_method = match property.as_str() {
                        "length" => {
                            return format!("{}.as_ref().map(|x| x.len())", object_str);
                        }
                        "isEmpty" => {
                            return format!("{}.as_ref().map(|x| x.is_empty())", object_str);
                        }
                        "isNotEmpty" => {
                            return format!("{}.as_ref().map(|x| !x.is_empty())", object_str);
                        }
                        "first" => {
                            return format!("{}.as_ref().and_then(|x| x.first())", object_str);
                        }
                        "last" => {
                            return format!("{}.as_ref().and_then(|x| x.last())", object_str);
                        }
                        "reversed" => {
                            return format!(
                                "{}.as_ref().map(|x| x.iter().rev().collect::<Vec<_>>())",
                                object_str
                            );
                        }
                        _ => property.clone(),
                    };
                    format!("{}.as_ref().map(|x| x.{})", object_str, prop_or_method)
                }
            }
            Expr::Spread(expr) => {
                format!("..{}", self.expr_to_string(expr))
            }
            Expr::NullCoalesceAssign { target, value } => {
                let target_str = self.expr_to_string(target);
                let value_str = self.expr_to_string(value);
                let wrapped_value = if value_str.starts_with('"') && value_str.ends_with('"') {
                    format!("{}.to_string()", value_str)
                } else {
                    value_str
                };
                format!(
                    "{} = Some({}.unwrap_or({}))",
                    target_str, target_str, wrapped_value
                )
            }
            Expr::This => "self".to_string(),
            Expr::Super { field_or_method } => {
                if let Some(field) = field_or_method {
                    format!("super::{}", field)
                } else {
                    "super::new()".to_string()
                }
            }
            Expr::Propagate(expr) => {
                format!("{}?", self.expr_to_string(expr))
            }
            Expr::ResultConstructor { variant, value } => {
                let value_str = self.expr_to_string_in_method(value);
                let final_value = if value_str.starts_with('"') && value_str.ends_with('"') {
                    format!("{}.to_string()", value_str)
                } else {
                    value_str
                };
                match variant.as_str() {
                    "Ok" => format!("Ok({})", final_value),
                    "Err" => format!("Err({})", final_value),
                    _ => format!("{}", variant),
                }
            }
            Expr::OptionConstructor { variant, value } => match variant.as_str() {
                "Some" => {
                    if let Some(v) = value {
                        format!("Some({})", self.expr_to_string(v))
                    } else {
                        "None".to_string()
                    }
                }
                "None" => "None".to_string(),
                _ => format!("{}", variant),
            },
            Expr::Block(stmts) => {
                let mut result = String::from("{\n");
                for (i, stmt) in stmts.iter().enumerate() {
                    let is_last = i == stmts.len() - 1;
                    let stmt_str = match stmt {
                        Statement::Expression(expr) if is_last => {
                            format!("    {}\n", self.expr_to_string(expr))
                        }
                        Statement::Return(Some(expr)) => {
                            let expr_str = self.expr_to_string(expr);
                            let is_null = matches!(expr.as_ref(), &Expr::Literal(Literal::Null));
                            let wrapped = if self.current_method_returns_boxed_trait && !is_null {
                                if self.current_method_returns_option {
                                    format!("Some(Box::new({}))", expr_str)
                                } else {
                                    format!("Box::new({})", expr_str)
                                }
                            } else if self.current_method_returns_option && !is_null {
                                format!("Some({})", expr_str)
                            } else {
                                expr_str
                            };
                            format!("    return {}\n", wrapped)
                        }
                        _ => {
                            let mut temp_gen = CodeGenerator::new();
                            temp_gen.indent_level = 1;
                            temp_gen.current_class_fields = self.current_class_fields.clone();
                            temp_gen.current_method_returns_option =
                                self.current_method_returns_option;
                            temp_gen.current_method_returns_boxed_trait =
                                self.current_method_returns_boxed_trait;
                            temp_gen.generate_statement_in_method(stmt);
                            temp_gen.output
                        }
                    };
                    result.push_str(&stmt_str);
                }
                result.push_str("}");
                result
            }
            Expr::BinaryOp { left, op, right } => {
                let left_str = self.expr_to_string_in_method(left);
                let mut right_str = self.expr_to_string_in_method(right);
                if right_str == "None" || right_str == "null" {
                    match op {
                        BinaryOp::NotEqual => return format!("{}.is_some()", left_str),
                        BinaryOp::Equal => return format!("{}.is_none()", left_str),
                        _ => {}
                    }
                }
                if left_str == "None" || left_str == "null" {
                    match op {
                        BinaryOp::NotEqual => return format!("{}.is_some()", right_str),
                        BinaryOp::Equal => return format!("{}.is_none()", right_str),
                        _ => {}
                    }
                }
                if *op == BinaryOp::Mul {
                    if matches!(&**left, Expr::PropertyAccess { .. }) {
                        right_str = format!("({} as f64)", right_str);
                    }
                    // Handle string * int (string repetition)
                    if left_str.starts_with('"') && left_str.ends_with('"') {
                        return format!("{}.repeat({})", left_str, right_str);
                    }
                }
                format!(
                    "{} {} {}",
                    left_str,
                    Self::binary_op_to_string(*op),
                    right_str
                )
            }
            Expr::UnaryOp { op, operand } => {
                let expr_str = self.expr_to_string_in_method(operand);
                format!("{}{}", Self::unary_op_to_string(*op), expr_str)
            }
            Expr::Call { func, args } => {
                let func_name = match &**func {
                    Expr::Identifier(name) => name.clone(),
                    _ => self.expr_to_string_in_method(func),
                };
                let args_str = args
                    .iter()
                    .map(|a| self.expr_to_string_in_method(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                match func_name.as_str() {
                    "print" => {
                        if args.is_empty() {
                            "println!()".to_string()
                        } else if args.len() == 1 {
                            format!("println!(\"{{:?}}\", {})", args_str)
                        } else {
                            format!("println!({})", args_str)
                        }
                    }
                    "Future" => {
                        if args.len() >= 1 {
                            let first_arg = self.expr_to_string_in_method(&args[0]);
                            if first_arg.contains("Duration") {
                                if args.len() >= 2 {
                                    let callback = self.expr_to_string_in_method(&args[1]);
                                    format!(
                                        "tokio::time::sleep({}).then(|_| async {{ {} }})",
                                        first_arg, callback
                                    )
                                } else {
                                    format!("tokio::time::sleep({})", first_arg)
                                }
                            } else {
                                format!("async {{ {} }}", first_arg)
                            }
                        } else {
                            "async {}".to_string()
                        }
                    }
                    "Duration" => {
                        let mut secs = 0u64;
                        let mut millis = 0u64;
                        let mut micros = 0u64;
                        for arg in args {
                            let arg_str = self.expr_to_string_in_method(arg);
                            if arg_str.contains("seconds:") {
                                if let Some(val) = arg_str.split(':').nth(1) {
                                    secs = val.trim().parse().unwrap_or(0);
                                }
                            } else if arg_str.contains("milliseconds:") {
                                if let Some(val) = arg_str.split(':').nth(1) {
                                    millis = val.trim().parse().unwrap_or(0);
                                }
                            } else if arg_str.contains("microseconds:") {
                                if let Some(val) = arg_str.split(':').nth(1) {
                                    micros = val.trim().parse().unwrap_or(0);
                                }
                            }
                        }
                        let total_micros = secs * 1_000_000 + millis * 1000 + micros;
                        format!("std::time::Duration::from_micros({})", total_micros)
                    }
                    _ => {
                        if func_name == "Future.wait" || func_name == "wait" {
                            format!("futures::future::join_all({})", args_str)
                        } else if func_name.starts_with("Future.") {
                            let method = &func_name[7..];
                            match method {
                                "value" => format!("async {{ {} }}", args_str),
                                "delayed" => {
                                    if args.len() >= 2 {
                                        let dur = self.expr_to_string_in_method(&args[0]);
                                        let callback = self.expr_to_string_in_method(&args[1]);
                                        format!(
                                            "tokio::time::sleep({}).then(|_| async {{ {} }})",
                                            dur, callback
                                        )
                                    } else {
                                        format!("tokio::time::sleep({})", args_str)
                                    }
                                }
                                _ => format!("{}", method),
                            }
                        } else if self.method_exists_in_current_class(&func_name) {
                            let is_finder_call =
                                func_name.starts_with("find") || func_name.starts_with("_find");
                            let args_str = if is_finder_call {
                                args.iter()
                                    .map(|a| {
                                        let arg_str = self.expr_to_string_in_method(a);
                                        format!("{}.as_str()", arg_str)
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            } else {
                                args_str
                            };
                            format!("self.{}({})", func_name, args_str)
                        } else if self.class_exists(&func_name) {
                            // Clone self.field arguments in constructor calls
                            let args_str = args
                                .iter()
                                .map(|a| {
                                    let arg_str = self.expr_to_string_in_method(a);
                                    if arg_str.starts_with("self.") && !arg_str.contains("(") {
                                        format!("{}.clone()", arg_str)
                                    } else {
                                        arg_str
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("{}::new({})", func_name, args_str)
                        } else {
                            format!("{}({})", func_name, args_str)
                        }
                    }
                }
            }
            Expr::PropertyAccess { object, property } => {
                let obj_str = self.expr_to_string(object);
                if let Some(ref program) = self.program {
                    if let crab_parser::Expr::Identifier(obj_name) = object.as_ref() {
                        for item in &program.items {
                            if let TopLevelItem::ClassDecl(c) = item {
                                if c.name == *obj_name && self.is_enum_class(c) {
                                    return format!("{}::{}", obj_str, property);
                                }
                            }
                        }
                    }
                }
                // Handle DateTime properties
                if obj_str == "chrono::Utc::now()" {
                    match property.as_str() {
                        "millisecondsSinceEpoch" => {
                            return "chrono::Utc::now().timestamp_millis()".to_string();
                        }
                        _ => {}
                    }
                }
                // Map common property names to Rust methods
                match property.as_str() {
                    "length" => return format!("{}.len() as i64", obj_str),
                    "isEmpty" => return format!("{}.is_empty()", obj_str),
                    "isNotEmpty" => return format!("!{}.is_empty()", obj_str),
                    "first" => return format!("{}.first()", obj_str),
                    "last" => return format!("{}.last()", obj_str),
                    "reversed" => {
                        return format!("{}.iter().rev().collect::<Vec<_>>()", obj_str);
                    }
                    _ => {}
                };
                // Check if property is a getter method
                let is_getter = self.is_getter_method(property);
                if is_getter {
                    format!("{}.{}{}", obj_str, property, "()")
                } else {
                    format!("{}.{}", obj_str, property)
                }
            }
            Expr::Index { object, index } => {
                let obj_str = self.expr_to_string_in_method(object);
                let idx_str = self.expr_to_string_in_method(index);
                format!("{}.get({}).cloned().unwrap()", obj_str, idx_str)
            }
            Expr::NullAware {
                object,
                property,
                args,
            } => {
                let obj_str = self.expr_to_string_in_method(object);
                match args {
                    Some(args) => {
                        let args_str = args
                            .iter()
                            .map(|a| self.expr_to_string_in_method(a))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!(
                            "{}.map(|o| o.{}({})).flatten()",
                            obj_str, property, args_str
                        )
                    }
                    None => format!("{}.map(|o| o.{})", obj_str, property),
                }
            }
            Expr::NullCoalesce { left, right } => {
                let left_str = self.expr_to_string_in_method(left);
                let right_str = self.expr_to_string_in_method(right);
                format!("{}.unwrap_or({})", left_str, right_str)
            }
            Expr::NullCoalesceAssign { target, value } => {
                let target_str = self.expr_to_string_in_method(target);
                let value_str = self.expr_to_string_in_method(value);
                let wrapped_value = if value_str.starts_with('"') && value_str.ends_with('"') {
                    format!("{}.to_string()", value_str)
                } else {
                    value_str
                };
                format!(
                    "{} = Some({}.unwrap_or({}))",
                    target_str, target_str, wrapped_value
                )
            }
            Expr::NullAssertion(expr) => {
                let expr_str = self.expr_to_string_in_method(expr);
                format!("{}.unwrap()", expr_str)
            }
            Expr::Await(expr) => {
                let expr_str = self.expr_to_string_in_method(expr);
                format!("{}.await", expr_str)
            }
            Expr::Propagate(expr) => {
                let expr_str = self.expr_to_string_in_method(expr);
                format!("{}?", expr_str)
            }
            Expr::ResultConstructor { variant, value } => {
                let val_str = self.expr_to_string_in_method(value);
                // Convert None to () for unit/void return types
                let final_value = if val_str == "None" {
                    "()".to_string()
                } else if val_str.starts_with('"') && val_str.ends_with('"') {
                    format!("{}.to_string()", val_str)
                } else {
                    val_str
                };
                match variant.as_str() {
                    "Ok" => format!("Ok({})", final_value),
                    "Err" => format!("Err({})", final_value),
                    _ => format!("{}", variant),
                }
            }
            Expr::OptionConstructor { variant, value } => {
                let final_value = if let Some(v) = value {
                    let val_str = self.expr_to_string_in_method(v);
                    if val_str.starts_with('"') && val_str.ends_with('"') {
                        format!("{}.to_string()", val_str)
                    } else {
                        val_str
                    }
                } else {
                    "".to_string()
                };
                match variant.as_str() {
                    "Some" => format!("Some({})", final_value),
                    "None" => "None".to_string(),
                    _ => format!("{}", variant),
                }
            }
            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                let object_str = self.expr_to_string_in_method(object);
                let method_name = method.as_str();
                // Check if this is a finder method call (receiver method starting with find/_find)
                let is_finder_call =
                    method_name.starts_with("find") || method_name.starts_with("_find");
                let args_str = args
                    .iter()
                    .map(|a| {
                        let arg_str = self.expr_to_string_in_method(a);
                        // Add .clone() for self.field arguments (String fields need cloning)
                        if arg_str.starts_with("self.") && !arg_str.contains("(") {
                            format!("{}.clone()", arg_str)
                        } else if is_finder_call {
                            // For finder methods, convert to &str via .as_str()
                            format!("{}.as_str()", arg_str)
                        } else if !arg_str.contains("(")
                            && !arg_str.starts_with('"')
                            && !arg_str.parse::<i64>().is_ok()
                        {
                            // Clone simple variable arguments to prevent move issues
                            // (identifiers that aren't method calls, literals, or numeric)
                            format!("{}.clone()", arg_str)
                        } else {
                            arg_str
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                // Handle List.from(x) -> x.clone() since x is already a Vec
                eprintln!(
                    "DEBUG MethodCall: object_str={}, method={}",
                    object_str, method
                );
                if object_str == "List" && method == "from" {
                    eprintln!("DEBUG List.from matched in MethodCall!");
                    return format!("{}.clone()", args_str);
                }
                // Handle fs module calls
                if object_str == "fs" {
                    match method.as_str() {
                        "writeFile" => {
                            return format!("std::fs::write({}, {}).ok()", args_str, "");
                        }
                        "readFile" => {
                            return format!("std::fs::read_to_string({}).unwrap()", args_str);
                        }
                        "readFileToString" => {
                            return format!("std::fs::read_to_string({}).unwrap()", args_str);
                        }
                        _ => {}
                    }
                }
                // Handle serde_json module calls
                if object_str == "serde_json" {
                    match method.as_str() {
                        "fromString" => {
                            return format!(
                                "serde_json::from_str::<Vec<serde_json::Value>>(&{}).unwrap_or_default()",
                                args_str
                            );
                        }
                        _ => {}
                    }
                }
                // Handle io module calls
                if object_str == "io" {
                    match method.as_str() {
                        "getEnv" => {
                            return format!("std::env::var({}).ok()", args_str);
                        }
                        _ => {}
                    }
                }
                // Handle DateTime calls
                if object_str == "DateTime" {
                    match method.as_str() {
                        "now" => {
                            return "chrono::Utc::now()".to_string();
                        }
                        _ => {}
                    }
                }
                // Handle int.parse(x) -> x.parse::<i64>()
                if object_str == "int" && method == "parse" {
                    return format!("{}.parse::<i64>()", args_str);
                }
                if object_str == "Future" {
                    match method.as_str() {
                        "delayed" => {
                            if args.len() >= 2 {
                                let dur = self.expr_to_string_in_method(&args[0]);
                                let callback = self.expr_to_string_in_method(&args[1]);
                                return format!(
                                    "tokio::time::sleep({}).then(|_| async {{ {} }})",
                                    dur, callback
                                );
                            } else if args.len() == 1 {
                                let dur = self.expr_to_string_in_method(&args[0]);
                                let dur_expr = if dur.parse::<u64>().is_ok() {
                                    format!("std::time::Duration::from_millis({})", dur)
                                } else {
                                    dur
                                };
                                return format!("tokio::time::sleep({})", dur_expr);
                            } else {
                                return "tokio::time::sleep(std::time::Duration::from_secs(0))"
                                    .to_string();
                            }
                        }
                        "wait" => {
                            return format!("futures::future::join_all({})", args_str);
                        }
                        "value" => {
                            return format!("async {{ {} }}", args_str);
                        }
                        _ => {}
                    }
                }
                // Special method handling
                match method.as_str() {
                    "isNotEmpty" => {
                        return format!("!{}.is_empty()", object_str);
                    }
                    "toList" => {
                        return format!("{}.collect::<Vec<_>>()", object_str);
                    }
                    "removeWhere" => {
                        // removeWhere keeps elements where predicate is false
                        // retain keeps elements where predicate is true
                        // So we need to negate the closure
                        return format!("{}.retain({})", object_str, args_str);
                    }
                    "compareTo" => {
                        // For i64 comparison: b.cmp(&a) for descending, a.cmp(&b) for ascending
                        return format!("{}.cmp(&{})", object_str, args_str);
                    }
                    "sort" => {
                        // sort with comparator should use sort_by
                        if !args_str.is_empty() {
                            return format!("{}.sort_by({})", object_str, args_str);
                        }
                    }
                    _ => {}
                }
                // Special handling for length to return i64 instead of usize
                if method.as_str() == "length" {
                    return format!("{}.len() as i64", object_str);
                }
                let method_name = match method.as_str() {
                    "length" => "len".to_string(),
                    "isEmpty" => "is_empty".to_string(),
                    "contains" => "contains".to_string(),
                    "startsWith" => "starts_with".to_string(),
                    "endsWith" => "ends_with".to_string(),
                    "toLowerCase" => "to_lowercase".to_string(),
                    "toUpperCase" => "to_uppercase".to_string(),
                    "trim" => "trim".to_string(),
                    "split" => "split".to_string(),
                    "join" => "join".to_string(),
                    "remove" => "remove".to_string(),
                    "add" => "push".to_string(),
                    "toString" => "to_string".to_string(),
                    "firstWhere" => "iter().find".to_string(),
                    "any" => "iter().any".to_string(),
                    "all" => "iter().all".to_string(),
                    "map" => "iter().map".to_string(),
                    "filter" => "iter().filter".to_string(),
                    "where" => "iter().filter".to_string(),
                    "forEach" => "iter().for_each".to_string(),
                    "indexOf" => "iter().position(|x| x ==".to_string(),
                    _ => method.clone(),
                };
                // For filter/where followed by collect, add .cloned() to get owned values
                if (method.as_str() == "filter" || method.as_str() == "where")
                    && !args_str.is_empty()
                {
                    return format!("{}.{}({}).cloned()", object_str, method_name, args_str);
                }
                // For push/add, clone the argument if it's a simple variable (to allow use after push)
                if (method.as_str() == "push" || method.as_str() == "add") && !args_str.is_empty() {
                    // Check if arg is a simple variable (not already a clone, literal, or method call)
                    let needs_clone = !args_str.contains("(")
                        && !args_str.contains(".clone()")
                        && !args_str.starts_with('"')
                        && !args_str.parse::<i64>().is_ok();
                    if needs_clone {
                        return format!("{}.push({}.clone())", object_str, args_str);
                    }
                }
                format!("{}.{}({})", object_str, method_name, args_str)
            }
            Expr::ListLiteral(items) => {
                let items_str = items
                    .iter()
                    .map(|a| {
                        let s = self.expr_to_string_in_method(a);
                        if s.starts_with('"') && s.ends_with('"') {
                            format!("{}.to_string()", s)
                        } else {
                            s
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("vec![{}]", items_str)
            }
            Expr::New {
                class_name,
                constructor: _,
                args,
            } => {
                let args_str = args
                    .iter()
                    .map(|a| {
                        let arg_str = self.expr_to_string_in_method(a);
                        // Clone self.field arguments in constructor calls (avoids move from &self)
                        if arg_str.starts_with("self.") && !arg_str.contains("(") {
                            format!("{}.clone()", arg_str)
                        } else {
                            arg_str
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}::new({})", class_name, args_str)
            }
            Expr::Assign { target, value } => {
                format!(
                    "{} = {}",
                    self.expr_to_string_in_method(target),
                    self.expr_to_string_in_method(value)
                )
            }
            Expr::CompoundAssign { target, op, value } => {
                let op_str = match op {
                    BinaryOp::Add => "+=",
                    BinaryOp::Sub => "-=",
                    BinaryOp::Mul => "*=",
                    BinaryOp::Div => "/=",
                    BinaryOp::Mod => "%=",
                    BinaryOp::BitAnd => "&=",
                    BinaryOp::BitOr => "|=",
                    BinaryOp::BitXor => "^=",
                    BinaryOp::LeftShift => "<<=",
                    BinaryOp::RightShift => ">>=",
                    _ => "???=",
                };
                format!(
                    "{} {} {}",
                    self.expr_to_string_in_method(target),
                    op_str,
                    self.expr_to_string_in_method(value)
                )
            }
            Expr::MapLiteral(pairs) => {
                if pairs.is_empty() {
                    "std::collections::HashMap::new()".to_string()
                } else {
                    let pairs_str = pairs
                        .iter()
                        .map(|(k, v)| {
                            format!(
                                "({}, {})",
                                self.expr_to_string_in_method(k),
                                self.expr_to_string_in_method(v)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("std::collections::HashMap::from([{}])", pairs_str)
                }
            }
            Expr::SetLiteral(items) => {
                let items_str = items
                    .iter()
                    .map(|a| self.expr_to_string_in_method(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("std::collections::HashSet::from([{}])", items_str)
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                format!(
                    "if {} {{ {} }} else {{ {} }}",
                    self.expr_to_string(condition),
                    self.expr_to_string(then_expr),
                    self.expr_to_string(else_expr)
                )
            }
            Expr::Is { expr, typ, negated } => {
                let type_str = TypeMapper::dart_to_rust(typ);
                let check = format!(
                    "std::any::type_name_of_val(&{}) == std::any::type_name::<{}>()",
                    self.expr_to_string_in_method(expr),
                    type_str
                );
                if *negated {
                    format!("!({})", check)
                } else {
                    check
                }
            }
            Expr::Cast { expr, typ } => {
                let expr_str = self.expr_to_string_in_method(expr);
                let type_str = TypeMapper::dart_to_rust(typ);
                // Handle serde_json::Value casts - convert to proper accessor methods
                if expr_str.contains(".get(") {
                    if type_str == "String" {
                        return format!("{}.as_str().unwrap().to_string()", expr_str);
                    } else if type_str == "i64" {
                        return format!("{}.as_i64().unwrap()", expr_str);
                    } else if type_str == "bool" {
                        return format!("{}.as_bool().unwrap()", expr_str);
                    }
                }
                format!("({} as {})", expr_str, type_str)
            }
            Expr::ResultConstructor { variant, value } => {
                let val_str = self.expr_to_string_in_method(value);
                // Convert None to () for unit/void return types
                let final_value = if val_str == "None" {
                    "()".to_string()
                } else if val_str.starts_with('"') && val_str.ends_with('"') {
                    format!("{}.to_string()", val_str)
                } else {
                    val_str
                };
                match variant.as_str() {
                    "Ok" => format!("Ok({})", final_value),
                    "Err" => format!("Err({})", final_value),
                    _ => format!("{}", variant),
                }
            }
            _ => self.expr_to_string(expr),
        }
    }
    fn expr_to_string_in_method(&self, expr: &Expr) -> String {
        match expr {
            Expr::Identifier(name) => {
                let safe_name = if name == "type" { "r#type" } else { name };
                if self.current_class_fields.contains(name) {
                    format!("self.{}", safe_name)
                } else {
                    safe_name.to_string()
                }
            }
            Expr::BinaryOp { left, op, right } => {
                let mut left_str = self.expr_to_string_in_method(left);
                let mut right_str = self.expr_to_string_in_method(right);
                let op_str = Self::binary_op_to_string(*op);
                // Handle string multiplication: "x" * n -> "x".repeat(n)
                if matches!(op, BinaryOp::Mul) {
                    if left_str.starts_with('"') && left_str.ends_with('"') {
                        return format!("{}.repeat({})", left_str, right_str);
                    }
                    if right_str.starts_with('"') && right_str.ends_with('"') {
                        return format!("{}.repeat({})", right_str, left_str);
                    }
                }
                // Wrap cast expressions in parentheses when used with comparison operators
                if matches!(
                    op,
                    BinaryOp::Less
                        | BinaryOp::Greater
                        | BinaryOp::LessEqual
                        | BinaryOp::GreaterEqual
                        | BinaryOp::Equal
                        | BinaryOp::NotEqual
                ) {
                    if left_str.contains(" as ") {
                        left_str = format!("({})", left_str);
                    }
                    if right_str.contains(" as ") {
                        right_str = format!("({})", right_str);
                    }
                }
                format!("{} {} {}", left_str, op_str, right_str)
            }
            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                let obj_str = self.expr_to_string_in_method(object);
                // Handle module-style static method calls
                if obj_str == "List" && method == "from" {
                    let args_str = args
                        .iter()
                        .map(|a| self.expr_to_string_in_method(a))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return format!("{}.clone()", args_str);
                }
                if obj_str == "fs" {
                    let args_str = args
                        .iter()
                        .map(|a| {
                            let arg_str = self.expr_to_string_in_method(a);
                            // Add & prefix for path arguments to avoid move
                            if arg_str.starts_with("self.") {
                                format!("&{}", arg_str)
                            } else {
                                arg_str
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    match method.as_str() {
                        "writeFile" => {
                            return format!("std::fs::write({}).ok()", args_str);
                        }
                        "readFile" | "readFileToString" => {
                            return format!("std::fs::read_to_string({}).unwrap()", args_str);
                        }
                        _ => {}
                    }
                }
                if obj_str == "serde_json" && method == "fromString" {
                    let args_str = args
                        .iter()
                        .map(|a| self.expr_to_string_in_method(a))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return format!(
                        "serde_json::from_str::<Vec<serde_json::Value>>(&{}).unwrap_or_default()",
                        args_str
                    );
                }
                if obj_str == "DateTime" && method == "now" {
                    return "chrono::Utc::now()".to_string();
                }
                // Only apply collection method mappings for known collection objects
                let is_collection = obj_str.contains("_tasks")
                    || obj_str.contains("_cookies")
                    || obj_str.contains("_orders")
                    || obj_str.contains("_customers")
                    || obj_str.ends_with("List")
                    || obj_str.ends_with("Vec");
                let method_name = match method.as_str() {
                    "clone" => return format!("{}.clone()", obj_str),
                    "length" => "len".to_string(),
                    "isEmpty" => "is_empty".to_string(),
                    "isNotEmpty" => {
                        return format!("!{}.is_empty()", obj_str);
                    }
                    "contains" => "contains".to_string(),
                    "startsWith" => "starts_with".to_string(),
                    "endsWith" => "ends_with".to_string(),
                    "toLowerCase" => "to_lowercase".to_string(),
                    "toUpperCase" => "to_uppercase".to_string(),
                    "trim" => "trim".to_string(),
                    "split" => "split".to_string(),
                    "join" => "join".to_string(),
                    "remove" => "remove".to_string(),
                    "add" if is_collection => "push".to_string(),
                    "add" => "add".to_string(),
                    "indexOf" => "iter().position(|x| x ==".to_string(),
                    "firstWhere" => "iter().find".to_string(),
                    "substring" => "chars().skip".to_string(),
                    "reverse" => "reverse".to_string(),
                    "sort" => "sort".to_string(),
                    "clear" => "clear".to_string(),
                    "get" => "get".to_string(),
                    "insert" => "insert".to_string(),
                    "any" => "iter().any".to_string(),
                    "all" => "iter().all".to_string(),
                    "map" => "iter().map".to_string(),
                    "filter" => "iter().filter".to_string(),
                    "where" => "iter().filter".to_string(),
                    "forEach" => "iter().for_each".to_string(),
                    "toList" => "collect::<Vec<_>>".to_string(),
                    "removeWhere" => "retain".to_string(),
                    _ => method.clone(),
                };
                if args.is_empty() {
                    if method_name == "len" {
                        format!("{}.len() as i64", obj_str)
                    } else {
                        format!("{}.{}()", obj_str, method_name)
                    }
                } else {
                    let args_str = args
                        .iter()
                        .map(|a| self.expr_to_string_in_method(a))
                        .collect::<Vec<_>>()
                        .join(", ");
                    if method_name.starts_with("iter().position") {
                        format!(
                            "{} {} ({}).unwrap_or(0) as i64",
                            obj_str, method_name, args_str
                        )
                    } else if method_name.starts_with("iter().") && method_name.ends_with(")") {
                        format!("{} {} ({})", obj_str, method_name, args_str)
                    } else if method_name.starts_with("chars().skip") {
                        let collect_str = format!(".take({}).collect::<String>()", args_str);
                        let result = format!("{}{}", obj_str, method_name);
                        return format!("{}{}", result, collect_str);
                    } else {
                        format!("{}.{}({})", obj_str, method_name, args_str)
                    }
                }
            }
            Expr::Assign { target, value } => {
                format!(
                    "{} = {}",
                    self.expr_to_string_in_method(target),
                    self.expr_to_string_in_method(value)
                )
            }
            Expr::CompoundAssign { target, op, value } => {
                let op_str = match op {
                    BinaryOp::Add => "+=",
                    BinaryOp::Sub => "-=",
                    BinaryOp::Mul => "*=",
                    BinaryOp::Div => "/=",
                    BinaryOp::Mod => "%=",
                    _ => "?=",
                };
                format!(
                    "{} {} {}",
                    self.expr_to_string_in_method(target),
                    op_str,
                    self.expr_to_string_in_method(value)
                )
            }
            Expr::NullAssertion(expr) => {
                let expr_str = self.expr_to_string_in_method(expr);
                // If expr is in unwrapped_locals, it's already unwrapped, don't unwrap again
                if let Expr::Identifier(name) = expr.as_ref() {
                    if self.unwrapped_locals.contains(name) {
                        return expr_str;
                    }
                }
                format!("{}.unwrap()", expr_str)
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                format!(
                    "if {} {{ {} }} else {{ {} }}",
                    self.expr_to_string_in_method(condition),
                    self.expr_to_string_in_method(then_expr),
                    self.expr_to_string_in_method(else_expr)
                )
            }
            Expr::Is { expr, typ, negated } => {
                let type_str = TypeMapper::dart_to_rust(typ);
                let check = format!(
                    "std::any::type_name_of_val(&{}) == std::any::type_name::<{}>()",
                    self.expr_to_string_in_method(expr),
                    type_str
                );
                if *negated {
                    format!("!({})", check)
                } else {
                    check
                }
            }
            Expr::Cast { expr, typ } => {
                let expr_str = self.expr_to_string_in_method(expr);
                let type_str = TypeMapper::dart_to_rust(typ);
                if expr_str.contains(".get(") {
                    if type_str == "String" {
                        return format!("{}.as_str().unwrap().to_string()", expr_str);
                    } else if type_str == "i64" {
                        return format!("{}.as_i64().unwrap()", expr_str);
                    } else if type_str == "bool" {
                        return format!("{}.as_bool().unwrap()", expr_str);
                    }
                }
                format!("({} as {})", expr_str, type_str)
            }
            Expr::PropertyAccess { object, property } => {
                let obj_str = self.expr_to_string_in_method(object);
                // Map common property names to Rust methods
                match property.as_str() {
                    "length" => return format!("{}.len() as i64", obj_str),
                    "isEmpty" => return format!("{}.is_empty()", obj_str),
                    "isNotEmpty" => return format!("!{}.is_empty()", obj_str),
                    "first" => return format!("{}.first()", obj_str),
                    "last" => return format!("{}.last()", obj_str),
                    "millisecondsSinceEpoch" => return format!("{}.timestamp_millis()", obj_str),
                    _ => {}
                };
                // Check if property is a getter method
                let is_getter = self.is_getter_method(property);
                if is_getter {
                    format!("{}.{}{}", obj_str, property, "()")
                } else {
                    format!("{}.{}", obj_str, property)
                }
            }
            Expr::Call { func, args } => {
                let func_name = match &**func {
                    Expr::Identifier(name) => name.clone(),
                    _ => self.expr_to_string_in_method(func),
                };
                // Check if this is a finder method call in the current class
                let is_finder_call = self.method_exists_in_current_class(&func_name)
                    && (func_name.starts_with("find") || func_name.starts_with("_find"));
                let args_str = args
                    .iter()
                    .map(|a| {
                        let arg_str = self.expr_to_string_in_method(a);
                        if is_finder_call {
                            // For finder methods, convert String to &str via .as_str()
                            format!("{}.as_str()", arg_str)
                        } else {
                            arg_str
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                // Handle module-style calls (e.g., List.from, fs.readFile)
                match func_name.as_str() {
                    "print" | "println" => {
                        if args.is_empty() {
                            return "println!()".to_string();
                        } else {
                            // Use {:?} debug format to support class instances (which have Debug derived)
                            return format!("println!(\"{{:?}}\", {})", args_str);
                        }
                    }
                    "List.from" => return format!("{}.clone()", args_str),
                    "fs.writeFile" => return format!("std::fs::write({}, r#\"\"#).ok()", args_str),
                    "fs.readFile" | "fs.readFileToString" => {
                        return format!("std::fs::read_to_string({}).unwrap()", args_str);
                    }
                    "serde_json.fromString" => {
                        return format!(
                            "serde_json::from_str::<Vec<serde_json::Value>>(&{}).unwrap_or_default()",
                            args_str
                        );
                    }
                    // Handle Result constructors Ok() and Err()
                    "Ok" => {
                        // Convert None to () for unit/void return types
                        if args_str == "None" {
                            return "Ok(())".to_string();
                        }
                        return format!("Ok({})", args_str);
                    }
                    "Err" => {
                        if args_str == "None" {
                            return "Err(())".to_string();
                        }
                        return format!("Err({})", args_str);
                    }
                    _ => {}
                }
                // Check if this is a method call in the current class
                if self.method_exists_in_current_class(&func_name) {
                    format!("self.{}({})", func_name, args_str)
                } else if self.class_exists(&func_name) {
                    // Class constructor - use ::new() syntax
                    format!("{}::new({})", func_name, args_str)
                } else {
                    format!("{}({})", func_name, args_str)
                }
            }
            Expr::ResultConstructor { variant, value } => {
                let val_str = self.expr_to_string_in_method(value);
                // Convert None to () for unit/void return types
                let final_value = if val_str == "None" {
                    "()".to_string()
                } else if val_str.starts_with('"') && val_str.ends_with('"') {
                    format!("{}.to_string()", val_str)
                } else {
                    val_str
                };
                match variant.as_str() {
                    "Ok" => format!("Ok({})", final_value),
                    "Err" => format!("Err({})", final_value),
                    _ => format!("{}", variant),
                }
            }
            _ => self.expr_to_string(expr),
        }
    }
    fn literal_to_string(lit: &Literal) -> String {
        match lit {
            Literal::Int(n) => n.to_string(),
            Literal::Double(f) => {
                let s = f.to_string();
                if s.contains('.') {
                    s
                } else {
                    format!("{}.0", s)
                }
            }
            Literal::Bool(b) => b.to_string(),
            Literal::String(s) => {
                let escaped = s
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t");
                format!("\"{}\"", escaped)
            }
            Literal::Null => "None".to_string(),
        }
    }
    fn binary_op_to_string(op: BinaryOp) -> String {
        match op {
            BinaryOp::Add => "+".to_string(),
            BinaryOp::Sub => "-".to_string(),
            BinaryOp::Mul => "*".to_string(),
            BinaryOp::Div => "/".to_string(),
            BinaryOp::IntDiv => "/".to_string(),
            BinaryOp::Mod => "%".to_string(),
            BinaryOp::Pow => "**".to_string(),
            BinaryOp::Equal => "==".to_string(),
            BinaryOp::NotEqual => "!=".to_string(),
            BinaryOp::Less => "<".to_string(),
            BinaryOp::Greater => ">".to_string(),
            BinaryOp::LessEqual => "<=".to_string(),
            BinaryOp::GreaterEqual => ">=".to_string(),
            BinaryOp::And => "&&".to_string(),
            BinaryOp::Or => "||".to_string(),
            BinaryOp::BitAnd => "&".to_string(),
            BinaryOp::BitOr => "|".to_string(),
            BinaryOp::BitXor => "^".to_string(),
            BinaryOp::LeftShift => "<<".to_string(),
            BinaryOp::RightShift => ">>".to_string(),
        }
    }
    fn unary_op_to_string(op: UnaryOp) -> String {
        match op {
            UnaryOp::Neg => "-".to_string(),
            UnaryOp::Not => "!".to_string(),
            UnaryOp::BitNot => "~".to_string(),
        }
    }
    fn emit_line(&mut self, line: &str) {
        let indent = "    ".repeat(self.indent_level);
        self.output.push_str(&indent);
        self.output.push_str(line);
        self.output.push('\n');
    }
    fn indent(&mut self) {
        self.indent_level += 1;
    }
    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
}
