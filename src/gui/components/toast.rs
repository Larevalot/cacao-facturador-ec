//! Sistema de notificaciones Toast flotantes para Iced 0.13.

use crate::gui::theme::ThemeMode;
use iced::widget::{button, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Shadow, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u64,
    pub toast_type: ToastType,
    pub title: String,
    pub message: String,
    pub created_at: std::time::Instant,
}

impl Toast {
    pub fn new(
        id: u64,
        toast_type: ToastType,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id,
            toast_type,
            title: title.into(),
            message: message.into(),
            created_at: std::time::Instant::now(),
        }
    }

    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        mode: ThemeMode,
        on_close: Message,
    ) -> Element<'a, Message> {
        let p = mode.palette();
        let (border_color, badge_text, icon) = match self.toast_type {
            ToastType::Success => (p.success, "Éxito", "✓"),
            ToastType::Error => (p.danger, "Error", "✕"),
            ToastType::Warning => (p.warning, "Advertencia", "⚠"),
            ToastType::Info => (p.accent, "Info", "ℹ"),
        };

        let title_text = if self.title.is_empty() {
            badge_text
        } else {
            &self.title
        };

        container(
            row![
                text(icon).size(18).color(border_color),
                text(title_text).size(14).color(p.text_main),
                text(&self.message).size(13).color(p.text_muted),
                button(text("✕").size(12))
                    .style(move |_, _| button::Style {
                        background: None,
                        text_color: p.text_muted,
                        border: Border::default(),
                        shadow: Shadow::default(),
                        ..Default::default()
                    })
                    .on_press(on_close)
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .padding([8, 16])
        .style(move |_| container::Style {
            background: Some(Background::Color(p.bg_card)),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        })
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_toast_creation_timestamp() {
        let toast = Toast::new(1, ToastType::Success, "Acceso Correcto", "Bienvenido");
        assert_eq!(toast.id, 1);
        assert_eq!(toast.toast_type, ToastType::Success);
        assert_eq!(toast.title, "Acceso Correcto");
        assert_eq!(toast.message, "Bienvenido");
        assert!(toast.created_at.elapsed() < Duration::from_secs(1));
    }
}
