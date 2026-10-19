use super::catalog::{self, BoardStyle, Icon, Overlay, PieceStyle};
use crate::game::Piece;
use crate::gui::settings::Settings;
use eframe::egui::{self, ColorImage, TextureFilter, TextureHandle, TextureOptions};
use egui::load::SizeHint;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// Pixel sizes the board and pieces are rasterized at. Each image is rasterized once,
/// then the GPU scales it to whatever size the window needs
const BOARD_PIXELS: u32 = 1600;
const PIECE_PIXELS: u32 = 256;
const OVERLAY_PIXELS: u32 = 256;
const ICON_PIXELS: u32 = 96;
const LOGO_PIXELS: u32 = 1200;
const BACKGROUND_PIXELS: u32 = 1920;
const APP_ICON_PIXELS: u32 = 256;

/// Smooth scaling, with mipmaps so images stay clean when drawn much smaller
const SMOOTH: TextureOptions = TextureOptions {
    mipmap_mode: Some(TextureFilter::Linear),
    ..TextureOptions::LINEAR
};

/// One image the app draws
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SpriteKey {
    Board(BoardStyle),
    Piece(PieceStyle, Piece),
    Overlay(Overlay),
    Icon(Icon),
    // The dark mode version, or the light one
    Logo(bool),
    Background(bool),
}

/// A finished rasterization, sent back from a background thread
type Rasterized = (SpriteKey, Result<ColorImage, String>);

/// Every image the app draws, rasterized from SVG on background threads
pub struct Sprites {
    textures: HashMap<SpriteKey, TextureHandle>,
    // Started but not finished, or failed, so neither gets started again
    pending: HashSet<SpriteKey>,
    failed: HashSet<SpriteKey>,
    sender: Sender<Rasterized>,
    receiver: Receiver<Rasterized>,
}

impl Sprites {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            textures: HashMap::new(),
            pending: HashSet::new(),
            failed: HashSet::new(),
            sender,
            receiver,
        }
    }

    /// Return the board image, or None while it's still being rasterized
    pub fn board(&mut self, ctx: &egui::Context, style: BoardStyle) -> Option<&TextureHandle> {
        self.get(ctx, SpriteKey::Board(style))
    }

    /// Return a piece image, or None while it's still being rasterized
    pub fn piece(
        &mut self,
        ctx: &egui::Context,
        style: PieceStyle,
        piece: Piece,
    ) -> Option<&TextureHandle> {
        self.get(ctx, SpriteKey::Piece(style, piece))
    }

    /// Return a square marker, or None while it's still being rasterized
    pub fn overlay(&mut self, ctx: &egui::Context, overlay: Overlay) -> Option<&TextureHandle> {
        self.get(ctx, SpriteKey::Overlay(overlay))
    }

    /// Return a button or label icon, or None while it's still being rasterized
    pub fn icon(&mut self, ctx: &egui::Context, icon: Icon) -> Option<&TextureHandle> {
        self.get(ctx, SpriteKey::Icon(icon))
    }

    /// Return the main menu's logo for dark or light mode, or None while it's still being rasterized
    pub fn logo(&mut self, ctx: &egui::Context, dark_mode: bool) -> Option<&TextureHandle> {
        self.get(ctx, SpriteKey::Logo(dark_mode))
    }

    /// Return the backdrop for dark or light mode, or None while it's still being rasterized
    pub fn background(&mut self, ctx: &egui::Context, dark_mode: bool) -> Option<&TextureHandle> {
        self.get(ctx, SpriteKey::Background(dark_mode))
    }

    /// Start rasterizing everything the screens draw with these settings, so it's ready before it's needed
    pub fn preload(&mut self, ctx: &egui::Context, settings: &Settings) {
        let interface = [
            SpriteKey::Logo(settings.dark_mode),
            SpriteKey::Background(settings.dark_mode),
        ];
        let icons = Icon::ALL.map(SpriteKey::Icon);
        for key in game_sprites(settings)
            .into_iter()
            .chain(interface)
            .chain(icons)
        {
            self.get(ctx, key);
        }
    }

    /// Return true once everything the game screen draws has loaded. Images that failed count, since they never will
    pub fn is_ready(&mut self, ctx: &egui::Context, settings: &Settings) -> bool {
        self.preload(ctx, settings);
        game_sprites(settings)
            .into_iter()
            .all(|key| self.textures.contains_key(&key) || self.failed.contains(&key))
    }

    /// Upload every image the background threads have finished since the last frame
    pub fn receive(&mut self, ctx: &egui::Context) {
        while let Ok((key, result)) = self.receiver.try_recv() {
            self.pending.remove(&key);
            match result {
                Ok(image) => {
                    let texture = ctx.load_texture(format!("{key:?}"), image, SMOOTH);
                    self.textures.insert(key, texture);
                }
                Err(error) => {
                    eprintln!("could not load {key:?}: {error}");
                    self.failed.insert(key);
                }
            }
        }
    }

    /// Return a texture if it's ready, otherwise make sure it's being rasterized
    fn get(&mut self, ctx: &egui::Context, key: SpriteKey) -> Option<&TextureHandle> {
        let started = self.pending.contains(&key) || self.failed.contains(&key);
        if !self.textures.contains_key(&key) && !started {
            self.start(ctx, key);
        }
        self.textures.get(&key)
    }

    /// Rasterize one image on a background thread, then wake the UI to pick it up
    fn start(&mut self, ctx: &egui::Context, key: SpriteKey) {
        self.pending.insert(key);

        let (path, pixels) = match key {
            SpriteKey::Board(style) => (style.path(), BOARD_PIXELS),
            SpriteKey::Piece(style, piece) => (style.path(piece), PIECE_PIXELS),
            SpriteKey::Overlay(overlay) => (overlay.path(), OVERLAY_PIXELS),
            SpriteKey::Icon(icon) => (icon.path(), ICON_PIXELS),
            SpriteKey::Logo(dark_mode) => (catalog::logo_path(dark_mode), LOGO_PIXELS),
            SpriteKey::Background(dark_mode) => {
                (catalog::background_path(dark_mode), BACKGROUND_PIXELS)
            }
        };
        let sender = self.sender.clone();
        let ctx = ctx.clone();

        thread::spawn(move || {
            let result = rasterize(&path, pixels);
            // The app may have closed, in which case nobody is listening anymore
            let _ = sender.send((key, result));
            ctx.request_repaint();
        });
    }
}

/// Every image the game screen draws: the board, all four pieces and the square markers
fn game_sprites(settings: &Settings) -> [SpriteKey; 8] {
    let piece = |piece| SpriteKey::Piece(settings.piece_style, piece);
    [
        SpriteKey::Board(settings.board_style),
        piece(Piece::BlackMan),
        piece(Piece::BlackKing),
        piece(Piece::WhiteMan),
        piece(Piece::WhiteKing),
        SpriteKey::Overlay(Overlay::Move),
        SpriteKey::Overlay(Overlay::Capture),
        SpriteKey::Overlay(Overlay::Selected),
    ]
}

/// Return the window icon: the red king from the classic set, the same piece as in the logo.
/// Rasterized right away, since the window needs it before it opens, which takes a few milliseconds
pub fn app_icon() -> Option<egui::IconData> {
    let path = PieceStyle::Classic.path(Piece::WhiteKing);
    let image = rasterize(&path, APP_ICON_PIXELS)
        .inspect_err(|error| eprintln!("could not load the app icon: {error}"))
        .ok()?;

    // The window wants plain RGBA bytes, where egui keeps colours with the alpha already applied
    let rgba = image
        .pixels
        .iter()
        .flat_map(|pixel| pixel.to_srgba_unmultiplied())
        .collect();
    Some(egui::IconData {
        rgba,
        width: image.size[0] as u32,
        height: image.size[1] as u32,
    })
}

/// Read an SVG file and rasterize it as big as fits in a square of the given size, keeping its shape
fn rasterize(path: &PathBuf, pixels: u32) -> Result<ColorImage, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let size = SizeHint::Size {
        width: pixels,
        height: pixels,
        maintain_aspect_ratio: true,
    };
    egui_extras::image::load_svg_bytes_with_size(&bytes, size, &resvg::usvg::Options::default())
}
