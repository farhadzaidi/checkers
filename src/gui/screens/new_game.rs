use super::Navigation;
use crate::game::{GameSession, Piece, Side};
use crate::gui::assets::{Icon, PieceStyle, Sprites};
use crate::gui::theme::{GROUP_GAP, SECTION_GAP, palette_of};
use crate::gui::widgets::{self, TILE_IMAGE_SIZE};
use eframe::egui::{self, Align, Layout, RichText};

/// Width of the screen's column
const COLUMN_WIDTH: f32 = 560.0;

/// Size of the warning icon beside a FEN error, a little smaller than the text it sits next to
const ERROR_ICON_SIZE: f32 = 18.0;

/// How many people are playing at this computer
#[derive(Clone, Copy, PartialEq, Eq)]
enum Players {
    One,
    Two,
}

/// The new game screen: the choices made on it, kept between visits
pub struct NewGameScreen {
    players: Players,
    // With one player, the side they play. With two, the side at the bottom of the board
    side: Side,
    // A PDN FEN to start from, like B:W18:B14, or empty for the normal starting position
    fen: String,
    // Why the last FEN couldn't be loaded, until it's fixed
    fen_error: Option<String>,
}

impl Default for NewGameScreen {
    fn default() -> Self {
        Self {
            players: Players::One,
            side: Side::Black,
            fen: String::new(),
            fen_error: None,
        }
    }
}

impl NewGameScreen {
    /// Draw the screen: number of players, the side and the starting position, then Start or Back
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        sprites: &mut Sprites,
        piece_style: PieceStyle,
    ) -> Option<Navigation> {
        let mut navigation = None;

        widgets::centered_column(ui, COLUMN_WIDTH, |ui| {
            widgets::screen_title(ui, "New game");
            widgets::card(ui, |ui| {
                self.players_choice(ui, sprites);
                ui.add_space(SECTION_GAP);
                self.side_choice(ui, sprites, piece_style);
                ui.add_space(SECTION_GAP);
                self.position_choice(ui, sprites);
                ui.add_space(GROUP_GAP);

                // Start on the right as the main action, Back beside it
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::menu_button(ui, sprites, Icon::Play, "Start", true).clicked() {
                        navigation = self.start();
                    }
                    if widgets::icon_button(ui, sprites, Icon::Back, "Back").clicked() {
                        navigation = Some(Navigation::Back);
                    }
                });
            });
        });

        navigation
    }

    /// One player against the computer, or two players taking turns at this computer
    fn players_choice(&mut self, ui: &mut egui::Ui, sprites: &mut Sprites) {
        let text = palette_of(ui).text;
        widgets::section_label(ui, "Players");
        ui.horizontal(|ui| {
            let one =
                widgets::icon_image(ui.ctx(), sprites, Icon::OnePlayer, TILE_IMAGE_SIZE, text);
            if widgets::choice_tile(ui, self.players == Players::One, one, "1 player").clicked() {
                self.players = Players::One;
            }

            let two =
                widgets::icon_image(ui.ctx(), sprites, Icon::TwoPlayers, TILE_IMAGE_SIZE, text);
            if widgets::choice_tile(ui, self.players == Players::Two, two, "2 players").clicked() {
                self.players = Players::Two;
            }
        });
    }

    /// Which side you play, or which side faces you with two players, shown as the real pieces
    fn side_choice(&mut self, ui: &mut egui::Ui, sprites: &mut Sprites, piece_style: PieceStyle) {
        let label = match self.players {
            Players::One => "You play",
            Players::Two => "Board faces",
        };
        widgets::section_label(ui, label);
        ui.horizontal(|ui| {
            for (side, label) in [(Side::Black, "Black"), (Side::White, "White")] {
                let piece = Piece::man(side);
                let image =
                    widgets::piece_image(ui.ctx(), sprites, piece_style, piece, TILE_IMAGE_SIZE);
                if widgets::choice_tile(ui, self.side == side, image, label).clicked() {
                    self.side = side;
                }
            }
        });
    }

    /// Blank for a normal game, or a pasted PDN FEN, with the reason underneath if it didn't load
    fn position_choice(&mut self, ui: &mut egui::Ui, sprites: &mut Sprites) {
        widgets::section_label(ui, "Start from position");
        let hint = "Normal start, or a PDN FEN like B:W18:B14";
        widgets::text_field(ui, sprites, Icon::Position, &mut self.fen, hint);

        if let Some(error) = &self.fen_error {
            let color = palette_of(ui).error;
            ui.horizontal_wrapped(|ui| {
                if let Some(icon) =
                    widgets::icon_image(ui.ctx(), sprites, Icon::Warning, ERROR_ICON_SIZE, color)
                {
                    ui.add(icon);
                }
                ui.label(RichText::new(error).color(color));
            });
        }
    }

    /// Start the game that's set up, or stay here and show why its FEN is wrong
    fn start(&mut self) -> Option<Navigation> {
        let fen = self.fen.trim();
        let session = if fen.is_empty() {
            Ok(GameSession::new())
        } else {
            GameSession::from_fen(fen)
        };

        match session {
            Ok(session) => {
                self.fen_error = None;
                // With one player, the computer takes the side they didn't pick
                let opponent = match self.players {
                    Players::One => Some(self.side.opposite()),
                    Players::Two => None,
                };
                Some(Navigation::StartGame {
                    session,
                    bottom_side: self.side,
                    opponent,
                })
            }
            Err(error) => {
                self.fen_error = Some(format!("Can't load that position: {error}"));
                None
            }
        }
    }
}
