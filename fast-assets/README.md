# fast-assets

Easy to use assets manager for Rust

## ⚠️ Warning ⚠️

This library is still under development, shared to get feedbacks.
If you found a missing feature [open an issue on github](https://github.com/eVisualUser/fast-assets/issues).
All unchecked features are planned.

## Features

- [X] Search for files
- [X] Index files
- [X] Load assets
- [X] Load compressed assets
- [ ] Write files (not compressed only)
- [X] Dependencie Checker

## Compression Support

- [x] ZIP (.zip)

## Load pre-defined index

Used to load files compressed

- [X] From CSV file

### Example

```rust
let mut index = fast_assets::index::Index::new("./", "\\w+\\.rs");
index.add_from_file("index/index.csv");
index.set_csv_separator('/');
```

```csv
folder/subfolder/file.txt
archives/archive.zip/file.txt
```

## Getting Started

### Initialization

```rust
// Search all un-compressed files and archives,
// using extern index allow to add files compressed,
// the manager will automatically manage the decompression
let mut index = fast_assets::index::Index::new("./", "\\w+\\.rs");
index.search();
index.add_from_file("index/index.csv");

// The decompression cache is what will manage your compressed files and their caches.
let dc = fast_assets::decompression_cache::DecompressionCache::default();

let mut manager = fast_assets::manager::AssetsManager::new(index, dc);
```

### Loading/Unloading a file in memory

```rust
// Load a compressed file
manager.load("index.json").unwrap();
// Load a not compressed file
manager.load("text.csv").unwrap();

// Load a not compressed file using full path
manager.load("fr\\text.csv").unwrap();
// Load a compressed file using full path
manager.load("lang.zip/fr/text.csv").unwrap();
```

```rust
// AssetsManager::unload() need two parameters:
// decompression_cache: bool => If the file was compressed and if true it will put the file in the cache.
// filename: &str => The file that will be unloaded

// UnLoad and put in cache a compressed file
manager.unload("index.json", true);
// UnLoad a not compressed file
manager.unload("text.csv", false);

// Tips: if you want always keep in cache, set always at true,
// it will have no impact on the unloading of a uncompressed file
```

### Accessing Data

You must know that two thing can failed:

- File not indexed
- File not exists

So you need to passthrough them to access the data:

```rust
manager.get("text.csv").unwrap().unwrap();
manager.get_ref("text.csv").unwrap().unwrap();
manager.get_mut("text.csv").unwrap().unwrap();

// In the case where you have multiple file with the same name:
// When not compressed
manager.get("en\\text.csv").unwrap();
manager.get_ref("fr\\text.csv").unwrap();
manager.get_mut("it\\text.csv").unwrap();
// When compressed (relative to the compressed directory)
manager.get("en/text.csv").unwrap();
manager.get_ref("fr/text.csv").unwrap();
manager.get_mut("it/text.csv").unwrap();
```

If the file was put in cache it will automatically reload it.

### Saving Data

⚠️ This features encounters io authorizations issues ⚠️

Only saves data of file that are not from compressed files.
Return a simple std::io::Result<()> as result.
If the file does not exist anymore it will create a new file.

```rust
manager.save("text.csv").unwrap();
```

### Remove a file reference

In the manager and DecompressionCache each file loaded will have his reference,
but them use memory, so to do a complete unload you need to remove them.
For the cached files, they will have their cache file removed (based on the trait Drop).

```rust
manager.remove("text.csv").unwrap().unwrap();

// Tips: You don't need to unload before excepted if you need to put in cache
```

### Dependencie Checker

The dependency checker (DependencieManager), it's job is to search the not indexed files in the manager.

#### JSON File source

As you can see in the example below, the JSON file define the dependencies for some files.
The organization of the JSON file is not recursive, so you cannot define dependencies of a file into the dependencies of another file.

```json
{
    "text.csv": [
        "index.json",
        "other.csv"
    ],
    "index.json": [
        "text.csv"
    ]
}
```

#### Initialize the dependencies

```rust
// Create DependencieManager object
let mut dependencie_manager = DependencieManager::default();
// Load a file containing the dependencies required
dependencie_manager.load_file(&mut manager, "deps.json");
```

#### Let's check the dependencies

Now you have to load the dependencies, and to check them you need to call three commands:

- update
- check_if_valid
- get_missing_dependencies

```rust

// Take a look in the manager to get if a dependency is missing
dependencie_manager.update(&mut manager);
// Return true if all dependencies are present
dependencie_manager.check_if_valid("text.csv");
// Get all the missing dependencies
dependencie_manager.get_missing_dependencies("text.csv");
```
