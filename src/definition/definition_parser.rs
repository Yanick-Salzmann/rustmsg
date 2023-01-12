use std::collections::LinkedList;

use regex::Regex;
use tl::{HTMLTag, NodeHandle};

use crate::definition::cached_http_loader::CachedHttpLoader;

use super::index_processor::{load_index, IndexEntry};

struct SrConfig {
    sr: String,
    base_url: String,
    index_topic: String,
}

struct FieldTableIndices {
    status: usize,
    tag: usize,
    name: usize,
    name_fallback: usize,
    qualifier: usize,
    link: usize,
}

fn process_definition(entry: &IndexEntry, downloader: &CachedHttpLoader, config: &SrConfig) {
    println!("Processing {}", entry.description);
    if Regex::new("MT[0-9]9[0-9]")
        .unwrap()
        .find(&entry.description)
        .is_some()
    {
        return;
    }

    let link = format!("{}/{}", config.base_url, entry.link);
    let html = downloader.download_string(&link).unwrap();
    let doc = tl::parse(&html, tl::ParserOptions::default()).unwrap();
    let parser = doc.parser();
    let table = doc
        .query_selector("div[id$=format-spec]")
        .unwrap()
        .next()
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap()
        .query_selector(parser, "table")
        .unwrap()
        .next()
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap();

    let headers: Vec<NodeHandle> = table.query_selector(parser, "th").unwrap().collect();
    let rows: Vec<Vec<NodeHandle>> = table.query_selector(parser, "tr")
        .unwrap()
        .map(|row| -> Vec<NodeHandle> {
            row.get(parser).unwrap().as_tag().unwrap().query_selector(parser, "td").unwrap().collect()
        })
        .collect();

    let indices = match headers.len() {
        5 => FieldTableIndices { status: 0, tag: 1, name: 2, name_fallback: 0, qualifier: 0, link: 4 },
        7 => FieldTableIndices { status: 0, tag: 1, name: 4, name_fallback: 3, qualifier: 2, link: 6 },
        _ => {
            println!("Could not determine format columns, header must have 5 or 7 columns but had {}", headers.len());
            return;
        }
    };

    for row in rows {
        if row.len() < 3 {
            continue;
        }

        let tag = row.get(indices.tag).unwrap().get(parser).unwrap().inner_text(parser).to_string();
        if tag.is_empty() {
            // there is one special row at the end of most messages just explaining the abbreviations in the table
            // the first field is empty in that row, otherwise there is always a tag
            continue;
        }

        let mut maybe_name = Some(
            row.get(indices.name).unwrap().get(parser).unwrap().inner_text(parser).to_string()
        ).filter(|s| !s.is_empty() && !s.contains("see qualifier description"));

        if maybe_name.is_none() && indices.name_fallback > 0 {
            maybe_name = maybe_name.or_else(|| Some(row.get(indices.name_fallback).unwrap().get(parser).unwrap().inner_text(parser).to_string()));
        }

        let name = maybe_name.unwrap_or(tag.clone());
        let link = row.get(indices.link)
            .unwrap()
            .get(parser)
            .unwrap()
            .as_tag()
            .unwrap()
            .query_selector(parser, "a")
            .unwrap()
            .next()
            .unwrap()
            .get(parser)
            .unwrap()
            .as_tag()
            .unwrap()
            .attributes()
            .get("href")
            .unwrap()
            .unwrap()
            .as_utf8_str()
            .to_string();

        process_field_definition(&link, &downloader, &config);
    }
}

fn process_field_definition(link: &str, downloader: &CachedHttpLoader, config: &SrConfig) {
    let url = format!("{}/{}", config.base_url, link);
    let html = downloader.download_string(&url).unwrap();
    let doc = tl::parse(&html, tl::ParserOptions::default()).unwrap();
    let parser = doc.parser();
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
            &ele.base_url,
            &format!("{}{}", ele.base_url, ele.index_topic),
            &downloader,
        )
            .iter()
            .for_each(|e| process_definition(&e, &downloader, cfg));
    }
}
