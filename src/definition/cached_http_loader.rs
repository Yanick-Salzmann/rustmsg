pub struct CachedHttpLoader {
    sr: String,
    cache_folder: String,

    client: reqwest::blocking::Client,
}

impl CachedHttpLoader {
    pub fn new(cache_folder: &str, sr: &str) -> CachedHttpLoader {
        let ret = CachedHttpLoader {
            sr: sr.into(),
            cache_folder: cache_folder.into(),
            client: create_http_client(),
        };
        ret.create_cache_folder();
        return ret;
    }

    fn create_cache_folder(&self) {
        let cache_dir = format!("{}/{}", self.cache_folder, self.sr);
        std::fs::create_dir_all(cache_dir).expect("Unable toc reate cache directory");
    }

    pub fn download_string(&self, url: &str) -> Result<String, reqwest::Error> {
        match read_from_cache(&self.cache_folder, url) {
            Some(content) => return Ok(content),
            None => return Ok(save_to_cache(&self.cache_folder, url, &self.client.get(url).send()?.text()?))
        }
    }
}

fn create_http_client() -> reqwest::blocking::Client {
    return reqwest::blocking::ClientBuilder::new()
        .cookie_store(true)
        .build()
        .expect("Unable to create reqwest client");
}

fn read_from_cache(folder: &str, url: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(url).unwrap();
    let path = parsed.path();
    let cache_location = format!("{}/{}", folder, path);
    return std::fs::read_to_string(cache_location).ok();
}

fn save_to_cache(folder: &str, url: &str, content: &str) -> String {
    let parsed = reqwest::Url::parse(url).unwrap();
    let path = parsed.path();
    let cache_location = format!("{}/{}", folder, path);
    let base_folder = std::path::Path::new(&cache_location).parent().unwrap();
    std::fs::create_dir_all(base_folder).unwrap();
    std::fs::write(cache_location, content).unwrap();
    return content.into();
}