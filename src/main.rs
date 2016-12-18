#![feature(proc_macro)]

extern crate hyper;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate ansi_term;

extern crate rocksdb;
extern crate clap;

#[macro_use(o, slog_log, slog_trace, slog_debug, slog_info, slog_warn, slog_error)]
extern crate slog;
extern crate slog_term;
// #[macro_use]
// extern crate slog_scope;
extern crate slog_envlogger;

mod formatter;
mod trans_type;

use std::io::{self, Read};

use hyper::Client;
use hyper::header::Connection;

use clap::{Arg, App};
use slog::DrainExt;

use std::env;
use std::str;

use rocksdb::DB;
use rocksdb::WriteBatch;

use formatter::YDCVFormatter;

const REQUEST_BASE: &'static str = "http://fanyi.youdao.com/openapi.do?keyfrom=ydcv-rust&key=379421805&type=data&doctype=json&version=1.1&q=";

fn get_remote_json_translation(query: &str, cache_db: &DB, update_cache: bool, logger: &slog::Logger) -> Result<String, YDCVError> {
    let mut request_url = REQUEST_BASE.to_string();

    request_url.push_str(query);

    let client = Client::new();

    slog_debug!(logger, "开始从网络获取翻译结果");

    let mut res = try!(client.get(&request_url).header(Connection::close()).send());

    let mut body = String::new();
    res.read_to_string(&mut body)?;

    slog_debug!(logger, "json content: {}", body);

    let mut trans_result: trans_type::Translation = try!(serde_json::from_str(body.as_str()));
    trans_result.logger = Some(logger);

    if update_cache {
        slog_debug!(logger, "更新本地缓存");
        let mut batch = WriteBatch::default();
        batch.delete(query.as_bytes());
        batch.put(query.as_bytes(), trans_result.translation_description().as_bytes());
        cache_db.write(batch);
    } else {
        slog_debug!(logger, "写入本地缓存");
        cache_db.put(query.as_bytes(), trans_result.translation_description().as_bytes())?;
    }

    Ok(trans_result.translation_description())
}

fn main() {
    slog_envlogger::init().unwrap();
    let drain = slog_term::streamer().compact().build().fuse();
    let root_log = slog::Logger::root(drain, o!("version" => "0.3.0"));

    let matches = App::new("YDCV")
        .version("0.3.0")
        .author("Greedwolf DSS <greedwolf.dss@gmail.com>")
        .about("Consolve version of Youdao")
        .arg(Arg::with_name("update").short("u").long("update").help("Update local cached word"))
        .arg(Arg::with_name("INPUT").required(true).index(1))
        .get_matches();

    let trans = matches.value_of("INPUT").unwrap();
    slog_info!(root_log, "trans: {}", trans);

    // 更新标记，是否更新本地缓存中的翻译结果
    let update_local_tag = matches.is_present("update");

    let cache_path = match env::home_dir().map(|mut path| {
        path.push(".cache/ydcv/cache");
        path
    }).and_then(|path| {
        path.to_str().map(|str| str.to_string())
    }) {
        Some(path) => path,
        None => {
            slog_error!(root_log, "没有缓存路径");
            return;
        },
    };
    slog_debug!(root_log, "cache path: {}", cache_path);

    let db = match DB::open_default(cache_path.as_str()) {
        Ok(db) => db,
        Err(err) => {
            slog_error!(root_log, "无法创建RocksDB的存储目录 error: {}", err);
            return;
        }
    };

    let db_key = trans.as_bytes();

    match db.get(db_key) {
        Ok(Some(ref value)) if !update_local_tag => {
            slog_info!(root_log, "从本地缓存读取");
            println!("{}", value.to_utf8().unwrap());
        },
        _ => {
            if let Some(translation) = get_remote_json_translation(trans, &db, update_local_tag, &root_log).ok() {
                println!("{}", translation);
            }
        }
    }
}

#[derive(Debug)]
enum YDCVError {
    Io(io::Error),
    Network(hyper::Error),
    Json(serde_json::Error),
    Cache(String),
    Rocksdb(rocksdb::Error),
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
impl From<rocksdb::Error> for YDCVError {
    fn from(err: rocksdb::Error) -> YDCVError {
        YDCVError::Rocksdb(err)
    }
}
