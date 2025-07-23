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
    CharacterLimitError(Box<AppState>), // previous_state
}
