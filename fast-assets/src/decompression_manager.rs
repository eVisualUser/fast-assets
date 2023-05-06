use std::{
    io::{Read, Write},
    path::PathBuf,
};

use crate::extension::Extension;

#[derive(Debug, Default)]
pub struct CachedFile {
    pub cache: Option<PathBuf>,
    pub origin: PathBuf,
    pub data: Option<Vec<u8>>,
}

impl Drop for CachedFile {
    fn drop(&mut self) {
        match &self.cache {
            Some(cache) => {
                match std::fs::remove_file(cache) {
                    _ => (),
                };
            }
            None => (),
        }
    }
}

impl CachedFile {
    pub fn set_cache_if_missing(&mut self, cache: &str) {
        match &self.cache {
            Some(_) => (),
            None => {
                let mut path = PathBuf::from(cache);
                while path.exists() {
                    path.set_file_name(&format!(
                        "_{}",
                        path.file_name().unwrap().to_string_lossy().to_string()
                    ));
                }
                self.cache = Some(path);
            }
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.data.is_some()
    }

    pub fn cache(&mut self) {
        match &self.cache {
            Some(path) => {
                let mut file = std::fs::File::options()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)
                    .unwrap();
                file.write_all(&self.data.clone().unwrap()).unwrap();
                self.data = None;
            }
            None => panic!("Cache path undefined"),
        }
    }

    pub fn load_cache(&mut self) {
        match &self.cache {
            Some(path) => {
                let mut file = std::fs::File::options().read(true).open(path).unwrap();
                let mut buffer = Vec::<u8>::new();
                file.read_to_end(&mut buffer).unwrap();
                self.data = Some(buffer);
            }
            None => panic!("Cache path undefined"),
        }
    }
}

#[derive(Debug, Default)]
pub struct DecompressionManager {
    cache_location: PathBuf,
    files: Vec<CachedFile>,
}

impl DecompressionManager {
    pub fn set_cache_location(&mut self, cache_location: &str) {
        self.cache_location = PathBuf::from(cache_location);
    }

    pub fn cache(&mut self, filename: &str) {
        let cache_location = self.cache_location.to_string_lossy().to_string();
        match self.get_mut(filename) {
            Some(file) => {
                file.set_cache_if_missing(&format!("{}/{}", cache_location, filename));
                file.cache();
            }
            None => (),
        }
    }

    pub fn get(&self, filename: &str) -> Option<&CachedFile> {
        for file in self.files.iter() {
            if file.origin.to_string_lossy() == filename {
                return Some(file);
            }
        }

        None
    }

    pub fn get_mut(&mut self, filename: &str) -> Option<&mut CachedFile> {
        for file in self.files.iter_mut() {
            if file.origin.to_string_lossy() == filename {
                return Some(file);
            }
        }

        None
    }

    /// Return a copy of the data and auto manage the cache
    pub fn get_data(&mut self, filename: &str) -> Option<Vec<u8>> {
        for file in self.files.iter_mut() {
            if file.origin.to_string_lossy() == filename {
                if file.data.is_none() {
                    if file.cache.is_some() {
                        file.load_cache();
                    }
                }
                return file.data.clone();
            }
        }
        None
    }

    /// Return a ref of the data and auto manage the cache
    pub fn get_data_ref(&mut self, filename: &str) -> Option<&Option<Vec<u8>>> {
        for file in self.files.iter_mut() {
            if file.origin.to_string_lossy() == filename {
                if file.data.is_none() {
                    if file.cache.is_some() {
                        file.load_cache();
                    }
                }
                return Some(&file.data);
            }
        }
        None
    }

    /// Return a &mut of the data and auto manage the cache
    pub fn get_data_mut(&mut self, filename: &str) -> Option<&mut Option<Vec<u8>>> {
        for file in self.files.iter_mut() {
            if file.origin.to_string_lossy() == filename {
                if file.data.is_none() {
                    if file.cache.is_some() {
                        file.load_cache();
                    }
                }
                return Some(&mut file.data);
            }
        }
        None
    }

    pub fn unload(&mut self, filename: &str) {
        for file in self.files.iter_mut() {
            if file.origin.to_string_lossy() == filename {
                file.data = None;
            }
        }
    }

    pub fn remove(&mut self, filename: &str) {
        for i in 0..self.files.len() {
            if self.files[i].origin.to_string_lossy() == filename {
                self.files.remove(i);
            }
        }
    }

    pub fn load_archive(
        &mut self,
        archive: &str,
        selection: Option<Vec<&str>>,
        process_pass_list: &mut Vec<Box<dyn Extension>>,
    ) {
        let path = PathBuf::from(archive);

        match path
            .extension()
            .unwrap()
            .to_string_lossy()
            .to_string()
            .as_str()
        {
            "zip" => {
                let archive = std::fs::File::open(&path).unwrap();
                let mut archive = zip::ZipArchive::new(archive).unwrap();
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    if file.is_file() {
                        match selection {
                            Some(ref selection) => {
                                for selected in selection.iter() {
                                    let file_name =
                                        file.enclosed_name().unwrap().to_string_lossy().to_string();
                                    if file_name == *selected {
                                        let mut cached_file = CachedFile::default();
                                        cached_file.origin = PathBuf::from(file_name);
                                        let mut data = Vec::<u8>::new();
                                        file.read_to_end(&mut data).unwrap();
                                        cached_file.data = Some(data);
                                        self.files.push(cached_file);
                                    }
                                }
                            }
                            None => {
                                let file_name =
                                    file.enclosed_name().unwrap().to_string_lossy().to_string();
                                let mut cached_file = CachedFile::default();
                                cached_file.origin = PathBuf::from(file_name);
                                let mut data = Vec::<u8>::new();
                                file.read_to_end(&mut data).unwrap();
                                cached_file.data = Some(data);
                                self.files.push(cached_file);
                            }
                        }
                    }
                }
            }
            extension => {
                for i in 0..process_pass_list.len() {
                    let mut process_pass = process_pass_list.swap_remove(i);
                    process_pass.on_archive(self, extension, &path);
                    process_pass_list.insert(i, process_pass);
                }
            }
        }
    }
}
