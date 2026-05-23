use std::collections::HashMap;

use colored::Colorize;
use tabled::settings::Style;
use tabled::Table;
use tabled::Tabled;

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

#[derive(Tabled)]
struct StatsRow {
    #[tabled(rename = "Type")]
    r#type: String,
    #[tabled(rename = "Pkgs")]
    pkgs: String,
    #[tabled(rename = "Files")]
    files: String,
    #[tabled(rename = "Size")]
    size: String,
    #[tabled(rename = "Status")]
    status: String,
}

pub fn render_stats(
    stats_data: &HashMap<&'static str, TypeStats>,
    total_pkgs: usize,
    total_files: usize,
    total_bytes: u64,
) {
    use crate::util::fmt_size;

    let repo_path = crate::config::DOTFILES_DIR.display();
    println!("WOTS Repository  —  {}\n", repo_path);

    let mut rows: Vec<StatsRow> = Vec::new();
    for pt in &crate::types::SYNCABLE_TYPES {
        let key = pt.value();
        if let Some(d) = stats_data.get(key) {
            if d.packages == 0 {
                continue;
            }
            rows.push(StatsRow {
                r#type: key.to_string(),
                pkgs: d.packages.to_string(),
                files: d.files.to_string(),
                size: d.size_human.clone(),
                status: d.status_text.clone(),
            });
        }
    }

    let total_pkgs_str = total_pkgs.to_string();
    let total_files_str = total_files.to_string();
    let total_size = fmt_size(total_bytes);
    rows.push(StatsRow {
        r#type: "TOTAL".bold().to_string(),
        pkgs: total_pkgs_str.bold().to_string(),
        files: total_files_str.bold().to_string(),
        size: total_size.bold().to_string(),
        status: "(synced + pending shown above)".clear().to_string(),
    });

    let mut table = Table::new(rows);
    table.with(Style::modern_rounded());
    println!("{}", table);
}

#[derive(Tabled)]
struct ListRowOut {
    #[tabled(rename = "Package")]
    name: String,
    #[tabled(rename = "Type")]
    r#type: String,
    #[tabled(rename = "Files")]
    files: String,
    #[tabled(rename = "Size")]
    size: String,
    #[tabled(rename = "Status")]
    status: String,
}

pub fn render_list(rows: &[ListRow]) {
    let out_rows: Vec<ListRowOut> = rows
        .iter()
        .map(|r| ListRowOut {
            name: r.name.cyan().bold().to_string(),
            r#type: r.pkg_type.green().to_string(),
            files: r.files.to_string(),
            size: r.size_human.clone(),
            status: r.status.clone(),
        })
        .collect();

    let mut table = Table::new(out_rows);
    table.with(Style::modern_rounded());
    println!("{}", table);
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
