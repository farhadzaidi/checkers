use super::images::icon_image;
use crate::gui::assets::{Icon, Sprites};
use crate::gui::theme::{BODY_TEXT_SIZE, LARGE_TEXT_SIZE, palette_of};
use eframe::egui::{self, CursorIcon, Response, RichText, Vec2, vec2};

/// Size of the big buttons on the menus
pub const MENU_BUTTON_SIZE: Vec2 = vec2(280.0, 50.0);

/// How much taller a button's icon is than its text, so they look the same size
const ICON_OVER_TEXT: f32 = 4.0;

/// A big menu button with an icon. The primary one is gold, for the main action of a screen
pub fn menu_button(
    ui: &mut egui::Ui,
    sprites: &mut Sprites,
    icon: Icon,
    text: &str,
    primary: bool,
) -> Response {
    icon_button_sized(
        ui,
        sprites,
        icon,
        text,
        primary,
        LARGE_TEXT_SIZE,
        MENU_BUTTON_SIZE,
    )
}

/// An ordinary button with an icon
pub fn icon_button(ui: &mut egui::Ui, sprites: &mut Sprites, icon: Icon, text: &str) -> Response {
    icon_button_sized(ui, sprites, icon, text, false, BODY_TEXT_SIZE, Vec2::ZERO)
}

/// A button with an icon that fills the width it's given, for stacks of buttons that line up
pub fn wide_button(ui: &mut egui::Ui, sprites: &mut Sprites, icon: Icon, text: &str) -> Response {
    let width = Vec2::new(ui.available_width(), 0.0);
    icon_button_sized(ui, sprites, icon, text, false, BODY_TEXT_SIZE, width)
}

/// A button with an icon before its text. The icon is left out until it has loaded
fn icon_button_sized(
    ui: &mut egui::Ui,
    sprites: &mut Sprites,
    icon: Icon,
    text: &str,
    primary: bool,
    text_size: f32,
    min_size: Vec2,
) -> Response {
    let palette = palette_of(ui);
    let color = if primary {
        palette.on_accent
    } else {
        palette.text
    };

    let image = icon_image(ui.ctx(), sprites, icon, text_size + ICON_OVER_TEXT, color);
    let label = RichText::new(text).size(text_size).color(color);
    // Button labels stay on one line, never breaking mid-word in a narrow space
    let mut button = egui::Button::opt_image_and_text(image, Some(label.into()))
        .min_size(min_size)
        .wrap_mode(egui::TextWrapMode::Extend);
    if primary {
        button = button.fill(palette.accent);
    }

    ui.add(button).on_hover_cursor(CursorIcon::PointingHand)
}
