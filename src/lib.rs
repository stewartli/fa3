use iced::{
    Element,
    Length::FillPortion,
    Size,
    widget::{button, column, container, row, rule, scrollable, text, text_input},
};

mod tree;

const SIZE_COL: f32 = 70.0;
const TYPE_COL: f32 = 70.0;
const CHILD_COL: f32 = 60.0;
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
    selected: Option<Vec<usize>>,
}

struct SubItem {
    name: String,
    amt: f64,
    kind: String,
    n_child: usize,
}

impl Account {
    #[allow(clippy::new_without_default)]
    pub fn new(path: &str) -> Result<Self, csv::Error> {
        let mut res = Self {
            path: "/home/stewart".into(),
            root: vec![],
            files: vec![
                SubItem {
                    name: "file1.txt".into(),
                    amt: 100.0,
                    kind: "file".into(),
                    n_child: 0,
                },
                SubItem {
                    name: "photo.jpg".into(),
                    amt: 2000.0,
                    kind: "image".into(),
                    n_child: 1,
                },
            ],
            selected: None,
        };

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_path(path)?;

        for x in reader.records() {
            let record = x?;
            let row: Vec<&str> = record
                .iter()
                .map(str::trim)
                .filter(|field| !field.is_empty())
                .collect();
            res.insert(&row);
        }

        Ok(res)
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

        let mut curr_node = &mut self.root[root_pos];
        for name in &row[1..] {
            if let Ok(value) = name.parse::<f64>() {
                curr_node.value = Some(value);
                break;
            }
            curr_node = curr_node.get_or_insert(name);
        }
    }
    pub fn toggle_folder(&mut self, row: &[usize]) {
        if row.is_empty() {
            return;
        }

        if let Some(root) = self.root.get_mut(row[0]) {
            root.toggle(&row[1..]);
        }

        let node = self.root.get(row[0]).and_then(|root| root.get(&row[1..]));

        if let Some(node) = node {
            let kind = if node.children.is_empty() {
                "child"
            } else {
                "parent"
            };

            self.files = vec![SubItem {
                name: node.name.clone(),
                amt: node.total_value(),
                kind: kind.into(),
                n_child: node.children.len(),
            }];
        }

        self.selected = Some(row.to_vec());
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
            text("Type").size(14).width(TYPE_COL),
            text("Child").size(14).width(CHILD_COL),
            text("Amount").size(14).width(SIZE_COL),
        ]
        .padding([4, 6]);

        let mut file_col = column![header, rule::horizontal(1)].spacing(2);
        for file in &self.files {
            file_col = file_col.push(
                row![
                    text(&file.name).size(14).width(iced::Length::Fill),
                    text(&file.kind).size(14).width(TYPE_COL),
                    text(file.n_child.to_string()).size(14).width(CHILD_COL),
                    text(format!("{:.2}", file.amt)).size(14).width(SIZE_COL),
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
