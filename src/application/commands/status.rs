use crate::models::common::Priority;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;
use std::collections::HashMap;

pub fn cmd_status() -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ListStorage::new(&paths.todo_file)?;
    let list = storage.list();

    // Calculate statistics
    let total_tasks = list.tasks.len();
    let completed_tasks = list.tasks.iter().filter(|t| t.completed).count();
    let open_tasks = total_tasks - completed_tasks;

    // Priority breakdown (open tasks only)
    let mut priority_counts = HashMap::new();
    for task in list.tasks.iter().filter(|t| !t.completed) {
        *priority_counts.entry(&task.priority).or_insert(0) += 1;
    }

    // Tag statistics
    let mut tag_counts: HashMap<String, (usize, usize)> = HashMap::new(); // (open, completed)
    for task in &list.tasks {
        for tag in &task.tags {
            let entry = tag_counts.entry(tag.clone()).or_insert((0, 0));
            if task.completed {
                entry.1 += 1;
            } else {
                entry.0 += 1;
            }
        }
    }

    println!("Project: {} {}\n", list.project_name, list.project_version);

    if total_tasks > 0 {
        let completion_rate = (completed_tasks as f64 / total_tasks as f64) * 100.0;
        println!("{} Task(s), {}% done:", total_tasks, completion_rate);
    } else {
        println!("{} Task(s):", completed_tasks);
    }

    println!("  Open: {}", open_tasks);
    println!("  Done: {}", completed_tasks);

    if open_tasks > 0 {
        println!("\nOpen Tasks:");
        let high = priority_counts.get(&Priority::High).unwrap_or(&0);
        let medium = priority_counts.get(&Priority::Medium).unwrap_or(&0);
        let low = priority_counts.get(&Priority::Low).unwrap_or(&0);

        if *high > 0 {
            println!("  High:   {}", high);
        }
        if *medium > 0 {
            println!("  Medium: {}", medium);
        }
        if *low > 0 {
            println!("  Low:    {}", low);
        }
    }

    if !tag_counts.is_empty() {
        println!("\nTags:");
        let mut sorted_tags: Vec<_> = tag_counts.iter().collect();
        sorted_tags.sort_by(|a, b| {
            let a_total = a.1.0 + a.1.1;
            let b_total = b.1.0 + b.1.1;
            b_total.cmp(&a_total).then_with(|| a.0.cmp(b.0))
        });

        for (tag, (open, completed)) in sorted_tags.iter().take(10) {
            let total = *open + *completed;
            if *open > 0 {
                println!(
                    "  #{}: {} open, {} done ({} total)",
                    tag, open, completed, total
                );
            } else {
                println!("  #{}: {} done", tag, completed);
            }
        }

        if sorted_tags.len() > 10 {
            println!("  ... and {} more", sorted_tags.len() - 10);
        }
    }

    Ok(())
}
