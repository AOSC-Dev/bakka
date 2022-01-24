use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Select};
use std::path::Path;

mod index;

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
}

#[derive(Parser, Debug)]
struct CdSubCommand {
    #[clap()]
    package: String,
}

fn main() {
    let abbs_tree_path;
    if let Ok(tree) = std::env::var("ABBS_TREE") {
        abbs_tree_path = tree;
    } else {
        eprintln!("env ABBS_TREE is not to set!\nTry to run `export ABBS_TREE=\"/path/to/tree\"` in your shell!");
        std::process::exit(1);
    }
    let abbs_tree_path = Path::new(&abbs_tree_path);
    let index = index::read_index(abbs_tree_path).unwrap();
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
    }
}

fn get_package_directory(index: Vec<(String, String)>, package: String) -> String {
    let mut sorted_correlation_list = Vec::new();
    for (index, (name, path)) in index.into_iter().enumerate() {
        let correlation = strsim::jaro_winkler(&name, &package);
        if correlation > 0.0 {
            sorted_correlation_list.push((name, path, correlation));
        }
        if index == 10 {
            break;
        }
    }
    sorted_correlation_list.sort_by(|(_, _, a), (_, _, b)| b.partial_cmp(a).unwrap());
    let correlation_name_list = sorted_correlation_list
        .iter()
        .map(|x| x.0.to_owned())
        .collect::<Vec<_>>();
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
