use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use http::HeaderValue;
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn request_id_layer(mut request: Request, next: Next) -> Response {
    let id = Uuid::new_v4().to_string();
    request
        .headers_mut()
        .insert(REQUEST_ID_HEADER, HeaderValue::from_str(&id).unwrap());

    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(REQUEST_ID_HEADER, HeaderValue::from_str(&id).unwrap());
    response
}
