use iced::{
    Element,
    Length::FillPortion,
    Size,
    widget::{button, column, container, row, rule, scrollable, text, text_input},
};

mod tree;

const TYPE_COL: f32 = 70.0;
const CHILD_COL: f32 = 60.0;
const SIZE_COL: f32 = 70.0;
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
    num: usize,
}

struct SubItem {
    name: String,
    kind: String,
    n_child: usize,
    amt: f64,
}

impl Account {
    #[allow(clippy::new_without_default)]
    pub fn new(path: &str) -> Result<Self, csv::Error> {
        let mut res = Self {
            path: String::new(),
            root: vec![],
            files: vec![],
            selected: None,
            num: 0,
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
            res.num += 1;
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

        self.select_path(row.to_vec());
    }
    fn select_path(&mut self, path: Vec<usize>) {
        let node = path
            .first()
            .and_then(|&i| self.root.get(i))
            .and_then(|root| root.get(&path[1..]));

        let Some(node) = node else {
            self.selected = Some(path);
            return;
        };

        let mut rows = vec![Self::row_for(node)];
        for child in &node.children {
            rows.push(Self::row_for(child));
        }
        self.files = rows;
        self.selected = Some(path);
    }
    fn row_for(node: &tree::Node) -> SubItem {
        let kind = if node.children.is_empty() {
            "Account"
        } else {
            "Parent"
        };
        SubItem {
            name: node.name.clone(),
            kind: kind.into(),
            n_child: node.children.len(),
            amt: node.total_value(),
        }
    }
    pub fn search(&mut self, query: &str) {
        self.path = query.to_string();
        let query = query.to_lowercase();
        if query.is_empty() {
            return;
        }

        for (i, root) in self.root.iter().enumerate() {
            if let Some(mut sub_path) = root.find_path(&query) {
                sub_path.insert(0, i);
                if let Some(root) = self.root.get_mut(i) {
                    root.expand_path(&sub_path[1..]);
                }
                self.select_path(sub_path);
                return;
            }
        }
    }
    pub fn collapse_all(&mut self) {
        for root in &mut self.root {
            root.collapse_all();
        }
        self.selected = None;
        self.files = vec![];
    }
    pub fn view(&self) -> Element<'_, Message> {
        // toolbar ui
        let toolbar = row![
            button("←").on_press(Message::Back).padding([4, 10]),
            button("→").on_press(Message::Forward).padding([4, 10]),
            button("↑").on_press(Message::Up).padding([4, 10]),
            button("⟳").on_press(Message::Refresh).padding([4, 10]),
            text_input("query account", &self.path)
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
        let statusbar = row![
            text(format!(
                "{} items / {} accounts",
                self.files.len(),
                self.num,
            ))
            .size(13)
        ]
        .padding([6, 10]);

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
