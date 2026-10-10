use proc_macro::{TokenStream, TokenTree};

fn target_type(input: TokenStream) -> Result<String, TokenStream> {
    let mut saw_item = false;
    for token in input {
        if let TokenTree::Ident(ident) = token {
            if saw_item {
                return Ok(ident.to_string());
            }
            saw_item = matches!(ident.to_string().as_str(), "struct" | "enum" | "union");
        }
    }
    Err("compile_error!(\"derive requires a named type\");"
        .parse()
        .unwrap_or_default())
}

#[proc_macro_derive(GameJoltDatastore)]
pub fn derive_datastore(input: TokenStream) -> TokenStream {
    let name = match target_type(input) {
        Ok(name) => name,
        Err(error) => return error,
    };
    format!(
        "impl ::gamejolt_rs::datastore::GameJoltDatastore for {name} {{ fn datastore_type_name() -> &'static str {{ stringify!({name}) }} }}"
    )
    .parse()
    .unwrap_or_default()
}

#[proc_macro_derive(GameJoltScore)]
pub fn derive_score(input: TokenStream) -> TokenStream {
    let name = match target_type(input) {
        Ok(name) => name,
        Err(error) => return error,
    };
    format!(
        "impl ::gamejolt_rs::scores::GameJoltScore for {name} {{ fn score_type_name() -> &'static str {{ stringify!({name}) }} }}"
    )
    .parse()
    .unwrap_or_default()
}
