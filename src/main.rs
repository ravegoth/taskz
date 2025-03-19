use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use strsim::levenshtein;
use colored::Colorize;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    description: String,
    created_at: i64,
}

impl Task {
    fn new(description: String) -> Task {
        Task {
            description,
            created_at: Utc::now().timestamp(),
        }
    }
}

fn get_tasks_file_path() -> io::Result<PathBuf> {
    let mut base_dir = if cfg!(target_os = "windows") {
        PathBuf::from(env::var("LOCALAPPDATA").unwrap_or_else(|_| "C:\\temp".to_string()))
    } else {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".local/share")
    };
    base_dir.push("taskz");
    fs::create_dir_all(&base_dir)?;
    base_dir.push("tasks.json");
    Ok(base_dir)
}

fn get_undo_file_path() -> io::Result<PathBuf> {
    let mut base_dir = if cfg!(target_os = "windows") {
        PathBuf::from(env::var("LOCALAPPDATA").unwrap_or_else(|_| "C:\\temp".to_string()))
    } else {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".local/share")
    };
    base_dir.push("taskz");
    fs::create_dir_all(&base_dir)?;
    base_dir.push("undo.json");
    Ok(base_dir)
}

fn load_tasks() -> io::Result<Vec<Task>> {
    let path = get_tasks_file_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let data = fs::read_to_string(&path)?;
    let tasks: Vec<Task> = serde_json::from_str(&data).unwrap_or_else(|_| vec![]);
    Ok(tasks)
}

fn save_tasks(tasks: &Vec<Task>) -> io::Result<()> {
    let path = get_tasks_file_path()?;
    let data = serde_json::to_string_pretty(tasks)?;
    fs::write(path, data)?;
    Ok(())
}

fn install() -> io::Result<()> {
    let current_exe = env::current_exe()?;
    let target_path = if cfg!(target_os = "windows") {
        PathBuf::from("C:\\Windows\\System32\\taskz.exe")
    } else {
        PathBuf::from("/usr/local/bin/taskz")
    };
    fs::copy(&current_exe, &target_path).map_err(|e| {
        eprintln!("{}", "run as administrator".red());
        e
    })?;
    println!("{}", format!("installed successfully to {:?}", target_path).green());
    Ok(())
}

fn uninstall() -> io::Result<()> {
    let target_path = if cfg!(target_os = "windows") {
        PathBuf::from("C:\\Windows\\System32\\taskz.exe")
    } else {
        PathBuf::from("/usr/local/bin/taskz")
    };
    if target_path.exists() {
        fs::remove_file(&target_path).map_err(|e| {
            eprintln!("{}", "run as administrator".red());
            e
        })?;
        println!("{}", format!("uninstalled successfully from {:?}", target_path).green());
    } else {
        println!("{}", "no installation found".red());
    }
    Ok(())
}

fn add_task(description: String) -> io::Result<()> {
    let mut tasks = load_tasks()?;
    tasks.push(Task::new(description));
    save_tasks(&tasks)?;
    println!("{}", "task added".green());
    Ok(())
}

fn list_tasks(alphabetical: bool) -> io::Result<()> {
    let mut tasks = load_tasks()?;
    if alphabetical {
        tasks.sort_by(|a, b| a.description.to_lowercase().cmp(&b.description.to_lowercase()));
    } else {
        tasks.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    }
    if tasks.is_empty() {
        println!("{}", "no tasks found".red());
    } else {
        for task in tasks {
            println!("{}", format!("[{}] {}", task.created_at, task.description).cyan());
        }
    }
    Ok(())
}

fn search_tasks(query: String) -> io::Result<()> {
    let tasks = load_tasks()?;
    let query_lower = query.to_lowercase();
    let filtered: Vec<&Task> = tasks.iter().filter(|task| task.description.to_lowercase().contains(&query_lower)).collect();
    if filtered.is_empty() {
        println!("{}", format!("no tasks found matching \"{}\"", query).red());
    } else {
        for task in filtered {
            println!("{}", format!("[{}] {}", task.created_at, task.description).cyan());
        }
    }
    Ok(())
}

fn find_closest_task(tasks: &[Task], query: &str) -> Option<usize> {
    tasks.iter().enumerate().min_by_key(|(_, task)| levenshtein(&task.description.to_lowercase(), &query.to_lowercase())).map(|(i, _)| i)
}

fn mark_done(query: String) -> io::Result<()> {
    let mut tasks = load_tasks()?;
    if let Some(index) = find_closest_task(&tasks, &query) {
        let removed = tasks.remove(index);
        save_tasks(&tasks)?;
        let undo_path = get_undo_file_path()?;
        let data = serde_json::to_string_pretty(&removed)?;
        fs::write(undo_path, data)?;
        println!("{}", format!("task done and removed: {}", removed.description).green());
    } else {
        println!("{}", "no matching task found".red());
    }
    Ok(())
}

fn undo_last() -> io::Result<()> {
    let undo_path = get_undo_file_path()?;
    if !undo_path.exists() {
        println!("{}", "no undo available".red());
        return Ok(());
    }
    let data = fs::read_to_string(&undo_path)?;
    let last_task: Task = serde_json::from_str(&data).unwrap_or_else(|_| {
        println!("{}", "failed to parse undo data".red());
        std::process::exit(1);
    });
    let mut tasks = load_tasks()?;
    tasks.push(last_task.clone());
    save_tasks(&tasks)?;
    fs::remove_file(undo_path)?;
    println!("{}", "undo successful: task restored".green());
    Ok(())
}

fn edit_task(query: String, new_description: String) -> io::Result<()> {
    let mut tasks = load_tasks()?;
    if let Some(index) = find_closest_task(&tasks, &query) {
        tasks[index].description = new_description.clone();
        save_tasks(&tasks)?;
        println!("{}", format!("task updated to: {}", new_description).green());
    } else {
        println!("{}", "no matching task found".red());
    }
    Ok(())
}

fn clear_tasks() -> io::Result<()> {
    save_tasks(&Vec::<Task>::new())?;
    println!("{}", "all tasks cleared".green());
    Ok(())
}

fn print_help() {
    println!("taskz - ultimate minimalistic todo list app in rust");
    println!();
    println!("usage:");
    println!("  taskz -i                    install the app globally");
    println!("  taskz -u                    uninstall the app");
    println!("  taskz add <task>            add a new task");
    println!("  taskz list [-a]             list tasks (use -a for alphabetical order)");
    println!("  taskz search <query>        search for tasks containing the query");
    println!("  taskz done <task>           mark the task as done (and remove it)");
    println!("  taskz undo                  undo the last removal");
    println!("  taskz edit <old> /// <new>  edit a task");
    println!("  taskz clear                 clear all tasks");
    println!("  taskz /? | -? | -h          show this help");
    println!();
    println!("made by tra1an.com");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("{}", "no command provided. usage: taskz [options]".red());
        return;
    }
    match args[1].as_str() {
        "-i" => {
            if let Err(e) = install() {
                eprintln!("{}", format!("installation failed: {}", e).red());
            }
        },
        "-u" => {
            if let Err(e) = uninstall() {
                eprintln!("{}", format!("uninstallation failed: {}", e).red());
            }
        },
        "add" => {
            if args.len() < 3 {
                eprintln!("{}", "please provide a task description".red());
                return;
            }
            let description = args[2..].join(" ");
            if let Err(e) = add_task(description) {
                eprintln!("{}", format!("failed to add task: {}", e).red());
            }
        },
        "list" => {
            let alphabetical = args.contains(&"-a".to_string());
            if let Err(e) = list_tasks(alphabetical) {
                eprintln!("{}", format!("failed to list tasks: {}", e).red());
            }
        },
        "search" => {
            if args.len() < 3 {
                eprintln!("{}", "please provide a search query".red());
                return;
            }
            let query = args[2..].join(" ");
            if let Err(e) = search_tasks(query) {
                eprintln!("{}", format!("failed to search tasks: {}", e).red());
            }
        },
        "done" => {
            if args.len() < 3 {
                eprintln!("{}", "please provide the task to mark as done".red());
                return;
            }
            let query = args[2..].join(" ");
            if let Err(e) = mark_done(query) {
                eprintln!("{}", format!("failed to mark task as done: {}", e).red());
            }
        },
        "undo" => {
            if let Err(e) = undo_last() {
                eprintln!("{}", format!("failed to undo: {}", e).red());
            }
        },
        "edit" => {
            let joined = args[2..].join(" ");
            let parts: Vec<&str> = joined.split("///").map(|s| s.trim()).collect();
            if parts.len() != 2 {
                eprintln!("{}", "please provide the edit command in format: taskz edit <query> /// <new description>".red());
                return;
            }
            let query = parts[0].to_string();
            let new_description = parts[1].to_string();
            if let Err(e) = edit_task(query, new_description) {
                eprintln!("{}", format!("failed to edit task: {}", e).red());
            }
        },
        "clear" => {
            if let Err(e) = clear_tasks() {
                eprintln!("{}", format!("failed to clear tasks: {}", e).red());
            }
        },
        "/?" | "-?" | "-h" => {
            print_help();
        },
        _ => {
            eprintln!("{}", "unknown command".red());
        }
    }
}
