#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod codec;
mod platform;
mod sample;
mod view;

use iced::window;
use iced::{Font, Size, Theme};

fn main() -> iced::Result {
    iced::application(app::App::new, app::App::update, app::App::view)
        .title("Base64 转图片")
        .theme(Theme::Light)
        .default_font(Font::with_name(platform::UI_FONT))
        .window(window::Settings {
            size: Size::new(1080.0, 830.0),
            min_size: Some(Size::new(960.0, 720.0)),
            ..window::Settings::default()
        })
        .antialiasing(true)
        .run()
}
