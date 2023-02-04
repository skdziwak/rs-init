//! This crate generates rust code that calls functions with the `#[init]` attribute in the correct order.
//! `#[init]` macro is provided by the `rs-init-macro` crate.
//! Example usage:
//! ```ignore
//! use rs_init_macro::init;
//!
//! #[init(stage = 0)]
//! fn init0() {
//!     println!("init0");
//! }
//!
//! #[init(stage = 1)]
//! fn init1() {
//!     println!("init1");
//! }
//!
//! fn main() {
//!     generated_init();
//! }
//! ```
//!
//! Build.rs:
//!
//! ```rust
//! fn main() {
//!     rs_init::default_setup();
//! }
//! ```
//!
//! `#[init]` macro can be used on function in any module, but the `rs-init` crate must be able to find the module.
//! This can be done by adding `pub(crate)` to the module declaration.
//!
//! You probably would not use this crate by itself, but rather to create some sort of framework and other macros that use it.
use std::str::FromStr;
use syn::Item;
use std::io::Write;

struct InitFunction {
    call: String,
    stage: u32,
}

struct InitContext {
    functions: Vec<InitFunction>,
}

/// This function is used by the build script to generate the `generated_init` function.
/// It scans the `src` directory for files with the `#[init]` attribute and generates a function that calls them in the correct order.
/// The `#[init]` attribute must have a `stage` parameter, which is used to determine the order in which the functions are called.
/// `cargo:rerun-if-changed=src` is added to the build script output, so that the build script is rerun when any file in the `src` directory changes.
pub fn default_setup() {
    println!("cargo:rerun-if-changed=src");
    generate_init_function("src");
}

/// This function is used by the build script to generate the `generated_init` function.
/// It allows you to specify the directory to scan for files with the `#[init]` attribute.
/// The `#[init]` attribute must have a `stage` parameter, which is used to determine the order in which the functions are called.
/// It does not add `cargo:rerun-if-changed=src` to the build script output, so you must add it yourself if you want the build script to be rerun when any file in the `src` directory changes.
pub fn generate_init_function(source_dir: &str) {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("init.rs");
    let mut context = InitContext {
        functions: Vec::new(),
    };
    scan_dir(&mut context, source_dir, "crate", 0);

    context.functions.sort_by(|a, b| a.stage.cmp(&b.stage));

    let writer = std::fs::File::create(&dest_path).unwrap();
    let mut writer = std::io::BufWriter::new(writer);
    writeln!(writer, "pub fn generated_init() {{").unwrap();
    for function in context.functions.iter() {
        writeln!(writer, "\t{};", function.call).unwrap();
    }
    writeln!(writer, "}}").unwrap();
}

fn scan_dir(context: &mut InitContext, dir: &str, prefix: &str, level: u32) {
    let paths = std::fs::read_dir(dir).unwrap();
    for path in paths {
        let path = path.expect("Failed to read path").path();
        let path_str = path.to_str().expect("Failed to read path");
        if path.is_dir() {
            let dir_name = path.file_name()
                .expect("Failed to get directory name")
                .to_str().expect("Failed to get directory name");
            let prefix = format!("{}::{}", prefix, path.file_name().unwrap().to_str().unwrap());
            scan_dir(context, path_str, &prefix, level + 1);
        } else {
            if path_str.ends_with(".rs") {
                if level == 0 {
                    scan_file(context, path_str, prefix);
                } else {
                    let file_name = path.file_name().expect("Failed to get file name").to_str().expect("Failed to get file name");
                    let mod_name = &file_name[..file_name.len() - 3];
                    let prefix = format!("{}::{}", prefix, mod_name);
                    scan_file(context, path_str, &prefix);
                }
            }
        }
    }
}

fn attr_to_map(attr: &syn::Attribute) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let tokens = attr.tokens.to_string();
    let tokens = tokens[1..tokens.len() - 1].trim();
    let tokens = tokens.split(",");
    for token in tokens {
        let token = token.trim();
        let token = token.split("=");
        let mut token = token.map(|t| t.trim());
        let key = token.next().expect("Failed to parse attribute: no key");
        let value = token.next().expect("Failed to parse attribute: no value");
        map.insert(key.to_string(), value.to_string());
    }
    map
}

fn scan_file(context: &mut InitContext, path: &str, prefix: &str) {
    let file_content = std::fs::read_to_string(path).unwrap();
    let stream = proc_macro2::TokenStream::from_str(&file_content).unwrap();
    let ast: syn::File = syn::parse2::<syn::File>(stream).unwrap();

    for item in ast.items {
        if let Item::Fn(f) = item {
            if let Some(attr) = f.attrs.iter().find(|a| a.path.is_ident("init")) {
                let name = f.sig.ident.to_string();
                let call_code = format!("{prefix}::{name}()");
                let map = attr_to_map(attr);
                let stage = map.get("stage").expect("No stage parameter defined. It should be a number greater than 0.")
                    .parse::<u32>().expect("Stage parameter should be a number greater than 0.");

                context.functions.push(InitFunction {
                    call: call_code,
                    stage,
                });
            }
        }
    }
}
