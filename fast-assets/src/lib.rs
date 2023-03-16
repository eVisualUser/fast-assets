pub mod decompression_manager;
pub mod dependencie_manager;
pub mod index;
pub mod manager;
pub mod process_pass;

#[cfg(test)]
mod test {
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
}
