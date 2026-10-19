use crate::game::{self, Board, Piece, Side, Square};
use crate::gui::assets::{Overlay, Sprites};
use crate::gui::settings::Settings;
use eframe::egui::{Color32, Painter, Pos2, Rect, Vec2, pos2};

/// Share of the board image taken by the frame on each side: 40px of 800 in every board SVG
const FRAME_FRACTION: f32 = 40.0 / 800.0;

/// Squares and pieces drawn as plain shapes while their images are still loading
const FALLBACK_LIGHT_SQUARE: Color32 = Color32::from_rgb(222, 196, 150);
const FALLBACK_DARK_SQUARE: Color32 = Color32::from_rgb(120, 80, 50);
const FALLBACK_BLACK_PIECE: Color32 = Color32::from_rgb(30, 30, 30);
const FALLBACK_WHITE_PIECE: Color32 = Color32::from_rgb(200, 40, 40);

/// Draw a whole texture, stretched over the rect
const FULL_UV: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));

/// Where the board sits on screen, and which way up it's drawn
#[derive(Clone, Copy)]
pub struct BoardGeometry {
    // The whole board image, frame included
    board: Rect,
    // Just the 8x8 squares inside the frame
    grid: Rect,
    flipped: bool,
}

impl BoardGeometry {
    /// Fit the biggest square board into the space, with the given side at the bottom
    pub fn fit(space: Rect, bottom_side: Side) -> Self {
        let size = space.width().min(space.height());
        let board = Rect::from_center_size(space.center(), Vec2::splat(size));
        let grid = board.shrink(size * FRAME_FRACTION);

        // Black starts at the top of the board, so turn it around to put Black at the bottom
        Self {
            board,
            grid,
            flipped: bottom_side == Side::Black,
        }
    }

    /// Return the width of one square on screen
    pub fn square_size(&self) -> f32 {
        self.grid.width() / 8.0
    }

    /// Return the screen rect of a square
    pub fn square_rect(&self, square: Square) -> Rect {
        let (row, col) = self.screen_cell(square);
        let size = self.square_size();
        let min = self.grid.min + Vec2::new(col as f32 * size, row as f32 * size);
        Rect::from_min_size(min, Vec2::splat(size))
    }

    /// Return the square under a screen position, or None over a light square or the frame
    pub fn square_at(&self, pos: Pos2) -> Option<Square> {
        if !self.grid.contains(pos) {
            return None;
        }

        // The far edge counts as inside the grid, so keep it in the last row and column
        let offset = (pos - self.grid.min) / self.square_size();
        let screen_row = (offset.y as usize).min(7);
        let screen_col = (offset.x as usize).min(7);
        let (row, col) = self.orient(screen_row, screen_col);
        game::square_at(row, col)
    }

    /// Return the row and column a square is drawn at, counted from the top left of the screen
    fn screen_cell(&self, square: Square) -> (usize, usize) {
        let (row, col) = game::row_and_col(square);
        self.orient(row, col)
    }

    /// Turn a row and column the board around when it's flipped. Doing it twice undoes it
    fn orient(&self, row: usize, col: usize) -> (usize, usize) {
        if self.flipped {
            (7 - row, 7 - col)
        } else {
            (row, col)
        }
    }
}

/// Draw the board image, or plain squares while it's still loading
pub fn draw_board(
    painter: &Painter,
    geometry: &BoardGeometry,
    sprites: &mut Sprites,
    settings: &Settings,
) {
    if let Some(texture) = sprites.board(painter.ctx(), settings.board_style) {
        painter.image(texture.id(), geometry.board, FULL_UV, Color32::WHITE);
        return;
    }

    painter.rect_filled(geometry.board, 0.0, FALLBACK_DARK_SQUARE);
    painter.rect_filled(geometry.grid, 0.0, FALLBACK_LIGHT_SQUARE);
    for square in 0..32 {
        painter.rect_filled(geometry.square_rect(square), 0.0, FALLBACK_DARK_SQUARE);
    }
}

/// Draw every piece on the board, except the hidden ones, which are being dragged or animated
pub fn draw_pieces(
    painter: &Painter,
    geometry: &BoardGeometry,
    board: &Board,
    hidden: &[Square],
    sprites: &mut Sprites,
    settings: &Settings,
) {
    for square in 0..32 {
        if hidden.contains(&square) {
            continue;
        }
        if let Some(piece) = board.piece_at(square) {
            let rect = geometry.square_rect(square);
            draw_piece(painter, rect, piece, 1.0, sprites, settings);
        }
    }
}

/// Draw a square marker in the given colour. Nothing shows until its image has loaded
pub fn draw_overlay(
    painter: &Painter,
    rect: Rect,
    overlay: Overlay,
    tint: Color32,
    sprites: &mut Sprites,
) {
    // The markers are white, so multiplying by the tint colours them
    if let Some(texture) = sprites.overlay(painter.ctx(), overlay) {
        painter.image(texture.id(), rect, FULL_UV, tint);
    }
}

/// Draw one piece filling the rect, or a plain disc while its image is still loading.
/// Opacity runs from 0 (invisible) to 1 (solid), for pieces fading out
pub fn draw_piece(
    painter: &Painter,
    rect: Rect,
    piece: Piece,
    opacity: f32,
    sprites: &mut Sprites,
    settings: &Settings,
) {
    if let Some(texture) = sprites.piece(painter.ctx(), settings.piece_style, piece) {
        let tint = Color32::WHITE.gamma_multiply(opacity);
        painter.image(texture.id(), rect, FULL_UV, tint);
        return;
    }

    let color = match piece {
        Piece::BlackMan | Piece::BlackKing => FALLBACK_BLACK_PIECE,
        Piece::WhiteMan | Piece::WhiteKing => FALLBACK_WHITE_PIECE,
    };
    painter.circle_filled(
        rect.center(),
        rect.width() * 0.4,
        color.gamma_multiply(opacity),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPACE: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));

    #[test]
    fn square_at_finds_the_square_drawn_there() {
        for side in [Side::Black, Side::White] {
            let geometry = BoardGeometry::fit(SPACE, side);
            for square in 0..32 {
                let center = geometry.square_rect(square).center();
                assert_eq!(
                    geometry.square_at(center),
                    Some(square),
                    "{side:?} at the bottom"
                );
            }
        }
    }

    #[test]
    fn bottom_side_starts_at_the_bottom() {
        // Square 0 is in Black's back row, so it's at the bottom only when Black is
        let black_bottom = BoardGeometry::fit(SPACE, Side::Black);
        let white_bottom = BoardGeometry::fit(SPACE, Side::White);
        assert!(black_bottom.square_rect(0).center().y > SPACE.center().y);
        assert!(white_bottom.square_rect(0).center().y < SPACE.center().y);
    }

    #[test]
    fn light_squares_and_frame_are_not_squares() {
        let geometry = BoardGeometry::fit(SPACE, Side::White);
        // The top-left square is light, and the frame's corner is outside the grid
        let top_left = geometry.grid.min + Vec2::splat(1.0);
        assert_eq!(geometry.square_at(top_left), None);
        assert_eq!(geometry.square_at(geometry.board.min), None);
    }
}
