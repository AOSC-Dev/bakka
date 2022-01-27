use anyhow::Result;
use std::fs;
use std::{path::Path, process::Command};

use dialoguer::{theme::ColorfulTheme, Select};
use walkdir::WalkDir;

const BUNDLE_SPEC: &[u8] = include_bytes!("../res/spec");
const BUNDLE_DEFINES: &[u8] = include_bytes!("../res/defines");

pub fn new_package(package: &str, tree: &Path, editor: &str) -> Result<()> {
    let category = get_all_category_in_tree(tree);
    let selected_index = Select::with_theme(&ColorfulTheme::default())
        .default(0)
        .with_prompt("Choose one category")
        .items(&category)
        .interact()?;
    let temp_path = tempfile::tempdir()?.into_path();
    let temp_autobuild_path = temp_path.join("autobuild");
    let temp_defines_path = temp_autobuild_path.join("defines");
    let temp_spec = temp_path.join("spec");
    fs::create_dir_all(temp_autobuild_path)?;
    fs::File::create(&temp_spec)?;
    fs::write(&temp_spec, BUNDLE_SPEC)?;
    Command::new(editor).arg(temp_spec).spawn()?.wait()?;
    fs::File::create(&temp_defines_path)?;
    fs::write(&temp_defines_path, BUNDLE_DEFINES)?;
    Command::new(editor)
        .arg(temp_defines_path)
        .spawn()?
        .wait()?;
    let category_path = tree.join(&category[selected_index]);
    fs::create_dir_all(&category_path)?;
    fs::copy(temp_path, category_path.join(package))?;

    Ok(())
}

fn get_all_category_in_tree(tree: &Path) -> Vec<String> {
    let mut result = WalkDir::new(tree)
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .flatten()
        .filter(|x| x.path().is_dir())
        .map(|x| x.file_name().to_string_lossy().to_string())
        .filter(|x| !x.starts_with('.') && x != "assets" && x != "groups")
        .collect::<Vec<_>>();
    result.sort();

    result
}
