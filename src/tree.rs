use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn gen_abbs_index(tree: &Path) -> Result<Vec<(String, String)>> {
    let mut result = Vec::new();
    std::env::set_current_dir(tree)?;
    for entry in WalkDir::new(".")
        .max_depth(2)
        .min_depth(2)
        .into_iter()
        .flatten()
    {
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

    Ok(result)
}

pub fn get_tree(directory: &Path) -> Result<PathBuf> {
    let mut tree = directory.canonicalize()?;
    let mut has_groups;
    loop {
        has_groups = Path::new(&format!("{}/groups", tree.display())).is_dir();
        if !has_groups && tree.to_str() == Some("/") {
            return Err(anyhow!("Cannot find ABBS tree!"));
        }
        if has_groups {
            return Ok(tree.to_path_buf());
        }
        tree.pop();
    }
}

pub fn get_all_category_in_tree(tree: &Path) -> Vec<String> {
    let mut result = WalkDir::new(tree)
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .flatten()
        .filter(|x| x.path().is_dir())
        .map(|x| x.file_name().to_string_lossy().to_string())
        .filter(|x| !x.starts_with(".") && x != "assets" && x != "groups")
        .collect::<Vec<_>>();
    result.sort();

    result
}
