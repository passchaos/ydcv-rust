#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate hyper;
extern crate serde_json;

extern crate ansi_term;

extern crate rocksdb;

extern crate getopts;

#[macro_use]
extern crate log;
extern crate env_logger;

mod trans_type;

use std::io::Read;

use hyper::Client;
use hyper::header::Connection;

use getopts::Options;

use std::env;
use std::str;

use rocksdb::{DB, Writable, WriteBatch};

const REQUEST_BASE: &'static str = "http://fanyi.youdao.com/openapi.do?keyfrom=ydcv-rust&key=379421805&type=data&doctype=json&version=1.1&q=";

fn get_remote_json_translation(query: &str, cache_db: &DB, update_cache: bool) -> Result<String, String> {
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

    if update_cache {
        debug!("更新本地缓存");
        let mut batch = WriteBatch::default();
        batch.delete(query.as_bytes());
        batch.put(query.as_bytes(), format!("{}", trans_result).as_bytes());
        cache_db.write(batch);
    } else {
        debug!("写入本地缓存");
        match cache_db.put(query.as_bytes(), format!("{}", trans_result).as_bytes()) {
            Err(err) => info!("{}", err),
            _ => {},        
        };        
    }
    
    Ok(format!("{}", trans_result))
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] WORD", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    env_logger::init().unwrap();
    
    let args: Vec<String> = env::args().collect();

    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("u", "update", "更新本地缓存中对应的数据", "WORD");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => {m}
        Err(f) => {
            debug!("{}", f.to_string());
            print_usage(&program, opts);
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let mut update_local_tag = false;

    let output = match matches.opt_str("u") {
        Some(str) => {
            debug!("{:?}", matches.free);
            update_local_tag = true;
            str
        },
        None => {
            debug!("{:?}", matches.free);
            if matches.free.is_empty() {
                print_usage(&program, opts);
                return;                
            } else {
                matches.free[0].clone()
            }
        }
    };

    let db = DB::open_default("/home/passchaos/.cache/ydcv/cache").unwrap();

    let db_key = output.as_bytes();

    match db.get(db_key) {
        Ok(Some(ref value)) if !update_local_tag => {
            info!("从本地缓存读取");
            println!("{}", value.to_utf8().unwrap());            
        },
        _ => {
            let translation = get_remote_json_translation(output.as_str(), &db, update_local_tag);
            println!("{}", translation.unwrap_or("网络出错".to_string()));
        }
    }
}
