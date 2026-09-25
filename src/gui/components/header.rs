//! Barra de título unificada (Titlebar) personalizada para Cacao Facturador SRI.

use crate::gui::icons::{
    icon_close, icon_lock, icon_maximize, icon_minimize, icon_refresh, icon_theme,
};
use crate::gui::theme::{Palette, ThemeMode};
use iced::widget::{button, container, mouse_area, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderAction {
    CheckUpdates,
    ToggleTheme,
    LockApp,
    Minimize,
    Maximize,
    Close,
    DragWindow,
}

pub fn view<'a, Message: Clone + 'a>(
    mode: ThemeMode,
    is_unlocked: bool,
    on_action: fn(HeaderAction) -> Message,
) -> Element<'a, Message> {
    let p = mode.palette();

    // 1. Título "Cacao Facturador" con tipografía Montserrat Bold + Tag de versión
    let title_section = row![
        text("Cacao Facturador")
            .font(iced::Font {
                family: iced::font::Family::Name("Montserrat"),
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .size(15)
            .color(p.text_main),
        container(text("v1.0.0").size(10).color(p.accent))
            .padding([2, 6])
            .style(move |_| container::Style {
                background: Some(Background::Color(p.bg_card)),
                border: Border {
                    color: p.border,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    // 2. Botones de acción principales (Solo íconos SVG)
    let is_dark = matches!(mode, ThemeMode::Dark);

    let btn_updates = button(icon_refresh(p.text_main, 16.0))
        .style(icon_button_style(p))
        .padding([6, 8])
        .on_press(on_action(HeaderAction::CheckUpdates));

    let btn_theme = button(icon_theme(p.text_main, is_dark, 16.0))
        .style(icon_button_style(p))
        .padding([6, 8])
        .on_press(on_action(HeaderAction::ToggleTheme));

    let mut actions_row = row![btn_updates, btn_theme]
        .spacing(4)
        .align_y(Alignment::Center);

    if is_unlocked {
        let btn_lock = button(icon_lock(p.text_main, 16.0))
            .style(icon_button_style(p))
            .padding([6, 8])
            .on_press(on_action(HeaderAction::LockApp));
        actions_row = actions_row.push(btn_lock);
    }

    // 3. Controles de ventana estándar (Minimizar, Maximizar, Cerrar - Solo SVG)
    let btn_minimize = button(icon_minimize(p.text_main, 14.0))
        .style(window_control_style(p))
        .padding([6, 10])
        .on_press(on_action(HeaderAction::Minimize));

    let btn_maximize = button(icon_maximize(p.text_main, 13.0))
        .style(window_control_style(p))
        .padding([6, 10])
        .on_press(on_action(HeaderAction::Maximize));

    let btn_close = button(icon_close(p.text_main, 13.0))
        .style(close_control_style(p))
        .padding([6, 12])
        .on_press(on_action(HeaderAction::Close));

    let window_controls = row![btn_minimize, btn_maximize, btn_close]
        .spacing(2)
        .align_y(Alignment::Center);

    // Barra de título completa: [Título] --- Espacio arrastrable --- [Acciones] [Separador] [Controles]
    let titlebar_content = row![
        title_section,
        row![].width(Length::Fill), // Espacio intermedio
        actions_row,
        container(row![])
            .width(Length::Fixed(1.0))
            .height(Length::Fixed(18.0))
            .style(move |_| container::Style {
                background: Some(Background::Color(p.border)),
                ..Default::default()
            }),
        window_controls,
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .padding([5, 12]);

    let titlebar_container = container(titlebar_content)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(p.bg_secondary)),
            border: Border {
                color: p.border,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

    // Envolver en mouse_area para permitir arrastrar la ventana y maximizar con doble clic
    mouse_area(titlebar_container)
        .on_press(on_action(HeaderAction::DragWindow))
        .on_double_click(on_action(HeaderAction::Maximize))
        .into()
}

fn icon_button_style(p: Palette) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = match status {
            button::Status::Hovered => Some(Background::Color(p.bg_card)),
            button::Status::Pressed => Some(Background::Color(p.border)),
            _ => None,
        };
        button::Style {
            background: bg,
            text_color: p.text_main,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 5.0.into(),
            },
            ..Default::default()
        }
    }
}

fn window_control_style(p: Palette) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = match status {
            button::Status::Hovered => Some(Background::Color(p.bg_card)),
            button::Status::Pressed => Some(Background::Color(p.border)),
            _ => None,
        };
        button::Style {
            background: bg,
            text_color: p.text_main,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}

fn close_control_style(p: Palette) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, status| {
        let (bg, text_color) = match status {
            button::Status::Hovered => (
                Some(Background::Color(Color::from_rgb8(0xDC, 0x26, 0x26))),
                Color::WHITE,
            ),
            button::Status::Pressed => (
                Some(Background::Color(Color::from_rgb8(0xB9, 0x1C, 0x1C))),
                Color::WHITE,
            ),
            _ => (None, p.text_main),
        };
        button::Style {
            background: bg,
            text_color,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}
