use super::game_over::{badge, result_text, side_name};
use super::screen::GameScreen;
use crate::game::Piece;
use crate::gui::assets::{Icon, Sprites};
use crate::gui::screens::Navigation;
use crate::gui::settings::Settings;
use crate::gui::theme::{GROUP_GAP, LARGE_TEXT_SIZE, SMALL_TEXT_SIZE, palette_of};
use crate::gui::widgets;
use eframe::egui::{self, RichText};

/// Width of the panel beside the board, the room around its card, and inside the card
pub const SIDE_PANEL_WIDTH: f32 = 300.0;
pub const PANEL_MARGIN: i8 = 18;
const CARD_MARGIN: i8 = 20;

/// Size of the piece or badge beside the game status
const STATUS_IMAGE_SIZE: f32 = 36.0;

impl GameScreen {
    /// Draw the game's status and controls in a card next to the board
    pub(super) fn side_panel(
        &mut self,
        ui: &mut egui::Ui,
        sprites: &mut Sprites,
        settings: &Settings,
    ) -> Option<Navigation> {
        let mut navigation = None;

        widgets::card_with_margin(ui, CARD_MARGIN, |ui| {
            ui.set_width(ui.available_width());
            self.status(ui, sprites, settings);
            ui.add_space(GROUP_GAP);

            self.undo_redo(ui, sprites);
            ui.add_space(GROUP_GAP);

            if widgets::wide_button(ui, sprites, Icon::FlipBoard, "Flip board").clicked() {
                self.flip();
            }
            let links = [
                (Icon::Settings, "Settings", Navigation::Settings),
                (Icon::NewGame, "New game", Navigation::NewGame),
                (Icon::Home, "Main menu", Navigation::MainMenu),
            ];
            for (icon, text, target) in links {
                if widgets::wide_button(ui, sprites, icon, text).clicked() {
                    navigation = Some(target);
                }
            }
        });

        navigation
    }

    /// Show whose turn it is with their piece, or how the game ended with its badge
    fn status(&self, ui: &mut egui::Ui, sprites: &mut Sprites, settings: &Settings) {
        let palette = palette_of(ui);
        let position = self.session.current_position();

        let (image, headline) = match self.session.outcome() {
            Some(outcome) => {
                let icon = badge(outcome);
                let image =
                    widgets::icon_image(ui.ctx(), sprites, icon, STATUS_IMAGE_SIZE, palette.accent);
                (image, result_text(outcome, &position).0)
            }
            None => {
                let side = position.side_to_move();
                let piece = Piece::man(side);
                let image = widgets::piece_image(
                    ui.ctx(),
                    sprites,
                    settings.piece_style,
                    piece,
                    STATUS_IMAGE_SIZE,
                );
                (image, format!("{} to move", side_name(side)))
            }
        };

        ui.horizontal(|ui| {
            if let Some(image) = image {
                ui.add(image);
            }
            let headline = RichText::new(headline)
                .size(LARGE_TEXT_SIZE)
                .strong()
                .color(palette.text);
            ui.label(headline);
        });
    }

    /// Undo and Redo side by side, greyed out when there's nothing to step to, with the keyboard hint below
    fn undo_redo(&mut self, ui: &mut egui::Ui, sprites: &mut Sprites) {
        let (can_undo, can_redo) = (self.session.can_undo(), self.session.can_redo());
        ui.columns(2, |columns| {
            let undo = columns[0].add_enabled_ui(can_undo, |ui| {
                widgets::wide_button(ui, sprites, Icon::Undo, "Undo")
            });
            let redo = columns[1].add_enabled_ui(can_redo, |ui| {
                widgets::wide_button(ui, sprites, Icon::Redo, "Redo")
            });
            if undo.inner.clicked() {
                self.undo();
            }
            if redo.inner.clicked() {
                self.redo();
            }
        });

        let muted = palette_of(ui).text_muted;
        ui.vertical_centered(|ui| {
            let hint = RichText::new("or use the ← and → keys")
                .size(SMALL_TEXT_SIZE)
                .color(muted);
            ui.label(hint);
        });
    }
}
