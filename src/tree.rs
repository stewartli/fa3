use crate::Message;
use iced::{
    Element,
    widget::{button, column, container, text},
};

pub struct Node {
    pub name: String,
    pub children: Vec<Node>,
    pub expanded: bool,
}

impl Node {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: vec![],
            expanded: true,
        }
    }
    #[allow(unused)]
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
    #[allow(clippy::only_used_in_recursion)]
    pub fn view(&self, path: Vec<usize>, depth: usize) -> Element<'_, Message> {
        let icon = if self.children.is_empty() {
            "•"
        } else if self.expanded {
            "▼"
        } else {
            "▶"
        };

        let mut col = column![
            button(text(format!("{icon} {}", self.name)))
                .on_press(Message::ToggleFolder(path.clone()))
        ];

        if self.expanded {
            for (i, child) in self.children.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(i);
                col = col.push(container(child.view(child_path, depth + 1)).padding(10));
            }
        }

        col.into()
    }
}
