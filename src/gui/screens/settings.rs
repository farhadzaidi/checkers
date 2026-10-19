use super::Navigation;
use crate::gui::assets::{BoardStyle, Icon, PieceStyle, Sprites};
use crate::gui::settings::Settings;
use crate::gui::theme::{self, GROUP_GAP};
use crate::gui::widgets;
use eframe::egui;

/// Width of the screen's column
const COLUMN_WIDTH: f32 = 500.0;

/// Gaps between the settings' columns and rows
const GRID_SPACING: [f32; 2] = [32.0, 18.0];

/// Width of the drop-downs, and the tallest their lists get: enough to show every style without scrolling
const COMBO_WIDTH: f32 = 220.0;
const COMBO_LIST_HEIGHT: f32 = 480.0;

/// Draw the settings screen. Changes apply right away
pub fn show(
    ui: &mut egui::Ui,
    sprites: &mut Sprites,
    settings: &mut Settings,
) -> Option<Navigation> {
    let mut navigation = None;

    widgets::centered_column(ui, COLUMN_WIDTH, |ui| {
        widgets::screen_title(ui, "Settings");
        widgets::card(ui, |ui| {
            egui::Grid::new("settings")
                .num_columns(2)
                .spacing(GRID_SPACING)
                .show(ui, |ui| rows(ui, sprites, settings));
            ui.add_space(GROUP_GAP);

            if widgets::icon_button(ui, sprites, Icon::Back, "Back").clicked() {
                navigation = Some(Navigation::Back);
            }
        });
    });

    navigation
}

/// One row per setting: its icon and name, then the control that changes it
fn rows(ui: &mut egui::Ui, sprites: &mut Sprites, settings: &mut Settings) {
    widgets::icon_label(ui, sprites, Icon::Board, "Board");
    style_picker(
        ui,
        "board",
        &mut settings.board_style,
        &BoardStyle::ALL,
        BoardStyle::label,
    );
    ui.end_row();

    widgets::icon_label(ui, sprites, Icon::Pieces, "Pieces");
    style_picker(
        ui,
        "pieces",
        &mut settings.piece_style,
        &PieceStyle::ALL,
        PieceStyle::label,
    );
    ui.end_row();

    // The icon shows the mode that's on: a moon for dark, a sun for light
    let mode_icon = if settings.dark_mode {
        Icon::DarkMode
    } else {
        Icon::LightMode
    };
    widgets::icon_label(ui, sprites, mode_icon, "Dark mode");
    if widgets::switch(ui, &mut settings.dark_mode).changed() {
        theme::set_dark_mode(ui.ctx(), settings.dark_mode);
    }
    ui.end_row();

    let moves_icon = if settings.show_legal_moves {
        Icon::ShowMoves
    } else {
        Icon::HideMoves
    };
    widgets::icon_label(ui, sprites, moves_icon, "Show legal moves");
    widgets::switch(ui, &mut settings.show_legal_moves);
    ui.end_row();
}

/// A drop-down for picking one of several styles
fn style_picker<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id: &str,
    current: &mut T,
    options: &[T],
    label: impl Fn(T) -> &'static str,
) {
    egui::ComboBox::from_id_salt(id)
        .width(COMBO_WIDTH)
        .height(COMBO_LIST_HEIGHT)
        .selected_text(label(*current))
        .show_ui(ui, |ui| {
            for &option in options {
                ui.selectable_value(current, option, label(option));
            }
        });
}
