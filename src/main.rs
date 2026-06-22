use iced::{
    Element, Subscription, Task, Theme,
    widget::{button, column, container, row},
    window,
};

use fa3::{Account, Message};

struct App {
    account: Account,
    curr_menu: Option<fa3::MenuKind>,
}

impl App {
    fn new() -> Self {
        Self {
            account: Account::new(),
            curr_menu: None,
        }
    }
    fn theme(&self) -> Theme {
        Theme::GruvboxLight
    }
    fn window() -> window::Settings {
        window::Settings {
            size: iced::Size::new(600.0, 500.0),
            resizable: true,
            decorations: true,
            ..Default::default()
        }
    }
    // fn subscription(&self) -> Subscription<Message> {
    //     window::resize_events().map(|(_id, size)| Message::A(size))
    // }
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::A(size) => {
                println!("{:?}", size);
            }
            Message::Back => {
                println!("back icon");
            }
            Message::Forward => {
                println!("forward icon");
            }
            Message::Up => {
                println!("up icon");
            }
            Message::Refresh => {
                println!("refresh icon");
            }
            Message::FolderSelected(x) => {
                println!("folder select {x}");
            }
            Message::ToggleFolder(x) => {
                self.account.toggle_folder(&x);
            }
            Message::FileSelected(x) => {
                println!("file select {x}");
            }
            Message::PathChanged(x) => {
                println!("path select {x}");
            }
            Message::ToggleMenu(menu) => {
                self.curr_menu = match self.curr_menu {
                    Some(m) if m == menu => None,
                    _ => Some(menu),
                };
            }
            Message::New => {
                self.curr_menu = None;
                println!("new");
            }
            Message::Open => {
                self.curr_menu = None;
                println!("open");
            }
            Message::Save => {
                self.curr_menu = None;
                println!("save");
            }
            Message::Exit => {
                println!("exit");
            }
            Message::ExpandAll => {
                self.curr_menu = None;
            }
            Message::CollapseAll => {
                self.curr_menu = None;
            }
        }
        Task::none()
    }
    fn view(&self) -> Element<'_, Message> {
        let menubar = row![
            button("File").on_press(Message::ToggleMenu(fa3::MenuKind::File)),
            button("Edit").on_press(Message::ToggleMenu(fa3::MenuKind::Edit)),
            button("View").on_press(Message::ToggleMenu(fa3::MenuKind::View)),
        ]
        .spacing(5);

        let menu = match self.curr_menu {
            Some(fa3::MenuKind::File) => column![
                button("New").on_press(Message::New),
                button("Open").on_press(Message::Open),
                button("Save").on_press(Message::Save),
                button("Exit").on_press(Message::Exit),
            ]
            .spacing(2),
            Some(fa3::MenuKind::Edit) => column![button("Rename"), button("Delete"),].spacing(2),
            Some(fa3::MenuKind::View) => column![
                button("Expand All").on_press(Message::ExpandAll),
                button("Collapse All").on_press(Message::CollapseAll),
            ]
            .spacing(2),
            None => column![],
        };

        column![
            menubar,
            container(menu).padding(5),
            self.account.view(),
            container("Account Tree")
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
        ]
        .into()
    }
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("CA Analytics")
        .theme(App::theme)
        // .subscription(App::subscription)
        .window(App::window())
        .settings(iced::Settings::default())
        .run()
}
