//! Theme tokens for Genasis TUI. M9 ties to docs/design-system.md.

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub fg: ratatui::style::Color,
    pub bg: ratatui::style::Color,
    pub accent: ratatui::style::Color,
    pub warn: ratatui::style::Color,
    pub error: ratatui::style::Color,
}

impl Default for Theme {
    fn default() -> Self {
        use ratatui::style::Color;
        Self {
            fg: Color::White,
            bg: Color::Reset,
            accent: Color::Cyan,
            warn: Color::Yellow,
            error: Color::Red,
        }
    }
}
