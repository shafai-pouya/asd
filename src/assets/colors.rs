#[allow(unused)]
pub mod colors {
    use ratatui::style::Color;

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
    
    
    pub static C_MENU_FG: Color = Color::Black;
    pub static C_MENU_BG: Color = Color::LightRed;


    pub static C_BG_NORMAL: Color = Color::Rgb(59, 34, 76);
    pub static C_FG_NORMAL: Color = Color::Rgb(163, 159, 231);
    
    
    pub static C_BG_BAR: Color = Color::Rgb(40, 23, 51);
    pub static C_FG_BAR: Color = Color::Rgb(208, 181, 228);
}