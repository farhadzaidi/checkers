use super::board::{self, BoardGeometry};
use super::screen::GameScreen;
use crate::game::{Board, Move, MovePath, Piece, Square};
use crate::gui::assets::Sprites;
use crate::gui::settings::Settings;
use eframe::egui::{self, Color32, Painter, Pos2, Rect, Vec2};
use std::f32::consts::TAU;

/// How long each hop of a sliding piece takes, in seconds
const SLIDE_SECONDS_PER_HOP: f64 = 0.18;

/// How long an illegal move is shown for, in seconds
const REJECT_SECONDS: f64 = 0.45;

/// The rejected piece swings side to side this many times, this far as a share of a square
const SHAKE_SWINGS: f32 = 3.0;
const SHAKE_DISTANCE: f32 = 0.08;

/// Red flashed over the square an illegal drop landed on, fading out
const REJECT_TINT: Color32 = Color32::from_rgba_unmultiplied_const(230, 40, 40, 150);

/// A piece sliding along its route, one hop at a time. For a played move, the pieces it took fade out
pub struct Slide {
    // The piece as it was before moving, so a man crowned by the move becomes a king as it lands
    piece: Piece,
    start: Square,
    // Where the piece was let go, when it was dragged there, instead of its start square
    dropped_at: Option<Pos2>,
    landings: MovePath,
    captured: Vec<(Square, Piece)>,
    // Where a played move put the piece, left empty while it slides there. None for a hop that isn't played yet
    pub lands_on: Option<Square>,
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
    /// Slide a move that was just played from the before position: from the start square (or the drop point)
    /// over the given landings, with the pieces it took fading out
    pub(super) fn slide_played_move(
        &mut self,
        before: &Board,
        mv: Move,
        start: Square,
        landings: MovePath,
        dropped_at: Option<Pos2>,
    ) {
        debug_assert!(!landings.is_empty(), "a slide needs somewhere to land");
        let Some(piece) = before.piece_at(mv.from()) else {
            return;
        };

        // The move knows which squares it captured; the position before it says what stood there
        let captured = mv
            .captured_squares()
            .into_iter()
            .filter_map(|square| Some((square, before.piece_at(square)?)))
            .collect();

        self.slide = Some(Slide {
            piece,
            start,
            dropped_at,
            landings,
            captured,
            lands_on: Some(mv.to()),
            started: None,
        });
    }

    /// Slide the selected piece over hops of a capture that isn't finished yet
    pub(super) fn slide_hops(
        &mut self,
        start: Square,
        landings: MovePath,
        dropped_at: Option<Pos2>,
    ) {
        debug_assert!(!landings.is_empty(), "a slide needs somewhere to land");
        let Some(piece) = self.selected_piece() else {
            return;
        };

        self.slide = Some(Slide {
            piece,
            start,
            dropped_at,
            landings,
            captured: Vec::new(),
            lands_on: None,
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
        let seconds = SLIDE_SECONDS_PER_HOP * slide.landings.len() as f64;
        let progress = progress(ui, started, now, seconds);
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

    /// Draw the sliding piece partway along its route, with the taken pieces fading out as it goes
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

        // The points the piece passes through: its start, then the middle of each square it lands on
        let start = slide
            .dropped_at
            .unwrap_or_else(|| geometry.square_rect(slide.start).center());
        let mut points = vec![start];
        points.extend(
            slide
                .landings
                .iter()
                .map(|&square| geometry.square_rect(square).center()),
        );

        // Split the progress across the hops: with 2 hops, 0.75 is halfway through the second
        let hops = slide.landings.len();
        let along = eased * hops as f32;
        let hop = (along as usize).min(hops - 1);
        let center = points[hop].lerp(points[hop + 1], along - hop as f32);

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
