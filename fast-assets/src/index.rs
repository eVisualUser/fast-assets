use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Index {
    pub root: PathBuf,
    pub files: Vec<PathBuf>,
    pub filter: Regex,
    current_pos: usize,
    csv_separator: char,
    redirect_list: HashMap<PathBuf, PathBuf>,
}

impl Index {
    pub fn new(root: &str, filter: &str) -> Self {
        Self {
            root: std::path::PathBuf::from(root),
            files: Vec::new(),
            filter: Regex::new(filter).unwrap(),
            current_pos: 0,
            csv_separator: ';',
            redirect_list: HashMap::new(),
        }
    }

    pub fn save_as_file(&self, filename: &str) -> std::io::Result<()> {
        let mut output = std::fs::File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(filename)?;

        for file in self.files.iter() {
            let mut line = String::new();
            for element in file.iter() {
                if element != "." {
                    let element = element.to_string_lossy();
                    line.push_str(&element);
                    line.push(self.csv_separator);
                }
            }
            line.pop();
            line.push('\n');
            output.write(line.as_bytes())?;
        }

        output.flush().unwrap();
        Ok(())
    }

    pub fn add_redirect(&mut self, origin: &str, target: &str) {
        self.redirect_list
            .insert(PathBuf::from(origin), PathBuf::from(target));
    }

    pub fn add_redirect_from_file(&mut self, file: &str) {
        let content = json::parse(&std::fs::read_to_string(&file).unwrap()).unwrap();

        content.entries().for_each(|entry| {
            if entry.0 == "redirect" {
                for redirect in entry.1.entries() {
                    let origin = self.get_path(redirect.0).expect("Missing file in index");
                    let target = self
                        .get_path(redirect.1.as_str().unwrap())
                        .expect("Missing file in index");

                    self.add_redirect(origin.as_str(), target.as_str());
                }
            }
        });
    }

    pub fn remove_redirect(&mut self, origin: &str) {
        self.redirect_list.remove(&PathBuf::from(origin));
    }

    pub fn set_csv_separator(&mut self, new_separator: char) {
        self.csv_separator = new_separator;
    }

    pub fn search(&mut self) {
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

    pub fn search_in_dir(&self, path: PathBuf) -> Vec<PathBuf> {
        let read_dir = path.read_dir().unwrap();

        let result = Arc::new(Mutex::new(Vec::<PathBuf>::new()));

        read_dir.par_bridge().for_each(|item| {
            let mut result = result.lock().unwrap();
            let item = item.unwrap();
            if item.path().is_file() {
                if self.filter.is_match(
                    &item
                        .path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                ) {
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
            Some(ext) => match ext.to_string_lossy().to_string().as_str() {
                "csv" => {
                    let mut csv = pro_csv::CSV::default();
                    csv.set_sperator_char(self.csv_separator);
                    csv.load_from_file(&file.to_string_lossy().to_string());
                    let buffer = Arc::new(Mutex::new(Vec::<PathBuf>::new()));
                    csv.par_bridge().for_each(|mut line| {
                        let mut buffer = buffer.lock().unwrap();
                        let mut path = String::new();
                        for element in 0..line.len() - 1 {
                            path.push_str(&format!("{}/", line[element]));
                        }
                        line.last_mut().unwrap().pop();
                        path.push_str(&line.last().unwrap());
                        buffer.push(PathBuf::from(path));
                    });
                    let mut buffer = buffer.lock().unwrap();
                    self.files.append(&mut buffer);
                }
                _ => (),
            },
            None => (),
        }
    }

    pub fn get_redirect(&self, filename: &str) -> Option<String> {
        for redirect in self.redirect_list.iter() {
            if redirect.0.to_string_lossy() == filename
                || redirect.0.file_name().unwrap().to_string_lossy() == filename
            {
                return Some(redirect.1.to_string_lossy().to_string());
            }
        }

        None
    }

    pub fn get_path(&self, filename: &str) -> Option<String> {
        let redirect = self.get_redirect(filename);
        match redirect {
            Some(path) => return self.get_path(path.to_string().as_str()),
            _ => (),
        }

        let result = Arc::new(Mutex::new(Option::<String>::None));

        self.files.par_iter().for_each(|path| {
            let mut result = result.lock().unwrap();
            println!("paht cmp: {}/{}", path.to_string_lossy(), filename);
            if path.to_string_lossy() == filename
                || path.file_name().unwrap().to_string_lossy() == filename
            {
                *result = Some(path.to_string_lossy().to_string());
            }
        });

        let result = result.lock().unwrap();
        result.clone()
    }

    pub fn have_file(&self, filename: &str) -> bool {
        let using_full_path = filename.contains("\\") || filename.contains("/");
        for file in self.files.iter() {
            if !using_full_path && file.file_name().unwrap().to_string_lossy() == filename {
                return true;
            } else if using_full_path && file.to_string_lossy() == filename {
                return true;
            }
        }
        false
    }

    /// Remove the index, and return if it was found
    pub fn remove_indexed_file(&mut self, filename: &str) -> bool {
        let path = self.get_path(filename);
        return match path {
            Some(path) => {
                for i in 0..self.files.len() {
                    if self.files[i].to_string_lossy() == path {
                        self.files.remove(i);
                        return true;
                    }
                }
                false
            }
            None => false,
        };
    }

    pub fn regex_search(&self, filter: &str) -> Vec<PathBuf> {
        let mut result = Vec::<PathBuf>::new();
        let regex = regex::Regex::new(filter).unwrap();
        for file in self.files.iter() {
            if regex.is_match(&file.to_string_lossy()) {
                result.push(file.clone());
            }
        }
        result
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
