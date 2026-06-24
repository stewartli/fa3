use iced::{
    Element,
    Length::FillPortion,
    widget::{button, column, container, row, rule, scrollable, text, text_input},
};

use crate::{Message, recon, tree};

const TYPE_COL: f32 = 70.0;
const CHILD_COL: f32 = 60.0;
const SIZE_COL: f32 = 70.0;

const DB_PATH: &str = "asset/gl.db";
const UPLOAD_PATH: &str = "check/car_truck_expense.csv";
const RECON_TOLERANCE: f64 = 0.01;

pub struct Account {
    pub path: String,
    root: Vec<tree::Node>,
    items: Vec<SubItem>,
    selected: Option<Vec<usize>>,
    n_acc: usize,
    recon_message: Option<String>,
}

struct SubItem {
    name: String,
    kind: String,
    n_child: usize,
    amt: f64,
    recon_status: tree::ReconStatus,
}

impl Account {
    pub fn new(src: &str) -> Result<Self, csv::Error> {
        let mut res = Self {
            path: String::new(),
            root: vec![],
            items: vec![],
            selected: None,
            n_acc: 0,
            recon_message: None,
        };

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_path(src)?;

        for x in reader.records() {
            let record = x?;
            let row: Vec<&str> = record
                .iter()
                .map(str::trim)
                .filter(|field| !field.is_empty())
                .collect();
            res.insert(&row);
            res.n_acc += 1;
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
            // coa.csv the last column is Node's value
            if let Ok(value) = name.parse::<f64>() {
                curr_node.value = Some(value);
                break;
            }
            curr_node = curr_node.get_or_insert(name);
        }
    }
    pub fn collapse_all(&mut self) {
        for root in &mut self.root {
            root.collapse_all();
        }
        self.selected = None;
        self.items = vec![];
    }
    pub fn toggle_folder(&mut self, row: &[usize]) {
        if row.is_empty() {
            return;
        }

        if let Some(root) = self.root.get_mut(row[0]) {
            root.collapse_toggle(&row[1..]);
        }

        self.select_path(row.to_vec());
    }
    fn select_path(&mut self, row: Vec<usize>) {
        let node = row
            .first()
            .and_then(|&i| self.root.get(i))
            .and_then(|root| root.get(&row[1..]));

        let Some(node) = node else {
            self.selected = Some(row);
            return;
        };

        // total value for parent and account
        let mut items = vec![Self::item_for(node)];
        for child in &node.children {
            items.push(Self::item_for(child));
        }
        self.items = items;
        self.selected = Some(row);
    }
    fn item_for(node: &tree::Node) -> SubItem {
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
    pub fn reconcile_selected(&mut self) {
        let Some(row) = self.selected.clone() else {
            self.recon_message = Some("Select an account first".into());
            return;
        };

        let node = row
            .first()
            .and_then(|&i| self.root.get(i))
            .and_then(|root| root.get(&row[1..]));

        let Some(node) = node else {
            self.recon_message = Some("Selected account does not exist".into());
            return;
        };

        if node.recon_status == tree::ReconStatus::Passed {
            self.recon_message = Some(format!("{}: passed (skipped re-check)", node.name));
            return;
        }

        let account_name = node.name.clone();
        let coa_amount = node.total_value();
        let leaf_names = node.leaf_names();

        match recon::reconcile(DB_PATH, UPLOAD_PATH, &account_name, &leaf_names) {
            Ok(gl_amount) => {
                let recon_check = (gl_amount - coa_amount).abs() <= RECON_TOLERANCE;

                if let Err(e) =
                    recon::record_status(DB_PATH, &account_name, coa_amount, gl_amount, recon_check)
                {
                    self.recon_message = Some(format!("Reconcile error (status write): {e}"));
                    return;
                }

                let node = row
                    .first()
                    .and_then(|&i| self.root.get_mut(i))
                    .and_then(|root| root.get_mut(&row[1..]));

                if let Some(node) = node {
                    node.recon_status = if recon_check {
                        tree::ReconStatus::Passed
                    } else {
                        tree::ReconStatus::Failed
                    };
                }

                self.recon_message = Some(if recon_check {
                    format!("{account_name}: passed (COA {coa_amount:.2} vs GL {gl_amount:.2})")
                } else {
                    format!("{account_name}: FAILED (COA {coa_amount:.2} vs GL {gl_amount:.2})")
                });

                self.select_path(row);
            }
            Err(e) => {
                self.recon_message = Some(format!("Reconcile error: {e}"));
            }
        }
    }
    fn load_recon_status(&mut self) {
        let rows = match recon::load_all_status(DB_PATH) {
            Ok(rows) => rows,
            Err(_) => return,
        };

        for (name, stat) in rows {
            let status = match stat.parse::<tree::ReconStatus>() {
                Ok(s) => s,
                _ => continue,
            };
            for root in &mut self.root {
                if Self::apply_status_by_name(root, &name, status) {
                    break;
                }
            }
        }
    }
    fn apply_status_by_name(node: &mut tree::Node, name: &str, status: tree::ReconStatus) -> bool {
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
                .on_input(Message::SearchPath)
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
        for file in &self.items {
            let amount_color = match file.recon_status {
                tree::ReconStatus::NotChecked => None,
                tree::ReconStatus::Passed => Some(iced::Color::from_rgb(0.0, 0.55, 0.0)),
                tree::ReconStatus::Failed => Some(iced::Color::from_rgb(0.75, 0.0, 0.0)),
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
            .unwrap_or_else(|| format!("{} accounts / {} items", self.n_acc, self.items.len()));

        let statusbar = row![text(status_text).size(13)].padding([6, 10]);

        // final account ui
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
