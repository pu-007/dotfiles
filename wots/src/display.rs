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

pub fn render_stats(stats_data: &HashMap<&'static str, TypeStats>, total_pkgs: usize, total_files: usize, total_bytes: u64) {
    use crate::util::fmt_size;

    println!(
        "WOTS Repository  —  {}\n",
        crate::config::DOTFILES_DIR.display()
    );
    println!(
        "{:<12} {:>5} {:>7} {:>10}  Status",
        "Type", "Pkgs", "Files", "Size"
    );
    println!("{}", "─".repeat(60));

    for pt in &crate::types::SYNCABLE_TYPES {
        let key = pt.value();
        if let Some(d) = stats_data.get(key) {
            if d.packages == 0 {
                continue;
            }
            let status = &d.status_text;
            println!(
                "{:<12} {:>5} {:>7} {:>10}  {}",
                key,
                d.packages,
                d.files,
                d.size_human,
                status,
            );
        }
    }

    println!("{}", "─".repeat(60));
    println!(
        "{:<12} {:>5} {:>7} {:>10}",
        "TOTAL".bold(),
        total_pkgs.to_string().bold(),
        total_files.to_string().bold(),
        fmt_size(total_bytes).bold(),
    );
}

pub fn render_list(rows: &[ListRow]) {
    println!(
        "{:<24} {:<8} {:>6} {:>10}  Status",
        "Package", "Type", "Files", "Size"
    );
    println!("{}", "─".repeat(68));

    for r in rows {
        println!(
            "{:<24} {:<8} {:>6} {:>10}  {}",
            r.name.cyan().bold(),
            r.pkg_type.green(),
            r.files,
            r.size_human,
            r.status,
        );
    }

    println!("\n{} package(s) total.", rows.len());
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
