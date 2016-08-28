#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate hyper;
extern crate serde_json;

extern crate ansi_term;

extern crate rocksdb;

#[macro_use]
extern crate log;
extern crate env_logger;

mod trans_type;

use std::io::Read;

use hyper::Client;
use hyper::header::Connection;

use std::env;
use std::str;

use rocksdb::{DB, Writable};

const REQUEST_BASE: &'static str = "http://fanyi.youdao.com/openapi.do?keyfrom=ydcv-rust&key=379421805&type=data&doctype=json&version=1.1&q=";

fn get_remote_json_translation(query: &str, cache_db: &DB) -> Result<String, String> {
    let mut request_url = REQUEST_BASE.to_string();

    request_url.push_str(query);

    let client = Client::new();

    info!("开始从网络获取翻译结果");
    let mut res = match client.get(&request_url).header(Connection::close()).send() {
        Ok(noun) => noun,
        Err(err) => {
            println!("网络错误，请稍候重试！ {}", err);
            return Err(format!("{}", err))
        },
    };

    let mut body = String::new();
    res.read_to_string(&mut body).expect("网络响应中没有字符段");

    debug!("json content: {}", body);

    let trans_result: trans_type::Translation = serde_json::from_str(body.as_str()).expect("需要更新本地json格式处理");

    info!("将 「{}」 的翻译结果写入缓存", query);
    match cache_db.put(query.as_bytes(), format!("{}", trans_result).as_bytes()) {
        Err(err) => info!("{}", err),
        _ => {},        
    };
    
    Ok(format!("{}", trans_result))
}

fn main() {
    env_logger::init().unwrap();
    
    let args: Vec<String> = env::args().collect();

    let ref arg = match args.get(1) {
        Some(noun) => noun,
        None => {
            println!("请输入需要翻译的词语");
            return;
        }
    };

    let db = DB::open_default("/home/passchaos/.cache/ydcv/cache").unwrap();

    let db_key = arg.as_bytes();

    match db.get(db_key) {
        Ok(Some(value)) => {
            info!("从本地缓存读取");
            println!("{}", value.to_utf8().unwrap());            
        },
        _ => {
            let translation = get_remote_json_translation(arg.as_str(), &db);
            println!("{}", translation.unwrap_or("网络出错".to_string()));
        }
    }
}
