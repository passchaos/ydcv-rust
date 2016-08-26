pub const REQUEST_BASE: &'static str = "http://fanyi.youdao.com/openapi.do?keyfrom=ydcv-rust&key=379421805&type=data&doctype=json&version=1.1&q=";

#[derive(Debug, Serialize, Deserialize)]
pub struct Basic {
    pub explains: Vec<String>,
    #[serde(rename="uk-phonetic")]
    uk_phonetic: String,
    #[serde(rename="us-phonetic")]
    us_phonetic: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reference {
    key: String,
    value: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Translation {
    pub basic: Basic,
    pub web: Vec<Reference>,
}
