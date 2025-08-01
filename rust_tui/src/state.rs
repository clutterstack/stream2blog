#[derive(Clone, Debug)]
pub enum AppState {
    ThreadList,
    ThreadView(String), // thread_id
    CreateThread,
    CreateEntry(String),                 // thread_id
    EditThread(String),                  // thread_id
    EditEntry(String, String),           // thread_id, entry_id
    ConfirmDeleteThread(String),         // thread_id
    ConfirmDeleteEntry(String, String),  // thread_id, entry_id
    ConfirmDiscardEntryChanges(String, String), // thread_id, entry_id
    ConfirmDiscardNewEntry(String), // thread_id
    ImageNaming(Box<AppState>, Vec<u8>), // previous_state, image_data
    ConfirmImageReplacement(Box<AppState>, Vec<u8>, String), // previous_state, new_image_data, current_image_path
    ConfirmImageRemoval(Box<AppState>, String), // previous_state, current_image_path
    CharacterLimitError(Box<AppState>), // previous_state
}
