use eframe::egui::{self, Color32};

/// The colours of one theme, picked to sit on the backgrounds: neutral greys for dark mode, warm stone for light,
/// with gold as the only colour. Neither has anything near white, to be easy on the eyes
#[derive(Clone, Copy)]
pub struct Palette {
    // Shown behind everything until the background art has loaded
    pub background: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    // Gold, for the main button on each screen, selections and highlights
    pub accent: Color32,
    pub on_accent: Color32,
    // Cards are slightly see-through so the background shows at the edges
    pub card: Color32,
    pub card_stroke: Color32,
    pub button: Color32,
    pub button_hovered: Color32,
    pub button_pressed: Color32,
    pub field: Color32,
    pub error: Color32,
}

pub const DARK: Palette = Palette {
    background: Color32::from_rgb(43, 45, 49),
    text: Color32::from_rgb(228, 230, 233),
    text_muted: Color32::from_rgb(152, 157, 165),
    accent: Color32::from_rgb(222, 172, 72),
    on_accent: Color32::from_rgb(34, 28, 18),
    card: Color32::from_rgba_unmultiplied_const(22, 23, 26, 220),
    card_stroke: Color32::from_rgba_unmultiplied_const(255, 255, 255, 22),
    button: Color32::from_rgb(50, 52, 57),
    button_hovered: Color32::from_rgb(64, 67, 73),
    button_pressed: Color32::from_rgb(78, 82, 89),
    field: Color32::from_rgb(30, 31, 35),
    error: Color32::from_rgb(236, 104, 86),
};

// Warm stone rather than cool grey, since gold looks muddy on blue-greys. Mid-toned, so the screen
// isn't bright, with cards a step lighter than the background so they stand off it
pub const LIGHT: Palette = Palette {
    background: Color32::from_rgb(168, 159, 146),
    text: Color32::from_rgb(42, 33, 24),
    text_muted: Color32::from_rgb(100, 88, 74),
    accent: Color32::from_rgb(160, 110, 34),
    on_accent: Color32::from_rgb(250, 243, 230),
    card: Color32::from_rgba_unmultiplied_const(214, 206, 193, 240),
    card_stroke: Color32::from_rgba_unmultiplied_const(70, 48, 20, 45),
    button: Color32::from_rgb(198, 189, 175),
    button_hovered: Color32::from_rgb(188, 178, 163),
    button_pressed: Color32::from_rgb(176, 165, 149),
    field: Color32::from_rgb(224, 217, 205),
    error: Color32::from_rgb(170, 48, 36),
};

/// Return the colours for dark or light mode
pub fn palette(dark_mode: bool) -> Palette {
    if dark_mode { DARK } else { LIGHT }
}

/// Return the colours for whichever mode this ui is drawn in
pub fn palette_of(ui: &egui::Ui) -> Palette {
    palette(ui.visuals().dark_mode)
}
