#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate hyper;
extern crate serde_json;

mod trans_type;

use std::io::Read;

use hyper::Client;
use hyper::header::Connection;

use serde_json::Value;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let ref arg = match args.get(1) {
        Some(noun) => noun,
        None => {
            println!("请输入需要翻译的词语");
            return;
        }
    };
    
    let mut request_url = trans_type::REQUEST_BASE.to_string();

    request_url.push_str(arg.as_str());

    let client = Client::new();

    let mut res = client.get(&request_url).header(Connection::close()).send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    let trans_result: trans_type::Translation = serde_json::from_str(body.as_str()).unwrap();

    println!("trans basic: {:?}\n webs: {:?}", trans_result.basic, trans_result.web);
    println!("explains: {}", trans_result.basic.explains
             .iter()
             .fold(String::new(), |mut acc, ref x| {
                 acc.push_str(format!("\n\t* {}", x).as_str());
                 acc
             }));

    // println!("webs: {}", trans_result.web
    //          .iter()
    //          .fold(String::new(), |mut acc, ref x| {
    //              acc.push_str(&*format!("\n\t* {}", x.key, x.va));
    //              acc
    //          }));
}
