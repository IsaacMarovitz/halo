use iced::application::Appearance;
use iced::widget::button::Status;
use iced::widget::pane_grid::Highlight;
use iced::widget::scrollable::Rail;
use iced::widget::{button, checkbox, container, pane_grid, scrollable, text, text_editor};
use iced::{application, Border, Color};
use std::default::Default;

struct Palette {
    pub base: Color,
    pub base_darker: Color,
    pub base_darkest: Color,
    pub base_lighter: Color,
    pub base_lightest: Color,
    pub background: Color,
    pub text: Color,
    pub disabled: Color,
    pub accent: Color,
    pub accent_secondary: Color,
    pub error: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            base: Color::from_rgb8(84, 111, 149),             // MEDIUM BLUE
            base_darker: Color::from_rgb8(38, 62, 99),        // DARK BLUE
            base_darkest: Color::from_rgb8(22, 30, 59),       // DARK BLUE
            base_lighter: Color::from_rgb8(113, 134, 162),    // LIGHTER BLUE
            base_lightest: Color::from_rgb8(158, 175, 195),   // LIGHTEST BLUE
            background: Color::from_rgb8(12, 12, 30),         // DARKEST BLUE
            text: Color::from_rgb8(226, 226, 226),            // LIGHT GREY
            disabled: Color::from_rgb8(153, 158, 162),        // MEDIUM GREY
            accent: Color::from_rgb8(235, 94, 85),            // SALMON
            accent_secondary: Color::from_rgb8(255, 159, 28), // GOLD
            error: Color::from_rgb8(255, 77, 77),             // ERROR RED
        }
    }
}

// Borders
const BORDER_RADIUS: f32 = 5.0;
const BORDER_WIDTH: f32 = 2.0;

/// Reduces intensity by 10%
fn dim(color: Color) -> Color {
    Color {
        r: color.r * 0.70,
        g: color.g * 0.70,
        b: color.b * 0.70,
        a: color.a,
    }
}

#[derive(Default)]
pub enum Theme {
    Light,
    #[default]
    Dark,
}

impl Theme {
    fn palette(&self) -> Palette {
        match self {
            // TODO: Light palette
            Theme::Light => Palette::default(),
            Theme::Dark => Palette::default(),
        }
    }
}

impl application::DefaultStyle for Theme {
    fn default_style(&self) -> Appearance {
        let palette = self.palette();

        Appearance {
            background_color: palette.background,
            text_color: palette.text,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum ContainerClass {
    Tooltip,
    Controls,
    Error,
    #[default]
    None,
}

impl container::Catalog for Theme {
    type Class<'a> = ContainerClass;

    fn default<'a>() -> Self::Class<'a> {
        ContainerClass::None
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        let mut style = container::Style::default();
        let palette = self.palette();

        match class {
            ContainerClass::Tooltip => {
                style.background = Some(palette.base_darker.into());
                style.text_color = Some(palette.text);
                style.border = Border {
                    radius: BORDER_RADIUS.into(),
                    width: BORDER_WIDTH,
                    color: palette.base_darkest,
                };
            },
            ContainerClass::Controls => {
                style.background = Some(palette.base_darker.into());
                style.text_color = Some(palette.text);
            },
            ContainerClass::Error => {
                style.background = Some(palette.base_darkest.into());
                style.text_color = Some(palette.error);
                style.border = Border {
                    radius: Default::default(),
                    width: 1.0,
                    color: palette.error,
                };
            },
            _ => {}
        };

        style
    }
}

#[derive(Default, Clone)]
pub enum TextClass {
    #[default]
    Primary,
    Error,
}

impl text::Catalog for Theme {
    type Class<'a> = TextClass;

    fn default<'a>() -> Self::Class<'a> {
        TextClass::Primary
    }

    fn style(&self, class: &Self::Class<'_>) -> text::Style {
        let palette = self.palette();

        match class {
            TextClass::Primary => text::Style {
                color: Some(palette.text),
            },
            TextClass::Error => text::Style {
                color: Some(palette.error),
            },
        }
    }
}

impl pane_grid::Catalog for Theme {
    type Class<'a> = ();

    fn default<'a>() -> <Self as pane_grid::Catalog>::Class<'a> {
        ()
    }

    fn style(&self, _class: &<Self as pane_grid::Catalog>::Class<'_>) -> pane_grid::Style {
        let palette = self.palette();

        pane_grid::Style {
            hovered_region: Highlight {
                background: palette.background.into(),
                border: Border {
                    radius: Default::default(),
                    width: 0.0,
                    color: Default::default(),
                },
            },
            picked_split: pane_grid::Line {
                color: palette.accent_secondary,
                width: 5.0,
            },
            hovered_split: pane_grid::Line {
                color: palette.accent,
                width: 5.0,
            },
        }
    }
}

#[derive(Default)]
pub enum ButtonClass {
    #[default]
    Control,
}

impl button::Catalog for Theme {
    type Class<'a> = ButtonClass;

    fn default<'a>() -> Self::Class<'a> {
        ButtonClass::Control
    }

    fn style(&self, _class: &Self::Class<'_>, status: Status) -> button::Style {
        let mut style = button::Style::default();
        let palette = self.palette();

        style.background = Some(palette.base_lighter.into());
        style.text_color = palette.text;
        style.border = Border {
            radius: 2.0.into(),
            width: 0.0,
            color: Default::default(),
        };

        match status {
            Status::Active => {
                style.background = Some(palette.base.into());
            },
            _ => {}
        };

        style
    }
}

impl checkbox::Catalog for Theme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>, status: checkbox::Status) -> checkbox::Style {
        let palette = self.palette();

        let base = checkbox::Style {
            background: palette.base.into(),
            icon_color: palette.text,
            border: Border {
                radius: 2.0.into(),
                width: 0.0,
                color: Default::default(),
            },
            text_color: Some(palette.text),
        };

        match status {
            checkbox::Status::Active { .. } => base,
            checkbox::Status::Hovered { .. } => {
                let mut hovered = base.clone();
                hovered.background = palette.base_lighter.into();

                hovered
            },
            checkbox::Status::Disabled { .. } => {
                let mut disabled = base.clone();
                disabled.background = palette.disabled.into();

                disabled
            },
        }
    }
}

impl text_editor::Catalog for Theme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>, status: text_editor::Status) -> text_editor::Style {
        let palette = self.palette();

        let style = text_editor::Style {
            background: palette.background.into(),
            border: Border {
                radius: Default::default(),
                width: 0.0,
                color: Default::default(),
            },
            icon: palette.text,
            placeholder: palette.disabled,
            value: palette.text,
            selection: palette.base_lighter,
        };

        match status {
            text_editor::Status::Active => style,
            text_editor::Status::Hovered => style,
            text_editor::Status::Focused => style,
            text_editor::Status::Disabled => style,
        }
    }
}

impl scrollable::Catalog for Theme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>, _status: scrollable::Status) -> scrollable::Style {
        let palette = self.palette();

        let rail = Rail {
            background: Some(palette.base_darker.into()),
            scroller: scrollable::Scroller {
                color: palette.error,
                border: Border::default(),
            },
            border: Border::default(),
        };

        scrollable::Style {
            container: Default::default(),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
        }
    }
}
