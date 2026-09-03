use ratatui::style::{Color, Modifier, Style};

use crate::backend::display_char::{BaseStyle, DeprecatedState, DiagnosticStyle};
macro_rules! rgb {
    ($hex:expr) => {{
        let hex = $hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
        Color::Rgb(r, g, b)
    }};
}

// pub static C_BG_DIALOG_PRIMARY: Color = Color::Rgb(121, 88, 220);
// pub static C_OTHER_GRAY: Color = Color::Rgb(90, 89, 119);
// pub static C_OTHER_GRAY_BOLD: Color = Color::Rgb(219, 191, 239);
// pub static C_FG_DIALOG_PRIMARY: Color = Color::Rgb(23, 20, 82);
// pub static C_BG_CURSOR_SELECTION: Color = Color::Rgb(111, 68, 240);
// pub static C_BG_SELECTION: Color = Color::Rgb(84, 0, 153);

// Log colors:

pub static C_LOG_ERROR: Color = Color::LightRed;
pub static C_LOG_INFO: Color = Color::LightYellow;
pub static C_LOG_HINT: Color = Color::LightYellow;
pub static C_LOG_WARNING: Color = Color::Yellow;
pub static C_LOG_TODO: Color = Color::Rgb(139, 179, 61);

// Cursor & selection colors:
pub static C_BG_SELECTION: Color = Color::Rgb(84, 0, 153);
pub static C_FG_SELECTION: Color = C_FG_NORMAL;
pub static C_BG_CURSOR_SELECTION: Color = Color::Rgb(111, 68, 240);
pub static C_FG_CURSOR_SELECTION: Color = C_FG_NORMAL;
pub static C_FG_CURSOR: Color = Color::Rgb(175, 171, 234);
pub static C_BG_CURSOR: Color = Color::Rgb(255, 255, 255);

// Tree colors:
pub static C_TREE_FG_FILE: Color = C_FG_NORMAL;
pub static C_TREE_FG_DIR: Color = Color::LightGreen;

// Parts: colors
pub static C_FG_LINE_NUMBERS: Color = Color::Rgb(131, 126, 186);
pub static C_FG_SCROLLBAR: Color = C_FG_LINE_NUMBERS;

pub static C_MENU_FG: Color = C_FG_BAR;
pub static C_MENU_BG: Color = C_BG_BAR;

pub static C_BG_NORMAL: Color = Color::Rgb(59, 34, 76);
pub static C_FG_NORMAL: Color = Color::Rgb(163, 159, 231);
pub static C_BG_SPECIAL_BYTES: [Color; 2] = [Color::Green, Color::Red];

pub static C_BG_BAR: Color = Color::Rgb(40, 23, 51);
pub static C_FG_BAR: Color = Color::Rgb(208, 181, 228);

pub fn apply_base_style(base_style: BaseStyle) -> Style {
    match base_style {
        BaseStyle::NonStyled => Style::new(),
        BaseStyle::Comment => Style::new().fg(rgb!("#7A7E85")),
        BaseStyle::DocComment => Style::new().fg(rgb!("#5F826B")),
        BaseStyle::ConstantOrField => Style::new().fg(rgb!("C77DBB")),
        BaseStyle::FunctionOrOverloadOperators => Style::new().fg(rgb!("#6AA2D7")),
        BaseStyle::Number => Style::new().fg(rgb!("#2AACB8")),
        BaseStyle::String => Style::new().fg(rgb!("#6AAB73")),
        BaseStyle::KeywordOrStringEscape => Style::new().fg(rgb!("#CF8E6D")),
        BaseStyle::Attribute => Style::new().fg(rgb!("#B3AE60")),
        BaseStyle::Macro => Style::new().fg(rgb!("#D5A563")),
        BaseStyle::LabelOrLifeTime => Style::new().fg(rgb!("#20999D")),
        BaseStyle::Self_ => Style::new().fg(rgb!("#E59EAE")),
        BaseStyle::TypeParameter => Style::new().fg(rgb!("#3CACAC")),
        BaseStyle::Type => Style::new().fg(rgb!("#A6BB77")),
        BaseStyle::EnumVariant => Style::new().fg(rgb!("#8CC8D4")),
        BaseStyle::Trait => Style::new().fg(rgb!("#8D91DC")),
        BaseStyle::UnsafeCall => Style::new().fg(rgb!("#6AA2D7")).bg(rgb!("#4E2C28")),
        BaseStyle::QuestionMark => Style::new().fg(rgb!("#D8A460")),
        BaseStyle::Todo => Style::new().fg(rgb!("#8BB33D")),
        BaseStyle::Unused => Style::new().fg(rgb!("#6F737A")),
        BaseStyle::ConditionallyDisabled => Style::new().fg(rgb!("#8A8E57")),
        BaseStyle::LspHint => Style::new().fg(rgb!("#858A94")).bg(rgb!("#393B40")),
        BaseStyle::LspLens => Style::new().fg(rgb!("#858A94")).bg(rgb!("#393B40")),
        BaseStyle::Autocompletion => Style::new().fg(rgb!("#2B2D30")),
    }
}

pub fn apply_diagnostic_style(diagnostic_style: DiagnosticStyle) -> Style {
    match diagnostic_style {
        DiagnosticStyle::None => Style::new(),
        DiagnosticStyle::Error => Style::new()
            .add_modifier(Modifier::UNDER_CURLED)
            .underline_color(rgb!("#FA6675")),
        DiagnosticStyle::UnknownSymbol => Style::new().fg(rgb!("#F75464")),
        DiagnosticStyle::RuntimeProblem => Style::new()
            .add_modifier(Modifier::UNDER_DOTTED)
            .underline_color(rgb!("#F2C55C")),
        DiagnosticStyle::Warning => Style::new()
            .add_modifier(Modifier::UNDER_CURLED)
            .underline_color(rgb!("#F2C55C")),
        DiagnosticStyle::Typo => Style::new()
            .add_modifier(Modifier::UNDER_CURLED)
            .underline_color(rgb!("#7EC482")),
    }
}

pub fn apply_deprecated_state(deprecated_state: DeprecatedState) -> Style {
    if deprecated_state.0 {
        Style::new().crossed_out()
    } else {
        Style::new()
    }
}
