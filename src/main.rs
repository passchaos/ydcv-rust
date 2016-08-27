#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate hyper;
extern crate serde_json;

extern crate ansi_term;

extern crate rocksdb;

mod trans_type;

use std::io::Read;

use hyper::Client;
use hyper::header::Connection;

use std::env;

use rocksdb::Writable;

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

    let mut db = rocksdb::DB::open_default("/home/passchaos/.cache/ydcv/cache").unwrap();

    let db_key = arg.as_bytes();
    if let Ok(Some(value)) = db.get(db_key) {
        println!("{}", value.to_utf8().unwrap());
        return
    }
    
    let mut request_url = REQUEST_BASE.to_string();

    request_url.push_str(arg.as_str());

    let client = Client::new();

    let mut res = match client.get(&request_url).header(Connection::close()).send() {
        Ok(noun) => noun,
        Err(err) => {
            println!("网络错误，请稍候重试！ {}", err);
            return
        }
    };

    let mut body = String::new();
    res.read_to_string(&mut body).expect("网络响应中没有字符段");

    let trans_result: trans_type::Translation = serde_json::from_str(body.as_str()).expect("需要更新本地json格式处理");
    db.put(db_key, format!("{}", trans_result).as_bytes());
    println!("{}", trans_result);
}
