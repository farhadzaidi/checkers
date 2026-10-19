use super::images::icon_image;
use crate::gui::assets::{Icon, Sprites};
use crate::gui::theme::{
    CARD_RADIUS, CARD_SHADOW, ICON_SIZE, Palette, SECTION_GAP, SMALL_TEXT_SIZE, TITLE_TEXT_SIZE,
    palette_of, title_font,
};
use eframe::egui::{self, RichText, Stroke};

/// Room around a card's contents
pub const CARD_MARGIN: i8 = 28;

/// How far down the window a screen's column starts, as a share of the window's height
const COLUMN_TOP: f32 = 0.08;
const COLUMN_MIN_TOP: f32 = 16.0;

/// Lay out a screen as a column of the given width, centred in the window, scrolling if the window is too short
pub fn centered_column(ui: &mut egui::Ui, width: f32, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let width = width.min(ui.available_width());
        let side_space = (ui.available_width() - width) / 2.0;
        // Sit a little above the middle, like a dialog
        let top_space = (ui.available_height() * COLUMN_TOP).max(COLUMN_MIN_TOP);

        ui.add_space(top_space);
        ui.horizontal(|ui| {
            ui.add_space(side_space);
            ui.vertical(|ui| {
                ui.set_width(width);
                add_contents(ui);
            });
        });
    });
}

/// A rounded panel with a soft shadow, grouping a screen's controls over the background
pub fn card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    card_with_margin(ui, CARD_MARGIN, add_contents)
}

/// A card with the given room around its contents, for narrow spaces like the game's side panel
pub fn card_with_margin<R>(
    ui: &mut egui::Ui,
    margin: i8,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    card_frame(&palette_of(ui), margin)
        .show(ui, add_contents)
        .inner
}

/// The look of a card: rounded, outlined and shadowed, with the given room around its contents.
/// Popups use it too, so they match the cards
pub fn card_frame(palette: &Palette, margin: i8) -> egui::Frame {
    egui::Frame::new()
        .fill(palette.card)
        .stroke(Stroke::new(1.0, palette.card_stroke))
        .corner_radius(CARD_RADIUS)
        .shadow(CARD_SHADOW)
        .inner_margin(margin)
}

/// A screen's title in the serif title font, centred above its card
pub fn screen_title(ui: &mut egui::Ui, text: &str) {
    let palette = palette_of(ui);
    ui.vertical_centered(|ui| {
        let title = RichText::new(text)
            .font(title_font(TITLE_TEXT_SIZE))
            .color(palette.text);
        ui.label(title);
    });
    ui.add_space(SECTION_GAP);
}

/// A heading for a group of choices, in small muted capitals
pub fn section_label(ui: &mut egui::Ui, text: &str) {
    let palette = palette_of(ui);
    let label = RichText::new(text.to_uppercase())
        .size(SMALL_TEXT_SIZE)
        .strong()
        .color(palette.text_muted);
    ui.label(label);
}

/// A small icon followed by a label, for the rows of a form
pub fn icon_label(ui: &mut egui::Ui, sprites: &mut Sprites, icon: Icon, text: &str) {
    let palette = palette_of(ui);
    ui.horizontal(|ui| {
        if let Some(image) = icon_image(ui.ctx(), sprites, icon, ICON_SIZE, palette.text_muted) {
            ui.add(image);
        }
        ui.label(RichText::new(text).color(palette.text));
    });
}
