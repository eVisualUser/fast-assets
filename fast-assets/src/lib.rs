pub mod decompression_manager;
pub mod dependencie_manager;
pub mod index;
pub mod manager;
pub mod process_pass;
pub mod downloader;

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    pub fn get_cargo_toml_path_index() {
        let mut index = crate::index::Index::new("./", "Cargo.toml");
        index.search();
        assert_eq!(index.get_path("./Cargo.toml"), Some(String::from("./Cargo.toml")));
    }

    #[test]
    pub fn get_cargo_toml_path_index_using_custom() {
        let mut index = crate::index::Index::new("./", "____________");
        index.set_csv_separator('/');
        index.search();
        index.add_from_file("test_resources/index.csv");
        assert_eq!(index.get_path("Cargo.toml"), Some(String::from("Cargo.toml")));
    }

    #[test]
    pub fn get_compressed_file_path() {
        let mut index = crate::index::Index::new("./", "____________");
        index.set_csv_separator('/');
        index.search();
        index.add_from_file("test_resources/index.csv");
        assert_eq!(index.get_path("index.json"), Some(String::from("index/index.zip/index.json")));
    }

    #[test]
    pub fn get_data() {
        let mut index = crate::index::Index::new("./", "____________");
        index.set_csv_separator('/');
        index.search();
        index.add_from_file("test_resources/index.csv");

        let dc = crate::decompression_manager::DecompressionManager::default();

        let mut manager = crate::manager::AssetsManager::new(index, dc);
        manager.load("index.csv").unwrap();
        assert_ne!(manager.get("index.csv"), None);

        manager.load("index.csv").unwrap();
        assert_ne!(manager.get_ref("index.csv"), None);
        assert_ne!(manager.get_ref("index.csv").unwrap().clone(), None);

        manager.load("index.csv").unwrap();
        assert_ne!(manager.get_mut("index.csv"), None);
        assert_ne!(manager.get_mut("index.csv").unwrap().clone(), None);
    }

    #[test]
    pub fn dependencie_manager() {
        let mut index = crate::index::Index::new("./", "____________");
        index.set_csv_separator('/');
        index.search();
        index.add_from_file("test_resources/index.csv");

        let dc = crate::decompression_manager::DecompressionManager::default();

        let mut manager = crate::manager::AssetsManager::new(index, dc);

        let mut deps = crate::dependencie_manager::DependencieManager::default();
        deps.load_file(&mut manager, "test_resources/deps.json");
        deps.update(&mut manager);

        assert_eq!(deps.get_missing_dependencies("index.csv").is_empty(), false);
    }

    #[test]
    pub fn redirect() {
        let mut index = crate::index::Index::new("./", "____________");
        index.set_csv_separator('/');
        index.search();
        index.add_from_file("test_resources/index.csv");
        index.add_redirect_from_file("test_resources/redirect.json");
        assert_eq!(index.get_path("Cargo.toml"), Some(String::from("other.toml")));
    }

    #[test]
    pub fn get_compressed() {
        let mut index = crate::index::Index::new("./", "____________");
        index.set_csv_separator('/');
        index.search();
        index.add_from_file("test_resources/index.csv");

        let dc = crate::decompression_manager::DecompressionManager::default();

        let mut manager = crate::manager::AssetsManager::new(index, dc);
        manager.load("localization.csv").unwrap();
        assert_ne!(manager.get("localization.csv"), None);

        assert_ne!(manager.get_ref("localization.csv"), None);
        assert_ne!(manager.get_ref("localization.csv").unwrap().clone(), None);

        assert_ne!(manager.get_mut("localization.csv"), None);
        assert_ne!(manager.get_mut("localization.csv").unwrap().clone(), None);
    }

    #[test]
    pub fn saving() -> std::io::Result<()> {
        let mut index = crate::index::Index::new("./", "____________");
        index.set_csv_separator('/');
        index.search();
        index.add_from_file("test_resources/index.csv");

        let dc = crate::decompression_manager::DecompressionManager::default();

        let mut manager = crate::manager::AssetsManager::new(index, dc);
        manager.load("index.csv").unwrap();
        manager.save("index.csv")
    }

    #[test]
    pub fn create() -> std::io::Result<()> {
        let mut index = crate::index::Index::new("./", "____________");
        index.set_csv_separator('/');
        index.search();
        index.add_from_file("test_resources/index.csv");

        let dc = crate::decompression_manager::DecompressionManager::default();

        let mut manager = crate::manager::AssetsManager::new(index, dc);
        manager.create_file("myFile.txt")?;
        manager.load("myFile.txt")?;
        assert_ne!(manager.get("myFile.txt"), None);
        Ok(())
    }

    #[test]
    pub fn downloader() {
        let downloader = crate::downloader::Downloader::default();

        let link_a = String::from("https://crates.io/assets/cargo.png");
        let link_b = String::from("https://www.rust-lang.org/");
        let link_c = String::from("https://github.com/eVisualUser/bellecour-gamebook/blob/main/hello_world/hello_world.zip");

        let out_a = String::from("crates.png");
        let out_b = String::from("rust_lang.html");
        let out_c = String::from("HelloWorld.zip");

        let th_a = downloader.download(link_a, out_a.clone());
        let th_b = downloader.download(link_b, out_b.clone());
        let th_c = downloader.download(link_c, out_c.clone());
        th_a.join().unwrap();
        th_b.join().unwrap();
        th_c.join().unwrap();

        assert!(PathBuf::from(out_a).exists());
        assert!(PathBuf::from(out_b).exists());
        assert!(PathBuf::from(out_c).exists());
    }
}
