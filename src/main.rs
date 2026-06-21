use iced::{Element, Subscription, Task, Theme, window};

use fa3::{Account, Message};

struct App {
    account: Account,
}

impl App {
    fn new() -> Self {
        Self {
            account: Account::new(),
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
    fn subscription(&self) -> Subscription<Message> {
        window::resize_events().map(|(_id, size)| Message::A(size))
    }
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::A(_size) => todo!(),
            Message::Back => todo!(),
            Message::Forward => todo!(),
            Message::Up => todo!(),
            Message::Refresh => todo!(),
            Message::FolderSelected(_) => todo!(),
            Message::ToggleFolder(x) => {
                self.account.toggle_folder(&x);
                Task::none()
            }
            Message::FileSelected(_) => todo!(),
            Message::PathChanged(_) => todo!(),
        }
    }
    fn view(&self) -> Element<'_, Message> {
        self.account.view()
    }
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("CA Analytics")
        .theme(App::theme)
        .subscription(App::subscription)
        .window(App::window())
        .settings(iced::Settings::default())
        .run()
}
