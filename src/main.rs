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

use std::error;

use std::io::{self, Read};

use hyper::Client;
use hyper::header::Connection;

use getopts::Options;

use std::env;
use std::str;

use rocksdb::{DB, Writable, WriteBatch};

#[derive(Debug)]
enum YDCVError {
    Io(io::Error),
    Network(hyper::Error),
    Json(serde_json::Error),
    Cache(String),
}

impl From<io::Error> for YDCVError {
    fn from(err: io::Error) -> YDCVError {
        YDCVError::Io(err)
    }
}
impl From<hyper::Error> for YDCVError {
    fn from(err: hyper::Error) -> YDCVError {
        YDCVError::Network(err)
    }
}
impl From<serde_json::Error> for YDCVError {
    fn from(err: serde_json::Error) -> YDCVError {
        YDCVError::Json(err)
    }
}
impl From<String> for YDCVError {
    fn from(err: String) -> YDCVError {
        YDCVError::Cache(err)
    }
}

const REQUEST_BASE: &'static str = "http://fanyi.youdao.com/openapi.do?keyfrom=ydcv-rust&key=379421805&type=data&doctype=json&version=1.1&q=";

fn get_remote_json_translation(query: &str, cache_db: &DB, update_cache: bool) -> Result<String, YDCVError> {
    let mut request_url = REQUEST_BASE.to_string();

    request_url.push_str(query);

    let client = Client::new();

    info!("开始从网络获取翻译结果");

    let mut res = try!(client.get(&request_url).header(Connection::close()).send());

    let mut body = String::new();
    try!(res.read_to_string(&mut body));

    debug!("json content: {}", body);

    let trans_result: trans_type::Translation = try!(serde_json::from_str(body.as_str()));

    if update_cache {
        debug!("更新本地缓存");
        let batch = WriteBatch::default();
        batch.delete(query.as_bytes());
        batch.put(query.as_bytes(), format!("{}", trans_result).as_bytes());
        cache_db.write(batch);
    } else {
        debug!("写入本地缓存");
        try!(cache_db.put(query.as_bytes(), format!("{}", trans_result).as_bytes()));
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

    let program = &args[0];

    let mut opts = Options::new();
    opts.optopt("u", "update", "更新本地缓存中对应的数据", "WORD");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
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

    let cache_path = match env::home_dir().map(|mut path| {
        path.push(".cache/ydcv/cache");
        path
    }).and_then(|path| {
        path.to_str().map(|str| str.to_string())
    }) {
        Some(path) => path,
        None => {
            println!("没有缓存路径");
            return;
        },
    };

    let db = match DB::open_default(cache_path.as_str()) {
        Ok(db) => db,
        Err(err) => {
            println!("无法创建RocksDB的存储目录 error: {}", err);
            return;
        }
    };

    let db_key = output.as_bytes();

    match db.get(db_key) {
        Ok(Some(ref value)) if !update_local_tag => {
            info!("从本地缓存读取");
            println!("{}", value.to_utf8().unwrap());            
        },
        _ => {
            let translation = get_remote_json_translation(output.as_str(), &db, update_local_tag);
            println!("{:?}", translation);
        }
    }
}
