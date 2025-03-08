use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ElasticsearchMgetResponse {
    pub docs: Vec<Doc>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Doc {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_index")]
    index: String,
    #[serde(rename = "_primary_term")]
    primary_term: Option<u64>,
    #[serde(rename = "_seq_no")]
    seq_no: Option<u64>,
    #[serde(rename = "_source")]
    pub source: Option<Source>,
    #[serde(rename = "_version")]
    version: Option<u64>,
    found: bool,
}

#[derive(Deserialize, Debug)]
pub struct Source {
    #[serde(rename = "MagnetUri")]
    pub magnet_uri: Option<String>,
}
