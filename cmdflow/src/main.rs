use std::collections::{HashMap, HashSet};
use std::fs::{OpenOptions, read_to_string};
use std::io::Write;
use std::path::PathBuf;
use colored::*;
use std::env;

fn main() {

    let args: Vec<String> = env::args().collect();

    // ---------- --version ----------
    if args.iter().any(|x| x == "--version") {
        println!("cmdflow v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let top_n: usize = if args.len() > 1 {
        args[1].parse().unwrap_or(10)
    } else {
        10
    };

    let _max_bar_len: usize = 20; // Максимальная длина бара


    let cmdflow_log: PathBuf = dirs::home_dir().unwrap().join(".local/share/cmdflow/log");
    let fish_history: PathBuf = dirs::home_dir().unwrap().join(".local/share/fish/fish_history");

    // ---------- Автокешинг ----------
    let mut existing_cmds = HashSet::new();
    if let Ok(existing_log) = read_to_string(&cmdflow_log) {
        for line in existing_log.lines() {
            existing_cmds.insert(line.trim().to_string());
        }
    }

    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&cmdflow_log)
        .unwrap();

    if let Ok(content) = read_to_string(&fish_history) {
        for line in content.lines() {
            if line.starts_with("- cmd: ") {
                let cmd_full = line[7..].trim();
                if !existing_cmds.contains(cmd_full) {
                    writeln!(log_file, "{}", cmd_full).unwrap();
                    existing_cmds.insert(cmd_full.to_string());
                }
            }
        }
    }

    // ---------- Подсчёт ----------
    let mut counter = HashMap::new();
    if let Ok(content) = read_to_string(&cmdflow_log) {
        for line in content.lines() {
            let cmd = line.split_whitespace().next().unwrap_or("").to_lowercase();
            if !cmd.is_empty() {
                *counter.entry(cmd).or_insert(0) += 1;
            }
        }
    }

    if counter.is_empty() {
        println!("Empty Log. Try to run some commands in Fish shell.");
        return;
    }

    let mut vec: Vec<_> = counter.into_iter().collect();
    vec.sort_by(|a, b| b.1.cmp(&a.1));
    let max_count = vec.get(0).map(|(_, c)| *c).unwrap_or(1);

    println!("{}", format!("Top {} command Fish:", top_n).bold().underline());

    // ---------- Мягкая радужная аллея ----------
    let soft_rainbow = [
        (200, 150, 255), (180, 180, 255), (150, 220, 255),
        (150, 255, 200), (180, 255, 150), (220, 255, 150),
        (255, 220, 150), (255, 180, 150), (255, 150, 180),
    ];

    for (i, (cmd, count)) in vec.into_iter().take(top_n).enumerate() {
    // Пропускаем команды с именем длиннее 11 символов
    if cmd.len() > 11 {
        continue;
    }

    // Вычисляем длину бара пропорционально
    let bar_len = ((count as f32 / max_count as f32) * 11.0).round() as usize;
    let bar = "███".repeat(bar_len.max(1));

    let line = format!("{:>5} │ {:<12} {}", count, cmd, bar);

    let colored_line = match i {
        0 => line.truecolor(255, 215, 0).bold(),      // золото
        1 => line.truecolor(192, 192, 192).bold(),    // серебро
        2 => line.truecolor(205, 127, 50).bold(),     // бронза
        _ => {
            let color = soft_rainbow[(i - 3) % soft_rainbow.len()];
            line.truecolor(color.0, color.1, color.2)
        }
    };

    println!("{}", colored_line);
    }
}
