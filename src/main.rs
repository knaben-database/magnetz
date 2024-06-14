use std::collections::HashMap;
use url::Url;

#[derive(Debug)]
struct Magnet {
    exact_topic: String,
    display_name: Option<String>,
    exact_length: Option<u64>,
    address_trackers: Vec<String>,
}
impl Magnet {
    fn new(link: &str) -> Magnet {
        let parsed = Url::parse(link).expect(format!("Unparsable url {}", link).as_str());
        let params: HashMap<_, _> = parsed.query_pairs().into_owned().collect();

        let exact_topic = params
            .get("xt")
            .expect("Required field xt not found")
            .to_string();
        let display_name = params.get("dn").map(String::from);
        let exact_length = params.get("xl").and_then(|s| s.parse().ok());

        let mut address_trackers = Vec::new();
        for (key, value) in parsed.query_pairs() {
            if key == "tr" {
                address_trackers.push(value.into_owned());
            }
        }

        Magnet {
            exact_topic,
            display_name,
            exact_length,
            address_trackers,
        }
    }
}

fn main() {
    let test_magnets : Vec<&str> = vec![
        "magnet:?xt=urn:btih:e536aa798701b027a098c8ebbca5943d3bda3a0c&dn=%5b2022.03.24%5d%20Blue%20Archive%20Original%20Soundtrack%20Vol.1%20%ef%bd%9eLonging%20for%20the%20memorable%20days%ef%bd%9e%20%5bFLAC%5d&tr=http%3a%2f%2fnyaa.tracker.wf%3a7777%2fannounce&tr=udp%3a%2f%2fopen.stealth.si%3a80%2fannounce&tr=udp%3a%2f%2ftracker.opentrackr.org%3a1337%2fannounce&tr=udp%3a%2f%2fexodus.desync.com%3a6969%2fannounce&tr=udp%3a%2f%2ftracker.torrent.eu.org%3a451%2fannounce",
        "magnet:?xt=urn:btih:5502910db0473c553d0aac7332ee7fadbdc05085&dn=%5b2022.07.17%5d%20Blue%20Archive%20Original%20Soundtrack%20Vol.2%20%ef%bd%9eSearching%20for%20the%20unknown%20truth%ef%bd%9e%20%5bFLAC%2048kHz%ef%bc%8f24bit%5d&tr=http%3a%2f%2fnyaa.tracker.wf%3a7777%2fannounce&tr=udp%3a%2f%2fopen.stealth.si%3a80%2fannounce&tr=udp%3a%2f%2ftracker.opentrackr.org%3a1337%2fannounce&tr=udp%3a%2f%2fexodus.desync.com%3a6969%2fannounce&tr=udp%3a%2f%2ftracker.torrent.eu.org%3a451%2fannounce",
        "magnet:?xt=urn:btih:70fc6581383f880642a6d95b16c2ceff08c67aa8&dn=%5b2023.01.23%5d%20Blue%20Archive%20Original%20Soundtrack%20Vol.3%20%ef%bd%9eReaching%20for%20the%20precious%20time%ef%bd%9e%20%5bFLAC%2048kHz%ef%bc%8f24bit%5d&tr=http%3a%2f%2fnyaa.tracker.wf%3a7777%2fannounce&tr=udp%3a%2f%2fopen.stealth.si%3a80%2fannounce&tr=udp%3a%2f%2ftracker.opentrackr.org%3a1337%2fannounce&tr=udp%3a%2f%2fexodus.desync.com%3a6969%2fannounce&tr=udp%3a%2f%2ftracker.torrent.eu.org%3a451%2fannounce"
    ];
    let magnet = Magnet::new(test_magnets[0]);
    println!("Hello, world! {:?}", magnet);
}
