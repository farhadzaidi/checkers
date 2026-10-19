mod fonts;
mod palette;
mod style;

pub use fonts::title_font;
pub use palette::{Palette, palette, palette_of};
pub use style::{
    BODY_TEXT_SIZE, CARD_RADIUS, CARD_SHADOW, GROUP_GAP, ICON_SIZE, LARGE_TEXT_SIZE, SECTION_GAP,
    SMALL_TEXT_SIZE, TITLE_TEXT_SIZE, WIDGET_RADIUS, install, set_dark_mode,
};
