use iced::{
    Element,
    widget::{button, column, container, row, text},
};

use crate::Message;

pub struct Node {
    pub name: String,
    pub children: Vec<Node>,
    pub expanded: bool,
    pub value: Option<f64>,
}

impl Node {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: vec![],
            expanded: true,
            value: None,
        }
    }
    pub fn get_or_insert(&mut self, name: &str) -> &mut Self {
        if let Some(pos) = self.children.iter().position(|n| n.name == name) {
            return &mut self.children[pos];
        }

        self.children.push(Node::new(name));
        self.children.last_mut().unwrap()
    }
    pub fn toggle(&mut self, row: &[usize]) {
        if row.is_empty() {
            self.expanded = !self.expanded;
            return;
        }

        if let Some(child) = self.children.get_mut(row[0]) {
            child.toggle(&row[1..]);
        }
    }
    pub fn get(&self, row: &[usize]) -> Option<&Self> {
        if row.is_empty() {
            return Some(self);
        }
        self.children.get(row[0])?.get(&row[1..])
    }
    pub fn total_value(&self) -> f64 {
        if self.children.is_empty() {
            return self.value.unwrap_or(0.0);
        }
        self.children.iter().map(Node::total_value).sum()
    }
    #[allow(clippy::only_used_in_recursion)]
    pub fn view(&self, path: Vec<usize>, depth: usize) -> Element<'_, Message> {
        let icon = if self.children.is_empty() {
            "•"
        } else if self.expanded {
            "▼"
        } else {
            "▶"
        };

        let row_label = button(
            row![text(icon).size(13).width(16), text(&self.name).size(14),]
                .spacing(4)
                .align_y(iced::Alignment::Center),
        )
        .on_press(Message::ToggleFolder(path.clone()))
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
                let mut child_path = path.clone();
                child_path.push(i);
                col = col.push(child.view(child_path, depth + 1));
            }
        }

        col.into()
    }
}
