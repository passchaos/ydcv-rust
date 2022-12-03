use std::time::Duration;

use anyhow::{bail, Context, Result};
use arboard::{Clipboard, GetExtLinux};
use clap::Parser;
use db::Answer;
use explain::YdcvResp;

mod db;
mod explain;

const REQUEST_BASE: &'static str = "http://fanyi.youdao.com/openapi.do?keyfrom=ydcv-rust&key=379421805&type=data&doctype=json&version=1.1&q=";

#[derive(Parser, Debug)]
struct Args {
    word: Option<String>,
    #[arg(short = 'd', long = "daemon", default_value_t = false)]
    daemon_mode: bool,
}

fn get_selected_text(clip: &mut Clipboard) -> Option<String> {
    let g = clip.get();
    let g = g.clipboard(arboard::LinuxClipboardKind::Primary);

    g.text()
        .map_err(|e| {
            // eprintln!("get primary clipboard meet failure: err= {e}");
            e
        })
        .ok()
}

fn main() -> Result<()> {
    let args = Args::parse();

    let fut = async {
        if args.daemon_mode {
            let mut clip = Clipboard::new()?;

            let mut initial_clip_content = get_selected_text(&mut clip);

            loop {
                let new_clip_content = get_selected_text(&mut clip);

                if new_clip_content != initial_clip_content {
                    initial_clip_content = new_clip_content.clone();

                    if let Some(text) = new_clip_content {
                        if let Err(e) = lookup(text.clone()).await {
                            eprintln!("lookup meet failure: content= {text} err= {e}");
                        }
                    }
                }

                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        } else {
            let Some(word) = args.word else {
                bail!("no word specified");
            };

            lookup(word).await?;
        }

        Ok::<_, anyhow::Error>(())
    };

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(fut)?;

    Ok(())
}

async fn lookup(word: String) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{REQUEST_BASE}{}", word);

    let web_explain_action = async {
        let resp = client
            .get(url)
            .timeout(Duration::from_secs(1))
            .send()
            .await?
            .text()
            .await?;

        let resp: YdcvResp = serde_json::from_str(&resp).context(format!(
            "parse resp to json meet failure: raw_resp= {}",
            resp
        ))?;

        let res = resp.colorized()?;

        Ok::<_, anyhow::Error>(res)
    };

    match web_explain_action.await {
        Ok(res) => {
            let Answer {
                explain,
                query_count,
            } = db::step_forward_with_web_result(&word, res)?;

            println!("query count: {query_count}\nweb result:\n\n{explain}");
        }
        Err(e) => {
            if let Some(Answer {
                explain,
                query_count,
            }) = db::step_forward_with_local_only(&word)?
            {
                println!("query count: {query_count}\nlocal result:\n\n{explain}");
            } else {
                bail!("no explain found: query= {word} query_err= {e}");
            }
        }
    }

    Ok(())
}
