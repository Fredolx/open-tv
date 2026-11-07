mod settings_view;

use gpui_kit::{App, px};
use gpui_kit::component::Theme;

pub use settings_view::SettingsView;

/// gpui-kit's base text size; every rem-based size in the UI scales from it.
const BASE_FONT_SIZE: f32 = 16.;

/// Scales the whole UI by the zoom setting (a percentage, default 100).
pub fn apply_zoom(zoom: Option<u16>, cx: &mut App) {
    let scale = f32::from(zoom.unwrap_or(100).clamp(50, 300)) / 100.;
    Theme::update(cx, |theme| theme.font_size = px(BASE_FONT_SIZE * scale));
}
