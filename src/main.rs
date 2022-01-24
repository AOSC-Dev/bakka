use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Select};
use std::path::Path;

mod index;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Switch to this package directory
    #[clap()]
    package: Option<String>,
    #[clap(long, short = 'c')]
    cd: Option<String>,
    #[clap(long, short = 'j')]
    jump: Option<String>,
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
    if let Some(package) = args.package {
        cd(index, package, abbs_tree_path);
        std::process::exit(0);
    }
    if let Some(package) = args.cd {
        cd(index, package, abbs_tree_path);
        std::process::exit(0);
    }
    if let Some(package) = args.jump {
        jump(index, abbs_tree_path, package);
        std::process::exit(0);
    }
}

fn jump(index: Vec<(String, String)>, abbs_tree_path: &Path, package: String) {
    let count = index.iter().position(|(x, _)| x == &package);
    match count {
        Some(count) => println!(
            "{}",
            abbs_tree_path.join(index[count].1.to_owned()).display()
        ),
        None => {
            eprintln!("Cannot find package: {}", package);
            std::process::exit(1);
        }
    }
}

fn cd(index: Vec<(String, String)>, package: String, abbs_tree_path: &Path) {
    println!(
        "{}",
        abbs_tree_path
            .join(get_package_directory(index, package))
            .display()
    );
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
