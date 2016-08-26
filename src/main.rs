#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate hyper;
extern crate serde_json;

mod trans_type;

use std::io::Read;

use hyper::Client;
use hyper::header::Connection;

use std::env;

const REQUEST_BASE: &'static str = "http://fanyi.youdao.com/openapi.do?keyfrom=ydcv-rust&key=379421805&type=data&doctype=json&version=1.1&q=";

fn main() {
    let args: Vec<String> = env::args().collect();

    let ref arg = match args.get(1) {
        Some(noun) => noun,
        None => {
            println!("请输入需要翻译的词语");
            return;
        }
    };
    
    let mut request_url = REQUEST_BASE.to_string();

    request_url.push_str(arg.as_str());

    let client = Client::new();

    let mut res = client.get(&request_url).header(Connection::close()).send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    let trans_result: trans_type::Translation = serde_json::from_str(body.as_str()).unwrap();

    println!("{}", trans_result);
}
