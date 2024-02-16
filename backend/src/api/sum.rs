use serde::Deserialize;
use warp::Reply;
use ts_rs::TS;
use warp::Filter;

#[make_endpoint::export_to("./bindings/sum.ts")]
pub struct QueryParams {
    a: i32,
    b: i32,
}

#[make_endpoint::make_endpoint]
pub fn sum(
    _token: String,
    params: QueryParams
) -> i32 {
    params.a + params.b
}

#[make_endpoint::make_endpoint]
pub fn sub(
    _token: String,
    params: QueryParams
) -> i32 {
    params.a - params.b
}