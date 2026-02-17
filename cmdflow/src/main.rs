use std::collections::HashMap;
use std::fs::{self, read_to_string, OpenOptions};
use std::io::Write;
use std::env;
use std::process::Command;
use colored::*;

fn read_log(path: &std::path::Path) -> Vec<String> {
    read_to_string(path)
        .map(|content| content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .map(|l| l.to_lowercase())
            .collect())
        .unwrap_or_default()
}

fn rebuild_log(src: &std::path::Path, dst: &std::path::Path, is_fish: bool) {
    if !src.exists() { return; }

    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(dst)
        .unwrap();

    if let Ok(content) = read_to_string(src) {
        for line in content.lines() {
            let cmd = if is_fish {
                line.strip_prefix("- cmd: ").map(str::trim)
            } else {
                let l = line.trim();
                if l.is_empty() || l.starts_with('#') { None } else { Some(l) }
            };

            if let Some(cmd) = cmd {
                let _ = writeln!(log_file, "{}", cmd);
            }
        }
    }
}

fn is_working(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", cmd))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn print_top(title: &str, data: &[(String, usize)], top_n: usize) {
    let soft_rainbow = [
        (200,150,255),(180,180,255),(150,220,255),
        (150,255,200),(180,255,150),(220,255,150),
        (255,220,150),(255,180,150),(255,150,180),
    ];

    let max_cmd_len = 12;
    let max_bar_len = 20;
    let max_count = data.get(0).map(|x| x.1).unwrap_or(1).max(1);

    println!("{}", title.bold().underline());

    for (i, (cmd, count)) in data.iter().take(top_n).enumerate() {
        let display_cmd = if cmd.len() > max_cmd_len {
            format!("{}…", &cmd[..max_cmd_len-1])
        } else { cmd.clone() };

        let bar_len = ((*count as f32 / max_count as f32) * max_bar_len as f32).round() as usize;
        let bar = "█".repeat(bar_len.max(1));

        let line = format!("{:>5} │ {:<12} {}", count, display_cmd, bar);

        // Радужная палитра по строкам
        let c = soft_rainbow[i % soft_rainbow.len()];
        println!("{}", line.truecolor(c.0, c.1, c.2));
    }

    println!();
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "--help") {
        println!("Usage:");
        println!("  --fish       Read only fish commands");
        println!("  --bash       Read only bash commands");
        println!("  --working    Show top N working commands");
        println!("  --broken     Show top N broken commands");
        println!("  --version    Show program version");
        println!("  --help       Show this help message");
        println!("  --version    Show project version");
        return;
    }

    if args.iter().any(|a| a == "--version") {
        println!("cmdflow v2.3.6");
        return;
    }

    let has_fish = args.contains(&"--fish".to_string());
    let has_bash = args.contains(&"--bash".to_string());
    let use_fish = has_fish || (!has_fish && !has_bash);
    let use_bash = has_bash || (!has_fish && !has_bash);

    let top_n: usize = args.iter().filter_map(|a| a.parse::<usize>().ok()).next().unwrap_or(10);

    let home = match dirs::home_dir() {
        Some(h) => h,
        None => { eprintln!("Could not determine home directory."); return; }
    };

    let fish_history = home.join(".local/share/fish/fish_history");
    let bash_history = home.join(".bash_history");
    let cmdflow_dir = home.join(".local/share/cmdflow");
    let fish_log = cmdflow_dir.join("fish.log");
    let bash_log = cmdflow_dir.join("bash.log");

    if let Err(_) = fs::create_dir_all(&cmdflow_dir) {
        eprintln!("Failed to create cmdflow directory");
        return;
    }

    let _ = fs::remove_file(&fish_log);
    let _ = fs::remove_file(&bash_log);

    if use_fish { rebuild_log(&fish_history, &fish_log, true); }
    if use_bash { rebuild_log(&bash_history, &bash_log, false); }

    // ---------- Считаем команды ----------
    let mut counter: HashMap<String, usize> = HashMap::new();
    let mut combined_logs = vec![];
    if use_fish && fish_log.exists() { combined_logs.extend(read_log(&fish_log)); }
    if use_bash && bash_log.exists() { combined_logs.extend(read_log(&bash_log)); }

    for line in combined_logs {
        let line = line.trim().to_lowercase();
        if line.is_empty() { continue; }

        let key = line.split_whitespace().next().unwrap();
        if !key.chars().next().unwrap().is_alphanumeric() { continue; }

        *counter.entry(key.to_string()).or_insert(0) += 1;
    }

    if counter.is_empty() {
        println!("No commands found for the selected shell(s).");
        return;
    }

    // ---------- Подготовка данных ----------
    let mut vec_main: Vec<_> = counter.iter().map(|(cmd, &c)| (cmd.clone(), c)).collect();

    // Фильтруем по рабочим/нерабочим, если аргумент есть
    if args.contains(&"--working".to_string()) {
        vec_main = vec_main.into_iter().filter(|(cmd, _)| is_working(cmd)).collect();
    } else if args.contains(&"--broken".to_string()) {
        vec_main = vec_main.into_iter().filter(|(cmd, _)| !is_working(cmd)).collect();
    }

    vec_main.sort_by(|a, b| b.1.cmp(&a.1));

    let mode = match (use_fish, use_bash) {
        (true, true) => "fish + bash",
        (true, false) => "fish",
        (false, true) => "bash",
        _ => "unknown",
    };

    let arg_label = if args.contains(&"--working".to_string()) {
        "Working Commands"
    } else if args.contains(&"--broken".to_string()) {
        "Broken Commands"
    } else {
        "Commands"
    };

    print_top(&format!("Top {} {} ({})", top_n, arg_label, mode), &vec_main, top_n);
}
