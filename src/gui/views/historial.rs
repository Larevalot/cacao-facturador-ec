//! Vista del Historial de Facturas Emitidas y visor de detalles en Iced.

use crate::db::facturas::FacturaGuardada;
use crate::gui::icons::{
    icon_ban, icon_copy, icon_download, icon_eye, icon_file_code, icon_file_pdf,
};
use crate::gui::theme::{
    ThemeMode, card_style, danger_button_style, primary_button_style, secondary_button_style,
};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Background, Border, Color, Element, Length};

#[derive(Debug, Clone)]
pub struct PagoExtraido {
    pub forma_pago: String,
    pub descripcion: String,
    pub total: f64,
    pub plazo: Option<String>,
    pub unidad_tiempo: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct HistorialState {
    pub facturas: Vec<FacturaGuardada>,
    pub factura_seleccionada: Option<FacturaGuardada>,
    pub copied_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum HistorialMessage {
    VerDetalle(FacturaGuardada),
    CerrarDetalle,
    CopiarClave(String),
    AnularEnSri(String),
    DescargarPdf(FacturaGuardada),
    DescargarXml(FacturaGuardada),
    DescargarAmbos(FacturaGuardada),
}

fn descripcion_forma_pago(codigo: &str) -> &'static str {
    match codigo {
        "01" => "SIN UTILIZACIÓN DEL SISTEMA FINANCIERO",
        "15" => "COMPENSACIÓN DE DEUDAS",
        "16" => "TARJETA DE DÉBITO",
        "17" => "DINERO ELECTRÓNICO",
        "18" => "TARJETA PREPAGO",
        "19" => "TARJETA DE CRÉDITO",
        "20" => "OTROS CON SISTEMA FINANCIERO",
        "21" => "ENDOSO DE TÍTULOS",
        _ => "OTROS",
    }
}

pub fn extraer_pagos_de_xml(xml: &str) -> Vec<PagoExtraido> {
    let mut pagos = Vec::new();
    let mut rest = xml;

    while let Some(start) = rest.find("<pago>") {
        rest = &rest[start + 6..];
        let end = match rest.find("</pago>") {
            Some(e) => e,
            None => break,
        };
        let pago_str = &rest[..end];
        rest = &rest[end + 7..];

        let forma_pago = extraer_tag(pago_str, "formaPago").unwrap_or_else(|| "01".to_string());
        let total = extraer_tag(pago_str, "total")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);
        let plazo = extraer_tag(pago_str, "plazo");
        let unidad_tiempo = extraer_tag(pago_str, "unidadTiempo");

        let desc = descripcion_forma_pago(&forma_pago).to_string();

        pagos.push(PagoExtraido {
            forma_pago,
            descripcion: desc,
            total,
            plazo,
            unidad_tiempo,
        });
    }

    pagos
}

fn extraer_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].trim().to_string())
}

impl HistorialState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, message: HistorialMessage) -> Option<String> {
        match message {
            HistorialMessage::VerDetalle(f) => {
                self.factura_seleccionada = Some(f);
                None
            }
            HistorialMessage::CerrarDetalle => {
                self.factura_seleccionada = None;
                None
            }
            HistorialMessage::CopiarClave(clave) => {
                self.copied_id = Some(clave.clone());
                Some(clave)
            }
            HistorialMessage::AnularEnSri(clave) => {
                let _ = open::that("https://srienlinea.sri.gob.ec");
                self.copied_id = Some(clave.clone());
                Some(clave)
            }
            HistorialMessage::DescargarPdf(_) => None,
            HistorialMessage::DescargarXml(_) => None,
            HistorialMessage::DescargarAmbos(_) => None,
        }
    }

    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        mode: ThemeMode,
        on_message: fn(HistorialMessage) -> Message,
    ) -> Element<'a, Message> {
        let p = mode.palette();

        let header = column![
            text("Historial de Facturas Emitidas")
                .size(20)
                .color(p.text_main),
            text("Consulta tus facturas enviadas al SRI, estado tributario y claves de acceso")
                .size(13)
                .color(p.text_muted),
        ]
        .spacing(4);

        let table_header = container(
            row![
                text("FECHA").size(12).color(p.text_muted).width(95),
                text("SECUENCIAL").size(12).color(p.text_muted).width(110),
                text("CLIENTE")
                    .size(12)
                    .color(p.text_muted)
                    .width(Length::Fill),
                text("IDENTIFICACIÓN")
                    .size(12)
                    .color(p.text_muted)
                    .width(130),
                text("TOTAL ($)").size(12).color(p.text_muted).width(90),
                text("ESTADO SRI").size(12).color(p.text_muted).width(120),
                text("ACCIONES").size(12).color(p.text_muted).width(250),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([8, 12])
        .style(move |_| container::Style {
            background: Some(Background::Color(p.bg_card)),
            border: Border {
                color: p.border,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        });

        let mut rows_col = column![].spacing(6);

        if self.facturas.is_empty() {
            rows_col = rows_col.push(
                container(
                    text("No se han emitido facturas aún. Utiliza el módulo Facturador para crear la primera.")
                        .size(13)
                        .color(p.text_muted),
                )
                .padding(24)
                .center_x(Length::Fill),
            );
        } else {
            for f in &self.facturas {
                let f_clone = f.clone();
                let clave = f.clave_acceso.clone();
                let is_copied = self.copied_id.as_deref() == Some(&clave);

                let status_color = if f.estado == "AUTORIZADO" {
                    p.success
                } else if f.estado.contains("RECIBIDA") || f.estado.contains("PROCESAMIENTO") {
                    p.warning
                } else {
                    p.danger
                };

                let row_item = container(
                    row![
                        text(&f.fecha_emision).size(13).color(p.text_main).width(95),
                        text(&f.secuencial).size(13).color(p.accent).width(110),
                        text(&f.cliente_razon)
                            .size(13)
                            .color(p.text_main)
                            .width(Length::Fill),
                        text(&f.cliente_identificacion)
                            .size(13)
                            .color(p.text_muted)
                            .width(130),
                        text(format!("${:.2}", f.importe_total))
                            .size(13)
                            .color(p.accent)
                            .width(90),
                        container(text(&f.estado).size(11).color(status_color))
                            .padding([2, 6])
                            .width(120),
                        row![
                            button(
                                row![icon_eye(p.text_main, 12.0), text("Detalle").size(11),]
                                    .spacing(4)
                                    .align_y(Alignment::Center),
                            )
                            .style(secondary_button_style(mode))
                            .padding([4, 8])
                            .on_press(on_message(HistorialMessage::VerDetalle(f_clone.clone()))),
                            button(
                                row![icon_file_pdf(p.text_main, 12.0), text("PDF").size(11),]
                                    .spacing(4)
                                    .align_y(Alignment::Center),
                            )
                            .style(secondary_button_style(mode))
                            .padding([4, 8])
                            .on_press(on_message(HistorialMessage::DescargarPdf(f_clone))),
                            button(
                                row![
                                    icon_copy(p.text_main, 12.0),
                                    text(if is_copied { "Copiado" } else { "Copiar N°" }).size(11),
                                ]
                                .spacing(4)
                                .align_y(Alignment::Center),
                            )
                            .style(secondary_button_style(mode))
                            .padding([4, 8])
                            .on_press(on_message(HistorialMessage::CopiarClave(clave.clone()))),
                            button(icon_ban(p.danger, 12.0))
                                .style(danger_button_style(mode))
                                .padding([4, 6])
                                .on_press(on_message(HistorialMessage::AnularEnSri(clave))),
                        ]
                        .spacing(4)
                        .align_y(Alignment::Center)
                        .width(250),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .padding([8, 12])
                .style(move |_| container::Style {
                    background: Some(Background::Color(p.bg_card)),
                    border: Border {
                        color: p.border,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                });

                rows_col = rows_col.push(row_item);
            }
        }

        let main_view = column![
            header,
            table_header,
            scrollable(rows_col).height(Length::Fill),
        ]
        .spacing(14);

        // Modal de detalle de factura si está seleccionado
        if let Some(factura) = &self.factura_seleccionada {
            let clave = factura.clave_acceso.clone();
            let xml = factura.xml_firmado.as_deref().unwrap_or("");
            let pagos = extraer_pagos_de_xml(xml);

            let mut pagos_col = column![].spacing(6);
            if pagos.is_empty() {
                pagos_col = pagos_col.push(
                    text("Sin formas de pago registradas")
                        .size(12)
                        .color(p.text_muted),
                );
            } else {
                for pago in pagos {
                    let plazo_str = if let Some(plazo) = pago.plazo {
                        let ut = pago
                            .unidad_tiempo
                            .unwrap_or_else(|| "días".to_string())
                            .to_uppercase();
                        format!("Plazo: {} {}", plazo, ut)
                    } else {
                        "Al contado (Sin plazo)".to_string()
                    };

                    pagos_col = pagos_col.push(
                        container(
                            row![
                                text(format!("{} - {}", pago.forma_pago, pago.descripcion))
                                    .size(12)
                                    .color(p.text_main)
                                    .width(Length::Fill),
                                text(format!("${:.2}", pago.total))
                                    .size(12)
                                    .color(p.accent)
                                    .width(90),
                                text(plazo_str).size(12).color(p.text_muted).width(150),
                            ]
                            .align_y(Alignment::Center),
                        )
                        .padding([4, 8]),
                    );
                }
            }

            let modal_content = column![
                row![
                    text(format!("Factura #{}", factura.secuencial))
                        .size(17)
                        .color(p.text_main),
                    row![].width(Length::Fill),
                    button(text("✕").size(14))
                        .style(secondary_button_style(mode))
                        .padding([4, 8])
                        .on_press(on_message(HistorialMessage::CerrarDetalle)),
                ]
                .align_y(Alignment::Center),
                container(
                    column![
                        row![
                            text(format!("Fecha: {}", factura.fecha_emision))
                                .size(12)
                                .color(p.text_main)
                                .width(Length::FillPortion(1)),
                            text(format!("Estado: {}", factura.estado))
                                .size(12)
                                .color(p.accent)
                                .width(Length::FillPortion(1)),
                        ],
                        row![
                            text(format!("Cliente: {}", factura.cliente_razon))
                                .size(12)
                                .color(p.text_main)
                                .width(Length::FillPortion(1)),
                            text(format!("RUC/Cédula: {}", factura.cliente_identificacion))
                                .size(12)
                                .color(p.text_muted)
                                .width(Length::FillPortion(1)),
                        ],
                        row![
                            text(format!(
                                "Subtotal Sin IVA: ${:.2}",
                                factura.total_sin_impuestos
                            ))
                            .size(12)
                            .color(p.text_main)
                            .width(Length::FillPortion(1)),
                            text(format!("IVA: ${:.2}", factura.total_iva))
                                .size(12)
                                .color(p.text_main)
                                .width(Length::FillPortion(1)),
                        ],
                        text(format!("TOTAL FACTURA: ${:.2}", factura.importe_total))
                            .size(14)
                            .color(p.accent),
                    ]
                    .spacing(6),
                )
                .padding(12)
                .style(move |_| container::Style {
                    background: Some(Background::Color(p.bg_input)),
                    border: Border {
                        color: p.border,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }),
                text("FORMAS DE PAGO, PLAZOS Y TIEMPO (SRI)")
                    .size(12)
                    .color(p.text_muted),
                pagos_col,
                container(
                    column![
                        text("Clave de Acceso (49 dígitos):")
                            .size(11)
                            .color(p.text_muted),
                        text(&factura.clave_acceso).size(11).color(p.accent),
                    ]
                    .spacing(2),
                )
                .padding(8)
                .style(move |_| container::Style {
                    background: Some(Background::Color(p.bg_input)),
                    border: Border {
                        color: p.border,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }),
                row![
                    button(
                        row![
                            icon_file_pdf(Color::WHITE, 13.0),
                            text("Descargar PDF").size(12),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    )
                    .style(primary_button_style(mode))
                    .padding([6, 12])
                    .on_press(on_message(HistorialMessage::DescargarPdf(factura.clone()))),
                    button(
                        row![
                            icon_file_code(p.text_main, 13.0),
                            text("Descargar XML").size(12),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    )
                    .style(secondary_button_style(mode))
                    .padding([6, 12])
                    .on_press(on_message(HistorialMessage::DescargarXml(factura.clone()))),
                    button(
                        row![
                            icon_download(p.text_main, 13.0),
                            text("Descargar Ambos").size(12),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    )
                    .style(secondary_button_style(mode))
                    .padding([6, 12])
                    .on_press(on_message(HistorialMessage::DescargarAmbos(
                        factura.clone()
                    ))),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    button(
                        row![icon_copy(p.text_main, 12.0), text("Copiar Clave").size(12),]
                            .spacing(4)
                            .align_y(Alignment::Center),
                    )
                    .style(secondary_button_style(mode))
                    .padding([6, 14])
                    .on_press(on_message(HistorialMessage::CopiarClave(clave))),
                    row![].width(Length::Fill),
                    button(text("Cerrar").size(12))
                        .style(primary_button_style(mode))
                        .padding([6, 16])
                        .on_press(on_message(HistorialMessage::CerrarDetalle)),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(14);

            let modal_box = container(modal_content)
                .width(550)
                .padding(24)
                .style(card_style(mode));

            container(modal_box)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        } else {
            container(main_view)
                .padding(24)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }
}
