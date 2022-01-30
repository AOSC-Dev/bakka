use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod new;
mod parser;
mod tree;
mod view;

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
                    .join(tree::get_package_directory(index, package))
                    .display()
            );
        }
        Command::Jump(CdSubCommand { package }) => match tree::search_package(&index, &package) {
            Ok(count) => {
                println!("{}", abbs_tree_path.join(&index[count].1).display());
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        },
        Command::View(ViewSubCommand { package }) => {
            let path = abbs_tree_path.join(tree::get_package_directory(index, package));
            view::view_main(path, editor);
        }
        Command::New(NewSubCommand { package }) => {
            if tree::search_package(&index, &package).is_err() {
                new::new_package(&package, &abbs_tree_path, &editor).unwrap();
            } else {
                eprintln!("Package {} is exist!", package);
                std::process::exit(1);
            }
        }
    }
}

