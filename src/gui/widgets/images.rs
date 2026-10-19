use crate::game::Piece;
use crate::gui::assets::{Icon, PieceStyle, Sprites};
use crate::gui::theme::{palette, palette_of};
use eframe::egui::{self, Color32, Image, LayerId, Rect, Vec2, pos2};
use std::f32::consts::TAU;

/// The loading icon's size, and how many turns it makes a second
const SPINNER_SIZE: f32 = 56.0;
const SPINNER_TURNS_PER_SECOND: f32 = 0.8;

/// Return an icon at the given size and colour, or None while it's still loading
pub fn icon_image(
    ctx: &egui::Context,
    sprites: &mut Sprites,
    icon: Icon,
    size: f32,
    tint: Color32,
) -> Option<Image<'static>> {
    let texture = sprites.icon(ctx, icon)?;
    Some(Image::new((texture.id(), Vec2::splat(size))).tint(tint))
}

/// Return a piece in the given style at the given size, or None while it's still loading
pub fn piece_image(
    ctx: &egui::Context,
    sprites: &mut Sprites,
    style: PieceStyle,
    piece: Piece,
    size: f32,
) -> Option<Image<'static>> {
    let texture = sprites.piece(ctx, style, piece)?;
    Some(Image::new((texture.id(), Vec2::splat(size))))
}

/// Paint the background art behind everything, scaled to cover the window and cropped to fit
pub fn background(ctx: &egui::Context, sprites: &mut Sprites, dark_mode: bool) {
    let window = ctx.content_rect();
    let painter = ctx.layer_painter(LayerId::background());
    match sprites.background(ctx, dark_mode) {
        Some(texture) => {
            let uv = cover_uv(texture.size_vec2(), window.size());
            painter.image(texture.id(), window, uv, Color32::WHITE);
        }
        None => {
            painter.rect_filled(window, 0.0, palette(dark_mode).background);
        }
    }
}

/// Return the part of an image that fills an area of a different shape without stretching:
/// a 16:9 image in a square window shows its middle square
fn cover_uv(image: Vec2, area: Vec2) -> Rect {
    let scale = (area.x / image.x).max(area.y / image.y);
    let visible = area / (image * scale);
    Rect::from_center_size(pos2(0.5, 0.5), visible)
}

/// A spinning loading icon in the middle of the rect
pub fn spinner(ui: &mut egui::Ui, sprites: &mut Sprites, rect: Rect) {
    let palette = palette_of(ui);
    // Anticlockwise, so the brightest dot leads the ring
    let angle = -(ui.input(|input| input.time) as f32) * SPINNER_TURNS_PER_SECOND * TAU;
    let spinner_rect = Rect::from_center_size(rect.center(), Vec2::splat(SPINNER_SIZE));

    match icon_image(
        ui.ctx(),
        sprites,
        Icon::Loading,
        SPINNER_SIZE,
        palette.accent,
    ) {
        Some(image) => image
            .rotate(angle, Vec2::splat(0.5))
            .paint_at(ui, spinner_rect),
        None => {
            ui.put(spinner_rect, egui::Spinner::new().size(SPINNER_SIZE));
        }
    }
    ui.ctx().request_repaint();
}
