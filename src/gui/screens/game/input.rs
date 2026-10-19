use super::board::BoardGeometry;
use super::screen::GameScreen;
use crate::game::Square;
use eframe::egui::{self, CursorIcon, Key, Pos2};

impl GameScreen {
    /// Left and right arrows step back and forward through the moves, like the Undo and Redo buttons
    pub(super) fn handle_keys(&mut self, ui: &egui::Ui) {
        let (left, right) = ui.input(|input| {
            (
                input.key_pressed(Key::ArrowLeft),
                input.key_pressed(Key::ArrowRight),
            )
        });
        if left {
            self.undo();
        }
        if right {
            self.redo();
        }
    }

    /// Turn this frame's clicks, drag starts and drops on the board into selections and moves
    pub(super) fn handle_pointer(
        &mut self,
        response: &egui::Response,
        geometry: &BoardGeometry,
        now: f64,
    ) {
        let square_under = |pos: Option<Pos2>| pos.and_then(|pos| geometry.square_at(pos));

        if response.clicked() {
            self.click(square_under(response.interact_pointer_pos()), now);
        }
        if response.drag_started() {
            // Where the button went down, not where the pointer is once it has moved far enough to count as a drag
            let origin = response.ctx.input(|input| input.pointer.press_origin());
            self.start_drag(square_under(origin), now);
        }
        if response.drag_stopped() {
            let pointer = response.interact_pointer_pos();
            self.drop(square_under(pointer), pointer, now);
        }
    }

    /// Show a grabbing hand while dragging, and a pointing hand over pieces that can move
    pub(super) fn set_cursor(
        &self,
        ui: &egui::Ui,
        response: &egui::Response,
        geometry: &BoardGeometry,
    ) {
        let hovered = response.hover_pos().and_then(|pos| geometry.square_at(pos));
        if self.dragging.is_some() {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        } else if hovered.is_some_and(|square| self.can_move(square)) {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
    }

    /// Play a move if the click lands on one of the selected piece's targets, otherwise change the selection
    fn click(&mut self, square: Option<Square>, now: f64) {
        self.rejection = None;
        if self.session.outcome().is_some() {
            self.selected = None;
            return;
        }

        if let Some(to) = square
            && let Some(mv) = self.selected_move_to(to)
        {
            self.play(mv, None);
            return;
        }

        // Shake a piece that's to move but can't, which happens when another piece must capture
        if let Some(square) = square
            && self.is_stuck(square)
        {
            self.reject(square, None, now);
            self.selected = None;
            return;
        }

        // Pick the clicked piece if it can move. Clicking it again, or anywhere else, clears the selection
        let clicked = square.filter(|&square| self.can_move(square));
        self.selected = if clicked == self.selected {
            None
        } else {
            clicked
        };
    }

    /// Pick up the piece under the pointer, if it can move
    fn start_drag(&mut self, square: Option<Square>, now: f64) {
        self.rejection = None;
        let Some(square) = square else {
            return;
        };
        if self.session.outcome().is_some() {
            return;
        }

        if self.can_move(square) {
            self.dragging = Some(square);
            self.selected = Some(square);
        } else if self.is_stuck(square) {
            self.reject(square, None, now);
        }
    }

    /// Play the dragged piece's move to the square it was dropped on, or send it back if that's not legal
    fn drop(&mut self, square: Option<Square>, pointer: Option<Pos2>, now: f64) {
        let Some(from) = self.dragging.take() else {
            return;
        };

        if let Some(to) = square
            && let Some(mv) = self.selected_move_to(to)
        {
            self.play(mv, pointer);
        } else if square != Some(from) {
            // Dropping it back where it started just leaves it selected
            self.reject(from, square, now);
        }
    }
}
