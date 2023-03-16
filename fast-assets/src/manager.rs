use crate::decompression_manager::DecompressionManager;
use crate::index::Index;
use crate::process_pass::ProcessPass;
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
            let mut file = std::fs::File::options()
                .read(true)
                .open(self.path.clone())?;
            file.read_to_end(&mut buffer)?;
            file.flush()?;
            self.data = Some(buffer);
            return Ok(());
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File \"{:?}\" not found", self.path),
        ))
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        let mut file = std::fs::File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(self.path.clone())?;
        match &self.data {
            Some(data) => {
                file.write_all(data.as_slice())?;
                file.flush()?;
            }
            None => (),
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct AssetsManager {
    pub index: Index,
    pub cache: DecompressionManager,
    files: Vec<File>,
    pub process_pass_list: Vec<Box<dyn ProcessPass>>,
    compression_formats: Vec<String>,
}

impl AssetsManager {
    pub fn new(index: Index, cache: DecompressionManager) -> Self {
        Self {
            index,
            cache,
            files: Vec::new(),
            process_pass_list: Vec::new(),
            compression_formats: vec![String::from("zip")],
        }
    }

    pub fn move_file(&mut self, origin: &str, target: &str) -> std::io::Result<()> {
        let origin = self.index.get_path(origin).unwrap();
        std::fs::copy(origin.clone(), target.clone())?;
        std::fs::remove_file(origin.clone())?;

        self.index.remove_indexed_file(&origin);
        self.index.add_file(PathBuf::from(target));

        Ok(())
    }

    pub fn remove_file(&mut self, origin: &str) -> std::io::Result<()> {
        let origin = self.index.get_path(origin).unwrap();
        std::fs::remove_file(origin.clone())?;
        self.index.remove_indexed_file(&origin);

        Ok(())
    }

    pub fn copy_file(&mut self, origin: &str, target: &str) -> std::io::Result<()> {
        let origin = self.index.get_path(origin).unwrap();
        std::fs::copy(origin.clone(), target.clone())?;

        self.index.add_file(PathBuf::from(target));

        Ok(())
    }

    pub fn create_file(&mut self, path: &str) -> std::io::Result<()> {
        std::fs::File::options().create(true).write(true).open(path)?;
        self.index.add_file(PathBuf::from(path));

        Ok(())
    }

    pub fn remove_process_pass(&mut self, name: &str) {
        for i in 0..self.process_pass_list.len() {
            if self.process_pass_list[i].get_name() == name {
                self.process_pass_list.remove(i);
            }
        }
    }

    pub fn add_compression_formats(&mut self, format: &str) {
        self.compression_formats.push(String::from(format));
    }

    pub fn add_process_pass(&mut self, process_pass: Box<dyn ProcessPass>) {
        self.process_pass_list.push(process_pass);
    }

    pub fn load(&mut self, base_path: &str) -> std::io::Result<()> {
        let mut path;
        if !(base_path.contains('\\') || base_path.contains('/')) {
            path = self.index.get_path(base_path);
        } else {
            path = Some(String::from(base_path));
        }

        for i in 0..self.process_pass_list.len() {
            let mut process_pass = self.process_pass_list.swap_remove(i);
            if !process_pass.on_load(self, &mut path) {
                return Ok(());
            }
            self.process_pass_list.insert(i, process_pass);
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

                    for fmt in self.compression_formats.iter() {
                        if cmp.ends_with(&format!(".{}", fmt)) {
                            in_archive = Some(i.as_os_str().to_string_lossy().to_string());
                        }
                    }
                }

                match in_archive {
                    Some(_) => {
                        let mut file = File::default();

                        let mut archive = String::new();
                        path_until_archive.iter().for_each(|elem| {
                            archive.push_str(&format!("{}/", elem));
                        });
                        archive.pop();
                        file.from_archive = true;

                        let mut path = String::new();
                        path_in_archive.iter().for_each(|elem| {
                            path.push_str(&format!("{}/", elem));
                        });
                        path.pop();
                        file.path = PathBuf::from(path.clone());

                        self.cache.load_archive(
                            &archive,
                            Some(vec![&path]),
                            &mut self.process_pass_list,
                        );

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
            None => {}
        }

        Ok(())
    }

    pub fn unload(&mut self, mut path: &str, mut cache_decompressed: bool) {
        for i in 0..self.process_pass_list.len() {
            let mut process_pass = self.process_pass_list.swap_remove(i);
            if !process_pass.on_unload(self, &mut path, &mut cache_decompressed) {
                return;
            }
            self.process_pass_list.insert(i, process_pass);
        }

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

    pub fn remove(&mut self, mut path: &str) {
        for i in 0..self.process_pass_list.len() {
            let mut process_pass = self.process_pass_list.swap_remove(i);
            if !process_pass.on_remove(self, &mut path) {
                return;
            }
            self.process_pass_list.insert(i, process_pass);
        }

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
        return match in_cache {
            Some(_) => in_cache,
            None => {
                let path = self.index.get_path(path).unwrap();
                let index;
                if path.contains('\\') || path.contains('/') {
                    index = self.find_file_index_using_full_path(path.as_str());
                } else {
                    index = self.find_file_index(path.as_str());
                }
                if index.is_none() {
                    return None;
                }
                self.files[index.unwrap()].data.clone()
            }
        }
    }

    pub fn get_ref(&mut self, path: &str) -> Option<&Option<Vec<u8>>> {
        let path_buf = self.index.get_path(path).unwrap();
        let is_full_path = path_buf.contains('\\') || path_buf.contains('/');
        let in_cache = self.cache.get_data_ref(path);
        match in_cache {
            Some(_) => return in_cache,
            None => {
                for file in self.files.iter() {
                    if is_full_path && file.path.to_string_lossy() == path_buf {
                        return Some(&file.data);
                    }
                }
            }
        }
        None
    }

    pub fn get_mut(&mut self, path: &str) -> Option<&mut Option<Vec<u8>>> {
        let path_buf = self.index.get_path(path).unwrap();
        let is_full_path = path_buf.contains('\\') || path_buf.contains('/');
        let in_cache = self.cache.get_data_mut(path);
        match in_cache {
            Some(_) => return in_cache,
            None => {
                for file in self.files.iter_mut() {
                    if is_full_path && file.path.to_string_lossy() == path_buf {
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

    pub fn get_files_matching_regex(&self, regex: &str) -> Vec<PathBuf> {
        self.index.regex_search(regex)
    }
}
