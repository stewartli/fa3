pub mod account;
mod recon;
mod tree;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    File,
    Edit,
    View,
}

#[derive(Clone)]
pub enum Message {
    // menu
    OpenMenu(MenuKind),
    CloseMenu,
    New,
    Open,
    Save,
    Exit,
    Rename,
    Delete,
    Expand,
    Collapse,
    // toolbar
    Back,
    Forward,
    Up,
    Refresh,
    TogglePath(Vec<usize>),
    SearchPath(String),
    // key
    A(iced::Size),
    EventOccurred(iced::Event),
}
