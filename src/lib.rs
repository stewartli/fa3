use iced::{
    Element,
    Length::FillPortion,
    Size,
    widget::{button, column, container, row, rule, scrollable, text, text_input},
};

mod recon;
mod tree;

pub use tree::ReconStatus;

const TYPE_COL: f32 = 70.0;
const CHILD_COL: f32 = 60.0;
const SIZE_COL: f32 = 70.0;
const _FONT_SIZE: i32 = 14;

const GL_DB_PATH: &str = "asset/gl.db";
const UPLOAD_PATH: &str = "asset/check.csv";
const RECON_TOLERANCE: f64 = 0.01;

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
    recon_message: Option<String>,
}

struct SubItem {
    name: String,
    kind: String,
    n_child: usize,
    amt: f64,
    recon_status: ReconStatus,
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
            recon_message: None,
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

        res.load_recon_status();

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
            recon_status: node.recon_status,
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
    pub fn reconcile_selected(&mut self) {
        let Some(path) = self.selected.clone() else {
            self.recon_message = Some("Select an account first.".into());
            return;
        };

        let node = path
            .first()
            .and_then(|&i| self.root.get(i))
            .and_then(|root| root.get(&path[1..]));
        let Some(node) = node else {
            self.recon_message = Some("Selected account no longer exists.".into());
            return;
        };

        if node.recon_status == ReconStatus::Passed {
            self.recon_message = Some(format!("{}: already passed (skipped re-check).", node.name));
            return;
        }

        let coa_amount = node.total_value();
        let account_name = node.name.clone();

        let mut leaf_names = vec![];
        node.leaf_names(&mut leaf_names);

        match recon::reconcile(
            GL_DB_PATH,
            &account_name,
            &leaf_names,
            UPLOAD_PATH,
            coa_amount,
            false,
        ) {
            Ok(gl_amount) => {
                let passed = (gl_amount - coa_amount).abs() <= RECON_TOLERANCE;

                // Re-record with the correct pass/fail now that we know it.
                let _ = recon::reconcile(
                    GL_DB_PATH,
                    &account_name,
                    &leaf_names,
                    UPLOAD_PATH,
                    coa_amount,
                    passed,
                );

                let node = path
                    .first()
                    .and_then(|&i| self.root.get_mut(i))
                    .and_then(|root| root.get_mut(&path[1..]));
                if let Some(node) = node {
                    node.recon_status = if passed {
                        ReconStatus::Passed
                    } else {
                        ReconStatus::Failed
                    };
                }

                self.recon_message = Some(if passed {
                    format!("{account_name}: passed (COA {coa_amount:.2} vs GL {gl_amount:.2})")
                } else {
                    format!("{account_name}: FAILED (COA {coa_amount:.2} vs GL {gl_amount:.2})")
                });

                self.select_path(path);
            }
            Err(e) => {
                self.recon_message = Some(format!("Reconcile error: {e}"));
            }
        }
    }
    fn load_recon_status(&mut self) {
        let rows = match recon::load_all_status(GL_DB_PATH) {
            Ok(rows) => rows,
            Err(_) => return,
        };

        for (name, status) in rows {
            let status = match status.as_str() {
                "passed" => ReconStatus::Passed,
                "failed" => ReconStatus::Failed,
                _ => continue,
            };
            for root in &mut self.root {
                if Self::apply_status_by_name(root, &name, status) {
                    break;
                }
            }
        }
    }
    fn apply_status_by_name(node: &mut tree::Node, name: &str, status: ReconStatus) -> bool {
        if node.name == name {
            node.recon_status = status;
            return true;
        }
        for child in &mut node.children {
            if Self::apply_status_by_name(child, name, status) {
                return true;
            }
        }
        false
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
            let amount_color = match file.recon_status {
                ReconStatus::NotChecked => None,
                ReconStatus::Passed => Some(iced::Color::from_rgb(0.0, 0.55, 0.0)),
                ReconStatus::Failed => Some(iced::Color::from_rgb(0.75, 0.0, 0.0)),
            };
            let mut amount_text = text(format!("{:.2}", file.amt)).size(14).width(SIZE_COL);
            if let Some(c) = amount_color {
                amount_text = amount_text.color(c);
            }

            file_col = file_col.push(
                row![
                    text(&file.name).size(14).width(iced::Length::Fill),
                    text(&file.kind).size(14).width(TYPE_COL),
                    text(file.n_child.to_string()).size(14).width(CHILD_COL),
                    amount_text,
                ]
                .padding([4, 6]),
            );
        }

        let file_panel = container(scrollable(file_col).width(iced::Length::Fill))
            .width(FillPortion(2))
            .padding(8);

        let content = row![folder_panel, rule::vertical(1), file_panel].height(iced::Length::Fill);

        // statusbar ui
        let status_text = self
            .recon_message
            .clone()
            .unwrap_or_else(|| format!("{} items / {} accounts", self.files.len(), self.num));
        let statusbar = row![text(status_text).size(13)].padding([6, 10]);

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
