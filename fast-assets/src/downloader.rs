use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::ops::{Deref, DerefMut};
use std::thread::JoinHandle;
use curl::easy::Easy;

#[derive(Default)]
pub struct Downloader {

}

impl Downloader {
    pub fn download(&self, url: String, output: String) -> JoinHandle<()> {
        std::thread::spawn(move ||{
            let mut easy = Easy::new();
            match easy.url(&url) {
                Err(err) => {
                    eprintln!("ERROR Downloader(Curl): {:?}", err);
                }
                _ => (),
            }

            let file = std::fs::File::options().write(true).truncate(true).create(true).open(&output);
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
        })
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
        }
    }
}
