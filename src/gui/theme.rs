//! Sistema de temas y paleta de colores Cacao para Iced 0.13.
//! Mantiene fidelidad total con el diseño original (Modo Oscuro / Modo Claro).

use iced::widget::{button, container, pick_list, text_input};
use iced::{Background, Border, Color, Shadow, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_card: Color,
    pub bg_card_hover: Color,
    pub bg_input: Color,
    pub border: Color,
    pub border_focus: Color,
    pub text_main: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub success: Color,
    pub danger: Color,
    pub warning: Color,
}

impl ThemeMode {
    pub fn palette(&self) -> Palette {
        match self {
            ThemeMode::Dark => Palette {
                bg_primary: Color::from_rgb8(0x14, 0x0E, 0x0A), // #140E0A
                bg_secondary: Color::from_rgb8(0x1E, 0x15, 0x10), // #1E1510
                bg_card: Color::from_rgb8(0x28, 0x1C, 0x15),    // #281C15
                bg_card_hover: Color::from_rgb8(0x35, 0x25, 0x1C), // #35251C
                bg_input: Color::from_rgb8(0x17, 0x10, 0x0B),   // #17100B
                border: Color::from_rgb8(0x42, 0x30, 0x24),     // #423024
                border_focus: Color::from_rgb8(0xC8, 0x6D, 0x27), // #C86D27
                text_main: Color::from_rgb8(0xFD, 0xFB, 0xF7),  // #FDFBF7
                text_muted: Color::from_rgb8(0xBB, 0xA9, 0x9B), // #BBA99B
                accent: Color::from_rgb8(0xD2, 0x7D, 0x2D),     // #D27D2D
                accent_hover: Color::from_rgb8(0xE0, 0x8A, 0x3C), // #E08A3C
                success: Color::from_rgb8(0x22, 0xC5, 0x5E),    // #22C55E
                danger: Color::from_rgb8(0xEF, 0x44, 0x44),     // #EF4444
                warning: Color::from_rgb8(0xF5, 0x9E, 0x0B),    // #F59E0B
            },
            ThemeMode::Light => Palette {
                bg_primary: Color::from_rgb8(0xFA, 0xF6, 0xF0), // #FAF6F0
                bg_secondary: Color::from_rgb8(0xF3, 0xEC, 0xE2), // #F3ECE2
                bg_card: Color::from_rgb8(0xFF, 0xFF, 0xFF),    // #FFFFFF
                bg_card_hover: Color::from_rgb8(0xFD, 0xFB, 0xF7), // #FDFBF7
                bg_input: Color::from_rgb8(0xFA, 0xF6, 0xF0),   // #FAF6F0
                border: Color::from_rgb8(0xEA, 0xDE, 0xCF),     // #EADECF
                border_focus: Color::from_rgb8(0x7B, 0x4A, 0x26), // #7B4A26
                text_main: Color::from_rgb8(0x2B, 0x17, 0x0B),  // #2B170B
                text_muted: Color::from_rgb8(0x7A, 0x66, 0x58), // #7A6658
                accent: Color::from_rgb8(0x7B, 0x4A, 0x26),     // #7B4A26
                accent_hover: Color::from_rgb8(0x5C, 0x35, 0x1B), // #5C351B
                success: Color::from_rgb8(0x16, 0xA3, 0x4A),    // #16A34A
                danger: Color::from_rgb8(0xDC, 0x26, 0x26),     // #DC2626
                warning: Color::from_rgb8(0xD9, 0x77, 0x06),    // #D97706
            },
        }
    }
}

// Estilos de Contenedor
pub fn primary_container_style(mode: ThemeMode) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| {
        let p = mode.palette();
        container::Style {
            background: Some(Background::Color(p.bg_primary)),
            text_color: Some(p.text_main),
            ..Default::default()
        }
    }
}

pub fn card_style(mode: ThemeMode) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| {
        let p = mode.palette();
        container::Style {
            background: Some(Background::Color(p.bg_card)),
            border: Border {
                color: p.border,
                width: 1.0,
                radius: 12.0.into(),
            },
            text_color: Some(p.text_main),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        }
    }
}

pub fn input_container_style(mode: ThemeMode) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| {
        let p = mode.palette();
        container::Style {
            background: Some(Background::Color(p.bg_input)),
            border: Border {
                color: p.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Some(p.text_main),
            ..Default::default()
        }
    }
}

// Estilos de Botón
pub fn primary_button_style(
    mode: ThemeMode,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, status| {
        let p = mode.palette();
        let bg = match status {
            button::Status::Hovered => p.accent_hover,
            button::Status::Pressed => p.accent,
            button::Status::Disabled => p.border,
            _ => p.accent,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn secondary_button_style(
    mode: ThemeMode,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, status| {
        let p = mode.palette();
        let bg = match status {
            button::Status::Hovered => p.bg_card_hover,
            button::Status::Pressed => p.bg_input,
            _ => p.bg_card,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.text_main,
            border: Border {
                color: p.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn danger_button_style(
    mode: ThemeMode,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, status| {
        let p = mode.palette();
        let bg = match status {
            button::Status::Hovered => p.danger,
            _ => Color::from_rgba(p.danger.r, p.danger.g, p.danger.b, 0.15),
        };
        let text_color = match status {
            button::Status::Hovered => Color::WHITE,
            _ => p.danger,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                color: p.danger,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn nav_button_style(
    mode: ThemeMode,
    is_active: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, status| {
        let p = mode.palette();
        let (bg, text) = if is_active {
            (p.accent, Color::WHITE)
        } else {
            match status {
                button::Status::Hovered => (p.bg_card_hover, p.text_main),
                _ => (Color::TRANSPARENT, p.text_muted),
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: text,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

// Estilos de Input
pub fn text_input_style(
    mode: ThemeMode,
) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style {
    move |_, status| {
        let p = mode.palette();
        let border_color = match status {
            text_input::Status::Focused { .. } => p.border_focus,
            text_input::Status::Hovered => p.accent,
            _ => p.border,
        };
        text_input::Style {
            background: Background::Color(p.bg_input),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 6.0.into(),
            },
            icon: p.text_muted,
            placeholder: p.text_muted,
            value: p.text_main,
            selection: p.accent,
        }
    }
}

// Estilos de PickList
pub fn pick_list_style(
    mode: ThemeMode,
) -> impl Fn(&iced::Theme, pick_list::Status) -> pick_list::Style {
    move |_, status| {
        let p = mode.palette();
        let border_color = match status {
            pick_list::Status::Opened { .. } => p.border_focus,
            pick_list::Status::Hovered => p.accent,
            _ => p.border,
        };
        pick_list::Style {
            text_color: p.text_main,
            placeholder_color: p.text_muted,
            handle_color: p.accent,
            background: Background::Color(p.bg_input),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 6.0.into(),
            },
        }
    }
}
