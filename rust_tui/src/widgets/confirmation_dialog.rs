use crate::widgets::modal::{Modal, ModalStyle};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    Frame,
};

#[derive(Debug, Clone)]
pub enum ConfirmationType {
    DeleteThread,
    DeleteEntry,
    DiscardChanges,
    DiscardNewEntry,
    RemoveImage,
    ReplaceImage,
    CharacterLimit,
}

pub struct ConfirmationDialog<'a> {
    confirmation_type: ConfirmationType,
    custom_message: Option<&'a str>,
    custom_options: Option<&'a str>,
}

impl<'a> ConfirmationDialog<'a> {
    pub fn new(confirmation_type: ConfirmationType) -> Self {
        Self {
            confirmation_type,
            custom_message: None,
            custom_options: None,
        }
    }

    pub fn with_custom_message(mut self, message: &'a str) -> Self {
        self.custom_message = Some(message);
        self
    }

    pub fn with_custom_options(mut self, options: &'a str) -> Self {
        self.custom_options = Some(options);
        self
    }

    pub fn render(&self, f: &mut Frame, background_area: Rect) -> Rect {
        let (title, message, options, style) = match self.confirmation_type {
            ConfirmationType::DeleteThread => (
                "Confirm Delete",
                "Are you sure you want to delete this thread?\n\nThis will delete the thread and all its entries.",
                "[Y] Yes   [N] No   [Esc] Cancel",
                ModalStyle::Warning,
            ),
            ConfirmationType::DeleteEntry => (
                "Confirm Delete",
                "Are you sure you want to delete this entry?",
                "[Y] Yes   [N] No   [Esc] Cancel",
                ModalStyle::Warning,
            ),
            ConfirmationType::DiscardChanges => (
                "Discard Changes?",
                "You have unsaved changes to the entry content.\n\nDiscard changes and exit?",
                "[Y] Yes   [N] No   [Esc] Cancel",
                ModalStyle::Warning,
            ),
            ConfirmationType::DiscardNewEntry => (
                "Discard New Entry?",
                "You have unsaved content for this new entry.\n\nDiscard new entry and exit?",
                "[Y] Yes   [N] No   [Esc] Cancel",
                ModalStyle::Warning,
            ),
            ConfirmationType::RemoveImage => (
                "Remove Image",
                "Are you sure you want to remove this image?\nThis action cannot be undone.",
                "[Y] Yes, Remove   [N] No   [Esc] Cancel",
                ModalStyle::Error,
            ),
            ConfirmationType::ReplaceImage => (
                "Image Already Exists",
                "This entry already has an image attached.\nWhat would you like to do?",
                "[R] Replace   [D] Delete Current   [Esc] Cancel",
                ModalStyle::Warning,
            ),
            ConfirmationType::CharacterLimit => (
                "Error",
                "Content exceeds 500 character limit.\nPlease shorten your text and try again.",
                "[Enter] OK",
                ModalStyle::Error,
            ),
        };

        let message_to_use = self.custom_message.unwrap_or(message);
        let options_to_use = self.custom_options.unwrap_or(options);

        let height = match self.confirmation_type {
            ConfirmationType::CharacterLimit => 6,
            ConfirmationType::ReplaceImage => 8,
            _ => 7,
        };

        let modal = Modal::new(title)
            .style(style)
            .size(60, height)
            .add_spacing(1)
            .add_text(message_to_use, Alignment::Center, Style::default())
            .add_spacing(1)
            .add_text(
                options_to_use,
                Alignment::Center,
                match self.confirmation_type {
                    ConfirmationType::DeleteThread | ConfirmationType::DeleteEntry => {
                        Style::default().fg(Color::Red)
                    }
                    ConfirmationType::RemoveImage => Style::default().fg(Color::Cyan),
                    _ => Style::default().fg(Color::Cyan),
                },
            );

        modal.render(f, background_area)
    }
}