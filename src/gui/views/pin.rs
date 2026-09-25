//! Pantalla de bloqueo y acceso por PIN en Iced.

use crate::gui::theme::{ThemeMode, card_style, primary_button_style, secondary_button_style};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Background, Border, Element, Length};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinOutcome {
    Unlocked,
    Created(String),
}

#[derive(Debug, Clone, Default)]
pub struct PinState {
    pub pin_input: String,
    pub is_new_pin: bool,
    pub error_msg: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PinMessage {
    DigitPressed(char),
    Backspace,
    Clear,
    Submit,
}

impl PinState {
    pub fn update(&mut self, message: PinMessage, stored_pin: Option<&str>) -> Option<PinOutcome> {
        match message {
            PinMessage::DigitPressed(c) => {
                if self.pin_input.len() < 4 {
                    self.pin_input.push(c);
                    self.error_msg = None;
                }
                None
            }
            PinMessage::Backspace => {
                self.pin_input.pop();
                self.error_msg = None;
                None
            }
            PinMessage::Clear => {
                self.pin_input.clear();
                self.error_msg = None;
                None
            }
            PinMessage::Submit => {
                if self.pin_input.len() < 4 {
                    self.error_msg = Some("El PIN debe tener 4 dígitos".to_string());
                    return None;
                }

                if let Some(expected) = stored_pin
                    && !expected.is_empty()
                {
                    if self.pin_input == expected {
                        self.pin_input.clear();
                        self.error_msg = None;
                        return Some(PinOutcome::Unlocked);
                    } else {
                        self.error_msg = Some("PIN incorrecto. Intenta de nuevo.".to_string());
                        self.pin_input.clear();
                        return None;
                    }
                }

                // Sin PIN previo: se crea el nuevo PIN
                let new_pin = self.pin_input.clone();
                self.pin_input.clear();
                self.error_msg = None;
                self.is_new_pin = false;
                Some(PinOutcome::Created(new_pin))
            }
        }
    }

    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        mode: ThemeMode,
        on_message: fn(PinMessage) -> Message,
    ) -> Element<'a, Message> {
        let p = mode.palette();

        let dots_row = row((0..4).map(|i| {
            let is_filled = i < self.pin_input.len();
            container("")
                .width(14)
                .height(14)
                .style(move |_| container::Style {
                    background: if is_filled {
                        Some(Background::Color(p.accent))
                    } else {
                        None
                    },
                    border: Border {
                        color: if is_filled { p.accent } else { p.border },
                        width: if is_filled { 0.0 } else { 2.0 },
                        radius: 7.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        }))
        .spacing(14)
        .align_y(Alignment::Center);

        let keypad_button = |content: Element<'a, Message>, msg: PinMessage| {
            button(
                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .width(68)
            .height(54)
            .padding(0)
            .style(secondary_button_style(mode))
            .on_press(on_message(msg))
        };

        let keypad_num_button = |label: &'static str, digit: char| {
            keypad_button(
                text(label)
                    .size(20)
                    .font(iced::Font {
                        family: iced::font::Family::Name("Montserrat"),
                        weight: iced::font::Weight::Semibold,
                        ..Default::default()
                    })
                    .color(p.text_main)
                    .into(),
                PinMessage::DigitPressed(digit),
            )
        };

        let row1 = row![
            keypad_num_button("1", '1'),
            keypad_num_button("2", '2'),
            keypad_num_button("3", '3'),
        ]
        .spacing(12);

        let row2 = row![
            keypad_num_button("4", '4'),
            keypad_num_button("5", '5'),
            keypad_num_button("6", '6'),
        ]
        .spacing(12);

        let row3 = row![
            keypad_num_button("7", '7'),
            keypad_num_button("8", '8'),
            keypad_num_button("9", '9'),
        ]
        .spacing(12);

        let clear_button = keypad_button(
            text("C")
                .size(18)
                .font(iced::Font {
                    family: iced::font::Family::Name("Montserrat"),
                    weight: iced::font::Weight::Semibold,
                    ..Default::default()
                })
                .color(p.text_muted)
                .into(),
            PinMessage::Clear,
        );

        let backspace_button = keypad_button(
            crate::gui::icons::icon_backspace(p.text_main, 20.0).into(),
            PinMessage::Backspace,
        );

        let row4 = row![clear_button, keypad_num_button("0", '0'), backspace_button,].spacing(12);

        let btn_label = if self.is_new_pin {
            "Crear PIN e Ingresar"
        } else {
            "Desbloquear App"
        };

        let submit_button = button(
            container(text(btn_label).size(14).font(iced::Font {
                family: iced::font::Family::Name("Montserrat"),
                weight: iced::font::Weight::Semibold,
                ..Default::default()
            }))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
        )
        .width(228)
        .height(44)
        .padding(0)
        .style(primary_button_style(mode))
        .on_press(on_message(PinMessage::Submit));

        let mut content = column![
            crate::gui::icons::icon_lock(p.accent, 32.0),
            text("Cacao Facturador")
                .font(iced::Font {
                    family: iced::font::Family::Name("Montserrat"),
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .size(20)
                .color(p.text_main),
            text(if self.is_new_pin {
                "Crea tu PIN de seguridad de 4 dígitos"
            } else {
                "Ingresa tu PIN de acceso"
            })
            .size(13)
            .color(p.text_muted),
            container(dots_row).padding([6, 16]),
            row1,
            row2,
            row3,
            row4,
            submit_button,
        ]
        .spacing(12)
        .align_x(Alignment::Center);

        if let Some(err) = &self.error_msg {
            content = content.push(text(err).size(12).color(p.danger));
        }

        container(container(content).padding(32).style(card_style(mode)))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_rechaza_numero_incorrecto() {
        let mut state = PinState::default();
        let stored_pin = "4321";

        // Ingresar 4 dígitos incorrectos
        state.update(PinMessage::DigitPressed('1'), Some(stored_pin));
        state.update(PinMessage::DigitPressed('2'), Some(stored_pin));
        state.update(PinMessage::DigitPressed('3'), Some(stored_pin));
        state.update(PinMessage::DigitPressed('4'), Some(stored_pin));

        let result = state.update(PinMessage::Submit, Some(stored_pin));
        assert_eq!(result, None, "Un PIN incorrecto no debe desbloquear la app");
        assert!(state.error_msg.is_some(), "Debe mostrar mensaje de error");
        assert!(
            state.pin_input.is_empty(),
            "Debe limpiar el input tras fallo"
        );
    }

    #[test]
    fn test_pin_acepta_numero_correcto() {
        let mut state = PinState::default();
        let stored_pin = "4321";

        // Ingresar 4 dígitos correctos
        state.update(PinMessage::DigitPressed('4'), Some(stored_pin));
        state.update(PinMessage::DigitPressed('3'), Some(stored_pin));
        state.update(PinMessage::DigitPressed('2'), Some(stored_pin));
        state.update(PinMessage::DigitPressed('1'), Some(stored_pin));

        let result = state.update(PinMessage::Submit, Some(stored_pin));
        assert_eq!(
            result,
            Some(PinOutcome::Unlocked),
            "El PIN exacto debe desbloquear exitosamente"
        );
        assert!(state.error_msg.is_none());
        assert!(state.pin_input.is_empty());
    }

    #[test]
    fn test_creacion_primer_pin() {
        let mut state = PinState {
            is_new_pin: true,
            ..Default::default()
        };

        state.update(PinMessage::DigitPressed('9'), None);
        state.update(PinMessage::DigitPressed('8'), None);
        state.update(PinMessage::DigitPressed('7'), None);
        state.update(PinMessage::DigitPressed('6'), None);

        let result = state.update(PinMessage::Submit, None);
        assert_eq!(
            result,
            Some(PinOutcome::Created("9876".to_string())),
            "Al no haber PIN previo, debe emitir PinOutcome::Created con el valor ingresado"
        );
    }

    #[test]
    fn test_pin_incompleto_falla() {
        let mut state = PinState::default();
        state.update(PinMessage::DigitPressed('1'), Some("1234"));
        state.update(PinMessage::DigitPressed('2'), Some("1234"));

        let result = state.update(PinMessage::Submit, Some("1234"));
        assert_eq!(result, None);
        assert!(state.error_msg.is_some());
    }

    #[test]
    fn test_pin_backspace_y_clear() {
        let mut state = PinState::default();
        state.update(PinMessage::DigitPressed('1'), None);
        state.update(PinMessage::DigitPressed('2'), None);
        assert_eq!(state.pin_input, "12");

        // Backspace borra el último dígito
        state.update(PinMessage::Backspace, None);
        assert_eq!(state.pin_input, "1");

        // Clear borra todo el input
        state.update(PinMessage::DigitPressed('5'), None);
        assert_eq!(state.pin_input, "15");
        state.update(PinMessage::Clear, None);
        assert_eq!(state.pin_input, "");
    }
}
