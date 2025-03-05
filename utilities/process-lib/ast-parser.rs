   //! ```cargo
   //! [dependencies]
   //! syn = { version = "2.0", features = ["full", "parsing", "visit"] }
   //! serde_json = "1.0"
   //! walkdir = "2.3"
   //! quote = "1.0"
   //! ```
use std::fs::{self, File as FsFile};
use std::io::Write;
use std::path::Path;
use syn::{File, Item, parse_file};
use syn::visit::{self, Visit};
use walkdir::WalkDir;
use quote::ToTokens;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root_dir = if args.len() < 2 {
        "."  // Default to current directory
    } else {
        &args[1]
    };
    
    let output_path = if args.len() < 3 {
        "StandardProcessLib.md"
    } else {
        &args[2]
    };
    
    let mut output = String::new();
    output.push_str("# Process-Lib API Documentation\n\n");
    
    // Walk through all files in the directory
    for entry in WalkDir::new(root_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        // Skip if not a .rs file
        if !path.is_file() || path.extension().map_or(false, |ext| ext != "rs") {
            continue;
        }
        
        // Skip test files
        if path.to_string_lossy().contains("test") {
            continue;
        }

        println!("Processing: {}", path.display());
        
        // Read and parse the file
        match fs::read_to_string(path) {
            Ok(content) => {
                match parse_file(&content) {
                    Ok(ast) => {
                        let file_name = path.file_name().unwrap().to_string_lossy();
                        output.push_str(&format!("## {}\n\n", file_name));
                        process_ast(&ast, &mut output);
                    },
                    Err(err) => eprintln!("Failed to parse {}: {}", path.display(), err),
                }
            },
            Err(err) => eprintln!("Failed to read {}: {}", path.display(), err),
        }
    }
    
    // Write output to file
    let mut file = FsFile::create(output_path).expect("Failed to create output file");
    file.write_all(output.as_bytes()).expect("Failed to write to output file");
    println!("Documentation written to {}", output_path);
}

fn process_ast(ast: &File, output: &mut String) {
    for item in &ast.items {
        match item {
            Item::Fn(func) => {
                // Process standalone function
                let name = &func.sig.ident;
                let visibility = &func.vis;
                
                // Skip private functions
                if !visibility.to_token_stream().to_string().contains("pub") {
                    continue;
                }
                
                // Function signature
                let sig = func.sig.to_token_stream().to_string();
                
                // Extract documentation comments
                let docs = extract_docs(&func.attrs);
                
                // Format the function in markdown
                output.push_str(&format!("#### `{}` function\n\n", name));
                
                output.push_str("```rust\n");
                output.push_str(&sig);
                output.push_str("\n```\n\n");
                
                if !docs.is_empty() {
                    output.push_str(&docs);
                    output.push_str("\n\n");
                }
            },
            Item::Impl(impl_block) => {
                let type_name = impl_block.self_ty.to_token_stream().to_string();
                output.push_str(&format!("### Impl for `{}`\n\n", type_name));
                
                for item in &impl_block.items {
                    if let syn::ImplItem::Fn(method) = item {
                        let name = &method.sig.ident;
                        let visibility = &method.vis;
                        
                        // Function signature
                        let sig = method.sig.to_token_stream().to_string();
                        
                        // Extract documentation comments
                        let docs = extract_docs(&method.attrs);
                        
                        // Format the method in markdown
                        output.push_str(&format!("#### `{}::{}` method\n\n", type_name, name));
                        
                        output.push_str("```rust\n");
                        output.push_str(&sig);
                        output.push_str("\n```\n\n");
                        
                        if !docs.is_empty() {
                            output.push_str(&docs);
                            output.push_str("\n\n");
                        }
                    }
                }
            },
            Item::Struct(struct_item) => {
                let name = &struct_item.ident;
                output.push_str(&format!("### Struct `{}`\n\n", name));
                
                // Extract struct documentation
                let docs = extract_docs(&struct_item.attrs);
                if !docs.is_empty() {
                    output.push_str(&format!("{}\n\n", docs));
                }
                
                // List fields if public
                output.push_str("**Fields:**\n\n");
                for field in &struct_item.fields {
                    if let Some(ident) = &field.ident {
                        let vis = &field.vis;
                        let ty = &field.ty;
                        if vis.to_token_stream().to_string().contains("pub") {
                            output.push_str(&format!("- `{}`: `{}`\n", ident, ty.to_token_stream()));
                        }
                    }
                }
                output.push_str("\n");
            },
            Item::Enum(enum_item) => {
                let name = &enum_item.ident;
                output.push_str(&format!("### Enum `{}`\n\n", name));
                
                // Extract enum documentation
                let docs = extract_docs(&enum_item.attrs);
                if !docs.is_empty() {
                    output.push_str(&format!("{}\n\n", docs));
                }
                
                // List variants
                output.push_str("**Variants:**\n\n");
                for variant in &enum_item.variants {
                    output.push_str(&format!("- `{}`\n", variant.ident));
                }
                output.push_str("\n");
            },
            _ => {}
        }
    }
}

fn extract_docs(attrs: &[syn::Attribute]) -> String {
    let mut docs = String::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Ok(doc) = attr.parse_args::<syn::LitStr>() {
                let doc_line = doc.value();
                docs.push_str(doc_line.trim());
                docs.push('\n');
            }
        }
    }
    docs.trim().to_string()
}