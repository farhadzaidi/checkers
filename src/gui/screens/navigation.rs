use crate::game::{GameSession, Side};

/// Where a screen asks the app to go next. Each screen returns one when a button takes the player elsewhere
pub enum Navigation {
    // The new game screen
    NewGame,
    // A game set up on the new game screen, ready to play. The computer plays the opponent side, if there is one
    StartGame {
        session: GameSession,
        bottom_side: Side,
        opponent: Option<Side>,
    },
    // The settings screen, coming back to the current screen afterwards
    Settings,
    // Back to the screen this one was opened from
    Back,
    MainMenu,
    Exit,
}
