use std::collections::HashMap;

use colored::Colorize;

pub fn info(msg: &str) {
    println!("{}", msg);
}

pub fn success(msg: &str) {
    println!("{} {}", "✓".green(), msg);
}

pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red(), msg);
}

pub fn warning(msg: &str) {
    println!("{} {}", "!".yellow(), msg);
}

pub fn dim(msg: &str) {
    println!("{}", msg.dimmed());
}

pub fn rule(title: &str) {
    if title.is_empty() {
        println!("{}", "─".repeat(50).dimmed());
    } else {
        println!("\n{} {}", "──".dimmed(), title.bold());
    }
}

pub fn render_stats(
    stats_data: &HashMap<&'static str, TypeStats>,
    total_pkgs: usize,
    total_files: usize,
    total_bytes: u64,
) {
    use crate::util::fmt_size;
    use comfy_table::presets::UTF8_FULL;
    use comfy_table::{Attribute, Cell, Table};

    let repo_path = crate::config::DOTFILES_DIR.display();
    println!("WOTS Repository  —  {}\n", repo_path);

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec![
            Cell::new("Type").add_attribute(Attribute::Bold),
            Cell::new("Pkgs").add_attribute(Attribute::Bold).set_alignment(comfy_table::CellAlignment::Right),
            Cell::new("Files").add_attribute(Attribute::Bold).set_alignment(comfy_table::CellAlignment::Right),
            Cell::new("Size").add_attribute(Attribute::Bold).set_alignment(comfy_table::CellAlignment::Right),
            Cell::new("Status").add_attribute(Attribute::Bold),
        ]);

    for pt in &crate::types::SYNCABLE_TYPES {
        let key = pt.value();
        if let Some(d) = stats_data.get(key) {
            if d.packages == 0 {
                continue;
            }
            table.add_row(vec![
                Cell::new(key),
                Cell::new(d.packages).set_alignment(comfy_table::CellAlignment::Right),
                Cell::new(d.files).set_alignment(comfy_table::CellAlignment::Right),
                Cell::new(&d.size_human).set_alignment(comfy_table::CellAlignment::Right),
                Cell::new(&d.status_text),
            ]);
        }
    }

    let total_size = fmt_size(total_bytes);
    table.add_row(vec![
        Cell::new("TOTAL").add_attribute(Attribute::Bold),
        Cell::new(total_pkgs).add_attribute(Attribute::Bold).set_alignment(comfy_table::CellAlignment::Right),
        Cell::new(total_files).add_attribute(Attribute::Bold).set_alignment(comfy_table::CellAlignment::Right),
        Cell::new(total_size).add_attribute(Attribute::Bold).set_alignment(comfy_table::CellAlignment::Right),
        Cell::new("(synced + pending shown above)").add_attribute(Attribute::Dim),
    ]);

    println!("{table}");
}

pub fn render_list(rows: &[ListRow]) {
    use comfy_table::presets::UTF8_FULL;
    use comfy_table::{Attribute, Cell, Table};

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec![
            Cell::new("Package").add_attribute(Attribute::Bold),
            Cell::new("Type").add_attribute(Attribute::Bold),
            Cell::new("Files").add_attribute(Attribute::Bold).set_alignment(comfy_table::CellAlignment::Right),
            Cell::new("Size").add_attribute(Attribute::Bold).set_alignment(comfy_table::CellAlignment::Right),
            Cell::new("Status").add_attribute(Attribute::Bold),
        ]);

    for r in rows {
        table.add_row(vec![
            Cell::new(&r.name),
            Cell::new(&r.pkg_type),
            Cell::new(r.files).set_alignment(comfy_table::CellAlignment::Right),
            Cell::new(&r.size_human).set_alignment(comfy_table::CellAlignment::Right),
            Cell::new(&r.status),
        ]);
    }

    println!("{table}");
    println!("\n  {} package(s) total.", rows.len());
}

pub struct TypeStats {
    pub packages: usize,
    pub files: usize,
    pub size_bytes: u64,
    pub size_human: String,
    pub status_text: String,
    pub names: Vec<String>,
}

pub struct ListRow {
    pub name: String,
    pub pkg_type: String,
    pub files: usize,
    pub size_bytes: u64,
    pub size_human: String,
    pub status: String,
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_stats_construction() {
        let ts = TypeStats {
            packages: 3,
            files: 10,
            size_bytes: 1024,
            size_human: "1.0 KB".into(),
            status_text: "stowed".into(),
            names: vec!["app".into()],
        };
        assert_eq!(ts.packages, 3);
        assert_eq!(ts.files, 10);
        assert_eq!(ts.size_bytes, 1024);
    }

    #[test]
    fn list_row_construction() {
        let row = ListRow {
            name: "test".into(),
            pkg_type: "user".into(),
            files: 5,
            size_bytes: 500,
            size_human: "500 B".into(),
            status: "stowed".into(),
            path: "/tmp/test.user".into(),
        };
        assert_eq!(row.name, "test");
        assert_eq!(row.pkg_type, "user");
        assert_eq!(row.files, 5);
        assert!(row.path.ends_with("test.user"));
    }

    #[test]
    fn info_does_not_panic() {
        info("test message");
    }

    #[test]
    fn success_does_not_panic() {
        success("done");
    }

    #[test]
    fn error_does_not_panic() {
        error("failed");
    }

    #[test]
    fn warning_does_not_panic() {
        warning("watch out");
    }

    #[test]
    fn dim_does_not_panic() {
        dim("faded");
    }

    #[test]
    fn rule_empty_does_not_panic() {
        rule("");
    }

    #[test]
    fn rule_with_title_does_not_panic() {
        rule("My Section");
    }

    #[test]
    fn render_stats_with_empty_data() {
        let data: HashMap<&str, TypeStats> = HashMap::new();
        render_stats(&data, 0, 0, 0);
    }

    #[test]
    fn render_stats_with_one_type() {
        let mut data: HashMap<&str, TypeStats> = HashMap::new();
        data.insert(
            "user",
            TypeStats {
                packages: 2,
                files: 10,
                size_bytes: 2048,
                size_human: "2.0 KB".into(),
                status_text: "10/10 stowed".into(),
                names: vec!["zsh".into(), "git".into()],
            },
        );
        render_stats(&data, 2, 10, 2048);
    }

    #[test]
    fn render_list_empty() {
        let rows: Vec<ListRow> = Vec::new();
        render_list(&rows);
    }

    #[test]
    fn render_list_with_entries() {
        let rows = vec![
            ListRow {
                name: "zsh".into(),
                pkg_type: "user".into(),
                files: 6,
                size_bytes: 1024,
                size_human: "1.0 KB".into(),
                status: "stowed".into(),
                path: "/tmp/zsh.user".into(),
            },
            ListRow {
                name: "nvim".into(),
                pkg_type: "config".into(),
                files: 30,
                size_bytes: 50000,
                size_human: "48.8 KB".into(),
                status: "30/30 stowed".into(),
                path: "/tmp/nvim.config".into(),
            },
        ];
        render_list(&rows);
    }

    #[test]
    fn prompt_confirm_default_yes_empty_input() {
        // Cannot test stdin interactively, but verify function signature compiles.
        // The function exists; actual behavior tested manually.
        let _ = prompt::confirm;
    }

    #[test]
    fn prompt_ask_function_exists() {
        let _ = prompt::ask;
    }

    #[test]
    fn prompt_ask_custom_function_exists() {
        let _ = prompt::ask_custom;
    }
}


pub mod prompt {
    use std::io::{self, Write};

    use colored::Colorize;

    pub fn confirm(msg: &str, default_yes: bool) -> bool {
        let prompt = if default_yes { "([Y]/n)" } else { "(y/[N])" };
        print!("{} {}: ", msg, prompt.dimmed());
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim().to_lowercase();
        if input.is_empty() {
            return default_yes;
        }
        input == "y" || input == "yes"
    }

    pub fn ask(msg: &str, default: &str) -> String {
        print!("{} [{}]: ", msg, default.cyan());
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim().to_string();
        if input.is_empty() {
            default.to_string()
        } else {
            input
        }
    }

    pub fn ask_custom(msg: &str, default: &str, options: &[&str]) -> String {
        let opts = options
            .iter()
            .map(|o| o.bright_black().to_string())
            .collect::<Vec<_>>()
            .join("/");
        print!("{} [{}/{}]: ", msg, default.cyan(), opts);
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim().to_string();
        if input.is_empty() {
            default.to_string()
        } else {
            input
        }
    }
}
