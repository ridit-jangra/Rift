use std::path::Path;

use gpui_component::tree::TreeItem;

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    "coverage",
    ".cache",
    "__pycache__",
    ".venv",
    "venv",
    "vendor",
];

/// Recursively scan a directory into TreeItems, skipping the usual noise
/// (dotfiles, .git, node_modules, target, build output, etc.) so the
/// explorer doesn't drown in dependency/build directories.
pub fn build_file_items(root: &Path) -> Vec<TreeItem> {
    scan_dir(root)
}

fn scan_dir(path: &Path) -> Vec<TreeItem> {
    let mut items = Vec::new();

    let Ok(entries) = std::fs::read_dir(path) else {
        return items;
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let file_name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        if file_name.starts_with('.') || SKIP_DIRS.contains(&file_name.as_str()) {
            continue;
        }

        let id = entry_path.to_string_lossy().to_string();

        if entry_path.is_dir() {
            let children = scan_dir(&entry_path);
            items.push(TreeItem::new(id, file_name).children(children));
        } else {
            items.push(TreeItem::new(id, file_name));
        }
    }

    items.sort_by(|a, b| {
        b.is_folder()
            .cmp(&a.is_folder())
            .then(a.label.cmp(&b.label))
    });
    items
}
