use std::io::Write;

use anyhow::Result;
use serde_derive::Deserialize;
use termcolor::{Buffer, ColorSpec, WriteColor};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Phonetic {
    En2Zh {
        #[serde(rename = "us-phonetic")]
        us: String,
        #[serde(rename = "uk-phonetic")]
        uk: String,
    },
    Zh2En {
        phonetic: String,
    },
}

#[derive(Debug, Deserialize)]
struct Basic {
    // 有些查询没有这个字段，比如 `cli`
    #[serde(flatten)]
    phonetic: Option<Phonetic>,
    explains: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Kv {
    key: String,
    value: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct YdcvResp {
    query: String,
    translation: Vec<String>,
    basic: Basic,
    web: Option<Vec<Kv>>,
}

impl YdcvResp {
    pub fn colorized(&self) -> Result<String> {
        let mut f = Buffer::ansi();

        f.set_color(ColorSpec::new().set_underline(true))?;

        write!(f, "{}", self.query)?;

        f.reset().unwrap();

        match &self.basic.phonetic {
            Some(Phonetic::En2Zh { us, uk }) => {
                for (k, v) in [("us", us), ("uk", uk)] {
                    write!(f, " {k}: [")?;
                    f.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)))?;
                    write!(f, "{v}")?;
                    f.reset()?;
                    write!(f, "]")?;
                }
            }
            Some(Phonetic::Zh2En { phonetic }) => {
                write!(f, " [")?;
                f.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)))?;
                write!(f, " {phonetic} ")?;
                f.reset()?;
                write!(f, "]")?;
            }
            None => {}
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

        if let Some(web) = &self.web {
            f.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)))?;
            write!(f, "\n  Web Reference:")?;
            f.reset()?;

            for Kv { key, value } in web {
                write!(f, "\n    * ")?;

                f.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)))?;
                write!(f, "{key}\n")?;
                f.reset()?;

                let v = value.join(",");
                f.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Magenta)))?;
                write!(f, "       {v}")?;
                f.reset()?;
            }
        }

        String::from_utf8(f.into_inner()).map_err(From::from)
    }
}
