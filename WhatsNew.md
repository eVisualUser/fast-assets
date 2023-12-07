# What's New

## (WIP) Version 0.1.7

### Now you can use a link in the AssetManager::Get(...) to download a file instead using the downloader:

```rust
let mut asset_manager = crate::manager::AssetsManager::default();
asset_manager.load("https://www.rust-lang.org/").unwrap();
let downloaded_file = asset_manager.get("https://www.rust-lang.org/");

let content = match downloaded_file {
	None => String::from(""),
	Some(new_content) => String::from_utf8(new_content).unwrap(),
};
```

### AssetManager::default() is now available:

```rust
let asset_manager = manager::AssetsManager::default();
```
