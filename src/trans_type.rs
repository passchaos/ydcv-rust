use std::fmt::{self, Formatter, Display};

use ansi_term::Colour::{self, RGB};
use ansi_term::Style;

use formatter::YDCVFormatter;

use slog;
use slog::DrainExt;
use slog_term;

#[derive(Debug, Deserialize)]
struct Basic {
    explains: Vec<String>,
    #[serde(rename="uk-phonetic")]
    uk_phonetic: Option<String>,
    #[serde(rename="us-phonetic")]
    us_phonetic: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Reference {
    key: String,
    #[serde(rename="value")]
    contents: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Translation<'a> {
    translation: Option<Vec<String>>,
    query: String,
    basic: Option<Basic>,
    web: Option<Vec<Reference>>,
    #[serde(skip_deserializing)]
    pub logger: Option<&'a slog::Logger>,
}

impl<'a> YDCVFormatter for Translation<'a> {
    fn translation_description(&self) -> String {
        let yellow_star = Colour::Yellow.paint("*");

        if let Some(ref logger) = self.logger {
            slog_info!(logger, "yellow_star: {}", yellow_star);
        };
        let mut header_str = String::new();
        if let Some(ref translations) = self.translation {
            header_str.push_str(&format!("  {}\n\t{} ", Colour::Purple.paint("Translation:"), yellow_star));
            if let Some(ref logger) = self.logger {
                slog_info!(logger, "current header_str: {}", header_str);
            };
            for (idx, value) in translations.iter().enumerate() {
                header_str.push_str(&value);
                if idx == translations.len() - 1 {
                    header_str.push_str("\n");
                } else {
                    header_str.push_str("; ");
                }
            }
        }

        let mut phonetic_str = String::new();
        if let Some(ref phonetic_basic) = self.basic {
            phonetic_str.push_str(&format!("  {}\n", Colour::Purple.paint("Word Explanation")));
            if let Some(ref uk_phonetic) = phonetic_basic.uk_phonetic {
                phonetic_str.push_str(&format!("\tUK: [{}]", Style::new().underline().paint(uk_phonetic.as_str())));
                if let Some(ref us_phonetic) = phonetic_basic.us_phonetic {
                    phonetic_str.push_str(&format!(" US: [{}]\n", Style::new().underline().paint(us_phonetic.as_str())));
                };
            } else {
                if let Some(ref us_phonetic) = phonetic_basic.us_phonetic {
                    phonetic_str.push_str(&format!("\tUS: [{}]\n", Style::new().underline().paint(us_phonetic.as_str())));
                }
            }

            for explain in &phonetic_basic.explains {
                phonetic_str.push_str(&format!("\t{} {}\n", yellow_star, explain));
            }
        }

        let mut reference_str = String::new();
        if let Some(ref web_ref) = self.web {
            reference_str.push_str(&format!("  {}\n", Colour::Purple.paint("Web Reference:")));
            for web in web_ref {
                reference_str.push_str(&format!("\t{} {}\n\t  ", yellow_star, web.key));
                for (idx, value) in web.contents.iter().enumerate() {
                    reference_str.push_str(&value);
                    if idx != web.contents.len() - 1 {
                        reference_str.push_str("; ");
                    } else {
                        reference_str.push_str("\n");
                    }
                }
            }
        }

        if !header_str.is_empty() {
            header_str.push_str("\n");
        }
        header_str.push_str(&phonetic_str);
        if !reference_str.is_empty() {
            header_str.push_str("\n");
        }
        header_str.push_str(&reference_str);
        header_str
    }
}
