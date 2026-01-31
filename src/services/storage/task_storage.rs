use crate::models::common::{Priority, Version};
use crate::models::tasks::{List, Task};
use crate::services::serializers::todo_serializer;
use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ListStorage {
    todo_list: List,
    list_file: PathBuf,
}

impl ListStorage {
    pub fn new(list_file: &Path) -> Result<Self> {
        let mut storage = Self {
            todo_list: List::new("", Version::new(0, 1, 0, false)),
            list_file: list_file.to_path_buf(),
        };
        storage.load_list()?;
        Ok(storage)
    }

    /// Load list from the TODO.md file
    pub fn load_list(&mut self) -> Result<()> {
        if !self.list_file.exists() {
            // Create a default list if file doesn't exist
            self.todo_list = List::new("Untitled", Version::new(0, 1, 0, false));
            return Ok(());
        }

        match fs::read_to_string(&self.list_file) {
            Ok(content) => {
                self.todo_list = todo_serializer::deserialize(&content)
                    .map_err(|e| anyhow!("Failed to parse TODO file: {}", e))?;
                Ok(())
            }
            Err(e) => Err(anyhow!("Failed to read TODO file: {}", e)),
        }
    }

    /// Save list to the TODO.md file
    pub fn save_list(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.list_file.parent() {
            fs::create_dir_all(parent).map_err(|e| anyhow!("Failed to create directory: {}", e))?;
        }

        let content = todo_serializer::serialize(&self.todo_list);

        fs::write(&self.list_file, content)
            .map_err(|e| anyhow!("Failed to write TODO file: {}", e))?;

        Ok(())
    }

    /// Get a reference to the entire list
    pub fn list(&self) -> &List {
        &self.todo_list
    }

    /// Get a mutable reference to the entire list
    pub fn list_mut(&mut self) -> &mut List {
        &mut self.todo_list
    }

    /// Add a new task and save
    pub fn add_task(&mut self, task: Task) -> Result<()> {
        self.todo_list.add_task(task);
        self.save_list()
    }

    /// Get all tasks
    pub fn tasks(&self) -> &[Task] {
        &self.todo_list.tasks
    }

    /// Get mutable reference to tasks
    pub fn tasks_mut(&mut self) -> &mut Vec<Task> {
        &mut self.todo_list.tasks
    }

    /// Remove a task by index and save
    pub fn remove_task(&mut self, index: usize) -> Result<Option<Task>> {
        if index < self.todo_list.tasks.len() {
            let task = self.todo_list.tasks.remove(index);
            self.todo_list.modified_at = chrono::Utc::now();
            self.save_list()?;
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }

    /// Mark a task as completed and save
    pub fn complete_task(&mut self, index: usize, version: Option<Version>) -> Result<()> {
        if let Some(task) = self.todo_list.tasks.get_mut(index) {
            task.completed = true;
            task.completed_at_time = Some(chrono::Utc::now());
            if let Some(v) = version {
                task.completed_at_version = Some(v);
            }
            self.todo_list.modified_at = chrono::Utc::now();
            self.save_list()?;
            Ok(())
        } else {
            Err(anyhow!("Task index {} out of bounds", index))
        }
    }

    /// Mark a task as incomplete and save
    pub fn uncomplete_task(&mut self, index: usize) -> Result<()> {
        if let Some(task) = self.todo_list.tasks.get_mut(index) {
            task.completed = false;
            task.completed_at_time = None;
            task.completed_at_version = None;
            task.completed_at_commit = None;
            self.todo_list.modified_at = chrono::Utc::now();
            self.save_list()?;
            Ok(())
        } else {
            Err(anyhow!("Task index {} out of bounds", index))
        }
    }

    /// Update task description and save
    pub fn update_task_description(&mut self, index: usize, description: String) -> Result<()> {
        if let Some(task) = self.todo_list.tasks.get_mut(index) {
            task.description = description;
            self.todo_list.modified_at = chrono::Utc::now();
            self.save_list()?;
            Ok(())
        } else {
            Err(anyhow!("Task index {} out of bounds", index))
        }
    }

    /// Update task priority and save
    pub fn update_task_priority(&mut self, index: usize, priority: Priority) -> Result<()> {
        if let Some(task) = self.todo_list.tasks.get_mut(index) {
            task.priority = priority;
            self.todo_list.modified_at = chrono::Utc::now();
            self.save_list()?;
            Ok(())
        } else {
            Err(anyhow!("Task index {} out of bounds", index))
        }
    }

    /// Add a tag to a task and save
    pub fn add_task_tag(&mut self, index: usize, tag: String) -> Result<()> {
        if let Some(task) = self.todo_list.tasks.get_mut(index) {
            if !task.tags.contains(&tag) {
                task.tags.push(tag);
                self.todo_list.modified_at = chrono::Utc::now();
                self.save_list()?;
            }
            Ok(())
        } else {
            Err(anyhow!("Task index {} out of bounds", index))
        }
    }

    /// Remove a tag from a task and save
    pub fn remove_task_tag(&mut self, index: usize, tag: &str) -> Result<()> {
        if let Some(task) = self.todo_list.tasks.get_mut(index) {
            task.tags.retain(|t| t != tag);
            self.todo_list.modified_at = chrono::Utc::now();
            self.save_list()?;
            Ok(())
        } else {
            Err(anyhow!("Task index {} out of bounds", index))
        }
    }

    /// Get tasks filtered by completion status
    pub fn tasks_by_status(&self, completed: bool) -> Vec<&Task> {
        self.todo_list
            .tasks
            .iter()
            .filter(|t| t.completed == completed)
            .collect()
    }

    /// Get tasks filtered by priority
    pub fn tasks_by_priority(&self, priority: Priority) -> Vec<&Task> {
        self.todo_list
            .tasks
            .iter()
            .filter(|t| t.priority == priority)
            .collect()
    }

    /// Get tasks filtered by tag
    pub fn tasks_by_tag(&self, tag: &str) -> Vec<&Task> {
        self.todo_list
            .tasks
            .iter()
            .filter(|t| t.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Get tasks for a specific version
    pub fn tasks_for_version(&self, version: &Version) -> Vec<&Task> {
        self.todo_list.tasks_for_version(version)
    }

    /// Get tasks between versions
    pub fn tasks_between_versions(&self, from: &Version, to: &Version) -> Vec<&Task> {
        self.todo_list.tasks_between_versions(from, to)
    }

    /// Assign version to all unversioned completed tasks and save
    pub fn assign_version_to_completed(&mut self, version: Version) -> Result<usize> {
        let count = self.todo_list.assign_version_to_completed(version);
        if count > 0 {
            self.save_list()?;
        }
        Ok(count)
    }

    /// Update the project name and save
    pub fn set_project_name(&mut self, name: String) -> Result<()> {
        self.todo_list.project_name = name;
        self.todo_list.modified_at = chrono::Utc::now();
        self.save_list()
    }

    /// Update the project version and save
    pub fn set_project_version(&mut self, version: Version) -> Result<()> {
        self.todo_list.project_version = version;
        self.todo_list.modified_at = chrono::Utc::now();
        self.save_list()
    }

    /// Get the project name
    pub fn project_name(&self) -> &str {
        &self.todo_list.project_name
    }

    /// Get the project version
    pub fn project_version(&self) -> &Version {
        &self.todo_list.project_version
    }

    /// Get the total number of tasks
    pub fn task_count(&self) -> usize {
        self.todo_list.tasks.len()
    }

    /// Get the number of completed tasks
    pub fn completed_count(&self) -> usize {
        self.todo_list.tasks.iter().filter(|t| t.completed).count()
    }

    /// Get the number of pending tasks
    pub fn pending_count(&self) -> usize {
        self.todo_list.tasks.iter().filter(|t| !t.completed).count()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.todo_list.tasks.is_empty()
    }

    /// Get the path to the list file
    pub fn list_path(&self) -> &Path {
        &self.list_file
    }

    /// Clear all tasks and save
    pub fn clear_tasks(&mut self) -> Result<()> {
        self.todo_list.tasks.clear();
        self.todo_list.modified_at = chrono::Utc::now();
        self.save_list()
    }

    /// Search tasks by description pattern
    pub fn search_tasks(&self, pattern: &str) -> Vec<&Task> {
        let pattern_lower = pattern.to_lowercase();
        self.todo_list
            .tasks
            .iter()
            .filter(|t| t.description.to_lowercase().contains(&pattern_lower))
            .collect()
    }
}
