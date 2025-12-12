use ratatui::style::Color;

/// Theme configuration for the application
#[derive(Clone)]
pub struct Theme {
    // Header colors
    pub header_bg: Color,
    pub header_fg: Color,
    pub button_fg: Color,

    // Panel borders (diff mode)
    pub focused_border: Color,
    pub unfocused_border: Color,

    // Diff highlighting
    pub diff_added: Color,
    pub diff_removed: Color,
    pub diff_modified: Color,

    // Cursor/selection
    pub cursor_bg: Color,

    // Syntax highlighting (future use)
    #[allow(dead_code)]
    pub syntax_string: Color,
    #[allow(dead_code)]
    pub syntax_number: Color,
    #[allow(dead_code)]
    pub syntax_boolean: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // Header: pastel purple background with white text
            header_bg: Color::Indexed(56),
            header_fg: Color::White,
            button_fg: Color::Yellow,

            // Panel borders
            focused_border: Color::Cyan,
            unfocused_border: Color::DarkGray,

            // Diff colors
            diff_added: Color::Green,
            diff_removed: Color::Red,
            diff_modified: Color::Yellow,

            // Cursor
            cursor_bg: Color::DarkGray,

            // Syntax highlighting
            syntax_string: Color::Cyan,
            syntax_number: Color::Magenta,
            syntax_boolean: Color::Blue,
        }
    }
}
