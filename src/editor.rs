mod file;
mod highlighter;
mod validation;

use crate::preferences::Preferences;
use crate::{preferences, FragmentShader, JETBRAINS_MONO};
use iced::alignment::Horizontal;
use iced::widget::text_editor::Action;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_editor, tooltip};
use iced::{keyboard, Alignment, Font, Length, Task};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use iced::keyboard::key::Named;
use crate::editor::highlighter::Highlighter;
use crate::theme::{ContainerClass, TextClass, Theme};

type Element<'a, Message> = iced::Element<'a, Message, Theme>;

#[derive(Clone, Debug)]
pub enum Message {
    Init(Result<(Preferences, Arc<FragmentShader>), preferences::Error>),
    Action(Action),
    Validate,
    Validated(Result<Arc<FragmentShader>, validation::Error>),
    AutoValidate(bool),
    New,
    Open,
    Opened(Result<(PathBuf, Arc<FragmentShader>), file::Error>),
    Save,
    Saved(Result<PathBuf, file::Error>),
    Undo,
    Redo,
    Search,
    Indent,
    PreferencesSaved(Result<(), preferences::Error>),
}

pub enum Event {
    None,
    UpdatePipeline(Arc<FragmentShader>),
}

pub struct Editor {
    content: text_editor::Content,
    theme: iced::highlighter::Theme,
    shader_path: Option<PathBuf>,
    validation_status: validation::Status,
    auto_validate: bool,
    is_loading: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            content: text_editor::Content::with_text(include_str!(
                "viewer/shaders/default_frag.wgsl"
            )),
            theme: iced::highlighter::Theme::Base16Mocha,
            shader_path: None,
            validation_status: validation::Status::default(),
            auto_validate: true,
            is_loading: true,
        }
    }
}

impl Editor {
    pub fn keypress(
        &self,
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
    ) -> Option<Message> {
        match key.as_ref() {
            keyboard::Key::Named(Named::Enter) if modifiers.control() => Some(Message::Validate),
            keyboard::Key::Character("s") if modifiers.command() => Some(Message::Save),
            keyboard::Key::Character("z") if modifiers.command() => Some(Message::Undo),
            keyboard::Key::Character("y") if modifiers.command() => Some(Message::Redo),
            keyboard::Key::Character("f") if modifiers.command() => Some(Message::Search),
            keyboard::Key::Named(Named::Tab) => Some(Message::Indent),
            _ => None,
        }
    }

    pub fn update(&mut self, update: Message) -> (Event, Task<Message>) {
        match update {
            Message::Init(result) => {
                let cmd = match result {
                    Ok((prefs, shader)) => {
                        self.auto_validate = prefs.auto_validate;
                        self.shader_path = prefs.last_shader_path;
                        self.content = text_editor::Content::with_text(&shader);
                        Task::perform(validation::validate(shader), Message::Validated)
                    }
                    Err(e) => {
                        println!("Error loading prefs: {e:?}");
                        Task::none()
                    }
                };

                self.is_loading = false;
                return (Event::None, cmd);
            }
            Message::Action(action) => {
                //TODO fix not being able to use hotkeys while text editor is focused
                let should_validate = action.is_edit() && self.auto_validate;
                self.content.perform(action);

                if should_validate {
                    return self.update(Message::Validate);
                }
            }
            Message::New => {
                let empty_shader = include_str!("viewer/shaders/empty_frag.wgsl");

                self.shader_path = None;
                self.content = text_editor::Content::with_text(empty_shader);

                return (
                    Event::UpdatePipeline(Arc::new(empty_shader.to_string())),
                    Task::none(),
                );
            }
            Message::Open => {
                let cmd = if self.is_loading {
                    Task::none()
                } else {
                    self.is_loading = true;
                    Task::perform(file::open(), Message::Opened)
                };

                return (Event::None, cmd);
            }
            Message::Opened(result) => {
                let cmds = if let Ok((path, shader)) = result {
                    self.shader_path = Some(path);
                    self.content = text_editor::Content::with_text(&shader);

                    Task::batch(vec![
                        self.save_prefs(),
                        Task::perform(validation::validate(shader), Message::Validated),
                    ])
                } else {
                    Task::none()
                };

                //TODO loading error msg
                self.is_loading = false;

                return (Event::None, cmds);
            }
            Message::Save => {
                return if self.is_loading {
                    (Event::None, Task::none())
                } else {
                    let shader = self.content.text();

                    (
                        Event::None,
                        Task::perform(
                            file::save(self.shader_path.clone(), shader),
                            Message::Saved,
                        ),
                    )
                }
            }
            Message::Saved(result) => {
                if let Ok(path) = result {
                    self.shader_path = Some(path);
                }
                //TODO handle error
                return (Event::None, self.save_prefs());
            }
            Message::Validate => {
                self.validation_status = validation::Status::Validating;
                let shader = Arc::new(self.content.text());

                return (
                    Event::None,
                    Task::perform(validation::validate(shader), Message::Validated),
                );
            }
            Message::Validated(result) => match result {
                Ok(shader) => {
                    self.validation_status = validation::Status::Validated;
                    return (Event::UpdatePipeline(shader), Task::none());
                }
                Err(error) => {
                    println!("Invalid: {error:?}");
                    self.validation_status = validation::Status::Invalid(error);
                }
            },
            Message::AutoValidate(checked) => {
                self.auto_validate = checked;
                return (Event::None, self.save_prefs());
            }
            Message::Undo => {
                //TODO!
            }
            Message::Redo => {
                //TODO!
            }
            Message::Indent => {
                //TODO!
            }
            Message::Search => {
                //TODO!
            }
            Message::PreferencesSaved(_) => {
                println!("Prefs saved");
            }
        }

        (Event::None, Task::none())
    }

    fn save_prefs(&self) -> Task<Message> {
        let prefs = Preferences {
            last_shader_path: self.shader_path.clone(),
            auto_validate: self.auto_validate,
        };

        Task::perform(preferences::save(prefs), Message::PreferencesSaved)
    }

    pub fn view(&'_ self) -> Element<'_, Message> {
        let errors =
            if let validation::Status::Invalid(validation::Error::Parse { message: _, errors }) =
                &self.validation_status
            {
                errors
                    .iter()
                    .map(|(range, _msg)| range)
                    .cloned()
                    .collect::<Vec<_>>()
            } else {
                vec![]
            };

        let text_editor = text_editor(&self.content)
            .font(JETBRAINS_MONO)
            .padding(10)
            .highlight_with::<Highlighter>(
                highlighter::Settings {
                    theme: self.theme,
                    errors,
                },
                |highlight, _theme| highlight.to_format(),  // Note: takes theme parameter
            )
            .on_action(Message::Action);

        let path = container(text(
            self.shader_path
                .as_ref()
                .map_or("".to_string(), |p| p.to_string_lossy().to_string()),
        ))
        .align_x(Horizontal::Left)
        .width(Length::Fill);

        let char_count = container(
            //TODO expose a len() function from iced editor to avoid extra allocation
            text(self.content.text().len()),
        )
        .align_x(Horizontal::Right);

        let info = row![path, char_count]
            .width(Length::Fill)
            .padding([5, 10]);

        let content =
            if let validation::Status::Invalid(validation::Error::Parse { message, errors }) =
                &self.validation_status
            {
                column![
                    text_editor,
                    tmp_error_view(message, &errors, &self.content.text()),
                    info,
                ]
            } else {
                column![text_editor, info]
            };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn title_bar(&'_ self) -> Element<'_, Message> {
        let new_icon = icon('\u{e804}');
        let open_icon = icon('\u{f115}');
        let save_icon = icon('\u{e800}');

        let validation_controls = container(
            row![
                container(self.validation_status.icon())
                    .width(24)
                    .center_y(24),
                checkbox("Auto", self.auto_validate)
                    .on_toggle(Message::AutoValidate),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Left);

        let file_controls = container(
            row![
                control_button(new_icon, "Create a new shader", Message::New),
                control_button(open_icon, "Open a shader file", Message::Open),
                control_button(save_icon, "Save current shader", Message::Save),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .align_x(Horizontal::Right);

        container(
            row![validation_controls, file_controls]
                .width(Length::Fill)
                .padding([10, 15])
                .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .class(ContainerClass::Controls)
        .into()
    }
}

fn control_button<'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Message,
) -> Element<'a, Message> {
    let button = button(container(content).center_x(30));

    tooltip(button.on_press(on_press), label, tooltip::Position::Bottom)
        .padding(10)
        .class(ContainerClass::Tooltip)
        .into()
}

//TODO colored icons once I have an actual theme
pub fn icon<'a, Message: 'static>(char: char) -> Element<'a, Message> {
    const FONT: Font = Font::with_name("halo-icons");

    text(char)
        .font(FONT)
        .into()
}

fn tmp_error_view<'a>(msg: &str, errors: &[(Range<usize>, String)], shader: &str) -> Element<'a, Message> {
    let errors = errors
        .iter()
        .map(|(range, err_msg)| {
            let slice = shader.get(range.start..=range.end);
            if let Some(slice) = slice {
                //TODO can't render tabs..?
                text(format!("{msg}:\n    {err_msg}:\n        {slice}"))
            } else {
                text(format!("{msg}:\n    {err_msg}"))
            }
            .class(TextClass::Error)
            .size(14)
            .into()
        })
        .collect::<Vec<Element<'a, Message>>>();

    container(
        scrollable(
            column(errors)
                .width(Length::Fill)
                .padding([10, 20])
                .spacing(10),
        )
            .width(Length::Fill)
            .height(100)
    ).width(Length::Fill)
    .class(ContainerClass::Error)
    .into()
}
