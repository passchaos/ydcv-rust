use std::io::Write;

use anyhow::Result;
use clap::Parser;
use redb::{Database, TableDefinition};
use serde_derive::Deserialize;
use termcolor::{Buffer, ColorSpec, WriteColor};

const REQUEST_BASE: &'static str = "http://fanyi.youdao.com/openapi.do?keyfrom=ydcv-rust&key=379421805&type=data&doctype=json&version=1.1&q=";

#[derive(Debug, Deserialize)]
struct Phonetic {
    #[serde(rename = "us-phonetic")]
    us: String,
    #[serde(rename = "uk-phonetic")]
    uk: String,
    explains: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Kv {
    key: String,
    value: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct YdcvResp {
    query: String,
    translation: Vec<String>,
    basic: Phonetic,
    web: Vec<Kv>,
}

impl YdcvResp {
    fn colorized(&self) -> Result<String> {
        let mut f = Buffer::ansi();

        f.set_color(ColorSpec::new().set_underline(true))?;

        write!(f, "{}", self.query)?;

        f.reset().unwrap();

        for (k, v) in [("us", &self.basic.us), ("uk", &self.basic.uk)] {
            write!(f, " {k}: [")?;
            f.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)))?;
            write!(f, "{}", v)?;
            f.reset()?;
            write!(f, "]")?;
        }

        for i in &self.translation {
            write!(f, " {}", i)?;
        }

        f.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)))?;
        write!(f, "\n  World Explanation:\n")?;
        f.reset()?;

        for i in &self.basic.explains {
            write!(f, "    * {}\n", i)?;
        }

        f.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)))?;
        write!(f, "\n  Web Reference:")?;
        f.reset()?;

        for Kv { key, value } in &self.web {
            write!(f, "\n    * ")?;

            f.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)))?;
            write!(f, "{key}\n")?;
            f.reset()?;

            let v = value.join(",");
            f.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Magenta)))?;
            write!(f, "       {v}")?;
            f.reset()?;
        }

        String::from_utf8(f.into_inner()).map_err(From::from)
    }
}

#[derive(Parser, Debug)]
struct Args {
    word: String,
}

fn save_dict_info(query: &str, result: &str) -> Result<()> {
    const TABLE: TableDefinition<str, str> = TableDefinition::new("ydcv");

    let Some(mut dir) = dirs::home_dir() else {
        return Ok(());
    };

    dir.push("proliferation/dict");

    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }

    dir.push("ydcv.redb");

    let db = unsafe { Database::create(dir, 1024 * 1024)? };

    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(TABLE)?;
        table.insert(query, result)?;
    }
    write_txn.commit()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let client = reqwest::Client::new();
    let url = format!("{REQUEST_BASE}{}", args.word);
    let resp: YdcvResp = client.get(url).send().await?.json().await?;

    let res = resp.colorized()?;
    println!("{}", res);

    save_dict_info(&args.word, &res)?;

    Ok(())
}
