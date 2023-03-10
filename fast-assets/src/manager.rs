use crate::index::Index;
use crate::decompression_cache::{DecompressionCache};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct File {
    pub from_archive: bool,
    pub path: PathBuf,
    pub data: Option<Vec<u8>>,
}

impl File {
    pub fn load(&mut self) -> std::io::Result<()> {
        if self.path.exists() {
            let mut buffer = Vec::<u8>::new();
            let mut file = std::fs::File::options().read(true).open(self.path.clone())?;
            file.read_to_end(&mut buffer)?;
            file.flush()?;
            self.data = Some(buffer);
        }
        Ok(())
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        println!("{:?}", self.path);
        let mut file = std::fs::File::options().write(true).truncate(true).create(true).open(self.path.clone())?;
        println!("File Opened");
        match &self.data {
            Some(data) => {
                println!("Try write");
                file.write_all(data.as_slice())?;
                println!("Success Write");
                file.flush()?;
                println!("Success flush");
            }
            None => (),
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct AssetsManager {
    index: Index,
    cache: DecompressionCache,
    files: Vec<File>,
}

impl AssetsManager {
    pub fn new(index: Index, cache: DecompressionCache) -> Self {
        Self {
            index,
            cache,
            files: Vec::new(),
        }
    }

    pub fn load(&mut self, base_path: &str) -> std::io::Result<()> {
        let mut path = Option::<String>::None;
        if !(base_path.contains('\\') || base_path.contains('/')) {
            path = self.index.get_path(base_path);
        } else {
            path = Some(String::from(base_path));
        }
        match path {
            Some(path) => {
                let path = PathBuf::from(path);
                
                let mut in_archive: Option<String> = None;
                let mut path_until_archive = Vec::<String>::new();
                let mut path_in_archive = Vec::<String>::new();
                for i in path.components() {
                    let cmp = i.as_os_str().to_string_lossy();

                    if in_archive.is_none() {
                        path_until_archive.push(cmp.to_string());
                    } else {
                        path_in_archive.push(cmp.to_string());
                    }

                    if cmp.ends_with(".zip") {
                        in_archive = Some(i.as_os_str().to_string_lossy().to_string());
                    }
                }

                match in_archive {
                    Some(_) => {
                        let mut file = File::default();

                        let mut archive = String::new();
                        path_until_archive.iter().for_each(|elem|{
                            archive.push_str(&format!("{}/", elem));
                        });
                        archive.pop();
                        file.from_archive = true;

                        let mut path = String::new();
                        path_in_archive.iter().for_each(|elem|{
                            path.push_str(&format!("{}/", elem));
                        });
                        path.pop();
                        file.path = PathBuf::from(path.clone());

                        self.cache.load_archive(&archive, Some(vec![&path]));

                        file.from_archive = true;
                        file.path = PathBuf::from(path);
                        self.files.push(file);
                    }
                    None => {
                        let mut file = File::default();
                        file.path = path;
                        file.load()?;
                        self.files.push(file);
                    }
                }
            }
            None => {
                
            }
        }

        Ok(())
    }

    pub fn unload(&mut self, path: &str, cache_decompressed: bool) {
        for file in self.files.iter_mut() {
            let file_path = file.path.to_string_lossy();
            if path == file_path {
                file.data = None;
                if file.from_archive {
                    if cache_decompressed {
                        self.cache.cache(&file_path);
                    } else {
                        self.cache.unload(&file_path);
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, path: &str) {
        for i in 0..self.files.len() {
            if i < self.files.len() {
            let file_path = self.files[i].path.to_string_lossy();
            if path == file_path {
                if self.files[i].from_archive {
                    self.cache.remove(path);
                }
                self.files.remove(i);
            }
        }
        }
    }

    pub fn find_file_index(&self, filename: &str) -> Option<usize> {
        for i in 0..self.files.len() {
            if self.files[i].path.file_name().unwrap().to_string_lossy() == filename {
                return Some(i);
            }
        }

        None
    }

    pub fn find_file_index_using_full_path(&self, path: &str) -> Option<usize> {
        for i in 0..self.files.len() {
            if self.files[i].path.to_string_lossy() == path {
                return Some(i);
            }
        }

        None
    }

    pub fn get(&mut self, path: &str) -> Option<Vec<u8>> {
        let in_cache = self.cache.get_data(path);
        match in_cache {
            Some(_) => return in_cache,
            None => {
                let mut index = Option::<usize>::None;
                if path.contains('\\') || path.contains('/') {
                    index = self.find_file_index_using_full_path(path);
                } else {
                    index = self.find_file_index(path);
                }
                if index.is_none() {
                    return None;
                }
                return self.files[index.unwrap()].data.clone();
            }
        }
    }

    pub fn get_ref(&mut self, path: &str) -> Option<&Option<Vec<u8>>> {
        let is_full_path = path.contains('\\') || path.contains('/'); 
        let in_cache = self.cache.get_data_ref(path);
        match in_cache {
            Some(_) => return in_cache,
            None => {
                for file in self.files.iter_mut() {
                    if is_full_path && file.path.to_string_lossy() == path {
                            return Some(&file.data);
                    } else if file.path.file_name().unwrap().to_string_lossy() == path {
                        return Some(&file.data);
                    }
                }
            }
        }
        None
    }

    pub fn get_mut(&mut self, path: &str) -> Option<&mut Option<Vec<u8>>> {
        let is_full_path = path.contains('\\') || path.contains('/'); 
        let in_cache = self.cache.get_data_mut(path);
        match in_cache {
            Some(_) => return in_cache,
            None => {
                for file in self.files.iter_mut() {
                if is_full_path && file.path.to_string_lossy() == path {
                    return Some(&mut file.data);
                } else if file.path.file_name().unwrap().to_string_lossy() == path {
                    return Some(&mut file.data);
                }
                }
            }
        }
        None
    }

    pub fn have_file(&self, filename: &str) -> bool {
        for file in self.files.iter() {
            if file.path.file_name().unwrap().to_string_lossy() == filename {
                return true;
            }
        }

        return self.index.have_file(filename);
    }

    pub fn save(&mut self, filename: &str) -> std::io::Result<()> {
        for file in self.files.iter_mut() {
            if file.path.file_name().unwrap().to_string_lossy() == filename {
                return file.save();
            }
        }

        Ok(())
    }
}
