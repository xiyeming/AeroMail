use tauri::State;
use tracing::instrument;

use crate::AppState;
use crate::models::error::ErrorPayload;
use crate::models::todo::TodoItem;

/// Lists all persisted todos.
///
/// # Errors
///
/// Returns an error if the database query fails.
#[tauri::command]
#[instrument(skip(state), err(Debug))]
pub async fn list_todos(state: State<'_, AppState>) -> Result<Vec<TodoItem>, ErrorPayload> {
    state.db.list_todos().map_err(|e| e.to_payload())
}

/// Creates or updates a todo item.
///
/// # Errors
///
/// Returns an error if the database write fails.
#[tauri::command]
#[instrument(skip(state, todo), fields(todo_id = %todo.id), err(Debug))]
pub async fn upsert_todo(todo: TodoItem, state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    state.db.upsert_todo(&todo).map_err(|e| e.to_payload())
}

/// Deletes a todo by ID.
///
/// # Errors
///
/// Returns an error if the database write fails.
#[tauri::command]
#[instrument(skip(state), fields(todo_id = %todo_id), err(Debug))]
pub async fn delete_todo(todo_id: String, state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    state.db.delete_todo(&todo_id).map_err(|e| e.to_payload())
}

/// Deletes all completed todos.
///
/// # Errors
///
/// Returns an error if the database write fails.
#[tauri::command]
#[instrument(skip(state), err(Debug))]
pub async fn clear_completed_todos(state: State<'_, AppState>) -> Result<usize, ErrorPayload> {
    state.db.clear_completed_todos().map_err(|e| e.to_payload())
}
