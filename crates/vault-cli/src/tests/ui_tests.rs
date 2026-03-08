use crate::ui::*;
use ratatui::prelude::{Color, Size};

#[test]
fn test_viewport_height_calculation() {
    let size = Size {
        width: 100,
        height: 50,
    };
    let vh = viewport_height(size);

    assert!(vh > 0);
    assert!(vh < 50);
}

#[test]
fn test_provider_style_mapping() {
    // We need to use direct access since it's now private or we can test via draw calls
    // But since we are in the same crate we can use crate::ui::body::provider_style
    // Actually, tests in src/tests follow the module hierarchy

    // For OpenAI (Green)
    let style = crate::ui::body::provider_style("OpenAI API Key");
    assert_eq!(style.fg, Some(Color::Rgb(0, 255, 0)));

    // Gemini (Blue)
    let style = crate::ui::body::provider_style("Gemini API Key");
    assert_eq!(style.fg, Some(Color::Rgb(0, 0, 255)));

    // Unrecognized
    let style = crate::ui::body::provider_style("Unknown");
    assert_eq!(style.fg, Some(Color::Gray));
}

#[test]
fn test_logo_dimensions() {
    let width = crate::ui::header::logo_max_width();
    assert!(width > 0);

    let count = crate::ui::header::logo_line_count();
    assert!(count > 0);
}

#[test]
fn test_preferred_height() {
    let h = crate::ui::header::preferred_height();
    assert!(h >= 11);
}
