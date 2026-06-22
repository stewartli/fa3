use iced::{
    Element,
    Length::FillPortion,
    Size,
    widget::{button, column, container, row, rule, scrollable, text, text_input},
};

mod tree;

const SIZE_COL: f32 = 70.0;
const TYPE_COL: f32 = 70.0;
const _FONT_SIZE: i32 = 14;

#[derive(Clone)]
pub enum Message {
    // toolbar
    Back,
    Forward,
    Up,
    Refresh,
    FolderSelected(usize),
    ToggleFolder(Vec<usize>),
    FileSelected(usize),
    PathChanged(String),
    // menu
    ToggleMenu(MenuKind),
    CloseMenu,
    New,
    Open,
    Save,
    Exit,
    ExpandAll,
    CollapseAll,
    // key
    A(Size),
    EventOccurred(iced::Event),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    File,
    Edit,
    View,
}

pub struct Account {
    pub path: String,
    root: Vec<tree::Node>,
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
        let mut res = Self {
            path: "/home/stewart".into(),
            root: vec![],
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
        };

        let coa = vec![
            vec!["Assets", "Fixed Assets", "Building"],
            vec!["Assets", "Fixed Assets", "Equipment"],
            vec!["Assets", "Current Assets", "Cash"],
            vec!["Assets", "Current Assets", "Accounts Receivable"],
            vec!["Liabilities", "Current Liabilities", "Accounts Payable"],
        ];

        for row in &coa {
            res.insert(row);
        }

        res
    }
    fn insert(&mut self, row: &[&str]) {
        if row.is_empty() {
            return;
        }

        let root_pos = if let Some(pos) = self.root.iter().position(|n| n.name == row[0]) {
            pos
        } else {
            self.root.push(tree::Node::new(row[0]));
            self.root.len() - 1
        };

        let mut curr_root_node = &mut self.root[root_pos];
        for name in &row[1..] {
            curr_root_node = curr_root_node.get_or_insert(name);
        }
    }
    pub fn toggle_folder(&mut self, row: &[usize]) {
        if row.is_empty() {
            return;
        }

        if let Some(root) = self.root.get_mut(row[0]) {
            root.toggle(&row[1..]);
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        // toolbar ui
        let toolbar = row![
            button("←").on_press(Message::Back).padding([4, 10]),
            button("→").on_press(Message::Forward).padding([4, 10]),
            button("↑").on_press(Message::Up).padding([4, 10]),
            button("⟳").on_press(Message::Refresh).padding([4, 10]),
            text_input("path", &self.path)
                .on_input(Message::PathChanged)
                .padding(6)
                .width(iced::Length::Fill),
        ]
        .spacing(6)
        .padding([8, 10])
        .align_y(iced::Alignment::Center);

        // content ui
        let mut folder_col = column![].spacing(2);
        for (i, node) in self.root.iter().enumerate() {
            folder_col = folder_col.push(node.view(vec![i], 0));
        }

        let folder_panel = container(scrollable(folder_col).width(iced::Length::Fill))
            .width(FillPortion(1))
            .padding(8);

        let header = row![
            text("Name").size(14).width(iced::Length::Fill),
            text("Size").size(14).width(SIZE_COL),
            text("Type").size(14).width(TYPE_COL),
        ]
        .padding([4, 6]);

        let mut file_col = column![header, rule::horizontal(1)].spacing(2);
        for file in &self.files {
            file_col = file_col.push(
                row![
                    text(&file.name).size(14).width(iced::Length::Fill),
                    text(file.size.to_string()).size(14).width(SIZE_COL),
                    text(&file.kind).size(14).width(TYPE_COL),
                ]
                .padding([4, 6]),
            );
        }

        let file_panel = container(scrollable(file_col).width(iced::Length::Fill))
            .width(FillPortion(2))
            .padding(8);

        let content = row![folder_panel, rule::vertical(1), file_panel].height(iced::Length::Fill);

        // statusbar ui
        let statusbar = row![text(format!("{} items", self.files.len())).size(13)].padding([6, 10]);

        // final ui
        column![
            toolbar,
            rule::horizontal(1),
            content,
            rule::horizontal(1),
            statusbar,
        ]
        .into()
    }
}
