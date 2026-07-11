use std::collections::HashMap;
use std::fs::{self, read_to_string, OpenOptions, read};
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

fn rebuild_log(src: &std::path::Path, dst: &std::path::Path, shell_type: &str) {
    if !src.exists() { return; }

    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(dst)
        .unwrap();

    if shell_type == "zsh" {
        if let Ok(bytes) = read(src) {
            let content = String::from_utf8_lossy(&bytes);
            for line in content.lines() {
                let l = line.trim();
                if l.is_empty() { continue; }
                
                let cmd = if l.starts_with(':') {
                    l.find(';').map(|idx| l[idx + 1..].trim())
                } else {
                    Some(l)
                };

                if let Some(cmd) = cmd {
                    if !cmd.is_empty() {
                        let _ = writeln!(log_file, "{}", cmd);
                    }
                }
            }
        }
        return;
    }

    if let Ok(content) = read_to_string(src) {
        for line in content.lines() {
            let cmd = if shell_type == "fish" {
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

fn fix_shell_histories(home: &std::path::Path) {
    println!("{}", ":: Checking shell configurations...".bold().blue());

    // Bash Configuration
    let bashrc_path = home.join(".bashrc");
    if bashrc_path.exists() {
        if let Ok(content) = read_to_string(&bashrc_path) {
            if !content.contains("export HISTCONTROL=") {
                let mut file = OpenOptions::new().append(true).open(&bashrc_path).unwrap();
                let _ = writeln!(file, "\n# Added by cmdflow\nexport HISTCONTROL=\nexport PROMPT_COMMAND=\"history -a; $PROMPT_COMMAND\"");
                println!("✓ Bash settings updated in ~/.bashrc");
            } else {
                println!("✓ Settings are already configured in ~/.bashrc");
            }
        }
    }

    // Fish Configuration
    let fish_config_path = home.join(".config/fish/config.fish");
    if fish_config_path.exists() {
        if let Ok(content) = read_to_string(&fish_config_path) {
            if !content.contains("history_merge_on_enter") {
                let mut file = OpenOptions::new().append(true).open(&fish_config_path).unwrap();
                let fish_hook = r#"
# Added by cmdflow
function history_merge_on_enter --on-event fish_postexec
    history merge
end
"#;
                let _ = writeln!(file, "{}", fish_hook);
                println!("✓ History recording hook added to ~/.config/fish/config.fish");
            } else {
                println!("✓ Hook is already present in config.fish");
            }
        }
    }

    // Zsh Configuration
    let zshrc_path = home.join(".zshrc");
    if zshrc_path.exists() {
        if let Ok(content) = read_to_string(&zshrc_path) {
            if !content.contains("# Added by cmdflow") {
                let mut file = OpenOptions::new().append(true).open(&zshrc_path).unwrap();
                let zsh_hook = r#"
# Added by cmdflow
unsetopt HIST_IGNORE_DUPS
unsetopt HIST_IGNORE_ALL_DUPS
unsetopt HIST_SAVE_BY_COPY
setopt APPEND_HISTORY
setopt INC_APPEND_HISTORY
"#;
                let _ = writeln!(file, "{}", zsh_hook);
                println!("✓ Zsh settings updated in ~/.zshrc");
            } else {
                println!("✓ Settings are already configured in ~/.zshrc");
            }
        }
    }
    
    println!("\n{}", "Done! Please restart your terminals for changes to take effect.".green().bold());
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

        let c = soft_rainbow[i % soft_rainbow.len()];
        println!("{}", line.truecolor(c.0, c.1, c.2));
    }

    println!();
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    const FLAGS: &[(&str, &str)] = &[
        ("--fish", "Read only fish commands"),
        ("--bash", "Read only bash commands"),
        ("--zsh", "Read only zsh commands"),
        ("--working", "Show top N working commands"),
        ("--broken", "Show top N broken commands"),
        ("--fix-history", "Inject duplicate-saving hooks into shell configs"),
        ("--version", "Show program version"),
        ("--help", "Show this help message"),
    ];

    let home = match dirs::home_dir() {
        Some(h) => h,
        None => { eprintln!("Could not determine home directory."); return; }
    };

    // Process system flags
    if args.iter().any(|a| a == "--help") {
        println!("Usage:");
        for (flag, description) in FLAGS {
            println!("  {:<14} {}", flag, description);
        }
        return;
    }

    if let Some(unknown) = args.iter().find(|a| {
        a.starts_with('-') && !FLAGS.iter().any(|(flag, _)| flag == &a.as_str())
    }) {
        eprintln!("Unknown option: {}", unknown);
        eprintln!("Use --help for usage.");
        return;
    }

    if args.iter().any(|a| a == "--version") {
        println!("cmdflow v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if args.iter().any(|a| a == "--fix-history") {
        fix_shell_histories(&home);
        return;
    }

    // Initialize shell flags
    let has_fish = args.contains(&"--fish".to_string());
    let has_bash = args.contains(&"--bash".to_string());
    let has_zsh = args.contains(&"--zsh".to_string());
    
    let no_flags = !has_fish && !has_bash && !has_zsh;
    let use_fish = has_fish || no_flags;
    let use_bash = has_bash || no_flags;
    let use_zsh = has_zsh || no_flags;

    let top_n: usize = args.iter().filter_map(|a| a.parse::<usize>().ok()).next().unwrap_or(10);

    // Paths to history files
    let fish_history = home.join(".local/share/fish/fish_history");
    let bash_history = home.join(".bash_history");
    let zsh_history = home.join(".zsh_history");
    
    let cmdflow_dir = home.join(".local/share/cmdflow");
    let fish_log = cmdflow_dir.join("fish.log");
    let bash_log = cmdflow_dir.join("bash.log");
    let zsh_log = cmdflow_dir.join("zsh.log");

    if let Err(_) = fs::create_dir_all(&cmdflow_dir) {
        eprintln!("Failed to create cmdflow directory");
        return;
    }

    let _ = fs::remove_file(&fish_log);
    let _ = fs::remove_file(&bash_log);
    let _ = fs::remove_file(&zsh_log);

    if use_fish { rebuild_log(&fish_history, &fish_log, "fish"); }
    if use_bash { rebuild_log(&bash_history, &bash_log, "bash"); }
    if use_zsh { rebuild_log(&zsh_history, &zsh_log, "zsh"); }

    // Count commands
    let mut counter: HashMap<String, usize> = HashMap::new();
    let mut combined_logs = vec![];
    
    if use_fish && fish_log.exists() { combined_logs.extend(read_log(&fish_log)); }
    if use_bash && bash_log.exists() { combined_logs.extend(read_log(&bash_log)); }
    if use_zsh && zsh_log.exists() { combined_logs.extend(read_log(&zsh_log)); }

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

    // Sorting and filtering
    let mut vec_main: Vec<_> = counter.iter().map(|(cmd, &c)| (cmd.clone(), c)).collect();

    if args.contains(&"--working".to_string()) {
        vec_main = vec_main.into_iter().filter(|(cmd, _)| is_working(cmd)).collect();
    } else if args.contains(&"--broken".to_string()) {
        vec_main = vec_main.into_iter().filter(|(cmd, _)| !is_working(cmd)).collect();
    }

    vec_main.sort_by(|a, b| b.1.cmp(&a.1));

    // Output results
    let mut active_shells = Vec::new();
    if use_fish { active_shells.push("fish"); }
    if use_bash { active_shells.push("bash"); }
    if use_zsh { active_shells.push("zsh"); }
    let mode = active_shells.join(" + ");

    let arg_label = if args.contains(&"--working".to_string()) {
        "Working Commands"
    } else if args.contains(&"--broken".to_string()) {
        "Broken Commands"
    } else {
        "Commands"
    };

    print_top(&format!("Top {} {} ({})", top_n, arg_label, mode), &vec_main, top_n);
}
