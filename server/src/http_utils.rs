use http;

enum MsgType{
    Json,
    Text,
    File
}

pub fn stringify_response(response: &http::Response<String>) -> String{
    let mut out = format!("{:?} {:?}\r\n", response.version(), response.status());
    for (name, value) in response.headers(){
        out = out + &format!("{}: {}\r\n",name.to_string(), value.to_str().unwrap())[..];
    }
    out = out + "\r\n" + response.body();

    out
}

pub fn stringify_request(req: &httparse::Request) -> String{
    let mut out = format!("\n\nmethod: {}\npath: {}\nversion: {}\nheaders:\n",req.method.unwrap(), req.path.unwrap(), req.version.unwrap());
    for header in req.headers.iter(){
        out += format!("{:?}\n", header).as_str();
    }
    out
}

pub fn ok() -> http::Response<()>{
    http::Response::builder()
        .status(200)
        .body(())
        .unwrap()
}

pub fn ok_body(msg: String, msg_type: MsgType) -> http::Response<>{

}

pub fn bad_request(msg: String) -> http::Response<String>{
    http::Response::builder()
        .status(400)
        .body(msg)
        .unwrap()
}

//TODO: 