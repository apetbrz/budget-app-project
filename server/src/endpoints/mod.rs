pub mod database;
pub mod files;
pub mod index;
pub mod users;

use std::path;

use http_bytes::http;

pub enum Endpoint{
    FunctionHandler(Box<dyn Fn(&mut path::Iter, Option<String>) -> Result<http::Response<Vec<u8>>, String>>),
    LoginRequest,
    RegisterRequest
}
impl Endpoint{
    pub fn func(func: Box<dyn Fn(&mut path::Iter, Option<String>) -> Result<http::Response<Vec<u8>>, String>>) -> Endpoint{
        Endpoint::FunctionHandler(func)
    }
}