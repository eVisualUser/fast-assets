use std::{path::PathBuf, borrow::{Borrow, BorrowMut}};
use regex::Regex;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Index {
    pub root: PathBuf,
    /// FileName + Path
    pub files: Vec<PathBuf>,
    pub filter: Regex,
    current_pos: usize,
}

impl Index {
    pub fn new(root: &str, filter: &str) -> Self {
        Self {
            root: std::path::PathBuf::from(root),
            files: Vec::new(),
            filter: Regex::new(filter).unwrap(),
            current_pos: 0,
        }
    }

    pub fn search(&mut self) {
        self.add_files(self.search_in_dir(self.root.to_path_buf()));
    }

    pub async fn async_search(&mut self) {
        self.add_files(self.search_in_dir(self.root.to_path_buf()));
    }

    pub fn add_file(&mut self, file: PathBuf) {
        self.files.push(file);
    }

    pub fn add_files(&mut self, files: Vec<PathBuf>) {
        self.files.extend(files);
    }

    pub fn clear(&mut self) {
        self.files.clear();
    }

    pub fn async_search_in_dir(&mut self, dir: &PathBuf) -> Vec<PathBuf> {
        self.search_in_dir(dir.to_path_buf())
    }

    pub fn search_in_dir(&self, path: PathBuf) -> Vec<PathBuf> {
        let read_dir = path.read_dir().unwrap();

        let result = Arc::new(Mutex::new(Vec::<PathBuf>::new()));

        read_dir.par_bridge().for_each(|item|{
            let mut result = result.lock().unwrap();
            let item = item.unwrap();
            if item.path().is_file() {
                if self.filter.is_match(&item.path().file_name().unwrap().to_string_lossy().to_string()) {
                    result.push(item.path());
                }
            } else if item.path().is_dir() {
                result.append(&mut self.search_in_dir(item.path()));
            }
        });

        let result = Arc::try_unwrap(result).unwrap().into_inner().unwrap();
        result
    }

    pub fn add_from_file(&mut self, file: &str) {
        let file = PathBuf::from(file);
        match file.extension() {
            Some(ext) => {
                match ext.to_string_lossy().to_string().as_str() {
                    "csv" => {
                        let mut csv = pro_csv::CSV::default();
                        csv.load_from_file(&file.to_string_lossy().to_string());
                        let buffer = Arc::new(Mutex::new(Vec::<PathBuf>::new()));
                        csv.par_bridge().for_each(|mut line|{
                            let mut buffer = buffer.lock().unwrap();
                            let mut path = String::new();
                            for element in 0..line.len()-1 {
                                path.push_str(&format!("{}\\", line[element]));
                            }
                            line.last_mut().unwrap().pop();
                            path.push_str(&line.last().unwrap());
                            buffer.push(PathBuf::from(path));
                        });
                        let mut buffer = buffer.lock().unwrap();
                        self.files.append(&mut buffer);
                    }
                    _ => (),
                }
            }
            None => (),
        }
    }

    pub fn get_path(&self, filename: &str) -> Option<String> {
        let mut result = Arc::new(Mutex::new(Option::<String>::None));

        self.files.par_iter().for_each(|path|{
            println!("Path: {:?}", path);
            let mut result = result.lock().unwrap();
            if path.file_name().unwrap().to_string_lossy() == filename {
                *result = Some(path.to_string_lossy().to_string());
            }
        });

        let result = result.lock().unwrap();
        result.clone()
    }
}

impl Iterator for Index {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos >= self.files.len() {
            return None;
        }
        let path = self.files[self.current_pos].clone();
        self.current_pos += 1;
        Some(path)
    }
}
