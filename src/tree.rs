use iced::{
    Element,
    widget::{button, column, container, row, text},
};

use crate::Message;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum ReconStatus {
    #[default]
    NotChecked,
    Passed,
    Failed,
}

impl std::str::FromStr for ReconStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "passed" => Ok(Self::Passed),
            "failed" => Ok(Self::Failed),
            "notchecked" => Ok(Self::NotChecked),
            _ => Err("invalid recon status".to_owned()),
        }
    }
}

pub struct Node {
    pub name: String,
    pub children: Vec<Node>,
    pub expanded: bool,
    pub value: Option<f64>,
    pub recon_status: ReconStatus,
}

impl Node {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: vec![],
            expanded: false,
            value: None,
            recon_status: ReconStatus::NotChecked,
        }
    }
    pub fn get_or_insert(&mut self, name: &str) -> &mut Self {
        if let Some(pos) = self.children.iter().position(|n| n.name == name) {
            return &mut self.children[pos];
        }
        self.children.push(Node::new(name));
        self.children.last_mut().unwrap()
    }
    pub fn get(&self, row: &[usize]) -> Option<&Self> {
        if row.is_empty() {
            return Some(self);
        }
        self.children.get(row[0])?.get(&row[1..])
    }
    pub fn get_mut(&mut self, row: &[usize]) -> Option<&mut Self> {
        if row.is_empty() {
            return Some(self);
        }
        self.children.get_mut(row[0])?.get_mut(&row[1..])
    }
    pub fn collapse_toggle(&mut self, row: &[usize]) {
        if row.is_empty() {
            self.expanded = !self.expanded;
            return;
        }
        if let Some(child) = self.children.get_mut(row[0]) {
            child.collapse_toggle(&row[1..]);
        }
    }
    pub fn collapse_all(&mut self) {
        self.expanded = false;
        for child in &mut self.children {
            child.collapse_all();
        }
    }
    pub fn total_value(&self) -> f64 {
        if self.children.is_empty() {
            return self.value.unwrap_or(0.0);
        }
        self.children.iter().map(Node::total_value).sum()
    }
    pub fn find_path(&self, query: &str) -> Option<Vec<usize>> {
        if self.name.to_lowercase().contains(query) {
            return Some(vec![]);
        }
        for (i, child) in self.children.iter().enumerate() {
            if let Some(mut sub_path) = child.find_path(query) {
                // maintain vec order
                sub_path.insert(0, i);
                return Some(sub_path);
            }
        }
        None
    }
    pub fn expand_path(&mut self, row: &[usize]) {
        self.expanded = true;
        if let Some((&first, rest)) = row.split_first()
            && let Some(child) = self.children.get_mut(first)
        {
            child.expand_path(rest);
        }
    }
    pub fn leaf_names(&self) -> Vec<String> {
        let mut out = vec![];
        if self.children.is_empty() {
            out.push(self.name.clone());
            return out;
        }
        for child in &self.children {
            child.leaf_names();
        }
        out
    }
    #[allow(clippy::only_used_in_recursion)]
    pub fn view(&self, row: Vec<usize>, depth: usize) -> Element<'_, Message> {
        let icon = if self.children.is_empty() {
            "•"
        } else if self.expanded {
            "▼"
        } else {
            "▶"
        };

        let row_label = button(
            row![text(icon).size(13).width(16), text(&self.name).size(14)]
                .spacing(4)
                .align_y(iced::Alignment::Center),
        )
        .on_press(Message::ToggleFolder(row.clone()))
        .padding([1, 6])
        .width(iced::Length::Fill)
        .style(button::text);

        let row_with_indent = row![
            container("").width(iced::Length::Fixed(depth as f32 * 16.0)),
            row_label,
        ];

        let mut col = column![row_with_indent].spacing(0);

        if self.expanded {
            for (i, child) in self.children.iter().enumerate() {
                let mut child_path = row.clone();
                child_path.push(i);
                col = col.push(child.view(child_path, depth + 1));
            }
        }

        col.into()
    }
}
