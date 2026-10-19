use crate::gui::assets::{BoardStyle, PieceStyle};
use serde::{Deserialize, Serialize};

/// Key the settings are saved under between runs
const SETTINGS_KEY: &str = "settings";

/// Everything the player can change on the settings screen
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
// Settings saved by an older version still load, with any new fields at their defaults
#[serde(default)]
pub struct Settings {
    pub board_style: BoardStyle,
    pub piece_style: PieceStyle,
    pub dark_mode: bool,
    pub show_legal_moves: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            board_style: BoardStyle::Walnut,
            piece_style: PieceStyle::Classic,
            dark_mode: true,
            show_legal_moves: true,
        }
    }
}

impl Settings {
    /// Load the settings saved by the last run, or the defaults the first time
    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        storage
            .and_then(|storage| eframe::get_value(storage, SETTINGS_KEY))
            .unwrap_or_default()
    }

    /// Save the settings for the next run
    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, SETTINGS_KEY, self);
    }
}
