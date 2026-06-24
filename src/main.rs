use iced::{
    Element, Length, Task, Theme, event,
    keyboard::Key,
    widget::{button, column, container, mouse_area, row, stack},
    window,
};

use fa3::{MenuKind, Message, account};

const MENU_BTN_WIDTH: f32 = 60.0;
const ROW_PADDING: f32 = 10.0;
const BTN_STRIDE: f32 = MENU_BTN_WIDTH + 5.0;

struct App {
    coa: account::Account,
    menu: Option<MenuKind>,
    theme: Theme,
}

impl App {
    fn new() -> Self {
        Self {
            coa: account::Account::new("asset/coa.csv").unwrap(),
            menu: None,
            theme: Theme::GruvboxLight,
        }
    }
    fn theme(&self) -> Theme {
        self.theme.clone()
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
            // menu
            Message::OpenMenu(menu) => {
                self.menu = match self.menu {
                    Some(m) if m == menu => None,
                    _ => Some(menu),
                };
            }
            Message::CloseMenu => {
                self.menu = None;
            }
            Message::New => {
                println!("new");
            }
            Message::Open => {
                println!("open");
            }
            Message::Save => {
                println!("save");
            }
            Message::Exit => {
                println!("exit");
            }
            Message::Rename => {
                println!("rename");
            }
            Message::Delete => {
                println!("delete");
            }
            Message::Expand => {
                self.menu = None;
            }
            Message::Collapse => {
                self.menu = None;
            }
            // toolbar
            Message::Back | Message::Forward => {
                let len = Theme::ALL.len();
                let idx = Theme::ALL
                    .iter()
                    .position(|t| &self.theme == t)
                    .unwrap_or(0);

                self.theme = if matches!(msg, Message::Back) {
                    Theme::ALL[(idx + len - 1) % len].clone()
                } else {
                    Theme::ALL[(idx + 1) % len].clone()
                };
            }
            Message::Up => {
                self.coa.reconcile_selected();
            }
            Message::Refresh => {
                self.coa.collapse_all();
            }
            Message::TogglePath(x) => {
                self.coa.toggle_folder(&x);
            }
            Message::SearchPath(x) => {
                self.coa.search(&x);
            }
            // key
            Message::A(x) => {
                println!("{:?}", x);
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
        let menubar = row![
            button("File")
                .on_press(Message::OpenMenu(MenuKind::File))
                .width(Length::Fixed(MENU_BTN_WIDTH)),
            button("Edit")
                .on_press(Message::OpenMenu(MenuKind::Edit))
                .width(Length::Fixed(MENU_BTN_WIDTH)),
            button("View")
                .on_press(Message::OpenMenu(MenuKind::View))
                .width(Length::Fixed(MENU_BTN_WIDTH)),
        ]
        .spacing(5)
        .padding([4, 10]);

        let base = column![menubar, self.coa.view()];

        let Some(menu_kind) = self.menu else {
            return base.into();
        };

        let items: Vec<Element<'_, Message>> = match menu_kind {
            MenuKind::File => vec![
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
            MenuKind::Edit => vec![
                button("Rename")
                    .on_press(Message::Rename)
                    .width(Length::Fill)
                    .into(),
                button("Delete")
                    .on_press(Message::Delete)
                    .width(Length::Fill)
                    .into(),
            ],
            MenuKind::View => vec![
                button("Expand All")
                    .on_press(Message::Expand)
                    .width(Length::Fill)
                    .into(),
                button("Collapse All")
                    .on_press(Message::Collapse)
                    .width(Length::Fill)
                    .into(),
            ],
        };

        let mut menu_col = column![].spacing(2).width(Length::Fixed(160.0));
        for item in items {
            menu_col = menu_col.push(item);
        }

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

        stack![base, dropdown, closeoff].into()
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
