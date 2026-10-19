use super::Navigation;
use crate::gui::assets::{Icon, Sprites};
use crate::gui::widgets::{self, MENU_BUTTON_SIZE};
use eframe::egui::{self, Image, Sense, vec2};

/// Width of the menu's column, and of the logo in it
const COLUMN_WIDTH: f32 = 520.0;
const LOGO_WIDTH: f32 = 500.0;

/// Room above the logo, and between it and the buttons
const ABOVE_LOGO: f32 = 24.0;
const BELOW_LOGO: f32 = 36.0;

/// Where the word's centre sits across the logo, as a share of its width: the gold rule under it is
/// centred at x = 373.5 of 600 in assets/ui/logo_*.svg. Update this if the logo's layout changes
const LOGO_WORD_CENTER: f32 = 373.5 / 600.0;

/// Draw the main menu: the logo over Play, Settings and Exit
pub fn show(ui: &mut egui::Ui, sprites: &mut Sprites) -> Option<Navigation> {
    let mut navigation = None;

    widgets::centered_column(ui, COLUMN_WIDTH, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(ABOVE_LOGO);
            logo(ui, sprites);
            ui.add_space(BELOW_LOGO);

            widgets::card(ui, |ui| {
                ui.set_width(MENU_BUTTON_SIZE.x);
                let buttons = [
                    (Icon::Play, "Play", Navigation::NewGame, true),
                    (Icon::Settings, "Settings", Navigation::Settings, false),
                    (Icon::Exit, "Exit", Navigation::Exit, false),
                ];
                for (icon, text, target, primary) in buttons {
                    if widgets::menu_button(ui, sprites, icon, text, primary).clicked() {
                        navigation = Some(target);
                    }
                }
            });
        });
    });

    navigation
}

/// Draw the logo, as wide as fits up to its full size
fn logo(ui: &mut egui::Ui, sprites: &mut Sprites) {
    let width = LOGO_WIDTH.min(ui.available_width());
    let Some(texture) = sprites.logo(ui.ctx(), ui.visuals().dark_mode) else {
        return;
    };
    let size = texture.size_vec2() * (width / texture.size_vec2().x);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());

    // Centre the word over the menu rather than the whole image, so the crown hangs out to the left
    let shift = (LOGO_WORD_CENTER - 0.5) * width;
    let image = Image::new((texture.id(), size));
    image.paint_at(ui, rect.translate(vec2(-shift, 0.0)));
}
