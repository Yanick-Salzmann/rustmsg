use crate::definition::cached_http_loader::CachedHttpLoader;

use super::index_processor::{load_index, IndexEntry};

struct SrConfig {
    sr: String,
    base_url: String,
    index_topic: String,
}

fn process_definition(entry: &IndexEntry, downloader: &CachedHttpLoader, config: &SrConfig) {
    println!("Processing {}", entry.description);
    let link = format!("{}/{}", config.base_url, entry.link);
    let html = downloader.download_string(&link).unwrap();
    let doc = tl::parse(&html, tl::ParserOptions::default()).unwrap();
}

pub fn process_definitions() {
    let service_releases = [SrConfig {
        sr: "sr2022".into(),
        base_url: "https://www2.swift.com/knowledgecentre/rest/v1/publications/usgf_20220722/2.0/"
            .into(),
        index_topic: "mt_messages.htm".into(),
    }];

    for ele in service_releases {
        let cfg = &ele;
        let downloader = CachedHttpLoader::new("./.cache".into(), &ele.sr);
        load_index(
            &*ele.base_url,
            &format!("{}{}", ele.base_url, ele.index_topic),
            &downloader,
        )
        .iter()
        .for_each(|e| process_definition(&e, &downloader, cfg));
    }
}
