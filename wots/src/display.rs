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

    let repo_path = crate::config::DOTFILES_DIR.display();
    println!("WOTS Repository  —  {}\n", repo_path);

    // Collect rows so we can compute max widths dynamically
    let mut rows: Vec<(String, usize, usize, String, String)> = Vec::new();
    for pt in &crate::types::SYNCABLE_TYPES {
        let key = pt.value();
        if let Some(d) = stats_data.get(key) {
            if d.packages == 0 {
                continue;
            }
            rows.push((
                key.to_string(),
                d.packages,
                d.files,
                d.size_human.clone(),
                d.status_text.clone(),
            ));
        }
    }

    // Headers
    let h_type = "Type";
    let h_pkgs = "Pkgs";
    let h_files = "Files";
    let h_size = "Size";
    let h_status = "Status";

    // Compute column widths
    let w_type = rows.iter().map(|r| r.0.len()).max().unwrap_or(12).max(h_type.len());
    let w_pkgs = rows.iter().map(|r| r.1.to_string().len()).max().unwrap_or(5).max(h_pkgs.len());
    let w_files = rows.iter().map(|r| r.2.to_string().len()).max().unwrap_or(5).max(h_files.len());
    let w_size = rows.iter().map(|r| r.3.len()).max().unwrap_or(8).max(h_size.len());
    let w_status = rows.iter().map(|r| r.4.len()).max().unwrap_or(20).max(h_status.len());

    let sep_w = w_type + w_pkgs + w_files + w_size + w_status + 10;

    // Header
    println!(
        "  {:<w_type$}  {:>w_pkgs$}  {:>w_files$}  {:>w_size$}  {}",
        h_type, h_pkgs, h_files, h_size, h_status,
    );
    println!("{}", "─".repeat(sep_w));

    // Data rows
    for (typ, pkgs, files, size, status) in &rows {
        println!(
            "  {:<w_type$}  {:>w_pkgs$}  {:>w_files$}  {:>w_size$}  {}",
            typ, pkgs, files, size, status,
        );
    }

    // Footer
    let total_pkgs_str = total_pkgs.to_string();
    let total_files_str = total_files.to_string();
    let total_size = fmt_size(total_bytes);
    println!("{}", "─".repeat(sep_w));
    println!(
        "  {:<w_type$}  {:>w_pkgs$}  {:>w_files$}  {:>w_size$}  (synced + pending shown above)",
        "TOTAL".bold(),
        total_pkgs_str.bold(),
        total_files_str.bold(),
        total_size.bold(),
    );
}

pub fn render_list(rows: &[ListRow]) {
    let h_pkg = "Package";
    let h_type = "Type";
    let h_files = "Files";
    let h_size = "Size";
    let h_status = "Status";

    let w_pkg = rows.iter().map(|r| r.name.len()).max().unwrap_or(12).max(h_pkg.len());
    let w_type = rows.iter().map(|r| r.pkg_type.len()).max().unwrap_or(8).max(h_type.len());
    let w_files = rows.iter().map(|r| r.files.to_string().len()).max().unwrap_or(5).max(h_files.len());
    let w_size = rows.iter().map(|r| r.size_human.len()).max().unwrap_or(8).max(h_size.len());
    let w_status = rows.iter().map(|r| r.status.len()).max().unwrap_or(24).max(h_status.len());
    let sep_w = w_pkg + w_type + w_files + w_size + w_status + 12;

    println!(
        "  {:<w_pkg$}  {:<w_type$}  {:>w_files$}  {:>w_size$}  {}",
        h_pkg, h_type, h_files, h_size, h_status,
    );
    println!("{}", "─".repeat(sep_w));

    for r in rows {
        println!(
            "  {:<w_pkg$}  {:<w_type$}  {:>w_files$}  {:>w_size$}  {}",
            r.name.cyan().bold(),
            r.pkg_type.green(),
            r.files,
            r.size_human,
            r.status,
        );
    }

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
