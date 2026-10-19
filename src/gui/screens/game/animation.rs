use super::board::{self, BoardGeometry};
use super::screen::GameScreen;
use crate::game::{Board, Move, Piece, Square};
use crate::gui::assets::Sprites;
use crate::gui::settings::Settings;
use eframe::egui::{self, Color32, Painter, Pos2, Rect, Vec2};
use std::f32::consts::TAU;

/// How long a played move takes to slide into place, in seconds
const SLIDE_SECONDS: f64 = 0.18;

/// How long an illegal move is shown for, in seconds
const REJECT_SECONDS: f64 = 0.45;

/// The rejected piece swings side to side this many times, this far as a share of a square
const SHAKE_SWINGS: f32 = 3.0;
const SHAKE_DISTANCE: f32 = 0.08;

/// Red flashed over the square an illegal drop landed on, fading out
const REJECT_TINT: Color32 = Color32::from_rgba_unmultiplied_const(230, 40, 40, 150);

/// A move sliding into place: the piece glides to its square, and the pieces it took fade out
pub struct Slide {
    pub mv: Move,
    // The piece as it was before moving, so a man crowned by the move becomes a king as it lands
    piece: Piece,
    captured: Vec<(Square, Piece)>,
    // Where the piece was let go, when it was dragged there, instead of its starting square
    dropped_at: Option<Pos2>,
    // Set on the first frame that draws it
    started: Option<f64>,
}

/// An illegal move being shown: the piece shakes, and the square it was dropped on flashes red
pub struct Rejection {
    pub piece: Square,
    // None when the piece was dropped off the board, or just can't move
    target: Option<Square>,
    started: f64,
}

impl GameScreen {
    /// Start animating the move that just led away from the before position
    pub(super) fn start_slide(&mut self, before: &Board, dropped_at: Option<Pos2>) {
        let Some(mv) = self.session.last_move() else {
            return;
        };
        let Some(piece) = before.piece_at(mv.from()) else {
            return;
        };

        // Captured pieces are the ones that were there before and are gone now, besides the mover
        let after = self.session.current_position();
        let captured = (0..32)
            .filter(|&square| square != mv.from() && after.piece_at(square).is_none())
            .filter_map(|square| Some((square, before.piece_at(square)?)))
            .collect();

        self.slide = Some(Slide {
            mv,
            piece,
            captured,
            dropped_at,
            started: None,
        });
    }

    /// Start showing an illegal move: a shake, and a red flash where it was dropped
    pub(super) fn reject(&mut self, piece: Square, target: Option<Square>, now: f64) {
        self.rejection = Some(Rejection {
            piece,
            target,
            started: now,
        });
    }

    /// Return how far the sliding move has got, from 0 to 1, or None once it has landed
    pub(super) fn advance_slide(&mut self, ui: &egui::Ui, now: f64) -> Option<f32> {
        let slide = self.slide.as_mut()?;
        let started = *slide.started.get_or_insert(now);
        let progress = progress(ui, started, now, SLIDE_SECONDS);
        if progress.is_none() {
            self.slide = None;
        }
        progress
    }

    /// Return how far the rejection has got, from 0 to 1, or None once it's over
    pub(super) fn advance_rejection(&mut self, ui: &egui::Ui, now: f64) -> Option<f32> {
        let started = self.rejection.as_ref()?.started;
        let progress = progress(ui, started, now, REJECT_SECONDS);
        if progress.is_none() {
            self.rejection = None;
        }
        progress
    }

    /// Draw the sliding move: the taken pieces fade out while the mover glides from its start to its square
    pub(super) fn draw_slide(
        &self,
        painter: &Painter,
        geometry: &BoardGeometry,
        sprites: &mut Sprites,
        settings: &Settings,
        progress: f32,
    ) {
        let Some(slide) = &self.slide else {
            return;
        };
        let eased = ease_out(progress);
        for &(square, piece) in &slide.captured {
            let rect = geometry.square_rect(square);
            board::draw_piece(painter, rect, piece, 1.0 - eased, sprites, settings);
        }

        let start = slide
            .dropped_at
            .unwrap_or_else(|| geometry.square_rect(slide.mv.from()).center());
        let end = geometry.square_rect(slide.mv.to()).center();
        let center = start.lerp(end, eased);
        let rect = Rect::from_center_size(center, Vec2::splat(geometry.square_size()));
        board::draw_piece(painter, rect, slide.piece, 1.0, sprites, settings);
    }

    /// Flash the square an illegal drop landed on red, fading out
    pub(super) fn draw_reject_flash(
        &self,
        painter: &Painter,
        geometry: &BoardGeometry,
        progress: f32,
    ) {
        if let Some(target) = self
            .rejection
            .as_ref()
            .and_then(|rejection| rejection.target)
        {
            let tint = REJECT_TINT.gamma_multiply(1.0 - progress);
            painter.rect_filled(geometry.square_rect(target), 0.0, tint);
        }
    }

    /// Shake the rejected piece side to side, settling as the rejection ends
    pub(super) fn draw_shake(
        &self,
        painter: &Painter,
        geometry: &BoardGeometry,
        sprites: &mut Sprites,
        settings: &Settings,
        progress: f32,
    ) {
        let Some(rejection) = &self.rejection else {
            return;
        };
        let Some(piece) = self.session.current_position().piece_at(rejection.piece) else {
            return;
        };

        let rect = geometry.square_rect(rejection.piece);
        let shake = (progress * SHAKE_SWINGS * TAU).sin() * (1.0 - progress);
        let offset = Vec2::new(shake * SHAKE_DISTANCE * rect.width(), 0.0);
        board::draw_piece(
            painter,
            rect.translate(offset),
            piece,
            1.0,
            sprites,
            settings,
        );
    }
}

/// Return how far an animation of the given length has got, from 0 to 1, or None once it's over.
/// While it's running, ask for another frame so it keeps moving
fn progress(ui: &egui::Ui, started: f64, now: f64, seconds: f64) -> Option<f32> {
    let progress = ((now - started) / seconds) as f32;
    if progress >= 1.0 {
        return None;
    }
    ui.ctx().request_repaint();
    Some(progress)
}

/// Start fast and settle gently: 0 at the start, 1 at the end, most of the way there by halfway
fn ease_out(progress: f32) -> f32 {
    1.0 - (1.0 - progress).powi(3)
}
