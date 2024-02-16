use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use syn::parse_file;
use quote::quote;

fn rust_to_typescript(rust_type: &str) -> String {
    match rust_type {
        // Primitive types
        "i32" | "u32" | "i64" | "u64" | "f32" | "f64" => "number".to_string(),
        "bool" => "boolean".to_string(),
        "char" => "string".to_string(),
        // Arrays
        "Vec<" => {
            let inner_type = extract_inner_type(rust_type);
            format!("Array<{}>", rust_to_typescript(inner_type))
        },
        // Tuples (up to 3 elements)
        "(" => {
            let element_types = extract_tuple_types(rust_type);
            format!("({})", element_types.iter().map(|t| rust_to_typescript(t)).collect::<Vec<_>>().join(", "))
        },
        // Ignore unit type
        "()" => "".to_string(),
        // HashMaps
        "HashMap<" => {
            let (key_type, value_type) = extract_map_types(rust_type);
            format!("{{ [key: {}]: {} }}", rust_to_typescript(key_type), rust_to_typescript(value_type))
        },
        // Unknown cases
        _ => {
            // Provide more context about the unknown type
            format!("{}", rust_type)
        }
    }
}

fn extract_inner_type(rust_type: &str) -> &str {
    rust_type.split('<').nth(1).unwrap().trim_end_matches('>')
}

fn extract_tuple_types(rust_type: &str) -> Vec<&str> {
    rust_type.split(',')
        .map(|part| part.trim().split(':').next().unwrap())
        .collect()
}

fn extract_map_types(rust_type: &str) -> (&str, &str) {
    let mut parts = rust_type.splitn(2, '<').skip(1);
    let key_type = parts.next().unwrap().trim_end_matches(',');
    let value_type = parts.next().unwrap().trim_end_matches('>');
    (key_type, value_type)
}

fn generate_typescript_bindings(file_path: &str) -> std::io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let file = parse_file(&contents).unwrap();

    let mut output = String::new();
    let template = std::fs::read_to_string("templates/function.ts")?;

    let mut included_first_line = false;

    for item in &file.items {
        if let syn::Item::Fn(item_fn) = item {
            let function_name = &item_fn.sig.ident;
            let function_return_type = &item_fn.sig.output;
            let mut arg_type = String::new();
            let mut return_type: String;

            // Stringify the return type
            let func_name = function_name.to_string();

            if let syn::ReturnType::Type(_, ty) = function_return_type {
                return_type = quote! { #ty }.to_string();
            } else {
                return_type = "void".to_string();
            }

            return_type = rust_to_typescript(&return_type);

            // Extract the argument types from the first function argument
            if let Some(first_arg) = item_fn.sig.inputs.iter().nth(1) {
                if let syn::FnArg::Typed(pat_type) = first_arg {
                    if let syn::Type::Path(path) = &*pat_type.ty {
                        for segment in &path.path.segments {
                            arg_type = segment.ident.to_string();
                        }
                    }
                }
            }

            let file_name = Path::new(file_path)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let route = if func_name == file_name {
                file_name
            } else {
                format!("{}/{}", file_name, func_name)
            };

            let function_name = route.replace("/", "_");

            let replaced_template = template.replace("$function_name$", &function_name)
                .replace("$arg_type$", &format!("{}", arg_type))
                .replace("$return_type$", &return_type)
                .replace("$route$", &format!("\"{}\"", route));

            if !included_first_line {
                output.push_str("\n");
                output.push_str(&replaced_template);
                included_first_line = true;
            } else {
                // Skip the first line of the template
                output.push_str("\n");
                let mut lines = replaced_template.lines();
                lines.next();
                for line in lines {
                    output.push_str(line);
                    output.push_str("\n");
                }
            }
        }
    }

    Ok(output)
}

fn generate_typescript_bindings_and_append(api_dir: &str, bindings_dir: &str) -> std::io::Result<()> {
    for entry in std::fs::read_dir(api_dir)? {
        let entry = entry?;
        let api_file_path = entry.path();

        if api_file_path.is_file() {
            let bindings_file_path = flatten_file_path(api_file_path.clone(), bindings_dir)?;

            let bindings = generate_typescript_bindings(api_file_path.to_str().unwrap())?;

            let mut bindings_file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(bindings_file_path.clone())?;

            println!("Writing to file: {:?}", bindings_file_path.clone());
            println!("{}", bindings.clone());
            bindings_file.write_all(bindings.as_bytes())?;
        }
    }

    Ok(())
}

fn flatten_file_path(original_path: PathBuf, target_dir: &str) -> std::io::Result<PathBuf> {
    let mut flattened_path = PathBuf::from(target_dir);

    for component in original_path.components().skip(2) {
        flattened_path.push(component.as_os_str().to_str().unwrap().replace("/", "."));
    }

    Ok(flattened_path.with_extension("ts"))
}

#[test]
fn test_add_functions_to_types() {
    let api_dir = "src/api";
    let bindings_dir = "bindings/";

    generate_typescript_bindings_and_append(api_dir, bindings_dir).unwrap();
}