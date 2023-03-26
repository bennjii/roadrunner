use std::convert::Infallible;
use warp::hyper::StatusCode;

pub async fn echo() -> Result<Box<dyn warp::Reply>, Infallible> {
    Ok(Box::new(StatusCode::OK))
}