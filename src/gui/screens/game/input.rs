use super::board::BoardGeometry;
use super::screen::GameScreen;
use crate::game::Square;
use eframe::egui::{self, CursorIcon, Key, Pos2};

impl GameScreen {
    /// Left and right arrows step back and forward through the moves, like the Undo and Redo buttons.
    /// Escape puts a piece partway through a capture back where it started
    pub(super) fn handle_keys(&mut self, ui: &egui::Ui) {
        let (left, right, escape) = ui.input(|input| {
            (
                input.key_pressed(Key::ArrowLeft),
                input.key_pressed(Key::ArrowRight),
                input.key_pressed(Key::Escape),
            )
        });
        if left {
            self.undo();
        }
        if right {
            self.redo();
        }
        if escape {
            self.cancel_hops();
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
        let grabbable = |square: Square| {
            if self.is_mid_capture() {
                Some(square) == self.piece_square()
            } else {
                self.can_move(square)
            }
        };

        if self.dragging.is_some() {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        } else if hovered.is_some_and(grabbable) {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
    }

    /// Move the selected piece if the click lands on one of its next squares, otherwise change the selection
    fn click(&mut self, square: Option<Square>, now: f64) {
        self.rejection = None;
        if self.session.outcome().is_some() {
            self.selected = None;
            return;
        }

        if let Some(target) = square
            && self.land(target, None)
        {
            return;
        }

        // Partway through a capture the piece has to keep jumping, so anywhere else is refused.
        // Clicking the piece itself does nothing. The piece is still on its starting square in the
        // position, so nothing stands on its hop square to shake; only the red flash shows
        if self.is_mid_capture() {
            if square != self.piece_square()
                && let Some(piece) = self.piece_square()
            {
                self.reject(piece, square, now);
            }
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

        // Partway through a capture, only the hopping piece can be picked up
        if self.is_mid_capture() {
            if Some(square) == self.piece_square() {
                self.dragging = Some(square);
            }
            return;
        }

        if self.can_move(square) {
            self.dragging = Some(square);
            self.selected = Some(square);
        } else if self.is_stuck(square) {
            self.reject(square, None, now);
        }
    }

    /// Move the dragged piece to the square it was dropped on, or send it back if no route goes there
    fn drop(&mut self, square: Option<Square>, pointer: Option<Pos2>, now: f64) {
        let Some(from) = self.dragging.take() else {
            return;
        };

        if let Some(target) = square
            && self.land(target, pointer)
        {
            return;
        }
        if square != Some(from) {
            // Dropping it back where it started just leaves it selected
            self.reject(from, square, now);
        }
    }
}
