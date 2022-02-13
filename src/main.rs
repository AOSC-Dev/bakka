use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

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
    /// Get package path
    GetPath(GetPathSubCommand),
    /// Add patch to package
    AddPatch(AddPatchSubCommand),
    /// init function to shell
    Init(InitSubCommand),
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

#[derive(Parser, Debug)]
struct GetPathSubCommand {
    #[clap()]
    package: String,
}

#[derive(Parser, Debug)]
struct AddPatchSubCommand {
    #[clap()]
    patch_file_path: String,
    package: String,
}

#[derive(Parser, Debug)]
struct InitSubCommand {
    #[clap()]
    shell: String,
}

const BASH_SCRIPT: &'static str = include_str!("../shell/bakka.bash");

fn main() {
    let editor = if let Ok(editor) = std::env::var("EDITOR") {
        editor
    } else {
        "nano".to_string()
    };
    let args = Args::parse();
    match args.subcommand {
        Command::Cd(CdSubCommand { package }) => {
            let (abbs_tree_path, index) = find_abbs_tree_and_index();
            println!(
                "{}",
                abbs_tree_path
                    .join(tree::select_package_to_directory(index, &package))
                    .display()
            );
        }
        Command::Jump(CdSubCommand { package })
        | Command::GetPath(GetPathSubCommand { package }) => {
            let (abbs_tree_path, index) = find_abbs_tree_and_index();
            println!(
                "{}",
                abbs_tree_path
                    .join(tree::get_package_path(index, &package, &abbs_tree_path).unwrap())
                    .display()
            );
        }
        Command::View(ViewSubCommand { package }) => {
            let (abbs_tree_path, index) = find_abbs_tree_and_index();
            let path = abbs_tree_path.join(tree::select_package_to_directory(index, &package));
            view::view_main(path, editor);
        }
        Command::New(NewSubCommand { package }) => {
            let (abbs_tree_path, index) = find_abbs_tree_and_index();
            if tree::get_package_path(index, &package, &abbs_tree_path).is_err() {
                new::new_package(&package, &abbs_tree_path, &editor).unwrap();
            } else {
                eprintln!("Package {} is exist!", package);
                std::process::exit(1);
            }
        }
        Command::AddPatch(AddPatchSubCommand {
            patch_file_path,
            package,
        }) => {
            let (abbs_tree_path, index) = find_abbs_tree_and_index();
            tree::add_patch(
                &package,
                index,
                &abbs_tree_path,
                Path::new(&patch_file_path),
            )
            .unwrap();
        }
        Command::Init(InitSubCommand { shell }) => {
            if shell == "bash" {
                print!("{}", BASH_SCRIPT);
            } else {
                eprintln!("Unsupport shell: {}", shell);
                std::process::exit(1);
            }
        }
    }
}

fn find_abbs_tree_and_index() -> (PathBuf, Vec<(String, String)>) {
    let abbs_tree_path;
    if let Ok(tree) = tree::get_tree(&std::env::current_dir().unwrap()) {
        abbs_tree_path = tree;
    } else if let Ok(tree) = std::env::var("ABBS_TREE") {
        abbs_tree_path = PathBuf::from(&tree);
    } else {
        eprintln!("Cannot find ABBS tree!\nTry to run `export ABBS_TREE=\"/path/to/tree\"` in your shell!");
        std::process::exit(1);
    }
    let index = tree::gen_abbs_index(&abbs_tree_path).unwrap();

    (abbs_tree_path, index)
}
