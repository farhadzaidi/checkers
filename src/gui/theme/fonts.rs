use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId, FontTweak};
use std::sync::Arc;

/// The fonts, built into the program so they're always there
const LATO: &[u8] = include_bytes!("../../../assets/fonts/Lato-Medium.ttf");
const SERIF: &[u8] = include_bytes!("../../../assets/fonts/NotoSerifDisplay-Bold.ttf");

/// Name of the font family used for screen titles
const TITLE_FAMILY: &str = "title";

/// Return the serif font for screen titles at the given size, matching the logo's lettering
pub fn title_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(TITLE_FAMILY.into()))
}

/// Use Lato for all text, and Noto Serif Display for titles. Their licences are in assets/fonts
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("lato".into(), Arc::new(FontData::from_static(LATO)));
    // Hinting snaps letters to the pixel grid, which keeps small text sharp but turns the curves and
    // thin strokes of a display serif at title size into stair steps. Titles are big, so leave it off
    let serif = FontData::from_static(SERIF).tweak(FontTweak {
        hinting: Some(false),
        ..FontTweak::default()
    });
    fonts.font_data.insert("serif".into(), Arc::new(serif));

    // Lato first, with egui's own fonts after it for any character Lato doesn't have
    if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
        proportional.insert(0, "lato".into());
    }
    fonts.families.insert(
        FontFamily::Name(TITLE_FAMILY.into()),
        vec!["serif".into(), "lato".into()],
    );
    ctx.set_fonts(fonts);
}
