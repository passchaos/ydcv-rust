use anyhow::{Context, Result};
use clap::Parser;
use explain::YdcvResp;

mod db;
mod explain;

const REQUEST_BASE: &'static str = "http://fanyi.youdao.com/openapi.do?keyfrom=ydcv-rust&key=379421805&type=data&doctype=json&version=1.1&q=";

#[derive(Parser, Debug)]
struct Args {
    word: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let client = reqwest::Client::new();
    let url = format!("{REQUEST_BASE}{}", args.word);

    let resp = client.get(url).send().await?.text().await?;

    let resp: YdcvResp = serde_json::from_str(&resp).context(format!(
        "parse resp to json meet failure: raw_resp= {}",
        resp
    ))?;

    let res = resp.colorized()?;
    // println!("{}", res);

    db::save_query_explain(&args.word, res)?;

    Ok(())
}
