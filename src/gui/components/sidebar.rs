//! Barra lateral (Sidebar) para navegación y créditos en Iced.

use crate::gui::icons::{icon_history, icon_home, icon_package, icon_receipt, icon_settings};
use crate::gui::theme::{ThemeMode, nav_button_style};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Background, Border, Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationTab {
    Dashboard,
    Facturador,
    Inventario,
    Historial,
    Configuracion,
}

impl NavigationTab {
    pub fn label(&self) -> &'static str {
        match self {
            NavigationTab::Dashboard => "Inicio",
            NavigationTab::Facturador => "Facturador",
            NavigationTab::Inventario => "Inventario",
            NavigationTab::Historial => "Historial",
            NavigationTab::Configuracion => "Configuración",
        }
    }
}

pub fn view<'a, Message: Clone + 'a>(
    mode: ThemeMode,
    active_tab: NavigationTab,
    on_tab_change: fn(NavigationTab) -> Message,
    on_credits_press: Message,
) -> Element<'a, Message> {
    let p = mode.palette();

    let tabs = [
        NavigationTab::Dashboard,
        NavigationTab::Facturador,
        NavigationTab::Inventario,
        NavigationTab::Historial,
        NavigationTab::Configuracion,
    ];

    let mut nav_col = column![].spacing(6).width(Length::Fill);

    for tab in tabs {
        let is_active = tab == active_tab;
        let icon_color = if is_active { p.accent } else { p.text_muted };

        let svg_icon = match tab {
            NavigationTab::Dashboard => icon_home(icon_color, 18.0),
            NavigationTab::Facturador => icon_receipt(icon_color, 18.0),
            NavigationTab::Inventario => icon_package(icon_color, 18.0),
            NavigationTab::Historial => icon_history(icon_color, 18.0),
            NavigationTab::Configuracion => icon_settings(icon_color, 18.0),
        };

        let btn_content = row![svg_icon, text(tab.label()).size(14),]
            .spacing(10)
            .align_y(Alignment::Center);

        nav_col = nav_col.push(
            button(btn_content)
                .width(Length::Fill)
                .padding([10, 14])
                .style(nav_button_style(mode, is_active))
                .on_press(on_tab_change(tab)),
        );
    }

    let credits_btn = button(
        container(
            text(">_ desarrollado por cacaoscript")
                .size(10)
                .color(p.text_muted),
        )
        .width(Length::Fill)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([8, 10])
    .style(move |_, status| {
        let bg = match status {
            button::Status::Hovered => p.bg_card_hover,
            button::Status::Pressed => p.bg_input,
            _ => p.bg_card,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: match status {
                button::Status::Hovered => p.accent,
                _ => p.text_muted,
            },
            border: Border {
                color: match status {
                    button::Status::Hovered => p.accent,
                    _ => p.border,
                },
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    })
    .on_press(on_credits_press);

    container(
        column![nav_col, column![].height(Length::Fill), credits_btn,]
            .spacing(12)
            .padding(12),
    )
    .width(200)
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(p.bg_secondary)),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}
