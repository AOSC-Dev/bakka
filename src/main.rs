use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Select};
use std::path::PathBuf;

mod new;
mod tree;
mod view;
mod parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(subcommand)]
    subcommand: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Switch to this package directory
    Cd(CdSubCommand),
    /// Switch to thic package directory (no match mode)
    Jump(CdSubCommand),
    /// View and modify this package autobuld file
    View(ViewSubCommand),
    /// Create new package
    New(NewSubCommand),
}

#[derive(Parser, Debug)]
struct CdSubCommand {
    #[clap()]
    package: String,
}

#[derive(Parser, Debug)]
struct ViewSubCommand {
    #[clap()]
    package: String,
}

#[derive(Parser, Debug)]
struct NewSubCommand {
    #[clap()]
    package: String,
}

fn main() {
    let abbs_tree_path;
    if let Ok(tree) = tree::get_tree(&std::env::current_dir().unwrap()) {
        abbs_tree_path = tree;
    } else if let Ok(tree) = std::env::var("ABBS_TREE") {
        abbs_tree_path = PathBuf::from(&tree);
    } else {
        eprintln!("Cannot find ABBS tree!\nTry to run `export ABBS_TREE=\"/path/to/tree\"` in your shell!");
        std::process::exit(1);
    }
    let editor = if let Ok(editor) = std::env::var("EDITOR") {
        editor
    } else {
        "nano".to_string()
    };
    let index = tree::gen_abbs_index(&abbs_tree_path).unwrap();
    let args = Args::parse();
    match args.subcommand {
        Command::Cd(CdSubCommand { package }) => {
            println!(
                "{}",
                abbs_tree_path
                    .join(get_package_directory(index, package))
                    .display()
            );
        }
        Command::Jump(CdSubCommand { package }) => match search_package(&index, &package) {
            Ok(count) => {
                println!("{}", abbs_tree_path.join(&index[count].1).display());
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        },
        Command::View(ViewSubCommand { package }) => {
            let path = abbs_tree_path.join(get_package_directory(index, package));
            view::view_main(path, editor);
        }
        Command::New(NewSubCommand { package }) => {
            if search_package(&index, &package).is_err() {
                new::new_package(&package, &abbs_tree_path, &editor).unwrap();
            } else {
                eprintln!("Package {} is exist!", package);
                std::process::exit(1);
            }
        }
    }
}

fn get_package_directory(index: Vec<(String, String)>, package: String) -> String {
    let mut sorted_correlation_list = Vec::new();
    for (name, path) in index {
        let correlation = strsim::jaro_winkler(&name, &package);
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

pub fn search_package(index: &[(String, String)], package: &str) -> Result<usize> {
    let count = index
        .iter()
        .position(|(x, _)| x == package)
        .ok_or_else(|| anyhow!("Cannot find package: {}", package))?;

    Ok(count)
}
