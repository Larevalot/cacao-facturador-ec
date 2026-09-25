//! Vista del Administrador de Inventario (Productos y Servicios) en Iced.

use crate::db::productos::{NuevoProductoRequest, Producto};
use crate::gui::icons::{icon_edit, icon_plus, icon_trash};
use crate::gui::theme::{
    ThemeMode, card_style, danger_button_style, pick_list_style, primary_button_style,
    secondary_button_style, text_input_style,
};
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoItem {
    Producto,
    Servicio,
}

impl std::fmt::Display for TipoItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TipoItem::Producto => write!(f, "PRODUCTO"),
            TipoItem::Servicio => write!(f, "SERVICIO"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TarifaIvaOption {
    Iva15,
    Iva0,
}

impl TarifaIvaOption {
    pub fn codigo(&self) -> &'static str {
        match self {
            TarifaIvaOption::Iva15 => "4",
            TarifaIvaOption::Iva0 => "0",
        }
    }

    pub fn tarifa(&self) -> f64 {
        match self {
            TarifaIvaOption::Iva15 => 15.0,
            TarifaIvaOption::Iva0 => 0.0,
        }
    }
}

impl std::fmt::Display for TarifaIvaOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TarifaIvaOption::Iva15 => write!(f, "15% (General)"),
            TarifaIvaOption::Iva0 => write!(f, "0% (Tarifa 0)"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FormModalState {
    pub editing_id: Option<i64>,
    pub tipo: Option<TipoItem>,
    pub codigo: String,
    pub descripcion: String,
    pub precio_sin_iva: String,
    pub precio_con_iva: String,
    pub iva: Option<TarifaIvaOption>,
    pub stock: String,
}

#[derive(Debug, Clone, Default)]
pub struct InventarioState {
    pub productos: Vec<Producto>,
    pub search_query: String,
    pub modal_form: Option<FormModalState>,
    pub deleting_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub enum InventarioMessage {
    SearchChanged(String),
    OpenNewModal,
    OpenEditModal(Producto),
    CloseModal,
    FormTipoChanged(TipoItem),
    FormCodigoChanged(String),
    FormDescripcionChanged(String),
    FormPrecioSinIvaChanged(String),
    FormPrecioConIvaChanged(String),
    FormIvaChanged(TarifaIvaOption),
    FormStockChanged(String),
    SaveForm,
    ConfirmDelete(i64),
    CancelDelete,
    ExecuteDelete(i64),
}

impl InventarioState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, message: InventarioMessage) -> Option<NuevoProductoRequest> {
        match message {
            InventarioMessage::SearchChanged(q) => {
                self.search_query = q;
                None
            }
            InventarioMessage::OpenNewModal => {
                self.modal_form = Some(FormModalState {
                    editing_id: None,
                    tipo: Some(TipoItem::Producto),
                    codigo: String::new(),
                    descripcion: String::new(),
                    precio_sin_iva: "0.00".to_string(),
                    precio_con_iva: "0.00".to_string(),
                    iva: Some(TarifaIvaOption::Iva15),
                    stock: "1.0".to_string(),
                });
                None
            }
            InventarioMessage::OpenEditModal(p) => {
                let tipo = if p.tipo.to_uppercase() == "SERVICIO" {
                    TipoItem::Servicio
                } else {
                    TipoItem::Producto
                };
                let iva_opt = if p.codigo_iva == "0" {
                    TarifaIvaOption::Iva0
                } else {
                    TarifaIvaOption::Iva15
                };
                let con_iva = p.precio_unitario * (1.0 + iva_opt.tarifa() / 100.0);

                self.modal_form = Some(FormModalState {
                    editing_id: Some(p.id),
                    tipo: Some(tipo),
                    codigo: p.codigo,
                    descripcion: p.descripcion,
                    precio_sin_iva: format!("{:.2}", p.precio_unitario),
                    precio_con_iva: format!("{:.2}", con_iva),
                    iva: Some(iva_opt),
                    stock: format!("{:.0}", p.stock),
                });
                None
            }
            InventarioMessage::CloseModal => {
                self.modal_form = None;
                None
            }
            InventarioMessage::FormTipoChanged(t) => {
                if let Some(form) = &mut self.modal_form {
                    form.tipo = Some(t);
                }
                None
            }
            InventarioMessage::FormCodigoChanged(val) => {
                if let Some(form) = &mut self.modal_form {
                    form.codigo = val;
                }
                None
            }
            InventarioMessage::FormDescripcionChanged(val) => {
                if let Some(form) = &mut self.modal_form {
                    form.descripcion = val;
                }
                None
            }
            InventarioMessage::FormPrecioSinIvaChanged(val) => {
                if let Some(form) = &mut self.modal_form {
                    form.precio_sin_iva = val.clone();
                    if let Ok(sin_iva) = val.parse::<f64>() {
                        let tarifa = form.iva.map(|i| i.tarifa()).unwrap_or(15.0);
                        form.precio_con_iva = format!("{:.2}", sin_iva * (1.0 + tarifa / 100.0));
                    }
                }
                None
            }
            InventarioMessage::FormPrecioConIvaChanged(val) => {
                if let Some(form) = &mut self.modal_form {
                    form.precio_con_iva = val.clone();
                    if let Ok(con_iva) = val.parse::<f64>() {
                        let tarifa = form.iva.map(|i| i.tarifa()).unwrap_or(15.0);
                        form.precio_sin_iva = format!("{:.2}", con_iva / (1.0 + tarifa / 100.0));
                    }
                }
                None
            }
            InventarioMessage::FormIvaChanged(iva) => {
                if let Some(form) = &mut self.modal_form {
                    form.iva = Some(iva);
                    if let Ok(sin_iva) = form.precio_sin_iva.parse::<f64>() {
                        form.precio_con_iva =
                            format!("{:.2}", sin_iva * (1.0 + iva.tarifa() / 100.0));
                    }
                }
                None
            }
            InventarioMessage::FormStockChanged(val) => {
                if let Some(form) = &mut self.modal_form {
                    form.stock = val;
                }
                None
            }
            InventarioMessage::SaveForm => {
                if let Some(form) = &self.modal_form {
                    let precio_unit = form.precio_sin_iva.parse::<f64>().unwrap_or(0.0);
                    let stock = form.stock.parse::<f64>().unwrap_or(0.0);
                    let iva_opt = form.iva.unwrap_or(TarifaIvaOption::Iva15);

                    let req = NuevoProductoRequest {
                        codigo: form.codigo.trim().to_string(),
                        codigo_auxiliar: None,
                        descripcion: form.descripcion.trim().to_string(),
                        precio_unitario: (precio_unit * 100.0).round() / 100.0,
                        stock,
                        codigo_iva: iva_opt.codigo().to_string(),
                        tarifa_iva: iva_opt.tarifa(),
                        tipo: Some(form.tipo.unwrap_or(TipoItem::Producto).to_string()),
                    };
                    return Some(req);
                }
                None
            }
            InventarioMessage::ConfirmDelete(id) => {
                self.deleting_id = Some(id);
                None
            }
            InventarioMessage::CancelDelete => {
                self.deleting_id = None;
                None
            }
            InventarioMessage::ExecuteDelete(_) => {
                self.deleting_id = None;
                None
            }
        }
    }

    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        mode: ThemeMode,
        on_message: fn(InventarioMessage) -> Message,
    ) -> Element<'a, Message> {
        let p = mode.palette();

        let header = row![
            column![
                text("Inventario de Productos y Servicios")
                    .size(20)
                    .color(p.text_main),
                text("Gestiona tu catálogo, precios y tarifas de IVA")
                    .size(13)
                    .color(p.text_muted),
            ]
            .spacing(4),
            row![].width(Length::Fill),
            button(
                row![
                    icon_plus(Color::WHITE, 14.0),
                    text("Nuevo Producto").size(14),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([8, 16])
            .style(primary_button_style(mode))
            .on_press(on_message(InventarioMessage::OpenNewModal)),
        ]
        .align_y(Alignment::Center);

        let search_bar = text_input("Buscar por código o descripción...", &self.search_query)
            .padding(10)
            .style(text_input_style(mode))
            .on_input(move |q| on_message(InventarioMessage::SearchChanged(q)));

        // Tabla de productos
        let table_header = container(
            row![
                text("TIPO").size(12).color(p.text_muted).width(100),
                text("CÓDIGO").size(12).color(p.text_muted).width(120),
                text("DESCRIPCIÓN")
                    .size(12)
                    .color(p.text_muted)
                    .width(Length::Fill),
                text("P.UNIT ($)").size(12).color(p.text_muted).width(90),
                text("IVA").size(12).color(p.text_muted).width(70),
                text("STOCK").size(12).color(p.text_muted).width(70),
                text("ACCIONES").size(12).color(p.text_muted).width(140),
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

        let filtered: Vec<&Producto> = self
            .productos
            .iter()
            .filter(|prod| {
                if self.search_query.is_empty() {
                    return true;
                }
                let q = self.search_query.to_lowercase();
                prod.codigo.to_lowercase().contains(&q)
                    || prod.descripcion.to_lowercase().contains(&q)
            })
            .collect();

        let mut rows_col = column![].spacing(6);

        if filtered.is_empty() {
            rows_col = rows_col.push(
                container(
                    text(if self.productos.is_empty() {
                        "No hay productos registrados en el inventario. Agrega uno con '+ Nuevo Producto'."
                    } else {
                        "No se encontraron productos coincidentes."
                    })
                    .size(13)
                    .color(p.text_muted),
                )
                .padding(20)
                .center_x(Length::Fill),
            );
        } else {
            for prod in filtered {
                let p_clone = prod.clone();
                let is_deleting = self.deleting_id == Some(prod.id);

                let actions_cell: Element<'a, Message> = if is_deleting {
                    row![
                        button(text("¿Seguro?").size(11))
                            .style(danger_button_style(mode))
                            .padding([4, 6])
                            .on_press(on_message(InventarioMessage::ExecuteDelete(prod.id))),
                        button(text("No").size(11))
                            .style(secondary_button_style(mode))
                            .padding([4, 6])
                            .on_press(on_message(InventarioMessage::CancelDelete)),
                    ]
                    .spacing(4)
                    .into()
                } else {
                    row![
                        button(
                            row![icon_edit(p.text_main, 12.0), text("Editar").size(11),]
                                .spacing(4)
                                .align_y(Alignment::Center),
                        )
                        .style(secondary_button_style(mode))
                        .padding([4, 8])
                        .on_press(on_message(InventarioMessage::OpenEditModal(p_clone))),
                        button(icon_trash(p.danger, 12.0))
                            .style(danger_button_style(mode))
                            .padding([4, 8])
                            .on_press(on_message(InventarioMessage::ConfirmDelete(prod.id))),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into()
                };

                let row_item = container(
                    row![
                        container(
                            text(if prod.tipo == "SERVICIO" {
                                "SERVICIO"
                            } else {
                                "PRODUCTO"
                            })
                            .size(11)
                            .color(p.accent)
                        )
                        .width(100),
                        text(&prod.codigo).size(13).color(p.text_main).width(120),
                        text(&prod.descripcion)
                            .size(13)
                            .color(p.text_main)
                            .width(Length::Fill),
                        text(format!("${:.2}", prod.precio_unitario))
                            .size(13)
                            .color(p.accent)
                            .width(90),
                        text(format!("{}%", prod.tarifa_iva as i64))
                            .size(13)
                            .color(p.text_muted)
                            .width(70),
                        text(if prod.tipo == "SERVICIO" {
                            "-".to_string()
                        } else {
                            format!("{:.0}", prod.stock)
                        })
                        .size(13)
                        .color(p.text_muted)
                        .width(70),
                        actions_cell,
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
            search_bar,
            table_header,
            scrollable(rows_col).height(Length::Fill),
        ]
        .spacing(14);

        // Render modal si está abierto
        if let Some(form) = &self.modal_form {
            let is_edit = form.editing_id.is_some();
            let modal_title = if is_edit {
                "Editar Producto / Servicio"
            } else {
                "Nuevo Producto / Servicio"
            };

            let tipos = vec![TipoItem::Producto, TipoItem::Servicio];
            let tipo_picker = pick_list(tipos, form.tipo, move |t| {
                on_message(InventarioMessage::FormTipoChanged(t))
            })
            .style(pick_list_style(mode))
            .width(Length::Fill);

            let ivas = vec![TarifaIvaOption::Iva15, TarifaIvaOption::Iva0];
            let iva_picker = pick_list(ivas, form.iva, move |i| {
                on_message(InventarioMessage::FormIvaChanged(i))
            })
            .style(pick_list_style(mode))
            .width(Length::Fill);

            let modal_content = column![
                row![
                    text(modal_title).size(17).color(p.text_main),
                    row![].width(Length::Fill),
                    button(text("✕").size(14))
                        .style(secondary_button_style(mode))
                        .padding([4, 8])
                        .on_press(on_message(InventarioMessage::CloseModal)),
                ]
                .align_y(Alignment::Center),
                row![
                    column![text("Tipo:").size(12).color(p.text_muted), tipo_picker]
                        .spacing(4)
                        .width(Length::FillPortion(1)),
                    column![
                        text("Código:").size(12).color(p.text_muted),
                        text_input("PRD-001", &form.codigo)
                            .style(text_input_style(mode))
                            .on_input(move |v| on_message(InventarioMessage::FormCodigoChanged(v)))
                    ]
                    .spacing(4)
                    .width(Length::FillPortion(1)),
                ]
                .spacing(10),
                column![
                    text("Descripción:").size(12).color(p.text_muted),
                    text_input("Nombre del producto o servicio...", &form.descripcion)
                        .style(text_input_style(mode))
                        .on_input(
                            move |v| on_message(InventarioMessage::FormDescripcionChanged(v))
                        )
                ]
                .spacing(4),
                row![
                    column![
                        text("Precio Sin IVA ($):").size(12).color(p.text_muted),
                        text_input("0.00", &form.precio_sin_iva)
                            .style(text_input_style(mode))
                            .on_input(move |v| on_message(
                                InventarioMessage::FormPrecioSinIvaChanged(v)
                            ))
                    ]
                    .spacing(4)
                    .width(Length::FillPortion(1)),
                    column![
                        text("Precio Con IVA ($):").size(12).color(p.text_muted),
                        text_input("0.00", &form.precio_con_iva)
                            .style(text_input_style(mode))
                            .on_input(move |v| on_message(
                                InventarioMessage::FormPrecioConIvaChanged(v)
                            ))
                    ]
                    .spacing(4)
                    .width(Length::FillPortion(1)),
                ]
                .spacing(10),
                row![
                    column![text("Tarifa IVA:").size(12).color(p.text_muted), iva_picker]
                        .spacing(4)
                        .width(Length::FillPortion(1)),
                    column![
                        text("Stock Inicial:").size(12).color(p.text_muted),
                        text_input("1.0", &form.stock)
                            .style(text_input_style(mode))
                            .on_input(move |v| on_message(InventarioMessage::FormStockChanged(v)))
                    ]
                    .spacing(4)
                    .width(Length::FillPortion(1)),
                ]
                .spacing(10),
                row![
                    button(text("Cancelar").size(13))
                        .style(secondary_button_style(mode))
                        .padding([8, 16])
                        .on_press(on_message(InventarioMessage::CloseModal)),
                    row![].width(Length::Fill),
                    button(
                        text(if is_edit {
                            "Guardar Cambios"
                        } else {
                            "Crear Producto"
                        })
                        .size(13)
                    )
                    .style(primary_button_style(mode))
                    .padding([8, 20])
                    .on_press(on_message(InventarioMessage::SaveForm)),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(14);

            let modal_box = container(modal_content)
                .width(480)
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
