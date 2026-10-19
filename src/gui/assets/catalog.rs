use crate::game::Piece;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Folder with the generated SVGs (see scripts/generate_assets.py)
const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

/// How the board looks, matching the files in assets/boards
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoardStyle {
    Walnut,
    Mahogany,
    Ebony,
    Oak,
    Tournament,
    Slate,
    Classic,
    Marble,
    Onyx,
}

impl BoardStyle {
    pub const ALL: [BoardStyle; 9] = [
        BoardStyle::Walnut,
        BoardStyle::Mahogany,
        BoardStyle::Ebony,
        BoardStyle::Oak,
        BoardStyle::Tournament,
        BoardStyle::Slate,
        BoardStyle::Classic,
        BoardStyle::Marble,
        BoardStyle::Onyx,
    ];

    /// Return the name shown in the settings
    pub fn label(self) -> &'static str {
        match self {
            BoardStyle::Walnut => "Walnut",
            BoardStyle::Mahogany => "Mahogany",
            BoardStyle::Ebony => "Ebony",
            BoardStyle::Oak => "Oak",
            BoardStyle::Tournament => "Tournament",
            BoardStyle::Slate => "Slate",
            BoardStyle::Classic => "Classic",
            BoardStyle::Marble => "Marble",
            BoardStyle::Onyx => "Onyx",
        }
    }

    /// Return the path of this board's SVG
    pub fn path(self) -> PathBuf {
        let file = format!("{}.svg", self.label().to_lowercase());
        PathBuf::from(ASSETS_DIR).join("boards").join(file)
    }
}

/// How the pieces look, matching the folders in assets/pieces
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PieceStyle {
    Classic,
    Ivory,
    Wood,
    // Replaced the slate set, so settings saved with it still load
    #[serde(alias = "Slate")]
    Marble,
    Amber,
    Jade,
}

impl PieceStyle {
    pub const ALL: [PieceStyle; 6] = [
        PieceStyle::Classic,
        PieceStyle::Ivory,
        PieceStyle::Wood,
        PieceStyle::Marble,
        PieceStyle::Amber,
        PieceStyle::Jade,
    ];

    /// Return the name shown in the settings
    pub fn label(self) -> &'static str {
        match self {
            PieceStyle::Classic => "Black and red",
            PieceStyle::Ivory => "Ebony and ivory",
            PieceStyle::Wood => "Wood",
            PieceStyle::Marble => "Marble",
            PieceStyle::Amber => "Lapis and amber",
            PieceStyle::Jade => "Jade",
        }
    }

    /// Return the path of the SVG for one piece in this style
    pub fn path(self, piece: Piece) -> PathBuf {
        let folder = match self {
            PieceStyle::Classic => "classic",
            PieceStyle::Ivory => "ivory",
            PieceStyle::Wood => "wood",
            PieceStyle::Marble => "marble",
            PieceStyle::Amber => "amber",
            PieceStyle::Jade => "jade",
        };

        // Black plays a set's dark pieces, White plays its light ones
        let file = match piece {
            Piece::BlackMan => "dark_man.svg",
            Piece::BlackKing => "dark_king.svg",
            Piece::WhiteMan => "light_man.svg",
            Piece::WhiteKing => "light_king.svg",
        };

        PathBuf::from(ASSETS_DIR)
            .join("pieces")
            .join(folder)
            .join(file)
    }
}

/// Markers drawn over squares, matching the files in assets/overlays. They're white, and the app tints them
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Overlay {
    // An empty square the selected piece can step to
    Move,
    // A square the selected piece can land on by jumping
    Capture,
    // The square of the selected piece
    Selected,
}

impl Overlay {
    /// Return the path of this marker's SVG
    pub fn path(self) -> PathBuf {
        let file = match self {
            Overlay::Move => "move.svg",
            Overlay::Capture => "capture.svg",
            Overlay::Selected => "selected.svg",
        };
        PathBuf::from(ASSETS_DIR).join("overlays").join(file)
    }
}

/// Icons for buttons and labels, matching the files in assets/icons. They're white, and the app tints them
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Back,
    Board,
    DarkMode,
    Draw,
    Exit,
    FlipBoard,
    HideMoves,
    Home,
    LightMode,
    Loading,
    MoveLimit,
    NewGame,
    OnePlayer,
    Pieces,
    Play,
    Position,
    Redo,
    Repetition,
    Settings,
    ShowMoves,
    TwoPlayers,
    Undo,
    Warning,
    Win,
}

impl Icon {
    pub const ALL: [Icon; 24] = [
        Icon::Back,
        Icon::Board,
        Icon::DarkMode,
        Icon::Draw,
        Icon::Exit,
        Icon::FlipBoard,
        Icon::HideMoves,
        Icon::Home,
        Icon::LightMode,
        Icon::Loading,
        Icon::MoveLimit,
        Icon::NewGame,
        Icon::OnePlayer,
        Icon::Pieces,
        Icon::Play,
        Icon::Position,
        Icon::Redo,
        Icon::Repetition,
        Icon::Settings,
        Icon::ShowMoves,
        Icon::TwoPlayers,
        Icon::Undo,
        Icon::Warning,
        Icon::Win,
    ];

    /// Return the path of this icon's SVG
    pub fn path(self) -> PathBuf {
        let file = match self {
            Icon::Back => "back.svg",
            Icon::Board => "board.svg",
            Icon::DarkMode => "dark_mode.svg",
            Icon::Draw => "draw.svg",
            Icon::Exit => "exit.svg",
            Icon::FlipBoard => "flip_board.svg",
            Icon::HideMoves => "hide_moves.svg",
            Icon::Home => "home.svg",
            Icon::LightMode => "light_mode.svg",
            Icon::Loading => "loading.svg",
            Icon::MoveLimit => "move_limit.svg",
            Icon::NewGame => "new_game.svg",
            Icon::OnePlayer => "one_player.svg",
            Icon::Pieces => "pieces.svg",
            Icon::Play => "play.svg",
            Icon::Position => "position.svg",
            Icon::Redo => "redo.svg",
            Icon::Repetition => "repetition.svg",
            Icon::Settings => "settings.svg",
            Icon::ShowMoves => "show_moves.svg",
            Icon::TwoPlayers => "two_players.svg",
            Icon::Undo => "undo.svg",
            Icon::Warning => "warning.svg",
            Icon::Win => "win.svg",
        };
        PathBuf::from(ASSETS_DIR).join("icons").join(file)
    }
}

/// Return the path of the main menu's logo, in the dark or light version
pub fn logo_path(dark_mode: bool) -> PathBuf {
    let file = if dark_mode {
        "logo_dark.svg"
    } else {
        "logo_light.svg"
    };
    PathBuf::from(ASSETS_DIR).join("ui").join(file)
}

/// Return the path of the backdrop behind every screen, in the dark or light version
pub fn background_path(dark_mode: bool) -> PathBuf {
    let file = if dark_mode {
        "background_dark.svg"
    } else {
        "background_light.svg"
    };
    PathBuf::from(ASSETS_DIR).join("ui").join(file)
}
