use super::fonts::install_fonts;
use super::palette::{DARK, LIGHT, Palette};
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Margin, Shadow, Stroke, Style, TextStyle, Theme, Vec2,
    style::ScrollStyle,
};

/// Text sizes: hints, ordinary text and buttons, big buttons and headlines, and screen titles
pub const SMALL_TEXT_SIZE: f32 = 13.0;
pub const BODY_TEXT_SIZE: f32 = 16.0;
pub const LARGE_TEXT_SIZE: f32 = 19.0;
pub const TITLE_TEXT_SIZE: f32 = 34.0;

/// Size of the icons beside labels and inside text fields
pub const ICON_SIZE: f32 = 20.0;

/// Gaps down a screen: between the parts of a card, and before a group of buttons
pub const SECTION_GAP: f32 = 14.0;
pub const GROUP_GAP: f32 = 20.0;

/// Rounding of buttons and fields, and of cards and popups
pub const WIDGET_RADIUS: u8 = 10;
pub const CARD_RADIUS: u8 = 18;

/// Soft shadow under cards and popups, so they float over the background
pub const CARD_SHADOW: Shadow = Shadow {
    offset: [0, 8],
    blur: 28,
    spread: 0,
    color: Color32::from_black_alpha(90),
};

/// Set up fonts and style both of egui's themes once at startup. Switching dark mode then picks one of them
pub fn install(ctx: &egui::Context) {
    install_fonts(ctx);
    ctx.style_mut_of(Theme::Dark, |style| apply(style, &DARK));
    ctx.style_mut_of(Theme::Light, |style| apply(style, &LIGHT));
}

/// Switch everything between dark and light mode
pub fn set_dark_mode(ctx: &egui::Context, dark_mode: bool) {
    let theme = if dark_mode { Theme::Dark } else { Theme::Light };
    ctx.set_theme(theme);
}

/// Set text sizes, spacing and every widget colour from a palette
fn apply(style: &mut Style, palette: &Palette) {
    style.text_styles = [
        (TextStyle::Heading, FontId::proportional(TITLE_TEXT_SIZE)),
        (TextStyle::Body, FontId::proportional(BODY_TEXT_SIZE)),
        (TextStyle::Button, FontId::proportional(BODY_TEXT_SIZE)),
        (TextStyle::Small, FontId::proportional(SMALL_TEXT_SIZE)),
        (TextStyle::Monospace, FontId::monospace(14.0)),
    ]
    .into();

    style.spacing.item_spacing = Vec2::new(10.0, 10.0);
    style.spacing.button_padding = Vec2::new(16.0, 9.0);
    style.spacing.interact_size.y = 34.0;
    // Room inside drop-down lists and other popups
    style.spacing.menu_margin = Margin::same(8);
    // Scroll bars get their own lane with a gap, instead of floating over the content,
    // so highlighted rows and outlines never touch them
    style.spacing.scroll = ScrollStyle {
        bar_inner_margin: 8.0,
        bar_outer_margin: 2.0,
        ..ScrollStyle::solid()
    };

    let visuals = &mut style.visuals;
    // Panels are see-through so the background shows behind every screen
    visuals.panel_fill = Color32::TRANSPARENT;
    visuals.window_fill = palette.card.to_opaque();
    visuals.window_stroke = Stroke::new(1.0, palette.card_stroke);
    visuals.window_corner_radius = CornerRadius::same(CARD_RADIUS);
    visuals.window_shadow = CARD_SHADOW;
    visuals.popup_shadow = CARD_SHADOW;
    visuals.menu_corner_radius = CornerRadius::same(WIDGET_RADIUS);

    visuals.extreme_bg_color = palette.field;
    visuals.text_edit_bg_color = Some(palette.field);
    visuals.error_fg_color = palette.error;
    visuals.hyperlink_color = palette.accent;
    visuals.selection.bg_fill = palette.accent.gamma_multiply(0.45);
    visuals.selection.stroke = Stroke::new(1.0, palette.accent);

    let radius = CornerRadius::same(WIDGET_RADIUS);
    let widgets = &mut visuals.widgets;
    widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.text);
    widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.card_stroke);
    widgets.noninteractive.corner_radius = radius;

    let states = [
        (&mut widgets.inactive, palette.button),
        (&mut widgets.hovered, palette.button_hovered),
        (&mut widgets.active, palette.button_pressed),
        (&mut widgets.open, palette.button_hovered),
    ];
    for (state, fill) in states {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::NONE;
        state.fg_stroke = Stroke::new(1.5, palette.text);
        state.corner_radius = radius;
        state.expansion = 0.0;
    }
    // A thin gold outline shows which button the pointer is over
    widgets.hovered.bg_stroke = Stroke::new(1.0, palette.accent.gamma_multiply(0.6));
}
