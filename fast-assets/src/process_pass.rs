use crate::decompression_manager::DecompressionManager;
use crate::manager::AssetsManager;
use std::fmt::Debug;
use std::path::PathBuf;

pub trait ProcessPass: Debug {
    /// Called when loading a file, and return true if continue the existing process
    fn on_load(&mut self, _: &mut AssetsManager, _: &mut Option<String>) -> bool {
        true
    }

    /// Called when unloading a file, and return true if continue the existing process
    fn on_unload(&mut self, _: &mut AssetsManager, _: &mut &str, _: &mut bool) -> bool {
        true
    }

    /// Called when remove a file reference, and return true if continue the existing process
    fn on_remove(&mut self, _: &mut AssetsManager, _: &mut &str) -> bool {
        true
    }

    /// Called when loading file/files from archive
    fn on_archive(&mut self, _: &mut DecompressionManager, _: &str, _: &PathBuf) {}
}
