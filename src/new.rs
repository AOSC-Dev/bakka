use std::{path::Path, process::Command};
use std::fs;
use anyhow::Result;

use dialoguer::{theme::ColorfulTheme, Select};

use crate::tree;

pub fn new_package(package: &str, tree: &Path, editor: &str) -> Result<()> {
    let category = tree::get_all_category_in_tree(tree);
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
    Command::new(editor).arg(temp_spec).spawn()?.wait()?;
    Command::new(editor).arg(temp_defines_path).spawn()?.wait()?;
    let category_path = tree.join(&category[selected_index]);
    fs::create_dir_all(&category_path)?;
    fs::copy(temp_path, category_path.join(package))?;
    
    Ok(())
}
