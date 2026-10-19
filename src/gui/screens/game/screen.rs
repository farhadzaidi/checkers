use super::animation::{Rejection, Slide};
use super::board::{self, BoardGeometry};
use super::side_panel::{PANEL_MARGIN, SIDE_PANEL_WIDTH};
use crate::game::{GameSession, Move, MoveList, Side, Square};
use crate::gui::assets::{Overlay, Sprites};
use crate::gui::screens::Navigation;
use crate::gui::settings::Settings;
use crate::gui::theme::palette_of;
use crate::gui::widgets;
use eframe::egui::{self, Color32, Painter, Pos2, Rect, Vec2};

/// A light wash over the two squares of the last move
const LAST_MOVE_TINT: Color32 = Color32::from_rgba_unmultiplied_const(255, 225, 110, 60);

/// A game being played on screen: the session plus what the player is doing with the mouse.
/// Its other parts are split across this folder: input, animation, the side panel and the game over popup
pub struct GameScreen {
    pub(super) session: GameSession,
    pub(super) bottom_side: Side,
    // The square of the piece the player picked, whose moves are showing
    pub(super) selected: Option<Square>,
    // The square of the piece being dragged, which follows the pointer instead
    pub(super) dragging: Option<Square>,
    pub(super) rejection: Option<Rejection>,
    pub(super) slide: Option<Slide>,
    // A move just ended the game, so its popup should show. Stepping through the moves later doesn't bring it back
    pub(super) result_pending: bool,
}

impl GameScreen {
    /// Show a game with the given side at the bottom of the board
    pub fn new(session: GameSession, bottom_side: Side) -> Self {
        // A position loaded from a FEN can already be over, so say so straight away
        let result_pending = session.outcome().is_some();
        Self {
            session,
            bottom_side,
            selected: None,
            dragging: None,
            rejection: None,
            slide: None,
            result_pending,
        }
    }

    /// Draw the side panel, the board and, once the game is over, its popup
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        sprites: &mut Sprites,
        settings: &Settings,
    ) -> Option<Navigation> {
        self.handle_keys(ui);

        // Panels must be added before the central panel, which takes the remaining space
        let mut navigation = egui::Panel::left("controls")
            .resizable(false)
            .exact_size(SIDE_PANEL_WIDTH)
            .frame(egui::Frame::NONE.inner_margin(PANEL_MARGIN))
            .show(ui, |ui| self.side_panel(ui, sprites, settings))
            .inner;

        egui::CentralPanel::default().show(ui, |ui| self.board_ui(ui, sprites, settings));

        if self.shows_result() {
            navigation = navigation.or(self.game_over_popup(ui.ctx(), sprites));
        }
        navigation
    }

    /// Turn the board around, so the other side is at the bottom
    pub(super) fn flip(&mut self) {
        self.bottom_side = self.bottom_side.opposite();
    }

    /// Take back the last move, if there is one
    pub(super) fn undo(&mut self) {
        if self.session.can_undo() {
            self.session.undo();
            self.clear_interaction();
        }
    }

    /// Replay the last undone move, if there is one, sliding it in like a fresh move
    pub(super) fn redo(&mut self) {
        if self.session.can_redo() {
            let before = self.session.current_position();
            self.session.redo();
            self.clear_interaction();
            self.start_slide(&before, None);
        }
    }

    /// Play a move and slide it into place, from the drop point if it was dragged there.
    /// If it ends the game, the popup shows once it lands
    pub(super) fn play(&mut self, mv: Move, dropped_at: Option<Pos2>) {
        let before = self.session.current_position();
        self.session.play(mv);
        self.selected = None;
        self.result_pending = self.session.outcome().is_some();
        self.start_slide(&before, dropped_at);
    }

    /// Return true if the game over popup should show: a move just ended the game, and it has landed
    fn shows_result(&self) -> bool {
        self.result_pending && self.slide.is_none() && self.session.outcome().is_some()
    }

    /// Forget the selection, drag, animations and popup, which belonged to the position before
    fn clear_interaction(&mut self) {
        self.selected = None;
        self.dragging = None;
        self.rejection = None;
        self.slide = None;
        self.result_pending = false;
    }

    /// Return true if the piece on this square has a legal move
    pub(super) fn can_move(&self, square: Square) -> bool {
        self.session.moves().iter().any(|mv| mv.from() == square)
    }

    /// Return true if the square holds a piece of the side to move that has no legal move
    pub(super) fn is_stuck(&self, square: Square) -> bool {
        let position = self.session.current_position();
        let ours = position
            .piece_at(square)
            .is_some_and(|piece| piece.side() == position.side_to_move());
        ours && !self.can_move(square)
    }

    /// Return the legal moves of the selected piece, or none when nothing is selected
    fn selected_moves(&self) -> MoveList {
        let Some(from) = self.selected else {
            return MoveList::new();
        };
        self.session
            .moves()
            .into_iter()
            .filter(|mv| mv.from() == from)
            .collect()
    }

    /// Return the selected piece's move to this square, if it has one
    pub(super) fn selected_move_to(&self, to: Square) -> Option<Move> {
        self.selected_moves().into_iter().find(|mv| mv.to() == to)
    }

    /// Draw the board as big as the space allows, and handle clicks and drags on it
    fn board_ui(&mut self, ui: &mut egui::Ui, sprites: &mut Sprites, settings: &Settings) {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let geometry = BoardGeometry::fit(response.rect, self.bottom_side);

        if !sprites.is_ready(ui.ctx(), settings) {
            widgets::spinner(ui, sprites, response.rect);
            return;
        }

        let now = ui.input(|input| input.time);
        self.handle_pointer(&response, &geometry, now);
        self.set_cursor(ui, &response, &geometry);
        self.draw(ui, &painter, &geometry, sprites, settings, now);
    }

    /// Draw everything, back to front: board, highlights under the pieces, pieces, markers, then the dragged piece
    fn draw(
        &mut self,
        ui: &egui::Ui,
        painter: &Painter,
        geometry: &BoardGeometry,
        sprites: &mut Sprites,
        settings: &Settings,
        now: f64,
    ) {
        let slide_progress = self.advance_slide(ui, now);
        let reject_progress = self.advance_rejection(ui, now);
        let accent = palette_of(ui).accent;

        board::draw_board(painter, geometry, sprites, settings);
        self.draw_last_move(painter, geometry);
        if let Some(square) = self.selected {
            let rect = geometry.square_rect(square);
            board::draw_overlay(painter, rect, Overlay::Selected, accent, sprites);
        }
        if let Some(progress) = reject_progress {
            self.draw_reject_flash(painter, geometry, progress);
        }

        // The dragged, shaking or sliding piece is drawn on its own below, so leave it out here
        let position = self.session.current_position();
        let hidden: Vec<Square> = [
            self.dragging,
            self.rejection.as_ref().map(|rejection| rejection.piece),
            self.slide.as_ref().map(|slide| slide.mv.to()),
        ]
        .into_iter()
        .flatten()
        .collect();
        board::draw_pieces(painter, geometry, &position, &hidden, sprites, settings);

        if let Some(progress) = slide_progress {
            self.draw_slide(painter, geometry, sprites, settings, progress);
        }
        if let Some(progress) = reject_progress {
            self.draw_shake(painter, geometry, sprites, settings, progress);
        }
        if settings.show_legal_moves {
            self.draw_move_markers(painter, geometry, sprites, accent);
        }
        self.draw_dragged(ui, painter, geometry, sprites, settings);
    }

    /// Wash over the squares the last move left and landed on
    fn draw_last_move(&self, painter: &Painter, geometry: &BoardGeometry) {
        if let Some(mv) = self.session.last_move() {
            painter.rect_filled(geometry.square_rect(mv.from()), 0.0, LAST_MOVE_TINT);
            painter.rect_filled(geometry.square_rect(mv.to()), 0.0, LAST_MOVE_TINT);
        }
    }

    /// Mark every square the selected piece can move to. Jumps get a different marker, so captures stand out
    fn draw_move_markers(
        &self,
        painter: &Painter,
        geometry: &BoardGeometry,
        sprites: &mut Sprites,
        tint: Color32,
    ) {
        for mv in self.selected_moves() {
            let marker = if mv.is_capture() {
                Overlay::Capture
            } else {
                Overlay::Move
            };
            let rect = geometry.square_rect(mv.to());
            board::draw_overlay(painter, rect, marker, tint, sprites);
        }
    }

    /// The dragged piece follows the pointer, on top of everything
    fn draw_dragged(
        &self,
        ui: &egui::Ui,
        painter: &Painter,
        geometry: &BoardGeometry,
        sprites: &mut Sprites,
        settings: &Settings,
    ) {
        let position = self.session.current_position();
        if let Some(from) = self.dragging
            && let Some(piece) = position.piece_at(from)
            && let Some(pointer) = ui.input(|input| input.pointer.interact_pos())
        {
            let rect = Rect::from_center_size(pointer, Vec2::splat(geometry.square_size()));
            board::draw_piece(painter, rect, piece, 1.0, sprites, settings);
        }
    }
}
