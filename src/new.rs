use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::io::Read;
use std::{path::Path, process::Command};
use walkdir::WalkDir;

use crate::parser::{self, handle_autobuild_file};

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
    fs::File::create(&temp_defines_path)?;
    fs::write(&temp_spec, BUNDLE_SPEC)?;
    Command::new(editor).arg(&temp_spec).spawn()?.wait()?;
    bye_bakka_comment(&temp_spec)?;
    fs::write(&temp_defines_path, BUNDLE_DEFINES)?;
    Command::new(editor)
        .arg(&temp_defines_path)
        .spawn()?
        .wait()?;
    bye_bakka_comment(&temp_defines_path)?;
    let category_path = tree.join(&category[selected_index]);
    copy_dir_all(temp_path, category_path.join(package))?;

    Ok(())
}

fn bye_bakka_comment(path: &Path) -> Result<()> {
    let mut file = fs::File::open(&path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    if let Some(last) = buf.last() {
        if last != &b'\n' {
            buf.push(b'\n');
        }
    }
    let parse_ab = handle_autobuild_file(&buf)
        .map_err(|e| anyhow!("Could not handle autobuild file! Why: {}", e))?;
    let flatten_ab = parser::flatten_autobuild_file(parse_ab.1)
        .into_iter()
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();
    fs::write(path, flatten_ab)?;

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

/// Function from https://stackoverflow.com/questions/26958489/how-to-copy-a-folder-recursively-in-rust
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }

    Ok(())
}
