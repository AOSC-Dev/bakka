// STD Dependencies -----------------------------------------------------------
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc::Sender;
// External Dependencies ------------------------------------------------------
use cursive::traits::*;
use cursive::views::Dialog;
use cursive::Cursive;

// Modules --------------------------------------------------------------------
use cursive_tree_view::{Placement, TreeView};

#[derive(Debug)]
struct TreeEntry {
    name: String,
    dir: Option<PathBuf>,
}

impl fmt::Display for TreeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn collect_entries(dir: &Path, entries: &mut Vec<TreeEntry>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                entries.push(TreeEntry {
                    name: entry
                        .file_name()
                        .into_string()
                        .unwrap_or_else(|_| "".to_string()),
                    dir: Some(path),
                });
            } else if path.is_file() {
                entries.push(TreeEntry {
                    name: entry
                        .file_name()
                        .into_string()
                        .unwrap_or_else(|_| "".to_string()),
                    dir: None,
                });
            }
        }
    }

    Ok(())
}

fn expand_tree(tree: &mut TreeView<TreeEntry>, parent_row: usize, dir: &Path) {
    let mut entries = Vec::new();
    collect_entries(dir, &mut entries).ok();

    entries.sort_by(|a, b| match (a.dir.is_some(), b.dir.is_some()) {
        (true, true) | (false, false) => a.name.cmp(&b.name),
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
    });

    for i in entries {
        if i.dir.is_some() {
            tree.insert_container_item(i, Placement::LastChild, parent_row);
        } else {
            tree.insert_item(i, Placement::LastChild, parent_row);
        }
    }
}

fn show_tree_with_working_directory(siv: &mut Cursive, directory: PathBuf, tx: Sender<String>) {
    let mut tree = TreeView::<TreeEntry>::new();
    let directory_selected_path = Rc::new(RefCell::new(directory.to_string_lossy().to_string()));
    let directory_selected_path_copy = Rc::clone(&directory_selected_path);

    tree.insert_item(
        TreeEntry {
            name: directory.file_name().unwrap().to_str().unwrap().to_string(),
            dir: Some(directory.clone()),
        },
        Placement::After,
        0,
    );

    expand_tree(&mut tree, 0, &directory);

    // Lazily insert directory listings for sub nodes
    tree.set_on_collapse(move |siv: &mut Cursive, row, is_collapsed, children| {
        let directory_selected_copy_copy = directory_selected_path_copy.clone();
        if !is_collapsed && children == 0 {
            siv.call_on_name("tree", move |tree: &mut TreeView<TreeEntry>| {
                if let Some(dir) = tree.borrow_item(row).unwrap().dir.clone() {
                    directory_selected_copy_copy
                        .replace(dir.canonicalize().unwrap().display().to_string());
                    expand_tree(tree, row, &dir);
                }
            });
        }
    });

    tree.set_on_submit(move |siv: &mut Cursive, row| {
        let file_name = siv.call_on_name("tree", move |tree: &mut TreeView<TreeEntry>| {
            tree.borrow_item(row).unwrap().to_string()
        });
        let directory_path = directory_selected_path.take();
        let path = Path::new(&directory_path).join(file_name.unwrap());
        tx.send(path.as_os_str().to_string_lossy().to_string())
            .unwrap();
        siv.pop_layer();
        siv.quit();
        let dump = siv.dump();
        siv.set_user_data(dump);
    });
    let tree_dialog = Dialog::around(tree.with_name("tree").scrollable()).title("File View");
    siv.add_layer(tree_dialog);
}

pub fn view_main(directory: PathBuf, editor: String) {
    let mut siv = cursive::default();
    let (tx, rx) = std::sync::mpsc::channel();
    let directory_clone = directory.clone();
    show_tree_with_working_directory(&mut siv, directory, tx.clone());
    siv.run();
    while let Some(dump) = siv.take_user_data::<cursive::Dump>() {
        if let Ok(path) = rx.recv() {
            drop(siv);
            Command::new(&editor)
                .arg(path)
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
            siv = cursive::default();
            siv.restore(dump);
            show_tree_with_working_directory(&mut siv, directory_clone.clone(), tx.clone());
            siv.run();
        }
    }
}
