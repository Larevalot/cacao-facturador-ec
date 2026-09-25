//! Vista del Facturador Electrónico SRI con Formas de Pago, Plazos y Totales en Iced.

use crate::db::clientes::Cliente;
use crate::db::productos::Producto;
use crate::gui::icons::{
    icon_alert_triangle, icon_check, icon_copy, icon_credit_card, icon_download, icon_file_code,
    icon_file_pdf, icon_plus, icon_send, icon_user, icon_zap,
};
use crate::gui::theme::{
    ThemeMode, card_style, danger_button_style, input_container_style, pick_list_style,
    primary_button_style, secondary_button_style, text_input_style,
};
use crate::sri::models::{ClienteInfo, DetalleFactura, FacturaRequest, FormaPago, RespuestaSRI};
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoIdentificacion {
    Cedula,
    Ruc,
    Pasaporte,
    ConsumidorFinal,
}

impl TipoIdentificacion {
    pub fn codigo(&self) -> &'static str {
        match self {
            TipoIdentificacion::Cedula => "05",
            TipoIdentificacion::Ruc => "04",
            TipoIdentificacion::Pasaporte => "06",
            TipoIdentificacion::ConsumidorFinal => "07",
        }
    }
}

impl std::fmt::Display for TipoIdentificacion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TipoIdentificacion::Cedula => write!(f, "05 - Cédula"),
            TipoIdentificacion::Ruc => write!(f, "04 - RUC"),
            TipoIdentificacion::Pasaporte => write!(f, "06 - Pasaporte"),
            TipoIdentificacion::ConsumidorFinal => write!(f, "07 - Consumidor Final"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormaPagoCodigo {
    SinSistemaFinanciero,
    CompensacionDeudas,
    TarjetaDebito,
    DineroElectronico,
    TarjetaPrepago,
    TarjetaCredito,
    OtrosSistemaFinanciero,
    EndosoTitulos,
}

impl FormaPagoCodigo {
    pub fn codigo(&self) -> &'static str {
        match self {
            FormaPagoCodigo::SinSistemaFinanciero => "01",
            FormaPagoCodigo::CompensacionDeudas => "15",
            FormaPagoCodigo::TarjetaDebito => "16",
            FormaPagoCodigo::DineroElectronico => "17",
            FormaPagoCodigo::TarjetaPrepago => "18",
            FormaPagoCodigo::TarjetaCredito => "19",
            FormaPagoCodigo::OtrosSistemaFinanciero => "20",
            FormaPagoCodigo::EndosoTitulos => "21",
        }
    }
}

impl std::fmt::Display for FormaPagoCodigo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormaPagoCodigo::SinSistemaFinanciero => {
                write!(f, "01 - SIN UTILIZACION DEL SISTEMA FINANCIERO")
            }
            FormaPagoCodigo::CompensacionDeudas => write!(f, "15 - COMPENSACIÓN DE DEUDAS"),
            FormaPagoCodigo::TarjetaDebito => write!(f, "16 - TARJETA DE DÉBITO"),
            FormaPagoCodigo::DineroElectronico => write!(f, "17 - DINERO ELECTRÓNICO"),
            FormaPagoCodigo::TarjetaPrepago => write!(f, "18 - TARJETA PREPAGO"),
            FormaPagoCodigo::TarjetaCredito => write!(f, "19 - TARJETA DE CRÉDITO"),
            FormaPagoCodigo::OtrosSistemaFinanciero => {
                write!(f, "20 - OTROS CON UTILIZACION DEL SISTEMA FINANCIERO")
            }
            FormaPagoCodigo::EndosoTitulos => write!(f, "21 - ENDOSO DE TÍTULOS"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnidadTiempoOption {
    Dias,
    Meses,
    Anios,
}

impl UnidadTiempoOption {
    pub fn codigo(&self) -> &'static str {
        match self {
            UnidadTiempoOption::Dias => "dias",
            UnidadTiempoOption::Meses => "meses",
            UnidadTiempoOption::Anios => "anios",
        }
    }
}

impl std::fmt::Display for UnidadTiempoOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnidadTiempoOption::Dias => write!(f, "Días"),
            UnidadTiempoOption::Meses => write!(f, "Meses"),
            UnidadTiempoOption::Anios => write!(f, "Años"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IvaOption {
    Iva15,
    Iva0,
}

impl std::fmt::Display for IvaOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IvaOption::Iva15 => write!(f, "15%"),
            IvaOption::Iva0 => write!(f, "0%"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FacturaItemRow {
    pub codigo: String,
    pub descripcion: String,
    pub cantidad: String,
    pub precio: String,
    pub descuento: String,
    pub iva: Option<IvaOption>,
}

#[derive(Debug, Clone)]
pub struct FormaPagoRow {
    pub forma_pago: Option<FormaPagoCodigo>,
    pub valor: String,
    pub plazo: String,
    pub tiempo: Option<UnidadTiempoOption>,
}

#[derive(Debug, Clone)]
pub struct FacturadorState {
    pub secuencial: String,
    pub fecha_emision: String,
    pub tipo_id: Option<TipoIdentificacion>,
    pub identificacion: String,
    pub razon_social: String,
    pub email: String,
    pub telefono: String,
    pub direccion: String,
    pub guia_remision: String,
    pub cliente_encontrado: bool,
    pub items: Vec<FacturaItemRow>,
    pub formas_pago: Vec<FormaPagoRow>,
    pub password_p12: String,
    pub is_emitiendo: bool,
    pub respuesta_sri: Option<RespuestaSRI>,
    pub ultimo_request: Option<FacturaRequest>,
}

#[derive(Debug, Clone)]
pub enum FacturadorMessage {
    SecuencialChanged(String),
    FechaChanged(String),
    TipoIdChanged(TipoIdentificacion),
    IdentificacionChanged(String),
    RazonSocialChanged(String),
    EmailChanged(String),
    TelefonoChanged(String),
    DireccionChanged(String),
    GuiaRemisionChanged(String),
    SetConsumidorFinal,
    // Items
    AddItem,
    RemoveItem(usize),
    SelectProducto(usize, Producto),
    ItemCodigoChanged(usize, String),
    ItemDescripcionChanged(usize, String),
    ItemCantidadChanged(usize, String),
    ItemPrecioChanged(usize, String),
    ItemDescuentoChanged(usize, String),
    ItemIvaChanged(usize, IvaOption),
    // Formas de pago
    AddFormaPago,
    RemoveFormaPago(usize),
    PagoCodigoChanged(usize, FormaPagoCodigo),
    PagoValorChanged(usize, String),
    PagoPlazoChanged(usize, String),
    PagoTiempoChanged(usize, UnidadTiempoOption),
    AjustarAlTotal,
    // Password y emisión
    PasswordChanged(String),
    EmitirFactura,
    CopiarClave(String),
    // Exportación
    GuardarPdf,
    GuardarXml,
    GuardarAmbos,
}

impl Default for FacturadorState {
    fn default() -> Self {
        let now = chrono::Local::now();
        let fecha = now.format("%d/%m/%Y").to_string();

        Self {
            secuencial: "000000001".to_string(),
            fecha_emision: fecha,
            tipo_id: Some(TipoIdentificacion::Cedula),
            identificacion: String::new(),
            razon_social: String::new(),
            email: String::new(),
            telefono: String::new(),
            direccion: String::new(),
            guia_remision: String::new(),
            cliente_encontrado: false,
            items: vec![FacturaItemRow {
                codigo: "PRD-001".to_string(),
                descripcion: "PRODUCTO GENERAL".to_string(),
                cantidad: "1".to_string(),
                precio: "0.00".to_string(),
                descuento: "0.00".to_string(),
                iva: Some(IvaOption::Iva15),
            }],
            formas_pago: vec![FormaPagoRow {
                forma_pago: Some(FormaPagoCodigo::SinSistemaFinanciero),
                valor: "0.00".to_string(),
                plazo: String::new(),
                tiempo: Some(UnidadTiempoOption::Dias),
            }],
            password_p12: String::new(),
            is_emitiendo: false,
            respuesta_sri: None,
            ultimo_request: None,
        }
    }
}

impl FacturadorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calcular_totales(&self) -> (f64, f64, f64, f64) {
        let mut subtotal15 = 0.0;
        let mut subtotal0 = 0.0;

        for item in &self.items {
            let cant = item.cantidad.parse::<f64>().unwrap_or(0.0);
            let precio = item.precio.parse::<f64>().unwrap_or(0.0);
            let desc = item.descuento.parse::<f64>().unwrap_or(0.0);
            let sub = ((cant * precio) - desc).max(0.0);

            if item.iva == Some(IvaOption::Iva15) {
                subtotal15 += sub;
            } else {
                subtotal0 += sub;
            }
        }

        let monto_iva = (subtotal15 * 0.15 * 100.0).round() / 100.0;
        let total = ((subtotal15 + subtotal0 + monto_iva) * 100.0).round() / 100.0;

        (subtotal15, subtotal0, monto_iva, total)
    }

    pub fn calcular_total_pagos(&self) -> f64 {
        self.formas_pago
            .iter()
            .map(|p| p.valor.parse::<f64>().unwrap_or(0.0))
            .sum()
    }

    pub fn auto_ajustar_si_unico_pago(&mut self) {
        let (_, _, _, total) = self.calcular_totales();
        if self.formas_pago.len() == 1 {
            self.formas_pago[0].valor = format!("{:.2}", total);
        }
    }

    pub fn autocompletar_cliente(&mut self, cliente: Option<Cliente>) {
        if let Some(c) = cliente {
            self.tipo_id = match c.tipo_identificacion.as_str() {
                "04" => Some(TipoIdentificacion::Ruc),
                "06" => Some(TipoIdentificacion::Pasaporte),
                "07" => Some(TipoIdentificacion::ConsumidorFinal),
                _ => Some(TipoIdentificacion::Cedula),
            };
            self.razon_social = c.razon_social;
            if let Some(d) = c.direccion {
                self.direccion = d;
            }
            if let Some(e) = c.email {
                self.email = e;
            }
            if let Some(t) = c.telefono {
                self.telefono = t;
            }
            self.cliente_encontrado = true;
        } else {
            self.cliente_encontrado = false;
        }
    }

    pub fn update(&mut self, message: FacturadorMessage) {
        match message {
            FacturadorMessage::SecuencialChanged(val) => self.secuencial = val,
            FacturadorMessage::FechaChanged(val) => self.fecha_emision = val,
            FacturadorMessage::TipoIdChanged(t) => {
                self.tipo_id = Some(t);
                if t == TipoIdentificacion::ConsumidorFinal {
                    self.identificacion = "9999999999999".to_string();
                    self.razon_social = "CONSUMIDOR FINAL".to_string();
                    self.direccion = "S/N".to_string();
                }
            }
            FacturadorMessage::IdentificacionChanged(val) => {
                self.identificacion = val;
            }
            FacturadorMessage::RazonSocialChanged(val) => self.razon_social = val,
            FacturadorMessage::EmailChanged(val) => self.email = val,
            FacturadorMessage::TelefonoChanged(val) => self.telefono = val,
            FacturadorMessage::DireccionChanged(val) => self.direccion = val,
            FacturadorMessage::GuiaRemisionChanged(val) => self.guia_remision = val,
            FacturadorMessage::SetConsumidorFinal => {
                self.tipo_id = Some(TipoIdentificacion::ConsumidorFinal);
                self.identificacion = "9999999999999".to_string();
                self.razon_social = "CONSUMIDOR FINAL".to_string();
                self.direccion = "S/N".to_string();
                self.email.clear();
                self.telefono.clear();
                self.cliente_encontrado = false;
            }
            FacturadorMessage::AddItem => {
                self.items.push(FacturaItemRow {
                    codigo: String::new(),
                    descripcion: String::new(),
                    cantidad: "1".to_string(),
                    precio: "0.00".to_string(),
                    descuento: "0.00".to_string(),
                    iva: Some(IvaOption::Iva15),
                });
                self.auto_ajustar_si_unico_pago();
            }
            FacturadorMessage::RemoveItem(idx) => {
                if self.items.len() > 1 {
                    self.items.remove(idx);
                    self.auto_ajustar_si_unico_pago();
                }
            }
            FacturadorMessage::SelectProducto(idx, prod) => {
                if let Some(item) = self.items.get_mut(idx) {
                    item.codigo = prod.codigo;
                    item.descripcion = prod.descripcion;
                    item.precio = format!("{:.2}", prod.precio_unitario);
                    item.descuento = "0.00".to_string();
                    item.iva = if prod.codigo_iva == "0" {
                        Some(IvaOption::Iva0)
                    } else {
                        Some(IvaOption::Iva15)
                    };
                }
                self.auto_ajustar_si_unico_pago();
            }
            FacturadorMessage::ItemCodigoChanged(idx, val) => {
                if let Some(item) = self.items.get_mut(idx) {
                    item.codigo = val;
                }
            }
            FacturadorMessage::ItemDescripcionChanged(idx, val) => {
                if let Some(item) = self.items.get_mut(idx) {
                    item.descripcion = val;
                }
            }
            FacturadorMessage::ItemCantidadChanged(idx, val) => {
                if let Some(item) = self.items.get_mut(idx) {
                    item.cantidad = val;
                }
                self.auto_ajustar_si_unico_pago();
            }
            FacturadorMessage::ItemPrecioChanged(idx, val) => {
                if let Some(item) = self.items.get_mut(idx) {
                    item.precio = val;
                }
                self.auto_ajustar_si_unico_pago();
            }
            FacturadorMessage::ItemDescuentoChanged(idx, val) => {
                if let Some(item) = self.items.get_mut(idx) {
                    item.descuento = val;
                }
                self.auto_ajustar_si_unico_pago();
            }
            FacturadorMessage::ItemIvaChanged(idx, iva) => {
                if let Some(item) = self.items.get_mut(idx) {
                    item.iva = Some(iva);
                }
                self.auto_ajustar_si_unico_pago();
            }
            FacturadorMessage::AddFormaPago => {
                let (_, _, _, total) = self.calcular_totales();
                let actual = self.calcular_total_pagos();
                let diff = (total - actual).max(0.0);

                self.formas_pago.push(FormaPagoRow {
                    forma_pago: Some(FormaPagoCodigo::SinSistemaFinanciero),
                    valor: format!("{:.2}", diff),
                    plazo: String::new(),
                    tiempo: Some(UnidadTiempoOption::Dias),
                });
            }
            FacturadorMessage::RemoveFormaPago(idx) => {
                if self.formas_pago.len() > 1 {
                    self.formas_pago.remove(idx);
                }
            }
            FacturadorMessage::PagoCodigoChanged(idx, codigo) => {
                if let Some(p) = self.formas_pago.get_mut(idx) {
                    p.forma_pago = Some(codigo);
                }
            }
            FacturadorMessage::PagoValorChanged(idx, val) => {
                if let Some(p) = self.formas_pago.get_mut(idx) {
                    p.valor = val;
                }
            }
            FacturadorMessage::PagoPlazoChanged(idx, val) => {
                if let Some(p) = self.formas_pago.get_mut(idx) {
                    p.plazo = val;
                }
            }
            FacturadorMessage::PagoTiempoChanged(idx, ut) => {
                if let Some(p) = self.formas_pago.get_mut(idx) {
                    p.tiempo = Some(ut);
                }
            }
            FacturadorMessage::AjustarAlTotal => {
                let (_, _, _, total) = self.calcular_totales();
                if self.formas_pago.len() == 1 {
                    self.formas_pago[0].valor = format!("{:.2}", total);
                } else if !self.formas_pago.is_empty() {
                    let ult_idx = self.formas_pago.len() - 1;
                    let anterior: f64 = self.formas_pago[..ult_idx]
                        .iter()
                        .map(|p| p.valor.parse::<f64>().unwrap_or(0.0))
                        .sum();
                    let diff = (total - anterior).max(0.0);
                    self.formas_pago[ult_idx].valor = format!("{:.2}", diff);
                }
            }
            FacturadorMessage::PasswordChanged(val) => self.password_p12 = val,
            FacturadorMessage::EmitirFactura => {
                self.is_emitiendo = true;
                self.respuesta_sri = None;
            }
            FacturadorMessage::CopiarClave(_) => {}
            FacturadorMessage::GuardarPdf => {}
            FacturadorMessage::GuardarXml => {}
            FacturadorMessage::GuardarAmbos => {}
        }
    }

    pub fn construir_factura_request(&self) -> Result<FacturaRequest, String> {
        let tipo_id_val = self.tipo_id.unwrap_or(TipoIdentificacion::Cedula);
        let final_tipo_id = tipo_id_val.codigo().to_string();
        let mut final_id = self.identificacion.trim().to_string();
        let mut final_razon = self.razon_social.trim().to_string();

        if tipo_id_val == TipoIdentificacion::ConsumidorFinal || final_id == "9999999999999" {
            final_id = "9999999999999".to_string();
            if final_razon.is_empty() {
                final_razon = "CONSUMIDOR FINAL".to_string();
            }
        } else {
            if final_id.is_empty() {
                return Err("Debe ingresar la identificación (Cédula/RUC) del cliente".to_string());
            }
            if final_razon.is_empty() {
                return Err("Debe ingresar la razón social o nombres del cliente".to_string());
            }
        }

        let direccion = if self.direccion.trim().is_empty() {
            None
        } else {
            Some(self.direccion.trim().to_string())
        };

        let cliente = ClienteInfo {
            tipo_identificacion: final_tipo_id,
            identificacion: final_id,
            razon_social: final_razon,
            direccion,
            email: if self.email.trim().is_empty() {
                None
            } else {
                Some(self.email.trim().to_string())
            },
            telefono: if self.telefono.trim().is_empty() {
                None
            } else {
                Some(self.telefono.trim().to_string())
            },
        };

        let mut detalles = Vec::new();
        for item in &self.items {
            let cant = item
                .cantidad
                .parse::<f64>()
                .map_err(|_| "Cantidad inválida en producto".to_string())?;
            let precio = item
                .precio
                .parse::<f64>()
                .map_err(|_| "Precio unitario inválido".to_string())?;
            let desc = item.descuento.parse::<f64>().unwrap_or(0.0);
            let iva_opt = item.iva.unwrap_or(IvaOption::Iva15);

            detalles.push(DetalleFactura {
                codigo_principal: if item.codigo.trim().is_empty() {
                    "PRD-001".to_string()
                } else {
                    item.codigo.trim().to_string()
                },
                codigo_auxiliar: None,
                descripcion: if item.descripcion.trim().is_empty() {
                    "PRODUCTO GENERAL".to_string()
                } else {
                    item.descripcion.trim().to_string()
                },
                cantidad: cant,
                precio_unitario: (precio * 100.0).round() / 100.0,
                descuento: (desc * 100.0).round() / 100.0,
                codigo_porcentaje_iva: if iva_opt == IvaOption::Iva0 {
                    "0".to_string()
                } else {
                    "4".to_string()
                },
                tarifa_iva: if iva_opt == IvaOption::Iva0 {
                    0.0
                } else {
                    15.0
                },
            });
        }

        let mut formas_pago = Vec::new();
        for fp in &self.formas_pago {
            let total = fp
                .valor
                .parse::<f64>()
                .map_err(|_| "Valor de forma de pago inválido".to_string())?;
            let plazo = fp.plazo.parse::<u32>().ok().filter(|&p| p > 0);
            let unidad_tiempo = if plazo.is_some() {
                fp.tiempo.map(|ut| ut.codigo().to_string())
            } else {
                None
            };

            formas_pago.push(FormaPago {
                forma_pago: fp
                    .forma_pago
                    .unwrap_or(FormaPagoCodigo::SinSistemaFinanciero)
                    .codigo()
                    .to_string(),
                total: (total * 100.0).round() / 100.0,
                plazo,
                unidad_tiempo,
            });
        }

        let guia_remision = if self.guia_remision.trim().is_empty() {
            None
        } else {
            Some(self.guia_remision.trim().to_string())
        };

        Ok(FacturaRequest {
            secuencial: format!("{:0>9}", self.secuencial.trim()),
            fecha_emision: self.fecha_emision.trim().to_string(),
            cliente,
            detalles,
            formas_pago,
            propina: 0.0,
            guia_remision,
        })
    }

    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        mode: ThemeMode,
        _productos_catalogo: &'a [Producto],
        on_message: fn(FacturadorMessage) -> Message,
    ) -> Element<'a, Message> {
        let p = mode.palette();
        let (sub15, sub0, iva, total) = self.calcular_totales();
        let total_pagos = self.calcular_total_pagos();
        let diff_pagos = (total - total_pagos).abs();

        let header = column![
            text("Emitir Factura Electrónica SRI")
                .size(20)
                .color(p.text_main),
            text(
                "Genera comprobantes autorizados con firma digital XAdES-BES y envío directo al SRI"
            )
            .size(13)
            .color(p.text_muted),
        ]
        .spacing(4);

        // Fila 1: Secuencial y Fecha
        let row_emision = row![
            column![
                text("Secuencial (9 dígitos):").size(12).color(p.text_muted),
                text_input("000000001", &self.secuencial)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(FacturadorMessage::SecuencialChanged(v)))
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text("Fecha Emisión (DD/MM/AAAA):")
                    .size(12)
                    .color(p.text_muted),
                text_input("DD/MM/AAAA", &self.fecha_emision)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(FacturadorMessage::FechaChanged(v)))
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12);

        // Datos del Cliente
        let client_header = row![
            text("DATOS DEL CLIENTE").size(13).color(p.accent),
            row![].width(Length::Fill),
            button(
                row![
                    icon_user(p.text_main, 14.0),
                    text("Consumidor Final").size(12),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .style(secondary_button_style(mode))
            .padding([4, 10])
            .on_press(on_message(FacturadorMessage::SetConsumidorFinal)),
        ]
        .align_y(Alignment::Center);

        let tipos_id = vec![
            TipoIdentificacion::Cedula,
            TipoIdentificacion::Ruc,
            TipoIdentificacion::Pasaporte,
            TipoIdentificacion::ConsumidorFinal,
        ];

        let row_cliente1 = row![
            column![
                text("Tipo Identificación:").size(12).color(p.text_muted),
                pick_list(tipos_id, self.tipo_id, move |v| on_message(
                    FacturadorMessage::TipoIdChanged(v)
                ))
                .style(pick_list_style(mode))
                .width(Length::Fill)
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                row![
                    text("Identificación (Cédula/RUC):")
                        .size(12)
                        .color(p.text_muted),
                    if self.cliente_encontrado {
                        row![
                            icon_check(p.success, 12.0),
                            text("Cliente Frecuente").size(11).color(p.success),
                        ]
                        .spacing(4)
                        .align_y(Alignment::Center)
                    } else {
                        row![].spacing(0)
                    }
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text_input("1712345678 o 9999999999999", &self.identificacion)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(FacturadorMessage::IdentificacionChanged(v)))
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12);

        let row_cliente2 = column![
            text("Nombres / Razón Social:").size(12).color(p.text_muted),
            text_input("CONSUMIDOR FINAL u Nombre del cliente", &self.razon_social)
                .style(text_input_style(mode))
                .on_input(move |v| on_message(FacturadorMessage::RazonSocialChanged(v))),
        ]
        .spacing(4);

        let row_cliente3 = row![
            column![
                text("Email (Opcional):").size(12).color(p.text_muted),
                text_input("cliente@ejemplo.com", &self.email)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(FacturadorMessage::EmailChanged(v)))
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text("Teléfono (Opcional):").size(12).color(p.text_muted),
                text_input("0991234567", &self.telefono)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(FacturadorMessage::TelefonoChanged(v)))
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12);

        let row_cliente4 = row![
            column![
                text("Dirección (Opcional):").size(12).color(p.text_muted),
                text_input("Av. Amazonas y Colón", &self.direccion)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(FacturadorMessage::DireccionChanged(v)))
            ]
            .spacing(4)
            .width(Length::FillPortion(2)),
            column![
                text("Guía de Remisión (Opcional):")
                    .size(12)
                    .color(p.text_muted),
                text_input("001-001-000000001", &self.guia_remision)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(FacturadorMessage::GuiaRemisionChanged(v)))
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12);

        // Ítems
        let items_header = row![
            text("ÍTEMS DE LA FACTURA").size(13).color(p.accent),
            row![].width(Length::Fill),
            button(
                row![icon_plus(p.text_main, 12.0), text("Agregar Ítem").size(12),]
                    .spacing(4)
                    .align_y(Alignment::Center),
            )
            .style(secondary_button_style(mode))
            .padding([4, 10])
            .on_press(on_message(FacturadorMessage::AddItem)),
        ]
        .align_y(Alignment::Center);

        let items_table_headers = row![
            text("CÓDIGO").size(11).color(p.text_muted).width(85),
            text("DESCRIPCIÓN")
                .size(11)
                .color(p.text_muted)
                .width(Length::Fill),
            text("CANT.").size(11).color(p.text_muted).width(65),
            text("P.UNIT ($)").size(11).color(p.text_muted).width(75),
            text("DESC ($)").size(11).color(p.text_muted).width(70),
            text("IVA").size(11).color(p.text_muted).width(80),
            text("").size(11).width(30),
        ]
        .spacing(8);

        let mut items_col = column![items_table_headers].spacing(8);
        for (idx, item) in self.items.iter().enumerate() {
            let ivapick = pick_list(
                vec![IvaOption::Iva15, IvaOption::Iva0],
                item.iva,
                move |iv| on_message(FacturadorMessage::ItemIvaChanged(idx, iv)),
            )
            .style(pick_list_style(mode))
            .width(80);

            let row_item = row![
                text_input("Cód...", &item.codigo)
                    .style(text_input_style(mode))
                    .width(85)
                    .on_input(move |v| on_message(FacturadorMessage::ItemCodigoChanged(idx, v))),
                text_input("Descripción del ítem...", &item.descripcion)
                    .style(text_input_style(mode))
                    .width(Length::Fill)
                    .on_input(
                        move |v| on_message(FacturadorMessage::ItemDescripcionChanged(idx, v))
                    ),
                text_input("Cant", &item.cantidad)
                    .style(text_input_style(mode))
                    .width(65)
                    .on_input(move |v| on_message(FacturadorMessage::ItemCantidadChanged(idx, v))),
                text_input("P.Unit", &item.precio)
                    .style(text_input_style(mode))
                    .width(75)
                    .on_input(move |v| on_message(FacturadorMessage::ItemPrecioChanged(idx, v))),
                text_input("Desc ($)", &item.descuento)
                    .style(text_input_style(mode))
                    .width(70)
                    .on_input(move |v| on_message(FacturadorMessage::ItemDescuentoChanged(idx, v))),
                ivapick,
                button(text("✕").size(12))
                    .style(danger_button_style(mode))
                    .padding([6, 8])
                    .on_press(on_message(FacturadorMessage::RemoveItem(idx))),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            items_col = items_col.push(row_item);
        }

        let total_desc: f64 = self
            .items
            .iter()
            .map(|it| it.descuento.parse::<f64>().unwrap_or(0.0))
            .sum();

        // Totales Card
        let totales_box = container(
            column![
                row![
                    text("Subtotal IVA 15%:")
                        .size(12)
                        .color(p.text_muted)
                        .width(160),
                    text(format!("${:.2}", sub15)).size(12).color(p.text_main)
                ],
                row![
                    text("Subtotal IVA 0%:")
                        .size(12)
                        .color(p.text_muted)
                        .width(160),
                    text(format!("${:.2}", sub0)).size(12).color(p.text_main)
                ],
                row![
                    text("Total Descuento:")
                        .size(12)
                        .color(p.text_muted)
                        .width(160),
                    text(format!("${:.2}", total_desc))
                        .size(12)
                        .color(p.text_main)
                ],
                row![
                    text("Monto IVA (15%):")
                        .size(12)
                        .color(p.text_muted)
                        .width(160),
                    text(format!("${:.2}", iva)).size(12).color(p.text_main)
                ],
                row![
                    text("TOTAL FACTURA:").size(14).color(p.accent).width(160),
                    text(format!("${:.2}", total)).size(16).color(p.accent)
                ],
            ]
            .spacing(4),
        )
        .padding(12)
        .style(card_style(mode));

        // Formas de Pago
        let fp_header = row![
            row![
                icon_credit_card(p.accent, 16.0),
                text("FORMAS DE PAGO (SRI)").size(13).color(p.accent),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![].width(Length::Fill),
            button(
                row![
                    icon_plus(p.text_main, 12.0),
                    text("Agregar Forma de Pago").size(12),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .style(secondary_button_style(mode))
            .padding([4, 10])
            .on_press(on_message(FacturadorMessage::AddFormaPago)),
        ]
        .align_y(Alignment::Center);

        let mut fp_col = column![].spacing(8);
        for (idx, fp) in self.formas_pago.iter().enumerate() {
            let fp_pick = pick_list(
                vec![
                    FormaPagoCodigo::SinSistemaFinanciero,
                    FormaPagoCodigo::CompensacionDeudas,
                    FormaPagoCodigo::TarjetaDebito,
                    FormaPagoCodigo::DineroElectronico,
                    FormaPagoCodigo::TarjetaPrepago,
                    FormaPagoCodigo::TarjetaCredito,
                    FormaPagoCodigo::OtrosSistemaFinanciero,
                    FormaPagoCodigo::EndosoTitulos,
                ],
                fp.forma_pago,
                move |c| on_message(FacturadorMessage::PagoCodigoChanged(idx, c)),
            )
            .style(pick_list_style(mode))
            .width(Length::Fill);

            let ut_pick = pick_list(
                vec![
                    UnidadTiempoOption::Dias,
                    UnidadTiempoOption::Meses,
                    UnidadTiempoOption::Anios,
                ],
                fp.tiempo,
                move |u| on_message(FacturadorMessage::PagoTiempoChanged(idx, u)),
            )
            .style(pick_list_style(mode))
            .width(95);

            let row_fp = row![
                fp_pick,
                text_input("Valor $", &fp.valor)
                    .style(text_input_style(mode))
                    .width(95)
                    .on_input(move |v| on_message(FacturadorMessage::PagoValorChanged(idx, v))),
                text_input("Plazo", &fp.plazo)
                    .style(text_input_style(mode))
                    .width(75)
                    .on_input(move |v| on_message(FacturadorMessage::PagoPlazoChanged(idx, v))),
                ut_pick,
                button(text("✕").size(12))
                    .style(danger_button_style(mode))
                    .padding([6, 8])
                    .on_press(on_message(FacturadorMessage::RemoveFormaPago(idx))),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            fp_col = fp_col.push(row_fp);
        }

        // Barra de Balance de Pagos
        let balance_bar = container(
            row![
                text(format!(
                    "Total Pagos: ${:.2} | Total Factura: ${:.2}",
                    total_pagos, total
                ))
                .size(12)
                .color(p.text_main),
                row![].width(Length::Fill),
                if diff_pagos < 0.01 {
                    row![
                        icon_check(p.success, 14.0),
                        text("Monto Cuadrado").size(12).color(p.success),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center)
                } else {
                    row![
                        icon_alert_triangle(p.danger, 14.0),
                        text(format!("Diferencia: ${:.2}", diff_pagos))
                            .size(12)
                            .color(p.danger),
                        button(
                            row![
                                icon_zap(p.text_main, 12.0),
                                text("Ajustar al Total").size(11),
                            ]
                            .spacing(4)
                            .align_y(Alignment::Center),
                        )
                        .style(secondary_button_style(mode))
                        .padding([3, 8])
                        .on_press(on_message(FacturadorMessage::AjustarAlTotal)),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                }
            ]
            .align_y(Alignment::Center),
        )
        .padding([8, 12])
        .style(input_container_style(mode));

        // Password P12 y Botón Emitir
        let pass_col = column![
            text("Contraseña Firma .p12 (Opcional si ya se guardó en Configuración):")
                .size(12)
                .color(p.text_muted),
            text_input("••••••••", &self.password_p12)
                .secure(true)
                .style(text_input_style(mode))
                .on_input(move |v| on_message(FacturadorMessage::PasswordChanged(v))),
        ]
        .spacing(4);

        let emitir_btn = button(
            row![
                icon_send(Color::WHITE, 16.0),
                text(if self.is_emitiendo {
                    "Procesando y Firmando..."
                } else {
                    "Emitir Factura Electrónica SRI"
                })
                .size(14),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .style(primary_button_style(mode))
        .padding([10, 24])
        .on_press(on_message(FacturadorMessage::EmitirFactura));

        // Tarjeta de Respuesta SRI si existe
        let mut content = column![
            header,
            container(row_emision).padding(16).style(card_style(mode)),
            container(
                column![
                    client_header,
                    row_cliente1,
                    row_cliente2,
                    row_cliente3,
                    row_cliente4,
                ]
                .spacing(10),
            )
            .padding(16)
            .style(card_style(mode)),
            container(column![items_header, items_col,].spacing(10),)
                .padding(16)
                .style(card_style(mode)),
            totales_box,
            container(column![fp_header, fp_col, balance_bar,].spacing(10),)
                .padding(16)
                .style(card_style(mode)),
            pass_col,
            row![emitir_btn].align_y(Alignment::Center),
        ]
        .spacing(16);

        if let Some(resp) = &self.respuesta_sri {
            let clave = resp.clave_acceso.clone();
            let is_ok = resp.estado == "AUTORIZADO";
            let badge_color = if is_ok { p.success } else { p.danger };

            let mut msg_col = column![].spacing(4);
            for m in &resp.mensajes {
                msg_col = msg_col.push(text(format!("• {}", m)).size(12).color(badge_color));
            }

            let resp_box = container(
                column![
                    row![
                        text("ESTADO SRI:").size(13).color(p.text_muted),
                        text(&resp.estado).size(14).color(badge_color),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    column![
                        text("Número de Autorización / Clave de Acceso (49 dígitos):")
                            .size(11)
                            .color(p.text_muted),
                        text(&resp.clave_acceso).size(11).color(p.accent),
                        button(
                            row![icon_copy(p.text_main, 12.0), text("Copiar Número").size(11),]
                                .spacing(4)
                                .align_y(Alignment::Center),
                        )
                        .style(secondary_button_style(mode))
                        .padding([4, 10])
                        .on_press(on_message(FacturadorMessage::CopiarClave(clave))),
                    ]
                    .spacing(4),
                    msg_col,
                    row![
                        button(
                            row![
                                icon_file_pdf(Color::WHITE, 14.0),
                                text("Guardar PDF").size(12),
                            ]
                            .spacing(6)
                            .align_y(Alignment::Center),
                        )
                        .style(primary_button_style(mode))
                        .padding([8, 14])
                        .on_press(on_message(FacturadorMessage::GuardarPdf)),
                        button(
                            row![
                                icon_file_code(p.text_main, 14.0),
                                text("Guardar XML").size(12),
                            ]
                            .spacing(6)
                            .align_y(Alignment::Center),
                        )
                        .style(secondary_button_style(mode))
                        .padding([8, 14])
                        .on_press(on_message(FacturadorMessage::GuardarXml)),
                        button(
                            row![
                                icon_download(p.text_main, 14.0),
                                text("Guardar Ambos (PDF + XML)").size(12),
                            ]
                            .spacing(6)
                            .align_y(Alignment::Center),
                        )
                        .style(secondary_button_style(mode))
                        .padding([8, 14])
                        .on_press(on_message(FacturadorMessage::GuardarAmbos)),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                ]
                .spacing(12),
            )
            .padding(16)
            .style(card_style(mode));

            content = content.push(resp_box);
        }

        scrollable(container(content).padding(24).width(Length::Fill)).into()
    }
}
