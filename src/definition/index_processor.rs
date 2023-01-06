use regex::Regex;

use super::cached_http_loader::CachedHttpLoader;

#[derive(Debug)]
pub struct IndexEntry {
    pub message_type: String,
    pub description: String,
    pub link: String,
}

pub fn load_index(
    base_url: &str,
    url: &str,
    downloader: &CachedHttpLoader,
) -> std::collections::LinkedList<IndexEntry> {
    let html = downloader.download_string(url).unwrap();
    let doc = tl::parse(&html, tl::ParserOptions::default()).unwrap();
    let parser = doc.parser();

    return doc
        .query_selector("a".into())
        .unwrap()
        .flat_map(|link| {
            let tag = link.get(parser).unwrap().as_tag().unwrap();
            let topic = tag
                .attributes()
                .get("href")
                .unwrap()
                .unwrap()
                .as_utf8_str()
                .to_string();
            return load_types_for_category(&format!("{}{}", base_url, topic), downloader);
        })
        .collect();
}

fn load_types_for_category(
    url: &str,
    downloader: &CachedHttpLoader,
) -> std::collections::LinkedList<IndexEntry> {
    let html = downloader.download_string(url).unwrap();
    let doc = tl::parse(&html, tl::ParserOptions::default()).unwrap();
    let parser = doc.parser();

    let invalid_char_regex = Regex::new("[^A-Za-z0-9 \\-]").unwrap();

    return doc
        .query_selector("a".into())
        .unwrap()
        .map(|link| {
            let tag = link.get(parser).unwrap().as_tag().unwrap();
            let mut name = tag.inner_text(parser).to_string();
            name = invalid_char_regex.replace_all(&name, "").into();

            let link = tag
                .attributes()
                .get("href")
                .unwrap()
                .unwrap()
                .as_utf8_str()
                .to_string();
            let mt = name.split_ascii_whitespace().take(1).last().unwrap();

            return IndexEntry {
                link,
                description: name.clone().into(),
                message_type: mt.to_string(),
            };
        })
        .collect();
}
