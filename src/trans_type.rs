use std::fmt::{self, Formatter, Display};

use ansi_term::Colour::{self, RGB};
use ansi_term::Style;

#[derive(Debug, Deserialize)]
struct Basic {
    explains: Vec<String>,
    phonetic: Option<String>,
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
pub struct Translation {
    translation: Option<Vec<String>>,
    query: String,
    basic: Option<Basic>,
    web: Option<Vec<Reference>>,
}

const HEADER_COLOR: Colour = RGB(26, 159, 160);
const PHONETIC_COLOR: Colour = RGB(220, 186, 40);
const REFERENCE_COLOR: Colour = RGB(138, 88, 164);

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut content = String::new();
        let sub_str_count = self.contents.len();

        for (index, str) in self.contents.iter().enumerate() {
            content.push_str(str);
            
            if index < sub_str_count - 1 {
                content.push_str("; ")
            }
        }

        write!(f, "\n\t* {}\n\t  {}", self.key, REFERENCE_COLOR.paint(content))
    }
}

impl Display for Basic {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut tmp_str = String::new();

        if let Some(ref phone) = self.phonetic {
            tmp_str = format!("\t[{}]\n", PHONETIC_COLOR.paint(phone.clone()));            
        }


        // if self.uk_phonetic.is_some() && self.us_phonetic.is_some() {
        //     tmp_str = format!("\tUK: [{}] US: [{}]\n", PHONETIC_COLOR.paint(self.uk_phonetic.clone().unwrap()), PHONETIC_COLOR.paint(self.us_phonetic.clone().unwrap()));
        // }
        if let (Some(uk), Some(us)) = (self.uk_phonetic.clone(), self.us_phonetic.clone()) {
            tmp_str = format!("\tUK: [{}] US: [{}]\n", PHONETIC_COLOR.paint(uk), PHONETIC_COLOR.paint(us));
        }

        write!(f, "{}\n  {}:{}",
               tmp_str, HEADER_COLOR.paint("Word Explanation"), self.explains
               .iter()
               .fold(String::new(), |mut acc, ref x| {
                   acc.push_str(format!("\n\t* {}", x).as_str());
                   acc
               }))
    }
}

impl Display for Translation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let tmp_trans = match self.translation {
            Some(ref trans) => format!("\n  {}\n\t* {}", HEADER_COLOR.paint("Translation:"), trans.first().expect("")),
            None => String::new(),
        };
        
        let tmp_basic = match self.basic {
            Some(ref bsc) => format!("\n{}\n", bsc),
            None => String::new(),
        };

        let tmp_web = match self.web {
            Some(ref vecs) => {
                let content = vecs.iter()
                    .fold(String::new(), |mut acc, ref x| {
                        acc.push_str(format!("{}", x).as_str());
                        acc
                    });
                format!("\n  {}:{}", HEADER_COLOR.paint("Web Reference"), content)
            }
            None => String::new(),
        };
        
        write!(f, "{}:{}{}{}",
               Style::new().underline().paint(self.query.clone()), tmp_trans, tmp_basic, tmp_web)
    }
}
