use iced::{
    Element, Size,
    widget::{button, column, container, row, scrollable, text, text_input},
};

#[derive(Clone)]
pub enum Message {
    A(Size),
    Back,
    Forward,
    Up,
    Refresh,
    FolderSelected(usize),
    FileSelected(usize),
    PathChanged(String),
}

pub struct Account {
    path: String,
    folders: Vec<String>,
    files: Vec<SubItem>,
}

struct SubItem {
    name: String,
    size: u64,
    kind: String,
}

impl Account {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            path: "/home/stewart".into(),
            folders: vec!["Documents".into(), "Downloads".into(), "Pictures".into()],
            files: vec![
                SubItem {
                    name: "file1.txt".into(),
                    size: 100,
                    kind: "file".into(),
                },
                SubItem {
                    name: "photo.jpg".into(),
                    size: 2000,
                    kind: "image".into(),
                },
            ],
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        // toolbar ui
        let toolbar = row![
            button("←").on_press(Message::Back),
            button("→").on_press(Message::Forward),
            button("↑").on_press(Message::Up),
            button("⟳").on_press(Message::Refresh),
            text_input("path", &self.path)
                .on_input(Message::PathChanged)
                .width(iced::Length::Fill),
        ]
        .spacing(5);
        // content ui
        let mut folder_col = column![];
        for (i, folder) in self.folders.iter().enumerate() {
            folder_col =
                folder_col.push(button(folder.as_str()).on_press(Message::FolderSelected(i)));
        }
        let folder_panel = scrollable(folder_col).width(250);

        let mut file_col = column![];
        for file in &self.files {
            file_col = file_col.push(row![
                text(&file.name).width(iced::Length::Fill),
                text(file.size.to_string()),
                text(&file.kind),
            ]);
        }
        let file_panel = container(scrollable(file_col)).width(iced::Length::Fill);
        let content = row![folder_panel, file_panel,].height(iced::Length::Fill);
        // statusbar ui
        let statusbar = row![text(format!("{} items", self.files.len()))];
        // final ui
        column!(toolbar, content, statusbar).into()
    }
}
