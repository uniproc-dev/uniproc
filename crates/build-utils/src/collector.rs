use proc_macro2::TokenStream;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Schema {
    pub ports: Vec<PortDef>,
    pub bindings: Vec<BindingDef>,
    pub dtos: Vec<DtoDef>,
    pub capabilities: Vec<CapabilityDef>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PortDef {
    pub name: String,
    pub global: String,
    pub source_file: String,
    pub methods: Vec<MethodDef>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BindingDef {
    pub name: String,
    pub global: String,
    pub source_file: String,
    pub methods: Vec<BindingMethodDef>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MethodDef {
    pub name: String,
    pub is_manual: bool,
    pub global_override: Option<String>,
    pub slint_name: Option<String>,
    pub args: Vec<ArgDef>,
    pub output_ty: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BindingMethodDef {
    pub name: String,
    pub is_manual: bool,
    pub tracing_skip: bool,
    pub tracing_target: Option<String>,
    pub slint_name: Option<String>,
    pub handler_args: Vec<ArgDef>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ArgDef {
    pub name: String,
    pub ty: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DtoDef {
    pub name: String,
    pub is_enum: bool,
    pub source_file: String,
    pub fields: Vec<DtoField>,
    pub variants: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DtoField {
    pub name: String,
    pub ty: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CapabilityDef {
    pub name: String,
    pub key: String,
    pub source_file: String,
}
pub fn find_workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let mut root = path.clone();

    while path.pop() {
        if path.join("target").exists() || path.join("Cargo.lock").exists() {
            root = path;
            break;
        }
    }
    root
}

pub fn get_schema_path() -> PathBuf {
    find_workspace_root().join("target/contracts-schema.json")
}

pub fn with_recompile_trigger(output: TokenStream) -> TokenStream {
    let path = get_schema_path().to_string_lossy().to_string();
    quote::quote! {
        const _: &[u8] = include_bytes!(#path);
        #output
    }
}

pub fn load_schema() -> Schema {
    if let Ok(val) = env::var("CONTRACTS_SCHEMA_PATH") {
        let p = PathBuf::from(val);
        if p.exists() {
            return serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap();
        }
    }

    let schema_path = get_schema_path();

    if schema_path.exists() {
        let json = fs::read_to_string(schema_path).expect("Failed to read schema");
        return serde_json::from_str(&json).expect("Failed to parse schema");
    }

    panic!("Schema not found in target/. Did you build the contracts crate first?");
}

pub struct SchemaCollector {
    src_dir: PathBuf,
    output_filename: String,
}

impl Default for SchemaCollector {
    fn default() -> Self {
        Self {
            src_dir: PathBuf::from("src/features"),
            output_filename: "contracts-schema.json".to_string(),
        }
    }
}

impl SchemaCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn walk_src(mut self, path: impl Into<PathBuf>) -> Self {
        self.src_dir = path.into();
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.output_filename = name.into();
        self
    }

    pub fn run(self) -> Schema {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("No CARGO_MANIFEST_DIR");
        let absolute_src = Path::new(&manifest_dir).join(&self.src_dir);
        let workspace_root = find_workspace_root();

        println!("cargo:rerun-if-changed={}", absolute_src.display());

        let mut schema = Schema::default();
        if absolute_src.exists() {
            for entry in WalkDir::new(&absolute_src)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().extension().unwrap_or_default() == "rs" {
                    parse_file(entry.path(), &workspace_root, &mut schema);
                }
            }
        }

        let json = serde_json::to_string_pretty(&schema).unwrap();

        if let Ok(out_dir) = env::var("OUT_DIR") {
            let _ = fs::write(Path::new(&out_dir).join(&self.output_filename), &json);
        }

        let schema_path = workspace_root.join("target").join(&self.output_filename);
        if let Some(parent) = schema_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&schema_path, &json).expect("Failed to write shared schema");

        schema
    }
}

fn parse_file(file_path: &Path, workspace_dir: &Path, schema: &mut Schema) {
    let content = fs::read_to_string(file_path).unwrap();
    let ast = match syn::parse_file(&content) {
        Ok(ast) => ast,
        Err(_) => return,
    };

    let relative_path = file_path
        .strip_prefix(workspace_dir)
        .unwrap_or(file_path)
        .to_string_lossy()
        .replace('\\', "/");

    for item in ast.items {
        match item {
            syn::Item::Trait(item_trait) => {
                let trait_name = item_trait.ident.to_string();

                if let Some(global) =
                    extract_attribute_arg(&item_trait.attrs, "slint_port", "global")
                {
                    schema.ports.push(PortDef {
                        name: trait_name.clone(),
                        global,
                        source_file: relative_path.clone(),
                        methods: parse_port_methods(&item_trait),
                    });
                } else if let Some(global) =
                    extract_attribute_arg(&item_trait.attrs, "slint_bindings", "global")
                {
                    schema.bindings.push(BindingDef {
                        name: trait_name.clone(),
                        global,
                        source_file: relative_path.clone(),
                        methods: parse_binding_methods(&item_trait),
                    });
                }
            }

            syn::Item::Struct(item_struct) => {
                if has_attribute(&item_struct.attrs, "slint_dto") {
                    let fields = item_struct
                        .fields
                        .iter()
                        .map(|f| DtoField {
                            name: f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default(),
                            ty: quote::quote!(#f.ty).to_string().replace(' ', ""),
                        })
                        .collect();

                    schema.dtos.push(DtoDef {
                        name: item_struct.ident.to_string(),
                        is_enum: false,
                        source_file: relative_path.clone(),
                        fields,
                        variants: vec![],
                    });
                }

                if let Some(key) = extract_name_value_attribute(&item_struct.attrs, "capability") {
                    schema.capabilities.push(CapabilityDef {
                        name: item_struct.ident.to_string(),
                        key,
                        source_file: relative_path.clone(),
                    });
                }
            }

            syn::Item::Enum(item_enum) => {
                if has_attribute(&item_enum.attrs, "slint_dto") {
                    let variants = item_enum
                        .variants
                        .iter()
                        .map(|v| v.ident.to_string())
                        .collect();

                    schema.dtos.push(DtoDef {
                        name: item_enum.ident.to_string(),
                        is_enum: true,
                        source_file: relative_path.clone(),
                        fields: vec![],
                        variants,
                    });
                }
            }

            _ => {}
        }
    }
}

fn parse_port_methods(item_trait: &syn::ItemTrait) -> Vec<MethodDef> {
    let mut methods = Vec::new();
    for item in &item_trait.items {
        if let syn::TraitItem::Fn(method) = item {
            let is_manual = has_attribute(&method.attrs, "manual");
            let global_override = extract_attribute_arg(&method.attrs, "slint", "global");
            let slint_name = extract_attribute_arg(&method.attrs, "slint", "name");

            methods.push(MethodDef {
                name: method.sig.ident.to_string(),
                is_manual,
                global_override,
                slint_name,
                args: extract_args(&method.sig),
                output_ty: extract_output_ty(&method.sig),
            });
        }
    }
    methods
}

fn parse_binding_methods(item_trait: &syn::ItemTrait) -> Vec<BindingMethodDef> {
    let mut methods = Vec::new();
    for item in &item_trait.items {
        if let syn::TraitItem::Fn(method) = item {
            let is_manual = has_attribute(&method.attrs, "manual");
            let tracing_skip = has_attribute(&method.attrs, "tracing")
                && has_attribute_flag(&method.attrs, "tracing", "skip");
            let tracing_target = extract_attribute_arg(&method.attrs, "tracing", "target");
            let slint_name = extract_attribute_arg(&method.attrs, "slint", "name");

            methods.push(BindingMethodDef {
                name: method.sig.ident.to_string(),
                is_manual,
                tracing_skip,
                tracing_target,
                slint_name,
                handler_args: extract_handler_args(method),
            });
        }
    }
    methods
}

fn extract_handler_args(method: &syn::TraitItemFn) -> Vec<ArgDef> {
    let Some(where_clause) = &method.sig.generics.where_clause else {
        return Vec::new();
    };

    for predicate in &where_clause.predicates {
        let syn::WherePredicate::Type(pred) = predicate else {
            continue;
        };
        let syn::Type::Path(type_path) = &pred.bounded_ty else {
            continue;
        };
        if type_path
            .path
            .segments
            .last()
            .map(|s| s.ident != "F")
            .unwrap_or(true)
        {
            continue;
        }
        for bound in &pred.bounds {
            let syn::TypeParamBound::Trait(trait_bound) = bound else {
                continue;
            };
            let Some(segment) = trait_bound.path.segments.last() else {
                continue;
            };
            if segment.ident != "Fn" && segment.ident != "FnMut" && segment.ident != "FnOnce" {
                continue;
            }
            let syn::PathArguments::Parenthesized(args) = &segment.arguments else {
                continue;
            };
            return args
                .inputs
                .iter()
                .enumerate()
                .map(|(idx, ty)| ArgDef {
                    name: format!("arg{}", idx + 1),
                    ty: quote::quote!(#ty).to_string().replace(' ', ""),
                })
                .collect();
        }
    }

    Vec::new()
}

fn extract_args(sig: &syn::Signature) -> Vec<ArgDef> {
    let mut args = Vec::new();
    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            let type_ast = &pat_type.ty;
            let type_str = quote::quote!(#type_ast).to_string();

            if type_str.contains("Fn (") || type_str.contains("FnOnce") {
                continue;
            }

            let name = if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                pat_ident.ident.to_string()
            } else {
                "unknown".to_string()
            };

            args.push(ArgDef {
                name,
                ty: type_str.replace(' ', ""),
            });
        }
    }
    args
}

fn extract_output_ty(sig: &syn::Signature) -> Option<String> {
    match &sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(quote::quote!(#ty).to_string().replace(' ', "")),
    }
}

fn has_attribute(attrs: &[syn::Attribute], attr_name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(attr_name))
}

fn has_attribute_flag(attrs: &[syn::Attribute], attr_name: &str, flag_name: &str) -> bool {
    for attr in attrs {
        if attr.path().is_ident(attr_name) {
            let mut found = false;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(flag_name) {
                    found = true;
                }
                Ok(())
            });
            if found {
                return true;
            }
        }
    }
    false
}

fn extract_attribute_arg(attrs: &[syn::Attribute], attr_name: &str, key: &str) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident(attr_name) {
            let mut value = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(key) {
                    if let Ok(v) = meta.value() {
                        if let Ok(s) = v.parse::<syn::LitStr>() {
                            value = Some(s.value());
                        }
                    }
                }
                Ok(())
            });
            if value.is_some() {
                return value;
            }
        }
    }
    None
}

fn extract_name_value_attribute(attrs: &[syn::Attribute], attr_name: &str) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident(attr_name) {
            if let syn::Meta::List(list) = &attr.meta {
                if let Ok(lit_str) = list.parse_args::<syn::LitStr>() {
                    return Some(lit_str.value());
                }
            }
        }
    }
    None
}
