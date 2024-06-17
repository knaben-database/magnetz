use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ElasticsearchMgetResponse {
    pub docs: Vec<Doc>,
}

#[derive(Deserialize, Debug)]
pub struct Doc {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_index")]
    index: String,
    #[serde(rename = "_primary_term")]
    primary_term: u64,
    #[serde(rename = "_seq_no")]
    seq_no: u64,
    #[serde(rename = "_source")]
    pub source: Source,
    #[serde(rename = "_type")]
    r#type: String,
    #[serde(rename = "_version")]
    version: u64,
    found: bool,
}

#[derive(Deserialize, Debug)]
pub struct Source {
    #[serde(rename = "MagnetUri")]
    pub magnet_uri: Option<String>,
}
