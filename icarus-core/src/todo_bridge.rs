// TodoWrite Bridge
// Enables task tracking within Markovian reasoning sessions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Todo item state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

/// A todo item in a reasoning session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    /// Todo content/description
    pub content: String,

    /// Current status
    pub status: TodoStatus,

    /// Active form (present continuous: "Doing X")
    pub active_form: String,

    /// Optional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Todo list for a session
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TodoList {
    /// Session ID this todo list belongs to
    pub session_id: Uuid,

    /// Ordered list of todos
    pub todos: Vec<TodoItem>,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl TodoList {
    pub fn new(session_id: Uuid) -> Self {
        let now = chrono::Utc::now();
        Self {
            session_id,
            todos: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_todo(&mut self, content: String, active_form: String) {
        self.todos.push(TodoItem {
            content,
            status: TodoStatus::Pending,
            active_form,
            metadata: serde_json::Value::Null,
        });
        self.updated_at = chrono::Utc::now();
    }

    pub fn update_status(&mut self, index: usize, status: TodoStatus) -> anyhow::Result<()> {
        if index >= self.todos.len() {
            anyhow::bail!("Todo index {} out of bounds (total: {})", index, self.todos.len());
        }
        self.todos[index].status = status;
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    pub fn get_summary(&self) -> TodoSummary {
        let total = self.todos.len();
        let completed = self.todos.iter().filter(|t| t.status == TodoStatus::Completed).count();
        let in_progress = self.todos.iter().filter(|t| t.status == TodoStatus::InProgress).count();
        let pending = self.todos.iter().filter(|t| t.status == TodoStatus::Pending).count();

        TodoSummary {
            total,
            completed,
            in_progress,
            pending,
        }
    }
}

/// Summary of todo list status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoSummary {
    pub total: usize,
    pub completed: usize,
    pub in_progress: usize,
    pub pending: usize,
}

/// TodoWrite Bridge - manages todo lists across sessions
#[derive(Clone)]
pub struct TodoBridge {
    /// Todo lists by session ID
    todos: Arc<Mutex<HashMap<Uuid, TodoList>>>,
}

impl TodoBridge {
    pub fn new() -> Self {
        Self {
            todos: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create or update todo list for a session
    pub fn set_todos(&self, session_id: Uuid, todos: Vec<TodoItem>) -> anyhow::Result<()> {
        let mut map = self.todos.lock().unwrap();
        let todo_list = map.entry(session_id).or_insert_with(|| TodoList::new(session_id));
        todo_list.todos = todos;
        todo_list.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// Get todo list for a session
    pub fn get_todos(&self, session_id: &Uuid) -> Option<TodoList> {
        let map = self.todos.lock().unwrap();
        map.get(session_id).cloned()
    }

    /// Add a single todo to a session
    pub fn add_todo(&self, session_id: Uuid, content: String, active_form: String) -> anyhow::Result<()> {
        let mut map = self.todos.lock().unwrap();
        let todo_list = map.entry(session_id).or_insert_with(|| TodoList::new(session_id));
        todo_list.add_todo(content, active_form);
        Ok(())
    }

    /// Update todo status
    pub fn update_todo_status(&self, session_id: &Uuid, index: usize, status: TodoStatus) -> anyhow::Result<()> {
        let mut map = self.todos.lock().unwrap();
        let todo_list = map.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("No todo list found for session {}", session_id))?;
        todo_list.update_status(index, status)
    }

    /// Get todo summary for a session
    pub fn get_summary(&self, session_id: &Uuid) -> Option<TodoSummary> {
        let map = self.todos.lock().unwrap();
        map.get(session_id).map(|list| list.get_summary())
    }

    /// List all sessions with todos
    pub fn list_sessions(&self) -> Vec<Uuid> {
        let map = self.todos.lock().unwrap();
        map.keys().copied().collect()
    }

    /// Clear todo list for a session
    pub fn clear_todos(&self, session_id: &Uuid) -> anyhow::Result<()> {
        let mut map = self.todos.lock().unwrap();
        map.remove(session_id)
            .ok_or_else(|| anyhow::anyhow!("No todo list found for session {}", session_id))?;
        Ok(())
    }
}

impl Default for TodoBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_list_creation() {
        let session_id = Uuid::new_v4();
        let mut list = TodoList::new(session_id);

        list.add_todo("Task 1".to_string(), "Doing task 1".to_string());
        list.add_todo("Task 2".to_string(), "Doing task 2".to_string());

        assert_eq!(list.todos.len(), 2);
        assert_eq!(list.todos[0].status, TodoStatus::Pending);
    }

    #[test]
    fn test_todo_status_update() {
        let session_id = Uuid::new_v4();
        let mut list = TodoList::new(session_id);

        list.add_todo("Task 1".to_string(), "Doing task 1".to_string());
        list.update_status(0, TodoStatus::InProgress).unwrap();

        assert_eq!(list.todos[0].status, TodoStatus::InProgress);
    }

    #[test]
    fn test_todo_summary() {
        let session_id = Uuid::new_v4();
        let mut list = TodoList::new(session_id);

        list.add_todo("Task 1".to_string(), "Doing task 1".to_string());
        list.add_todo("Task 2".to_string(), "Doing task 2".to_string());
        list.add_todo("Task 3".to_string(), "Doing task 3".to_string());

        list.update_status(0, TodoStatus::Completed).unwrap();
        list.update_status(1, TodoStatus::InProgress).unwrap();

        let summary = list.get_summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.completed, 1);
        assert_eq!(summary.in_progress, 1);
        assert_eq!(summary.pending, 1);
    }

    #[test]
    fn test_todo_bridge() {
        let bridge = TodoBridge::new();
        let session_id = Uuid::new_v4();

        bridge.add_todo(session_id, "Task 1".to_string(), "Doing task 1".to_string()).unwrap();
        bridge.add_todo(session_id, "Task 2".to_string(), "Doing task 2".to_string()).unwrap();

        let list = bridge.get_todos(&session_id).unwrap();
        assert_eq!(list.todos.len(), 2);

        bridge.update_todo_status(&session_id, 0, TodoStatus::Completed).unwrap();

        let summary = bridge.get_summary(&session_id).unwrap();
        assert_eq!(summary.completed, 1);
        assert_eq!(summary.pending, 1);
    }
}
