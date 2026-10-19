use super::images::icon_image;
use crate::gui::assets::{Icon, Sprites};
use crate::gui::theme::{BODY_TEXT_SIZE, ICON_SIZE, WIDGET_RADIUS, palette_of};
use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, Image, Margin, Rect, Response, Sense, Stroke,
    StrokeKind, Vec2, pos2, vec2,
};

/// Size of a choice tile, the picture in it, and the room above the picture and below the label
const TILE_SIZE: Vec2 = vec2(150.0, 120.0);
pub const TILE_IMAGE_SIZE: f32 = 54.0;
const TILE_PADDING: f32 = 15.0;

/// Room inside a text field, around the icon and text
const FIELD_MARGIN: Margin = Margin::symmetric(12, 10);

/// Size of an on/off switch, and the gap between its knob and track
const SWITCH_SIZE: Vec2 = vec2(46.0, 26.0);
const SWITCH_KNOB_INSET: f32 = 3.0;

/// A large tile with a picture and a label, outlined in gold when it's the chosen one
pub fn choice_tile(
    ui: &mut egui::Ui,
    selected: bool,
    image: Option<Image<'_>>,
    label: &str,
) -> Response {
    let palette = palette_of(ui);
    let (rect, response) = ui.allocate_exact_size(TILE_SIZE, Sense::click());

    let fill = if selected {
        palette.accent.gamma_multiply(0.22)
    } else if response.hovered() {
        palette.button_hovered
    } else {
        palette.button
    };
    let stroke = if selected {
        Stroke::new(2.0, palette.accent)
    } else {
        Stroke::NONE
    };
    let painter = ui.painter();
    painter.rect(rect, WIDGET_RADIUS, fill, stroke, StrokeKind::Inside);

    // Picture in the upper part, label along the bottom
    if let Some(image) = image {
        let center = rect.center_top() + vec2(0.0, TILE_PADDING + TILE_IMAGE_SIZE / 2.0);
        let image_rect = Rect::from_center_size(center, Vec2::splat(TILE_IMAGE_SIZE));
        image.paint_at(ui, image_rect);
    }
    let label_pos = rect.center_bottom() - vec2(0.0, TILE_PADDING);
    let font = FontId::proportional(BODY_TEXT_SIZE);
    painter.text(label_pos, Align2::CENTER_BOTTOM, label, font, palette.text);

    response.on_hover_cursor(CursorIcon::PointingHand)
}

/// A one-line text box styled like the rest of the app: an icon inside on the left,
/// a soft outline at rest, and a gold one while typing in it
pub fn text_field(
    ui: &mut egui::Ui,
    sprites: &mut Sprites,
    icon: Icon,
    text: &mut String,
    hint: &str,
) -> Response {
    let palette = palette_of(ui);
    let id = egui::Id::new(("text field", hint));
    let focused = ui.memory(|memory| memory.has_focus(id));
    let (stroke, icon_color) = if focused {
        (Stroke::new(1.5, palette.accent), palette.accent)
    } else {
        (Stroke::new(1.0, palette.card_stroke), palette.text_muted)
    };

    egui::Frame::new()
        .fill(palette.field)
        .stroke(stroke)
        .corner_radius(WIDGET_RADIUS)
        .inner_margin(FIELD_MARGIN)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(image) = icon_image(ui.ctx(), sprites, icon, ICON_SIZE, icon_color) {
                    ui.add(image);
                }
                // The frame above draws the box, so the text edit itself has none
                let edit = egui::TextEdit::singleline(text)
                    .id(id)
                    .hint_text(hint)
                    .frame(egui::Frame::NONE)
                    .desired_width(f32::INFINITY);
                ui.add(edit)
            })
            .inner
        })
        .inner
}

/// An on/off switch, with the knob sliding across and the track turning gold when on
pub fn switch(ui: &mut egui::Ui, on: &mut bool) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(SWITCH_SIZE, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    // Slides from 0 (off) to 1 (on) over a few frames instead of jumping
    let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
    let palette = palette_of(ui);
    let track = palette.button_pressed.lerp_to_gamma(palette.accent, how_on);
    let radius = rect.height() / 2.0;
    let knob_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);

    let painter = ui.painter();
    painter.rect_filled(rect, radius, track);
    let knob = pos2(knob_x, rect.center().y);
    painter.circle_filled(knob, radius - SWITCH_KNOB_INSET, Color32::WHITE);

    response.on_hover_cursor(CursorIcon::PointingHand)
}
