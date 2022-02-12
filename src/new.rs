use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use std::fs;
use std::io::Read;
use std::{path::Path, process::Command};
use walkdir::WalkDir;

use crate::parser;

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
    editor_file(editor, &temp_spec)?;
    bye_bakka_comment(&temp_spec)?;
    fs::write(&temp_defines_path, BUNDLE_DEFINES)?;
    editor_file(editor, &temp_defines_path)?;
    bye_bakka_comment(&temp_defines_path)?;
    let category_path = tree.join(&category[selected_index]);
    copy_dir_all(temp_path, category_path.join(package))?;
    println!(
        r#"All Done! {} package directory in {}!
Try to use `bakka jump {}` to open it! or use `bakka view {}` to view directory!"#,
        package,
        category_path.join(package).display(),
        package,
        package
    );

    Ok(())
}

fn editor_file(editor: &str, file_path: &Path) -> Result<()> {
    Command::new(editor).arg(file_path).spawn()?.wait()?;
    loop {
        match question_whether_to_save_file(
            file_path
                .file_name()
                .ok_or_else(|| anyhow!("Could not get file name!"))?
                .to_str()
                .ok_or_else(|| anyhow!("Could not file name to str!"))?,
        ) {
            true => break,
            false => {
                Command::new(editor).arg(file_path).spawn()?.wait()?;
            }
        }
    }

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
    let parse_ab = parser::handle_autobuild_file(&buf)?;
    fs::write(path, parse_ab)?;

    Ok(())
}

fn get_all_category_in_tree(tree: &Path) -> Vec<String> {
    let mut result = WalkDir::new(tree)
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .flatten()
        .filter(|x| x.path().is_dir())
        .map(|x| {
            x.file_name()
                .to_str()
                .ok_or_else(|| anyhow!("Cannot convert filename to string!"))
                .unwrap()
                .to_string()
        })
        .filter(|x| !x.starts_with('.') && x != &"assets" && x != &"groups")
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

fn question_whether_to_save_file(filename: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Do you apply the {} file you just edited?",
            filename
        ))
        .default(true)
        .interact()
        .unwrap()
}
