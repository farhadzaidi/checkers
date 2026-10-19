use crate::gui::assets::{self, Sprites};
use crate::gui::screens::{self, GameScreen, Navigation, NewGameScreen};
use crate::gui::settings::Settings;
use crate::gui::{theme, widgets};
use eframe::egui;

/// Names the app to the desktop, and the folder its settings are saved in
const APP_ID: &str = "checkers";

/// Window size the app opens at, and the smallest it can be resized to
const DEFAULT_WINDOW_SIZE: [f32; 2] = [1100.0, 800.0];
const MIN_WINDOW_SIZE: [f32; 2] = [700.0, 520.0];

/// How long a new screen takes to fade in, in seconds
const SCREEN_FADE_SECONDS: f64 = 0.2;

/// Open the checkers window and run until it's closed
pub fn run() -> eframe::Result {
    // On Wayland the dock finds the icon through the app id, in a checkers.desktop file. Elsewhere the window icon is used
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Checkers")
        .with_app_id(APP_ID)
        .with_inner_size(DEFAULT_WINDOW_SIZE)
        .with_min_inner_size(MIN_WINDOW_SIZE);
    if let Some(icon) = assets::app_icon() {
        viewport = viewport.with_icon(icon);
    }

    // The OpenGL renderer, because it builds mipmaps for our textures and egui's wgpu one doesn't.
    // Without them, images drawn much smaller than they were rasterized get jagged edges.
    // Open in the middle of the screen; Wayland doesn't let apps place their own windows, so there it's up to the desktop
    let options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        APP_ID,
        options,
        Box::new(|creation| Ok(Box::new(CheckersApp::new(creation)))),
    )
}

/// Which screen the window is showing
#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    MainMenu,
    NewGame,
    Settings,
    Game,
}

/// The app: which screen is showing, the screens' state, and the images they share
struct CheckersApp {
    screen: Screen,
    // Where Back goes from the new game and settings screens: wherever they were opened from
    back_to: Screen,
    // The screen drawn last frame, and when it first appeared, for fading new screens in
    shown_screen: Screen,
    shown_at: f64,
    settings: Settings,
    new_game: NewGameScreen,
    // None until the first game starts
    game: Option<GameScreen>,
    sprites: Sprites,
}

impl CheckersApp {
    /// Set up the app when the window opens, restoring the last run's settings
    fn new(creation: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load(creation.storage);
        theme::install(&creation.egui_ctx);
        theme::set_dark_mode(&creation.egui_ctx, settings.dark_mode);

        Self {
            screen: Screen::MainMenu,
            back_to: Screen::MainMenu,
            shown_screen: Screen::MainMenu,
            shown_at: 0.0,
            settings,
            new_game: NewGameScreen::default(),
            game: None,
            sprites: Sprites::new(),
        }
    }

    /// Draw the current screen, and return where it asks to go next
    fn show_screen(&mut self, ui: &mut egui::Ui) -> Option<Navigation> {
        let sprites = &mut self.sprites;
        match self.screen {
            Screen::MainMenu => in_window(ui, |ui| screens::main_menu::show(ui, sprites)),
            Screen::NewGame => {
                let piece_style = self.settings.piece_style;
                in_window(ui, |ui| self.new_game.show(ui, sprites, piece_style))
            }
            Screen::Settings => in_window(ui, |ui| {
                screens::settings::show(ui, sprites, &mut self.settings)
            }),
            // The game screen lays out its own panels
            Screen::Game => match &mut self.game {
                Some(game) => game.show(ui, sprites, &self.settings),
                None => Some(Navigation::MainMenu),
            },
        }
    }

    /// Go where a screen asked
    fn navigate(&mut self, navigation: Navigation, ctx: &egui::Context) {
        match navigation {
            Navigation::NewGame => self.open(Screen::NewGame),
            Navigation::Settings => self.open(Screen::Settings),
            Navigation::StartGame {
                session,
                bottom_side,
            } => {
                self.game = Some(GameScreen::new(session, bottom_side));
                self.screen = Screen::Game;
            }
            Navigation::Back => self.screen = self.back_to,
            Navigation::MainMenu => self.screen = Screen::MainMenu,
            Navigation::Exit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }

    /// Show a screen that Back returns from, to wherever it was opened from
    fn open(&mut self, screen: Screen) {
        self.back_to = self.screen;
        self.screen = screen;
    }

    /// Fade a screen in over its first moments, leaving the background in place behind it
    fn fade_in_new_screen(&mut self, ui: &mut egui::Ui) {
        let now = ui.input(|input| input.time);
        if self.screen != self.shown_screen {
            self.shown_screen = self.screen;
            self.shown_at = now;
        }

        let opacity = ((now - self.shown_at) / SCREEN_FADE_SECONDS) as f32;
        if opacity < 1.0 {
            ui.multiply_opacity(opacity);
            ui.ctx().request_repaint();
        }
    }
}

impl eframe::App for CheckersApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Pick up any images finished on background threads, and start any the current settings need,
        // so a new style starts loading the moment it's picked
        self.sprites.receive(ui.ctx());
        self.sprites.preload(ui.ctx(), &self.settings);
        widgets::background(ui.ctx(), &mut self.sprites, self.settings.dark_mode);
        self.fade_in_new_screen(ui);

        if let Some(navigation) = self.show_screen(ui) {
            self.navigate(navigation, ui.ctx());
        }
    }

    /// Save the settings when the app closes, and every so often while it runs
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.settings.save(storage);
    }
}

/// Draw a screen that fills the window
fn in_window<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::CentralPanel::default().show(ui, add_contents).inner
}
