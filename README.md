# fast-assets

Easy to use assets manager, that can manage any kind of file by manipulating files as Vec<u8>,
it's made to fit in any software/game/framework.

## Features

- [X] Search for files
- [X] Index files
- [X] Load assets
- [X] Load compressed assets
- [X] Dependency Checker
- [X] Process-Pass
- [X] File Redirect
- [X] Write files (not compressed only)
- [X] Downloader (Download files from web)
- [X] Easy file move/copy/remove

## Compression Support

- [x] ZIP (.zip)

## Load pre-defined index

Used to load files compressed

- [X] From CSV file

### Example

```rust
let mut index = fast_assets::index::Index::new("./", "\\w+\\.rs");
index.set_csv_separator('/');
index.add_from_file("index/index.csv");
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
let dc = fast_assets::decompression_manager::DecompressionManager::default();

let mut manager = fast_assets::manager::AssetsManager::new(index, dc);

// Create a file and add it to the index
manage.create_file("myFile.text");
```

### Loading/Unloading a file in memory

#### Load

```rust
// Load a compressed file
manager.load("index.json").unwrap();
// Load a not compressed file
manager.load("text.csv").unwrap();

// Load a not compressed file using full path
manager.load("fr/text.csv").unwrap();
// Load a compressed file using full path
manager.load("lang.zip/fr/text.csv").unwrap();
```

#### UnLoad

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

#### Use Regex

```rust

// Get all the files that matching the regex into a Vec<Pathbuf>
let files = manager.get_files_matching_regex("\\w+\\.csv");

// Using it:
manager.get(&files[0].file_name().unwrap().to_string_lossy());

```

### Accessing Data

You must know that two things can fail:

- File not indexed
- File not exists

So you need to pass through them to access the data:

```rust
manager.get("text.csv").unwrap().unwrap();
manager.get_ref("text.csv").unwrap().unwrap();
manager.get_mut("text.csv").unwrap().unwrap();

// In the case where you have multiple file with the same name:
manager.get("en/text.csv").unwrap();
manager.get_ref("fr/text.csv").unwrap();
manager.get_mut("it/text.csv").unwrap();

// If you to set the data you can call:
manager.set_data("text.csv", b"Hello, World!".to_vec());
```

If the file was put in the cache and will automatically reload it.

### Saving Data

Only saves data of files that are not from compressed files.
Return a simple std::io::Result<()> as result.
If the file does not exist anymore it will create a new file.

```rust
manager.save("text.csv").unwrap();
```

### Remove a file reference

In the manager and DecompressionManager each file loaded will have its reference, but they use memory, so to do a complete unload you need to remove them.
For the cached files, they will have their cache file removed (based on the trait Drop).

```rust
manager.remove("text.csv").unwrap().unwrap();

// Tips: You don't need to unload before excepted if you need to put in cache
```

### Dependencies Checker

The dependency checker (DependencieManager), the goal is to search the not indexed files in the manager.

#### JSON File source

As you can see in the example below, the JSON file defines the dependencies for some files.
The organization of the JSON file is not recursive, so you cannot define the dependencies of a file into the dependencies of another file.

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

### ProcessPass

This is an easy way to add custom features.
It targets to let you add support for new compression formats...

#### Create a ProcessPass

A ProcessPass is a trait that adds the following functions:

```rust
/// Called when loading a file, and return true if continue the existing process
fn on_load(&mut self, _: &mut AssetsManager, path: &mut Option<String>) -> bool;

/// Called when unloading a file, and return true if continue the existing process
fn on_unload(&mut self, _: &mut AssetsManager, path: &mut &str, use_cache: &mut bool);

/// Called when remove a file reference, and return true if continue the existing process
fn on_remove(&mut self, _: &mut AssetsManager, path: &mut &str) -> bool;

/// Called when loading file/files from archive
fn on_archive(&mut self, _: &mut DecompressionManager, ext: &str, path: &str);
```

#### Add it to the AssetsManager

```rust
let my_process_pass = MyProcessPass::default();

manager.add_process_pass(Box::new(my_process_pass));
```

### Redirect System

Sometimes it's useful to specify a path and in background the assets manager use the good file path.
So it's implemented.

#### Add redirect

##### From simple command

```rust
index.add_redirect("base_path", "new_path");
```

##### From file

```json
{
  "redirect": {
    "base_path": "new_path"
  }
}
```

```rust
index.add_redirect_from_file("redirect.json");
```

### Downloader

```rust
// Create an instance of the downloader
let downloader = crate::downloader::Downloader::default();

// If you want prevent a download failure
downloader.can_download("https://crates.io/assets/cargo.png");
downloader.can_download("https://www.rust-lang.org/");
downloader.can_download("https://github.com/eVisualUser/bellecour-gamebook/blob/main/hello_world/hello_world.zip");

// Here using _sync method version to not have to handle the async.
// All errors produced will be output in the console.
downloader.download_sync(String::from("https://crates.io/assets/cargo.png"), String::from("crates.png"));
downloader.download_sync(String::from("https://www.rust-lang.org/"), String::from("rust_lang.html"));
downloader.download_sync(String::from("https://github.com/eVisualUser/bellecour-gamebook/blob/main/hello_world/hello_world.zip"), String::from("HelloWorld.zip"));
```

### Easy File move/copy/remove

There is few useful methods to control your file, and update the index as well.

#### Create a File

```rust
// Create the file and add it to the index
manager.create_file("myFile.txt");
```

#### Copy a file

```rust
// Copy the file to another location
manager.copy_file("myFile.txt", "folder/myFile.txt");
```

#### Move a file

```rust
// Copy the file and remove the original
manager.move_file("myFile.txt", "folder/myFile.txt");
```

#### Remove a file

```rust
// Remove a file from the index and from the directory
manager.remove_file("myFile.txt");
```
