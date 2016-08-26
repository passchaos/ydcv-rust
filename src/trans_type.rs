use std::fmt::{self, Formatter, Display};

#[derive(Debug, Serialize, Deserialize)]
struct Basic {
    explains: Vec<String>,
    phonetic: String,
    #[serde(rename="uk-phonetic")]
    uk_phonetic: Option<String>,
    #[serde(rename="us-phonetic")]
    us_phonetic: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Reference {
    key: String,
    value: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Translation {
    translation: Vec<String>,
    query: String,
    basic: Basic,
    web: Vec<Reference>,
}

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "\n\t* {}\n\t  {}", self.key, self.value.iter()
               .fold(String::new(), |mut acc, ref x| {
                   acc.push_str(x);
                   acc
               }))
    }
}

impl Display for Basic {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut tmp_str = format!("[{}]", self.phonetic);

        if let Some(ref uk) = self.uk_phonetic {
            if let Some(ref us) = self.us_phonetic {
                tmp_str = format!("UK: [{}] US: [{}]", uk, us);
            }
        }
        
        write!(f, "\t{}\n\n  Word Explanation:{}",
               tmp_str, self.explains
               .iter()
               .fold(String::new(), |mut acc, ref x| {
                   acc.push_str(format!("\n\t* {}", x).as_str());
                   acc
               }))
    }
}

impl Display for Translation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:\n\t{}\n{}\n\n  Web Reference:{}",
               self.query, self.translation.first().expect(""), self.basic, self.web
               .iter()
               .fold(String::new(), |mut acc, ref x| {
                   acc.push_str(format!("{}", x).as_str());
                   acc
               }))
    }
}
