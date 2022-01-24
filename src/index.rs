use anyhow::{anyhow, Result};
use std::{io::Read, path::Path};
use walkdir::WalkDir;

const INDEX_PATH: &str = "./.index";

fn gen_abbs_index(tree: &Path) -> Result<Vec<(String, String)>> {
    let mut result = Vec::new();
    std::env::set_current_dir(tree)?;
    for entry in WalkDir::new(".").max_depth(2).min_depth(2).into_iter() {
        if let Ok(entry) = entry {
            if entry.file_type().is_dir() {
                let name = entry
                    .file_name()
                    .to_str()
                    .ok_or_else(|| anyhow!("Path error!"))?;
                let path = entry
                    .path()
                    .to_str()
                    .ok_or_else(|| anyhow!("Path error!"))?;
                result.push((name.to_owned(), path.to_owned()));
            }
        }
    }
    let index_str = serde_json::to_string(&result).unwrap();
    std::fs::write(INDEX_PATH, index_str).unwrap();

    Ok(result)
}

pub fn read_index(tree: &Path) -> Result<Vec<(String, String)>> {
    std::env::set_current_dir(tree)?;
    let mut index_file;
    match std::fs::File::open(tree.join(INDEX_PATH)) {
        Ok(file) => index_file = file,
        Err(_) => return gen_abbs_index(tree),
    }
    let mut buf = Vec::new();
    index_file.read_to_end(&mut buf)?;
    let index: Vec<(String, String)> = serde_json::from_slice(&buf)?;

    Ok(index)
}
