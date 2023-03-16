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
            easy.url(&url).unwrap();

            let mut file = std::fs::File::options().write(true).truncate(true).create(true).open(&output).unwrap();

            let mut transfer = easy.transfer();
            transfer.write_function(move |data| {
                file.write_all(data).unwrap();
                file.flush().unwrap();
                file.sync_all().unwrap();
                Ok(data.len())
            }).unwrap();
            transfer.perform().unwrap();
        })
    }
}
