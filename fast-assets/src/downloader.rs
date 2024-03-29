use curl::easy::Easy;
use std::io::Write;

/// It have all features to support web download using Curl
#[derive(Default, Debug)]
pub struct Downloader {}

impl Downloader {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn download(&self, url: String, output: String) {
        let mut easy = Easy::new();
        match easy.url(&url) {
            Err(err) => {
                eprintln!("ERROR Downloader(Curl): {:?}", err);
            }
            _ => (),
        }

        let file = std::fs::File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&output);
        let mut file = match file {
            Err(err) => {
                panic!("ERROR Downloader(FS): {:?}", err);
            }
            Ok(ok) => ok,
        };

        let mut transfer = easy.transfer();
        match transfer.write_function(move |data| {
            file.write_all(data).unwrap();
            file.flush().unwrap();
            file.sync_all().unwrap();
            Ok(data.len())
        }) {
            Err(err) => {
                eprintln!("ERROR Downloader(Curl): {:?}", err);
            }
            _ => (),
        }

        match transfer.perform() {
            Err(err) => {
                eprintln!("ERROR Downloader(Curl): {:?}", err);
            }
            _ => (),
        }
    }

    pub fn download_sync(&self, url: String, output: String) {
        pollster::block_on(self.download(url, output));
    }

    pub fn can_download(&self, target: &str) -> bool {
        let mut handle = Easy::new();
        match handle.url(target) {
            Ok(_) => (),
            _ => return false,
        }
        return match handle.perform() {
            Ok(_) => true,
            _ => false,
        };
    }
}
