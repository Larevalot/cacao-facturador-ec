//! Pantalla de Inicio / Dashboard de Resumen para Cacao Facturador SRI.

use crate::db::facturas::FacturaGuardada;
use crate::db::productos::Producto;
use crate::gui::components::sidebar::NavigationTab;
use crate::gui::icons::{
    icon_alert_triangle, icon_arrow_right, icon_check, icon_check_circle, icon_history, icon_info,
    icon_package, icon_plus, icon_receipt, icon_settings,
};
use crate::gui::theme::{ThemeMode, card_style, primary_button_style, secondary_button_style};
use crate::sri::models::EmisorConfig;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Background, Border, Color, Element, Length};

#[derive(Debug, Clone)]
pub struct ConfigAudit {
    pub is_complete: bool,
    pub missing_items: Vec<&'static str>,
    pub completed_count: usize,
    pub total_count: usize,
}

pub fn auditar_configuracion(config: &EmisorConfig) -> ConfigAudit {
    let mut missing = Vec::new();
    let mut completed = 0usize;
    let total_count = 7;

    // 1. RUC
    let ruc = config.ruc.trim();
    if !ruc.is_empty() && ruc != "1790000000001" && ruc.len() == 13 {
        completed += 1;
    } else {
        missing.push("RUC del Emisor (requiere 13 dígitos válidos)");
    }

    // 2. Razón Social
    let razon = config.razon_social.trim();
    if !razon.is_empty() && razon != "MI EMPRESA S.A." {
        completed += 1;
    } else {
        missing.push("Razón Social del negocio");
    }

    // 3. Dirección Matriz
    let dir = config.dir_matriz.trim();
    if !dir.is_empty() && dir != "Quito, Ecuador" {
        completed += 1;
    } else if !dir.is_empty() {
        completed += 1;
    } else {
        missing.push("Dirección Matriz");
    }

    // 4. Establecimiento y Punto de Emisión
    if !config.cod_establecimiento.trim().is_empty() && !config.pto_emision.trim().is_empty() {
        completed += 1;
    } else {
        missing.push("Código de Establecimiento y Punto de Emisión");
    }

    // 5. Certificado Digital .p12
    if let Some(p12) = &config.p12_path {
        if !p12.trim().is_empty() && std::path::Path::new(p12).exists() {
            completed += 1;
        } else {
            missing.push("Archivo de Firma Electrónica (.p12 / .pfx) en disco");
        }
    } else {
        missing.push("Certificado Digital de Firma Electrónica (.p12)");
    }

    // 6. Contraseña del Certificado
    if let Some(pass) = &config.p12_password {
        if !pass.trim().is_empty() {
            completed += 1;
        } else {
            missing.push("Contraseña del Certificado Digital");
        }
    } else {
        missing.push("Contraseña del Certificado Digital");
    }

    // 7. PIN de Bloqueo de la App
    if let Some(pin) = &config.pin {
        if !pin.trim().is_empty() {
            completed += 1;
        } else {
            missing.push("PIN de Seguridad de la App");
        }
    } else {
        missing.push("PIN de Seguridad de la App");
    }

    ConfigAudit {
        is_complete: missing.is_empty(),
        missing_items: missing,
        completed_count: completed,
        total_count,
    }
}

pub fn view<'a, Message: Clone + 'a>(
    mode: ThemeMode,
    ultima_factura: Option<&'a FacturaGuardada>,
    ultimo_producto: Option<&'a Producto>,
    config: &'a EmisorConfig,
    on_navigate: fn(NavigationTab) -> Message,
) -> Element<'a, Message> {
    let p = mode.palette();
    let audit = auditar_configuracion(config);

    // 1. Cabecera principal
    let ambiente_label = if config.ambiente == "2" {
        "Ambiente: Producción"
    } else {
        "Ambiente: Pruebas (SRI)"
    };

    let header_section = row![
        column![
            text("Resumen General")
                .font(iced::Font {
                    family: iced::font::Family::Name("Montserrat"),
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .size(22)
                .color(p.text_main),
            text("Estado operativo de tu facturación electrónica e inventario")
                .size(13)
                .color(p.text_muted),
        ]
        .spacing(4),
        row![].width(Length::Fill),
        container(
            row![
                icon_info(p.accent, 14.0),
                text(ambiente_label).size(12).color(p.accent),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([6, 12])
        .style(move |_| container::Style {
            background: Some(Background::Color(p.bg_card)),
            border: Border {
                color: p.border,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }),
    ]
    .align_y(Alignment::Center);

    // 2. Tarjeta: Estado de las Configuraciones
    let config_card = {
        let (icon_widget, status_title, title_color) = if audit.is_complete {
            (
                icon_check_circle(p.accent, 22.0),
                "Configuraciones Completas y Listas para Emitir",
                p.accent,
            )
        } else {
            (
                icon_alert_triangle(p.danger, 22.0),
                "Configuración Incompleta",
                p.danger,
            )
        };

        let btn_config = button(
            row![
                icon_settings(p.text_main, 14.0),
                text("Gestionar Configuración").size(12),
                icon_arrow_right(p.text_main, 14.0),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .style(secondary_button_style(mode))
        .padding([6, 12])
        .on_press(on_navigate(NavigationTab::Configuracion));

        let top_row = row![
            row![
                icon_widget,
                column![
                    text(status_title)
                        .font(iced::Font {
                            family: iced::font::Family::Name("Montserrat"),
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .size(15)
                        .color(title_color),
                    text(format!(
                        "{}/{} parámetros requeridos configurados",
                        audit.completed_count, audit.total_count
                    ))
                    .size(12)
                    .color(p.text_muted),
                ]
                .spacing(2),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![].width(Length::Fill),
            btn_config,
        ]
        .align_y(Alignment::Center);

        let body_content: Element<Message> = if audit.is_complete {
            row![
                summary_pill("RUC", &config.ruc, p.bg_secondary, p.border, p.text_main),
                summary_pill(
                    "Emisor",
                    &config.razon_social,
                    p.bg_secondary,
                    p.border,
                    p.text_main
                ),
                summary_pill(
                    "Establecimiento",
                    format!("{}-{}", config.cod_establecimiento, config.pto_emision),
                    p.bg_secondary,
                    p.border,
                    p.text_main
                ),
                summary_pill(
                    "Firma Digital",
                    "Cargada (.p12)",
                    p.bg_secondary,
                    p.border,
                    p.accent
                ),
            ]
            .spacing(12)
            .into()
        } else {
            let mut missing_list = column![].spacing(6);
            for item in &audit.missing_items {
                let item_row = row![
                    icon_alert_triangle(p.danger, 13.0),
                    text(*item).size(12).color(p.text_muted),
                ]
                .spacing(8)
                .align_y(Alignment::Center);
                missing_list = missing_list.push(item_row);
            }
            column![
                text("Faltan los siguientes parámetros para emitir comprobantes autorizados:")
                    .size(12)
                    .color(p.text_muted),
                missing_list,
            ]
            .spacing(8)
            .into()
        };

        container(
            column![
                top_row,
                container(row![])
                    .height(Length::Fixed(1.0))
                    .width(Length::Fill)
                    .style(move |_| container::Style {
                        background: Some(Background::Color(p.border)),
                        ..Default::default()
                    }),
                body_content,
            ]
            .spacing(14),
        )
        .padding(18)
        .width(Length::Fill)
        .style(card_style(mode))
    };

    // 3. Tarjeta: Última Factura Emitida
    let factura_card = {
        let header_row = row![
            row![
                icon_receipt(p.accent, 18.0),
                text("Última Factura Emitida")
                    .font(iced::Font {
                        family: iced::font::Family::Name("Montserrat"),
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .size(15)
                    .color(p.text_main),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![].width(Length::Fill),
            button(
                row![
                    icon_history(p.text_main, 14.0),
                    text("Ver Historial").size(12),
                    icon_arrow_right(p.text_main, 14.0),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .style(secondary_button_style(mode))
            .padding([6, 12])
            .on_press(on_navigate(NavigationTab::Historial)),
        ]
        .align_y(Alignment::Center);

        let content_box: Element<Message> = if let Some(fac) = ultima_factura {
            let estado_color = if fac.estado.to_uppercase().contains("AUTORIZAD") {
                p.accent
            } else if fac.estado.to_uppercase().contains("DEVUELT")
                || fac.estado.to_uppercase().contains("RECHAZAD")
            {
                p.danger
            } else {
                Color::from_rgb8(0x3B, 0x82, 0xF6)
            };

            let status_badge = container(
                row![
                    if fac.estado.to_uppercase().contains("AUTORIZAD") {
                        icon_check(estado_color, 12.0)
                    } else {
                        icon_info(estado_color, 12.0)
                    },
                    text(&fac.estado).size(11).color(estado_color),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .padding([3, 8])
            .style(move |_| container::Style {
                background: Some(Background::Color(p.bg_secondary)),
                border: Border {
                    color: estado_color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

            column![
                row![
                    column![
                        text(format!("Secuencial: #{}", fac.secuencial))
                            .font(iced::Font {
                                family: iced::font::Family::Name("Montserrat"),
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                            .size(14)
                            .color(p.text_main),
                        text(format!("Fecha: {}", fac.fecha_emision))
                            .size(12)
                            .color(p.text_muted),
                    ]
                    .spacing(2),
                    row![].width(Length::Fill),
                    status_badge,
                ]
                .align_y(Alignment::Center),
                container(row![])
                    .height(Length::Fixed(1.0))
                    .width(Length::Fill)
                    .style(move |_| container::Style {
                        background: Some(Background::Color(p.border)),
                        ..Default::default()
                    }),
                row![
                    column![
                        text("Cliente:").size(11).color(p.text_muted),
                        text(&fac.cliente_razon).size(13).color(p.text_main),
                        text(format!("ID: {}", fac.cliente_identificacion))
                            .size(11)
                            .color(p.text_muted),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    column![
                        text("Total Factura:").size(11).color(p.text_muted),
                        text(format!("${:.2}", fac.importe_total))
                            .font(iced::Font {
                                family: iced::font::Family::Name("Montserrat"),
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                            .size(18)
                            .color(p.accent),
                        text(format!("IVA: ${:.2}", fac.total_iva))
                            .size(11)
                            .color(p.text_muted),
                    ]
                    .spacing(2)
                    .align_x(Alignment::End),
                ]
                .align_y(Alignment::Center),
                button(
                    row![
                        icon_plus(Color::WHITE, 14.0),
                        text("Emitir Nueva Factura").size(13),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .padding([8, 14])
                .style(primary_button_style(mode))
                .on_press(on_navigate(NavigationTab::Facturador)),
            ]
            .spacing(12)
            .into()
        } else {
            column![
                icon_receipt(p.text_muted, 32.0),
                text("Aún no has emitido facturas")
                    .size(14)
                    .color(p.text_main),
                text("Crea y autoriza tu primer comprobante electrónico directamente con el SRI.")
                    .size(12)
                    .color(p.text_muted),
                button(
                    row![
                        icon_plus(Color::WHITE, 14.0),
                        text("Emitir Primera Factura").size(13),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([8, 16])
                .style(primary_button_style(mode))
                .on_press(on_navigate(NavigationTab::Facturador)),
            ]
            .spacing(10)
            .align_x(Alignment::Center)
            .into()
        };

        container(
            column![
                header_row,
                container(row![])
                    .height(Length::Fixed(1.0))
                    .width(Length::Fill)
                    .style(move |_| container::Style {
                        background: Some(Background::Color(p.border)),
                        ..Default::default()
                    }),
                content_box,
            ]
            .spacing(12),
        )
        .padding(18)
        .width(Length::FillPortion(1))
        .style(card_style(mode))
    };

    // 4. Tarjeta: Último Producto o Servicio Agregado
    let producto_card = {
        let header_row = row![
            row![
                icon_package(p.accent, 18.0),
                text("Último Producto o Servicio")
                    .font(iced::Font {
                        family: iced::font::Family::Name("Montserrat"),
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .size(15)
                    .color(p.text_main),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![].width(Length::Fill),
            button(
                row![
                    icon_package(p.text_main, 14.0),
                    text("Ver Catálogo").size(12),
                    icon_arrow_right(p.text_main, 14.0),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .style(secondary_button_style(mode))
            .padding([6, 12])
            .on_press(on_navigate(NavigationTab::Inventario)),
        ]
        .align_y(Alignment::Center);

        let content_box: Element<Message> = if let Some(prod) = ultimo_producto {
            let is_service = prod.tipo.to_uppercase() == "SERVICIO";
            let type_badge = container(
                text(if is_service { "SERVICIO" } else { "PRODUCTO" })
                    .size(10)
                    .color(p.text_main),
            )
            .padding([3, 8])
            .style(move |_| container::Style {
                background: Some(Background::Color(p.bg_secondary)),
                border: Border {
                    color: p.border,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

            column![
                row![
                    column![
                        text(&prod.descripcion)
                            .font(iced::Font {
                                family: iced::font::Family::Name("Montserrat"),
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                            .size(14)
                            .color(p.text_main),
                        text(format!("Código: {}", prod.codigo))
                            .size(12)
                            .color(p.text_muted),
                    ]
                    .spacing(2),
                    row![].width(Length::Fill),
                    type_badge,
                ]
                .align_y(Alignment::Center),
                container(row![])
                    .height(Length::Fixed(1.0))
                    .width(Length::Fill)
                    .style(move |_| container::Style {
                        background: Some(Background::Color(p.border)),
                        ..Default::default()
                    }),
                row![
                    column![
                        text("Stock / Disponibilidad:").size(11).color(p.text_muted),
                        text(if is_service {
                            "Sin límite (Servicio)".to_string()
                        } else {
                            format!("{:.2} unidades", prod.stock)
                        })
                        .size(13)
                        .color(p.text_main),
                        text(format!("Tarifa IVA: {}%", prod.tarifa_iva as i64))
                            .size(11)
                            .color(p.text_muted),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    column![
                        text("Precio Unitario:").size(11).color(p.text_muted),
                        text(format!("${:.2}", prod.precio_unitario))
                            .font(iced::Font {
                                family: iced::font::Family::Name("Montserrat"),
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                            .size(18)
                            .color(p.accent),
                        text(format!(
                            "PVP con IVA: ${:.2}",
                            prod.precio_unitario * (1.0 + prod.tarifa_iva / 100.0)
                        ))
                        .size(11)
                        .color(p.text_muted),
                    ]
                    .spacing(2)
                    .align_x(Alignment::End),
                ]
                .align_y(Alignment::Center),
                button(
                    row![
                        icon_plus(Color::WHITE, 14.0),
                        text("Agregar Nuevo Producto").size(13),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .padding([8, 14])
                .style(primary_button_style(mode))
                .on_press(on_navigate(NavigationTab::Inventario)),
            ]
            .spacing(12)
            .into()
        } else {
            column![
                icon_package(p.text_muted, 32.0),
                text("No hay productos o servicios registrados")
                    .size(14)
                    .color(p.text_main),
                text("Agrega ítems a tu inventario para seleccionarlos rápidamente al emitir facturas.")
                    .size(12)
                    .color(p.text_muted),
                button(
                    row![
                        icon_plus(Color::WHITE, 14.0),
                        text("Agregar Primer Producto").size(13),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([8, 16])
                .style(primary_button_style(mode))
                .on_press(on_navigate(NavigationTab::Inventario)),
            ]
            .spacing(10)
            .align_x(Alignment::Center)
            .into()
        };

        container(
            column![
                header_row,
                container(row![])
                    .height(Length::Fixed(1.0))
                    .width(Length::Fill)
                    .style(move |_| container::Style {
                        background: Some(Background::Color(p.border)),
                        ..Default::default()
                    }),
                content_box,
            ]
            .spacing(12),
        )
        .padding(18)
        .width(Length::FillPortion(1))
        .style(card_style(mode))
    };

    let details_row = row![factura_card, producto_card].spacing(16);

    let content = column![header_section, config_card, details_row,].spacing(20);

    scrollable(container(content).padding(24).width(Length::Fill)).into()
}

fn summary_pill<'a, Message: 'a>(
    label: &'static str,
    val: impl Into<String>,
    bg: Color,
    border_color: Color,
    text_color: Color,
) -> Element<'a, Message> {
    container(
        column![
            text(label)
                .size(10)
                .color(Color::from_rgb8(0x94, 0xA3, 0xB8)),
            text(val.into()).size(12).color(text_color),
        ]
        .spacing(2),
    )
    .padding([6, 12])
    .style(move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auditoria_detecta_config_incompleta_por_defecto() {
        let config = EmisorConfig::default();
        let audit = auditar_configuracion(&config);

        assert!(
            !audit.is_complete,
            "La configuración por defecto debe marcarse como incompleta"
        );
        assert!(
            audit
                .missing_items
                .iter()
                .any(|m| m.contains("Firma Electrónica"))
        );
        assert!(audit.missing_items.iter().any(|m| m.contains("PIN")));
    }

    #[test]
    fn test_auditoria_detecta_config_completa() {
        // Crear un archivo temporal simulando el .p12
        let temp_dir = std::env::temp_dir();
        let p12_dummy = temp_dir.join("dummy_firma_test.p12");
        let _ = std::fs::write(&p12_dummy, b"dummy content");

        let config = EmisorConfig {
            ruc: "1792345678001".to_string(),
            razon_social: "CORPORACION CHOCOLATERA CIA. LTDA.".to_string(),
            nombre_comercial: Some("CACAO CORP".to_string()),
            dir_matriz: "Av. Amazonas y Colón, Quito".to_string(),
            dir_establecimiento: "Av. Amazonas y Colón, Quito".to_string(),
            cod_establecimiento: "001".to_string(),
            pto_emision: "002".to_string(),
            obligado_contabilidad: "SI".to_string(),
            contribuyente_especial: None,
            regimen_microempresas: None,
            regimen_rimpe: Some("CONTRIBUYENTE RÉGIMEN RIMPE".to_string()),
            ambiente: "1".to_string(),
            p12_path: Some(p12_dummy.display().to_string()),
            p12_password: Some("clave_segura123".to_string()),
            pin: Some("5678".to_string()),
            logo_path: None,
            plantilla_pdf: "clasica".to_string(),
        };

        let audit = auditar_configuracion(&config);
        let _ = std::fs::remove_file(&p12_dummy);

        assert!(
            audit.is_complete,
            "La configuración completa debe dar is_complete = true"
        );
        assert_eq!(audit.missing_items.len(), 0);
        assert_eq!(audit.completed_count, audit.total_count);
    }
}
