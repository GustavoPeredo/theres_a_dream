extern crate pretty_env_logger;
#[macro_use] extern crate log;

use warp::Filter;
use std::collections::HashMap;
use serde_json::json;
use dotenv::dotenv;
use warp::http::StatusCode;
use std::convert::Infallible;
use warp::http::Method;
use std::env;
use warp::Reply;

use jwt;

mod api;

async fn custom_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    if err.find::<jwt::InvalidJwt>().is_some() {
        Ok(warp::reply::with_status("Unauthorized", StatusCode::UNAUTHORIZED))
    } else {
        Ok(warp::reply::with_status("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR))
    }
}

pub async fn login_handler(body: HashMap<String, String>) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(user_id) = body.get("user_id") {
        let token = jwt::encode_jwt(user_id.clone());
        Ok(warp::reply::json(&json!({"token": token})))
    } else {
        Err(warp::reject::not_found())
    }
}

pub async fn secure_handler(_token: String) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&"Access granted to secured resource."))
}

fn static_files_with_mime(dir: String) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    warp::fs::dir(dir.clone())
        .map(move |reply: warp::fs::File| {
            let mime_type = match reply.path().extension().and_then(|ext| ext.to_str()) {
                Some("js") => "application/javascript",
                Some("css") => "text/css",
                // Add more MIME types as needed
                _ => "text/html", // Default MIME type or use match to handle other cases
            };
            warp::reply::with_header(reply.into_response(), "Content-Type", mime_type)
        })
}
#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "todos=info");
    }
    pretty_env_logger::init();
    let WEB_APP_DIR = env::var("WEB_APP_DIR").expect("WEB_APP_DIR must be set");
    dotenv().ok(); // Load .env file

    let cors = warp::cors()
        .allow_any_origin() // .allow_origin("http://example.com")
        .allow_headers(vec!["Authorization", "Content-Type", "Accept"])
        .allow_methods(vec![Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .build();

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::body::json())
        .and_then(login_handler);

    let secure = warp::get() 
        .and(warp::path("secure"))
        .and(jwt::with_auth())
        .and_then(secure_handler);

    let apis = warp::path("api").and(
            login
            .or(secure)
                        .or(api::sum::sum(warp::path!("sum")))
            .or(api::sum::sub(warp::path!("sub")))
        ).with(cors).boxed(); //.recover(custom_rejection)

    let frontend = static_files_with_mime(WEB_APP_DIR.clone())
        .or(warp::fs::file(format!("{}/index.html", WEB_APP_DIR)));
        
    let routes = apis.or(frontend).with(warp::log("todos"));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
    