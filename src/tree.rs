use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn gen_abbs_index(tree: &Path) -> Result<Vec<(String, String)>> {
    let mut result = Vec::new();
    std::env::set_current_dir(tree)
        .map_err(|e| anyhow!("Cannot switch to tree directory! why: {}", e))?;
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
                .ok_or_else(|| anyhow!("Cannot convert OsStr to str!"))?;
            let path = entry
                .path()
                .to_str()
                .ok_or_else(|| anyhow!("Cannot convert path to str!"))?;
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

pub fn select_package_to_directory(index: Vec<(String, String)>, package: &str) -> String {
    let mut sorted_correlation_list = Vec::new();
    let package = if package.contains('/') {
        let (_, package) = package.split_once('/').unwrap();

        package
    } else {
        package
    };
    for (name, path) in index {
        let correlation = strsim::jaro_winkler(&name, package);
        if correlation > 0.0 {
            sorted_correlation_list.push((name, path, correlation));
        }
    }
    sorted_correlation_list.sort_by(|(_, _, a), (_, _, b)| b.partial_cmp(a).unwrap());
    let mut correlation_name_list = Vec::new();
    for (name, _, _) in sorted_correlation_list.iter().take(10) {
        correlation_name_list.push(name);
    }
    let selected_package_index: usize = if correlation_name_list.len() == 1 {
        0
    } else {
        Select::with_theme(&ColorfulTheme::default())
            .default(0)
            .with_prompt("Choose one package to switch directory")
            .items(&correlation_name_list)
            .interact()
            .unwrap()
    };

    sorted_correlation_list[selected_package_index]
        .1
        .to_string()
}

pub fn get_package_path(
    index: Vec<(String, String)>,
    package: &str,
    abbs_tree_path: &Path,
) -> Result<PathBuf> {
    let count = index
        .iter()
        .position(|(x, _)| x == package)
        .ok_or_else(|| anyhow!("Cannot find package: {}", package))?;

    Ok(abbs_tree_path.join(&index[count].1))
}

pub fn add_patch(
    package: &str,
    index: Vec<(String, String)>,
    abbs_tree_path: &Path,
    patch_path: &Path,
) -> Result<()> {
    let package_path = get_package_path(index, package, abbs_tree_path)?;
    let to = package_path.join("autobuild/patches");
    if !to.exists() {
        std::fs::create_dir_all(&to)?;
    } else if !to.is_dir() {
        return Err(anyhow!("Package {} patches directory not a dir!", package));
    };
    std::fs::copy(patch_path, to)?;

    Ok(())
}
