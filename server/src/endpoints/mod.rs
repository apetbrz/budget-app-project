pub mod database;
pub mod files;
pub mod index;
pub mod users;

use std::path;

use http_bytes::http;

pub enum Content {
    File(String),
    HandlerFunction(
        Box<dyn Fn(&mut path::Iter, Option<String>) -> Result<http::Response<Vec<u8>>, String>>,
    ),
    LoginRequest,
    RegisterRequest,
    LogoutRequest,
    UserDataRequest,
    UserCommand
}
pub fn new_func_endpoint(
    func: Box<
        dyn Fn(&mut path::Iter, Option<String>) -> Result<http::Response<Vec<u8>>, String>,
    >,
) -> Content {
    Content::HandlerFunction(func)
}