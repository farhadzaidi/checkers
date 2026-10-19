use super::screen::GameScreen;
use crate::game::{Board, DrawReason, GameOutcome, Side};
use crate::gui::assets::{Icon, Sprites};
use crate::gui::screens::Navigation;
use crate::gui::theme::{self, GROUP_GAP, TITLE_TEXT_SIZE};
use crate::gui::widgets::{self, CARD_MARGIN, MENU_BUTTON_SIZE};
use eframe::egui::{self, RichText};

/// Size of the result badge at the top of the popup
const BADGE_SIZE: f32 = 64.0;

/// Room above the badge
const ABOVE_BADGE: f32 = 8.0;

/// What the player picked on the popup
enum Choice {
    NewGame,
    Undo,
    // Close the popup to look at the final position
    ViewBoard,
    MainMenu,
}

impl GameScreen {
    /// Show how the game ended over the board, and let the player start again, take a move back or look at the board
    pub(super) fn game_over_popup(
        &mut self,
        ctx: &egui::Context,
        sprites: &mut Sprites,
    ) -> Option<Navigation> {
        let outcome = self.session.outcome()?;
        let (title, detail) = result_text(outcome, &self.session.current_position());

        match popup(ctx, sprites, badge(outcome), &title, &detail)? {
            Choice::NewGame => Some(Navigation::NewGame),
            Choice::MainMenu => Some(Navigation::MainMenu),
            Choice::Undo => {
                self.undo();
                None
            }
            Choice::ViewBoard => {
                self.result_pending = false;
                None
            }
        }
    }
}

/// Draw the popup: a badge, the result, and what happens next
fn popup(
    ctx: &egui::Context,
    sprites: &mut Sprites,
    badge: Icon,
    title: &str,
    detail: &str,
) -> Option<Choice> {
    let mut choice = None;

    // Styled like the cards, but solid, since it sits over the board
    let palette = theme::palette(ctx.theme() == egui::Theme::Dark);
    let frame = widgets::card_frame(&palette, CARD_MARGIN).fill(palette.card.to_opaque());

    let modal = egui::Modal::new(egui::Id::new("game over"))
        .frame(frame)
        .show(ctx, |ui| {
            ui.set_width(MENU_BUTTON_SIZE.x);
            ui.vertical_centered(|ui| {
                ui.add_space(ABOVE_BADGE);
                if let Some(image) =
                    widgets::icon_image(ui.ctx(), sprites, badge, BADGE_SIZE, palette.accent)
                {
                    ui.add(image);
                }
                let title = RichText::new(title)
                    .font(theme::title_font(TITLE_TEXT_SIZE))
                    .color(palette.text);
                ui.label(title);
                ui.label(RichText::new(detail).color(palette.text_muted));
                ui.add_space(GROUP_GAP);

                let buttons = [
                    (Icon::NewGame, "New game", Choice::NewGame, true),
                    (Icon::Undo, "Undo last move", Choice::Undo, false),
                    (Icon::Board, "View board", Choice::ViewBoard, false),
                    (Icon::Home, "Main menu", Choice::MainMenu, false),
                ];
                for (icon, text, picked, primary) in buttons {
                    if widgets::menu_button(ui, sprites, icon, text, primary).clicked() {
                        choice = Some(picked);
                    }
                }
            });
        });

    // Escape or a click outside the popup closes it, the same as View board
    if modal.should_close() && choice.is_none() {
        choice = Some(Choice::ViewBoard);
    }
    choice
}

/// The badge for how a game ended: a crown for a win, or the reason for a draw
pub(super) fn badge(outcome: GameOutcome) -> Icon {
    match outcome {
        GameOutcome::Win(_) => Icon::Win,
        GameOutcome::Draw(DrawReason::ThreefoldRepetition) => Icon::Repetition,
        GameOutcome::Draw(DrawReason::FortyMoveRule) => Icon::MoveLimit,
    }
}

/// Return a headline for how the game ended, and a line explaining why
pub(super) fn result_text(outcome: GameOutcome, position: &Board) -> (String, String) {
    match outcome {
        GameOutcome::Win(winner) => {
            // A side loses with no pieces left, or with pieces that are all blocked
            let loser = winner.opposite();
            let reason = if position.has_pieces(loser) {
                "has no legal moves"
            } else {
                "has no pieces left"
            };
            (
                format!("{} wins", side_name(winner)),
                format!("{} {reason}", side_name(loser)),
            )
        }
        GameOutcome::Draw(DrawReason::ThreefoldRepetition) => (
            "Draw".to_string(),
            "The same position came up three times".to_string(),
        ),
        GameOutcome::Draw(DrawReason::FortyMoveRule) => (
            "Draw".to_string(),
            "40 moves each without a capture or a man moving".to_string(),
        ),
    }
}

/// Return the name of a side as players read it
pub(super) fn side_name(side: Side) -> &'static str {
    match side {
        Side::Black => "Black",
        Side::White => "White",
    }
}
