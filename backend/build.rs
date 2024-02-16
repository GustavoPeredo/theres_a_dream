use std::fs::{self, File};
use std::io::{Write, Error};
use std::path::{Path, PathBuf};
use syn::{ItemFn, parse_file};
use quote::quote;


fn main() -> Result<(), Error> {
    let api_path = Path::new("src/api");
    let mut routes = Vec::new();

    if api_path.exists() && api_path.is_dir() {
        read_directory(&api_path, &mut routes, Vec::new())?;
    }

    // Generate the main.rs content with dynamic routing based on the API structure
    let main_rs_content = generate_main_rs_content(&routes);
    
    // Write the generated content to main.rs
    let mut file = File::create("src/main.rs")?;
    file.write_all(main_rs_content.as_bytes())?;

    Ok(())
}

fn read_directory(path: &Path, routes: &mut Vec<String>, mod_path: Vec<String>) -> Result<(), Error> {
    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_str().unwrap().to_owned();
            let mut new_mod_path = mod_path.clone();
            new_mod_path.push(dir_name);
            read_directory(&path, routes, new_mod_path)?;
        } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            process_file(&path, routes, &mod_path).expect("Error processing file");
        }
    }
    Ok(())
}

fn process_file(path: &PathBuf, routes: &mut Vec<String>, mod_path: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let file = parse_file(&content)?;

    
    for item in &file.items {
        if let syn::Item::Fn(item_fn) = item {
            let function_name = &item_fn.sig.ident;
            let function_return_type = &item_fn.sig.output;
            let mut arg_type = String::new();

            // Stringify the return type
            let return_type_str = quote! { #function_return_type }.to_string();

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
            
            // Construct the route path
            let mut route_path = mod_path.clone();
            route_path.push(function_name.to_string());
            let route_string = format!("\"{}\"", route_path.join("\" / \""));
            
            // Construct the module path
            let module_path = format!("api{}::{}", mod_path.join("::"), path.file_stem().unwrap().to_str().unwrap());
            
            // Push the route string to the routes vector
            routes.push(format!(r#"            .or({}::{}(warp::path!({})))"#, module_path, function_name, route_string));
            
        }
    }
    
    Ok(())
}

fn generate_main_rs_content(routes: &Vec<String>) -> String {
    // Read from templates/main.rs, substitute the /* ROUTES */ placeholder with the generated routes
    let main_rs_template = include_str!("templates/main.rs");
    let routes_str = routes.join("\n");
    main_rs_template.replace("/* ROUTES */", &routes_str)
}
