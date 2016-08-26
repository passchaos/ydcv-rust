use std::fmt::{self, Formatter, Display};

#[derive(Debug, Serialize, Deserialize)]
struct Basic {
    explains: Vec<String>,
    #[serde(rename="uk-phonetic")]
    uk_phonetic: String,
    #[serde(rename="us-phonetic")]
    us_phonetic: String,
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
        write!(f, "\tUK: [{}] US: [{}]\n\n  Word Explanation:{}",
               self.uk_phonetic, self.us_phonetic, self.explains
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
