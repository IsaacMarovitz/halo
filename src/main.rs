mod editor;
mod preferences;
mod theme;
mod viewer;
mod widget;

use crate::editor::{Editor, Event};
use crate::preferences::Preferences;
use crate::theme::Theme;
use crate::viewer::Viewer;
use iced::font::{Family, Stretch, Style, Weight};
use iced::widget::pane_grid::Configuration;
use iced::widget::{container, pane_grid};
use iced::{keyboard, Element, Font, Length, Size, Subscription, Task};
use std::sync::Arc;

pub type FragmentShader = String;

const HALO: &str = "Halo";

const JETBRAINS_MONO: Font = Font {
    family: Family::Name("JetBrains Mono"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

fn main() -> iced::Result {
    iced::application(Halo::title, Halo::update, Halo::view)
        .theme(Halo::theme)
        .subscription(Halo::subscription)
        .font(include_bytes!("../fonts/JetBrainsMono-Regular.ttf").as_slice())
        .font(include_bytes!("../fonts/halo-icons.ttf").as_slice())
        .default_font(Font::MONOSPACE)
        .window_size(Size::new(1600.0, 900.0))
        .run_with(|| Halo::new(()))
}

struct Halo {
    viewer: Viewer,
    editor: Editor,
    panes: pane_grid::State<Pane>,
}

//TODO toggle editor
#[derive(Clone, Debug)]
enum Message {
    PaneResized(pane_grid::ResizeEvent),
    Editor(editor::Message),
    KeyPressed {
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
    },
    Loaded(Result<(Preferences, Arc<FragmentShader>), preferences::Error>),
}

impl Halo {
    fn new(_flags: ()) -> (Self, Task<Message>) {
        (
            //TODO save settings
            Self {
                viewer: Viewer::default(),
                editor: Editor::default(),
                panes: pane_grid::State::with_configuration(Configuration::Split {
                    axis: pane_grid::Axis::Vertical,
                    ratio: 0.5,
                    a: Box::new(Configuration::Pane(Pane::Viewer)),
                    b: Box::new(Configuration::Pane(Pane::Editor)),
                }),
            },
            //TODO load last shader file from settings
            Task::perform(preferences::load(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        HALO.to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Editor(msg) => {
                let (event, cmd) = self.editor.update(msg);

                match event {
                    Event::UpdatePipeline(shader) => {
                        self.viewer.last_valid_shader = shader;
                        self.viewer.version += 1;
                    }
                    _ => {}
                };

                return cmd.map(Message::Editor);
            }
            Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
            }
            Message::KeyPressed { key, modifiers } => {
                if let Some(msg) = self.editor.keypress(key, modifiers).map(Message::Editor) {
                    return self.update(msg);
                }
            }
            Message::Loaded(result) => {
                return self.update(Message::Editor(editor::Message::Init(result)));
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message, Theme> {
        let panes = pane_grid(&self.panes, |_id, pane, _is_maximized| {
            pane.view(&self.editor, &self.viewer).into()
        })
        .on_resize(10, Message::PaneResized);

        container(panes)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| Some(Message::KeyPressed { key, modifiers }))
    }
}

enum Pane {
    Viewer,
    Editor,
}

impl Pane {
    fn view<'a>(
        &'a self,
        editor: &'a Editor,
        viewer: &'a Viewer,
    ) -> pane_grid::Content<'a, Message, Theme> {
        match self {
            Self::Viewer => viewer.content(),
            Self::Editor => pane_grid::Content::new(editor.view().map(Message::Editor))
                .title_bar(pane_grid::TitleBar::new(
                    editor.title_bar().map(Message::Editor),
                )),
        }
    }
}
