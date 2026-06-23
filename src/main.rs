use iced::{
    Element, Length, Task, Theme, event,
    keyboard::Key,
    widget::{button, column, container, mouse_area, row, stack},
    window,
};

use fa3::{Account, Message};

struct App {
    account: Account,
    menu: Option<fa3::MenuKind>,
}

impl App {
    fn new() -> Self {
        Self {
            account: Account::new("asset/coa.csv").unwrap(),
            menu: None,
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
    //     event::listen().map(Message::EventOccurred)
    // }
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
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
                self.account.collapse_all();
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
                self.account.search(&x);
            }
            Message::ToggleMenu(menu) => {
                self.menu = match self.menu {
                    Some(m) if m == menu => None,
                    _ => Some(menu),
                };
            }
            Message::CloseMenu => {
                self.menu = None;
            }
            Message::New => {
                self.menu = None;
                println!("new");
            }
            Message::Open => {
                self.menu = None;
                println!("open");
            }
            Message::Save => {
                self.menu = None;
                println!("save");
            }
            Message::Exit => {
                println!("exit");
            }
            Message::ExpandAll => {
                self.menu = None;
            }
            Message::CollapseAll => {
                self.menu = None;
            }
            Message::A(size) => {
                println!("{:?}", size);
            }
            Message::EventOccurred(x) => {
                if let event::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = x
                    && let Key::Character(res) = key
                {
                    println!("key {res}");
                }
            }
        }
        Task::none()
    }
    fn view(&self) -> Element<'_, Message> {
        const MENU_BTN_WIDTH: f32 = 60.0;
        let menubar = row![
            button("File")
                .on_press(Message::ToggleMenu(fa3::MenuKind::File))
                .width(Length::Fixed(MENU_BTN_WIDTH)),
            button("Edit")
                .on_press(Message::ToggleMenu(fa3::MenuKind::Edit))
                .width(Length::Fixed(MENU_BTN_WIDTH)),
            button("View")
                .on_press(Message::ToggleMenu(fa3::MenuKind::View))
                .width(Length::Fixed(MENU_BTN_WIDTH)),
        ]
        .spacing(5)
        .padding([4, 10]);

        let base = column![menubar, self.account.view()];

        let Some(menu_kind) = self.menu else {
            return base.into();
        };

        let items: Vec<Element<'_, Message>> = match menu_kind {
            fa3::MenuKind::File => vec![
                button("New")
                    .on_press(Message::New)
                    .width(Length::Fill)
                    .into(),
                button("Open")
                    .on_press(Message::Open)
                    .width(Length::Fill)
                    .into(),
                button("Save")
                    .on_press(Message::Save)
                    .width(Length::Fill)
                    .into(),
                button("Exit")
                    .on_press(Message::Exit)
                    .width(Length::Fill)
                    .into(),
            ],
            fa3::MenuKind::Edit => vec![
                button("Rename").width(Length::Fill).into(),
                button("Delete").width(Length::Fill).into(),
            ],
            fa3::MenuKind::View => vec![
                button("Expand All")
                    .on_press(Message::ExpandAll)
                    .width(Length::Fill)
                    .into(),
                button("Collapse All")
                    .on_press(Message::CollapseAll)
                    .width(Length::Fill)
                    .into(),
            ],
        };

        let mut menu_col = column![].spacing(2).width(Length::Fixed(160.0));
        for item in items {
            menu_col = menu_col.push(item);
        }

        const ROW_PADDING: f32 = 10.0;
        const BTN_STRIDE: f32 = MENU_BTN_WIDTH + 5.0;

        let left_offset = match menu_kind {
            fa3::MenuKind::File => ROW_PADDING,
            fa3::MenuKind::Edit => ROW_PADDING + BTN_STRIDE,
            fa3::MenuKind::View => ROW_PADDING + BTN_STRIDE * 2.0,
        };

        let dropdown = container(column![
            container("").height(Length::Fixed(36.0)),
            row![
                container("").width(Length::Fixed(left_offset)),
                container(menu_col).padding(4).style(container::rounded_box),
            ]
        ]);

        let closeoff = mouse_area(container("").width(Length::Fill).height(Length::Fill))
            .on_press(Message::CloseMenu);

        stack![base, closeoff, dropdown].into()
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
