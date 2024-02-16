extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, LitStr, ItemFn, FnArg};

#[proc_macro_attribute]
pub fn make_endpoint(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_block = &input.block;
    let attrs = &input.attrs;

    // Attempt to extract the type of the second argument for the body parameters.
    let body_param_type = input.sig.inputs.iter().nth(1).map(|arg| {
        match arg {
            FnArg::Typed(pat_type) => pat_type.ty.clone(),
            _ => panic!("Expected the second argument to be a typed parameter"),
        }
    }).expect("Failed to extract body parameter type");

    let output = quote! {
        #(#attrs)*
        pub fn #fn_name<F>(
            warp_filter: F
        ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone 
            where F: warp::Filter<Extract = (), Error = warp::Rejection> + Clone
        {
            warp_filter
                .and(warp::post())
                .and(jwt::with_auth())
                .and(warp::body::json::<#body_param_type>()) // Changed to use body::json for deserializing request body
                .and_then(move |token: String, params: #body_param_type| async move {
                    let result = async { #fn_block }.await; // Execute the original function block
                    match serde_json::to_string(&result) {
                        Ok(json) => Ok(warp::reply::with_status(json, warp::http::StatusCode::OK).into_response()),
                        Err(_) => Err(warp::reject::custom(jwt::InvalidJwt)),
                    }
                })
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn export_to(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_str = parse_macro_input!(attr as LitStr);
    let attr_value = attr_str.value();

    let item = parse_macro_input!(item as ItemStruct);

    let expanded = quote! {
        #[derive(Deserialize, TS)]
        #[ts(rename_all = "lowercase")]
        #[ts(export, export_to = #attr_value)]
        #item
    };
    TokenStream::from(expanded)
}