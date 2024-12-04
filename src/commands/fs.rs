use std::sync::Arc;

use parking_lot::Mutex;
use rhai::{Array, Dynamic, EvalAltResult, Position};

use crate::{state::SharedState, Environment};

pub fn read_file(path: &str) -> Result<String, Box<EvalAltResult>> {
    std::fs::read_to_string(path).map_err(|e| {
        let msg = format!("Failed to read file: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

pub fn write_file(path: &str, content: &str) -> Result<(), Box<EvalAltResult>> {
    std::fs::write(path, content).map_err(|e| {
        let msg = format!("Failed to write file: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

pub fn remove(path: &str) -> Result<(), Box<EvalAltResult>> {
    std::fs::remove_dir_all(path).map_err(|e| {
        let msg = format!("Failed to remove path: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

pub fn mkdir(path: &str) -> Result<(), Box<EvalAltResult>> {
    std::fs::create_dir_all(path).map_err(|e| {
        let msg = format!("Failed to create directory: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

pub fn ls(path: &str) -> Result<Array, Box<EvalAltResult>> {
    let metadata = std::fs::metadata(path).map_err(|e| {
        let msg = format!("Failed to get metadata: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })?;

    if metadata.is_file() {
        return Ok(vec![Dynamic::from(
            path.split('/').last().unwrap_or(path).to_string(),
        )]);
    }

    let entries = std::fs::read_dir(path)
        .map_err(|e| {
            let msg = format!("Failed to list directory: {}", e);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .map(Dynamic::from)
        .collect();
    log::debug!("Directory contents: {:?}", entries);
    Ok(entries)
}

pub fn file_exists(path: &str) -> bool {
    std::fs::metadata(path).is_ok()
}

pub fn temp_dir<E: Environment>(
    state: Arc<Mutex<SharedState<E>>>,
    prefix: &str,
) -> Result<String, Box<EvalAltResult>> {
    let temp_dir = tempdir::TempDir::new(prefix).map_err(|e| {
        let msg = format!("Failed to create temporary directory: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })?;
    let path = temp_dir.path().to_string_lossy().to_string();
    state.lock().temp_dirs.push(temp_dir);
    Ok(path)
}

// Get file metadata like size, modified time, etc.
pub fn stat(path: &str) -> Result<Dynamic, Box<EvalAltResult>> {
    let metadata = std::fs::metadata(path).map_err(|e| {
        let msg = format!("Failed to get metadata: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })?;
    
    // Convert metadata to a Dynamic map
    let mut map = rhai::Map::new();
    map.insert("size".into(), Dynamic::from(metadata.len()));
    map.insert("is_file".into(), Dynamic::from(metadata.is_file()));
    map.insert("is_dir".into(), Dynamic::from(metadata.is_dir()));
    map.insert("modified".into(), Dynamic::from(metadata.modified().ok().map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()).unwrap_or(0)));
    
    Ok(Dynamic::from(map))
}

// Copy a file or directory
pub fn copy(src: &str, dst: &str) -> Result<(), Box<EvalAltResult>> {
    let metadata = std::fs::metadata(src).map_err(|e| {
        let msg = format!("Failed to get source metadata: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })?;

    if metadata.is_file() {
        std::fs::copy(src, dst).map_err(|e| {
            let msg = format!("Failed to copy file: {}", e);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })?;
    } else {
        copy_dir_all(src, dst).map_err(|e| {
            let msg = format!("Failed to copy directory: {}", e);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })?;
    }
    Ok(())
}

// Rename/move a file or directory
pub fn rename(src: &str, dst: &str) -> Result<(), Box<EvalAltResult>> {
    std::fs::rename(src, dst).map_err(|e| {
        let msg = format!("Failed to rename: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

// Check if path is a directory
pub fn is_dir(path: &str) -> bool {
    std::fs::metadata(path)
        .map(|m| m.is_dir())
        .unwrap_or(false)
}

// Check if path is a file
pub fn is_file(path: &str) -> bool {
    std::fs::metadata(path)
        .map(|m| m.is_file())
        .unwrap_or(false)
}

// Get absolute path
pub fn absolute_path(path: &str) -> Result<String, Box<EvalAltResult>> {
    std::fs::canonicalize(path)
        .map_err(|e| {
            let msg = format!("Failed to get absolute path: {}", e);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })
        .map(|p| p.to_string_lossy().to_string())
}

// Helper function for recursive directory copying
fn copy_dir_all(src: &str, dst: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = std::path::Path::new(dst).join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(
                src_path.to_str().unwrap(),
                dst_path.to_str().unwrap()
            )?;
        } else {
            std::fs::copy(src_path, dst_path)?;
        }
    }
    Ok(())
}

