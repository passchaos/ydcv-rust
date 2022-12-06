use std::{io::BufReader, path::Path, time::Duration};

use anyhow::{bail, Context, Result};
use arboard::{Clipboard, GetExtLinux};
use chinese_detection::{classify, ClassificationResult};
use clap::Parser;
use db::Answer;
use explain::YdcvResp;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use reqwest::Client;
use tempdir::TempDir;
use tokio::io::{self};
use tokio_stream::StreamExt;
use tokio_util::io::StreamReader;

mod db;
mod explain;

const REQUEST_BASE: &'static str = "http://fanyi.youdao.com/openapi.do?keyfrom=ydcv-rust&key=379421805&type=data&doctype=json&version=1.1&q=";

static CACHED_CONTENT: RwLock<(Option<String>, Option<String>)> = RwLock::new((None, None));

#[derive(Parser, Debug)]
struct Args {
    word: Option<String>,
    #[arg(short = 'd', long = "daemon", default_value_t = false)]
    daemon_mode: bool,
}

fn get_selected_text(clip: &mut Clipboard) -> Option<String> {
    let g = clip.get();
    let g = g.clipboard(arboard::LinuxClipboardKind::Primary);

    g.text().ok()
}

fn get_copied_text(clip: &mut Clipboard) -> Option<String> {
    let g = clip.get();
    let g = g.clipboard(arboard::LinuxClipboardKind::Clipboard);

    g.text().ok()
}

fn get_clipboard_content(clip: &mut Clipboard) -> Option<String> {
    let copied_text = get_copied_text(clip);
    let selected_text = get_selected_text(clip);

    if let Some(copied_text) = copied_text {
        if CACHED_CONTENT.read().0.as_ref() != Some(&copied_text) {
            CACHED_CONTENT.write().0 = Some(copied_text.clone());

            return Some(copied_text);
        }
    }

    if let Some(selected_text) = selected_text {
        if CACHED_CONTENT.read().1.as_ref() != Some(&selected_text) {
            CACHED_CONTENT.write().1 = Some(selected_text.clone());

            Some(selected_text)
        }
    }

    None
}

fn main() -> Result<()> {
    let args = Args::parse();

    let fut = async {
        if args.daemon_mode {
            let mut clip = Clipboard::new()?;

            let mut initial_clip_content = get_clipboard_content(&mut clip);

            loop {
                let new_clip_content = get_clipboard_content(&mut clip);

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

async fn tts(word: &str) -> Result<()> {
    // type:
    // * 为1时，是英音
    // * 为2时，是美音
    const URL: &str = "http://dict.youdao.com/dictvoice?type=1&audio=";
    let client = HTTP_CLIENT.clone();
    let url = format!("{URL}{word}");

    let stream = client
        .get(url)
        .timeout(Duration::from_secs(1))
        .send()
        .await?
        .bytes_stream()
        .map(|r| r.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));

    let mut reader = StreamReader::new(stream);

    let tmp_dir = TempDir::new("dict_tts_uk")?;

    let file_path = tmp_dir.path().join("{word}.mp3");

    let mut writer = tokio::fs::File::create(&file_path).await?;
    let data_len = io::copy_buf(&mut reader, &mut writer).await?;

    println!("mp3 data len: {data_len}");

    play_mp3(file_path.as_ref())?;

    Ok(())
}

fn play_mp3(file_path: &Path) -> Result<()> {
    let (_stream, handle) = rodio::OutputStream::try_default()?;
    let sink = rodio::Sink::try_new(&handle)?;

    let f = std::fs::File::open(file_path)?;
    sink.append(rodio::Decoder::new_mp3(BufReader::new(f))?);

    sink.sleep_until_end();

    Ok(())
}

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

async fn lookup(word: String) -> Result<()> {
    if classify(&word) == ClassificationResult::EN {
        if let Err(e) = tts(&word).await {
            eprintln!("tts meet failure: err= {e}");
        }
    }

    let client = HTTP_CLIENT.clone();
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
