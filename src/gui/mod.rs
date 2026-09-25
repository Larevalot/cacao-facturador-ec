//! Módulo principal de la GUI en Iced para Cacao Facturador SRI.

pub mod components;
pub mod icons;
pub mod theme;
pub mod views;

use crate::config::{cargar_configuracion, guardar_configuracion};
use crate::db::clientes::buscar_cliente_por_identificacion;
use crate::db::facturas::{guardar_factura_db, listar_historial_facturas};
use crate::db::productos::{
    actualizar_producto, crear_producto, eliminar_producto, listar_productos,
};
use crate::gui::components::header::{HeaderAction, view as header_view};
use crate::gui::components::sidebar::{NavigationTab, view as sidebar_view};
use crate::gui::components::toast::{Toast, ToastType};
use crate::gui::theme::{ThemeMode, primary_container_style};
use crate::gui::views::configuracion::{ConfiguracionMessage, ConfiguracionState};
use crate::gui::views::dashboard::view as dashboard_view;
use crate::gui::views::facturador::{FacturadorMessage, FacturadorState};
use crate::gui::views::historial::{HistorialMessage, HistorialState};
use crate::gui::views::inventario::{InventarioMessage, InventarioState};
use crate::gui::views::pin::{PinMessage, PinOutcome, PinState};
use crate::sri::clave_acceso::generar_clave_acceso;
use crate::sri::client::SriClient;
use crate::sri::models::{EmisorConfig, FacturaRequest, RespuestaSRI};
use crate::sri::pdf_builder::{DatosRide, extraer_datos_ride_de_xml, generar_ride_pdf};
use crate::sri::xades_signer::firmar_xml;
use crate::sri::xml_builder::construir_xml_factura;

use iced::widget::{column, container, row, stack};
use iced::{Alignment, Element, Length, Size, Task, clipboard};
use sqlx::SqlitePool;

pub struct CacaoApp {
    pub mode: ThemeMode,
    pub active_tab: NavigationTab,
    pub is_unlocked: bool,
    pub stored_pin: Option<String>,
    pub pin_state: PinState,
    pub window_id: Option<iced::window::Id>,
    pub dashboard_state: (),
    pub facturador_state: FacturadorState,
    pub inventario_state: InventarioState,
    pub historial_state: HistorialState,
    pub configuracion_state: ConfiguracionState,
    pub toast: Option<Toast>,
    pub toast_counter: u64,
    pub pool: SqlitePool,
    pub emisor_config: EmisorConfig,
}

#[derive(Debug, Clone)]
pub enum Message {
    Header(HeaderAction),
    WindowOpened(iced::window::Id),
    Navigate(NavigationTab),
    Pin(PinMessage),
    Facturador(FacturadorMessage),
    Inventario(InventarioMessage),
    Historial(HistorialMessage),
    Configuracion(ConfiguracionMessage),
    CloseToast,
    ToastTick,
    ClientFound(Option<crate::db::clientes::Cliente>),
    ProductsLoaded(Vec<crate::db::productos::Producto>),
    InvoicesLoaded(Vec<crate::db::facturas::FacturaGuardada>),
    InvoiceEmitted(Result<RespuestaSRI, String>),
    OpenCredits,
}

impl CacaoApp {
    pub fn new(pool: SqlitePool) -> (Self, Task<Message>) {
        let config = cargar_configuracion();
        let conf_state = ConfiguracionState::from_config(&config);
        let stored_pin = config.pin.clone();
        let has_pin = stored_pin.as_ref().is_some_and(|p| !p.trim().is_empty());
        let is_unlocked = !has_pin;
        let pin_state = PinState {
            is_new_pin: !has_pin,
            ..Default::default()
        };

        let app = Self {
            mode: ThemeMode::Dark,
            active_tab: NavigationTab::Dashboard,
            is_unlocked,
            stored_pin,
            pin_state,
            window_id: None,
            dashboard_state: (),
            facturador_state: FacturadorState::default(),
            inventario_state: InventarioState::default(),
            historial_state: HistorialState::default(),
            configuracion_state: conf_state,
            toast: None,
            toast_counter: 0,
            pool: pool.clone(),
            emisor_config: config,
        };

        let load_products = {
            let pool = pool.clone();
            Task::perform(
                async move { listar_productos(&pool).await.unwrap_or_default() },
                Message::ProductsLoaded,
            )
        };

        let load_invoices = {
            let pool = pool.clone();
            Task::perform(
                async move { listar_historial_facturas(&pool).await.unwrap_or_default() },
                Message::InvoicesLoaded,
            )
        };

        (app, Task::batch([load_products, load_invoices]))
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        let mut subs = Vec::new();
        subs.push(iced::window::open_events().map(Message::WindowOpened));

        if !self.is_unlocked {
            let key_sub = iced::keyboard::listen().filter_map(|event| {
                if let iced::keyboard::Event::KeyPressed { key, text, .. } = event {
                    match key {
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace) => {
                            Some(Message::Pin(PinMessage::Backspace))
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter) => {
                            Some(Message::Pin(PinMessage::Submit))
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => {
                            Some(Message::Pin(PinMessage::Clear))
                        }
                        _ => {
                            if let Some(t) = text
                                && let Some(c) = t.chars().next()
                            {
                                if c.is_ascii_digit() {
                                    return Some(Message::Pin(PinMessage::DigitPressed(c)));
                                } else if c == 'c' || c == 'C' {
                                    return Some(Message::Pin(PinMessage::Clear));
                                }
                            }
                            if let iced::keyboard::Key::Character(c_str) = key
                                && let Some(c) = c_str.chars().next()
                            {
                                if c.is_ascii_digit() {
                                    return Some(Message::Pin(PinMessage::DigitPressed(c)));
                                } else if c == 'c' || c == 'C' {
                                    return Some(Message::Pin(PinMessage::Clear));
                                }
                            }
                            None
                        }
                    }
                } else {
                    None
                }
            });
            subs.push(key_sub);
        }

        if self.toast.is_some() {
            subs.push(
                iced::time::every(std::time::Duration::from_millis(250))
                    .map(|_| Message::ToastTick),
            );
        }

        iced::Subscription::batch(subs)
    }

    pub fn title(&self) -> String {
        "Cacao Facturador SRI & Inventario".to_string()
    }

    pub fn show_toast(
        &mut self,
        toast_type: ToastType,
        title: impl Into<String>,
        msg: impl Into<String>,
    ) {
        self.toast_counter += 1;
        self.toast = Some(Toast::new(self.toast_counter, toast_type, title, msg));
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(id) => {
                self.window_id = Some(id);
                Task::none()
            }
            Message::Header(action) => match action {
                HeaderAction::ToggleTheme => {
                    self.mode = match self.mode {
                        ThemeMode::Dark => ThemeMode::Light,
                        ThemeMode::Light => ThemeMode::Dark,
                    };
                    Task::none()
                }
                HeaderAction::CheckUpdates => {
                    self.show_toast(
                        ToastType::Info,
                        "Actualizaciones",
                        "Tienes la versión v1.0.0 más reciente.",
                    );
                    Task::none()
                }
                HeaderAction::LockApp => {
                    self.is_unlocked = false;
                    self.pin_state = PinState {
                        is_new_pin: self.stored_pin.is_none(),
                        ..Default::default()
                    };
                    Task::none()
                }
                HeaderAction::Minimize => {
                    if let Some(id) = self.window_id {
                        iced::window::minimize(id, true)
                    } else {
                        Task::none()
                    }
                }
                HeaderAction::Maximize => {
                    if let Some(id) = self.window_id {
                        iced::window::toggle_maximize(id)
                    } else {
                        Task::none()
                    }
                }
                HeaderAction::Close => {
                    if let Some(id) = self.window_id {
                        iced::window::close(id)
                    } else {
                        Task::none()
                    }
                }
                HeaderAction::DragWindow => {
                    if let Some(id) = self.window_id {
                        iced::window::drag(id)
                    } else {
                        Task::none()
                    }
                }
            },
            Message::Navigate(tab) => {
                self.active_tab = tab;
                match tab {
                    NavigationTab::Dashboard => {
                        let pool_prod = self.pool.clone();
                        let load_products = Task::perform(
                            async move { listar_productos(&pool_prod).await.unwrap_or_default() },
                            Message::ProductsLoaded,
                        );
                        let pool_inv = self.pool.clone();
                        let load_invoices = Task::perform(
                            async move {
                                listar_historial_facturas(&pool_inv)
                                    .await
                                    .unwrap_or_default()
                            },
                            Message::InvoicesLoaded,
                        );
                        Task::batch([load_products, load_invoices])
                    }
                    NavigationTab::Inventario => {
                        let pool = self.pool.clone();
                        Task::perform(
                            async move { listar_productos(&pool).await.unwrap_or_default() },
                            Message::ProductsLoaded,
                        )
                    }
                    NavigationTab::Historial => {
                        let pool = self.pool.clone();
                        Task::perform(
                            async move { listar_historial_facturas(&pool).await.unwrap_or_default() },
                            Message::InvoicesLoaded,
                        )
                    }
                    _ => Task::none(),
                }
            }
            Message::Pin(msg) => {
                if let Some(outcome) = self.pin_state.update(msg, self.stored_pin.as_deref()) {
                    match outcome {
                        PinOutcome::Unlocked => {
                            self.is_unlocked = true;
                            self.show_toast(
                                ToastType::Success,
                                "Acceso Correcto",
                                "Bienvenido a Cacao Facturador.",
                            );
                        }
                        PinOutcome::Created(new_pin) => {
                            self.stored_pin = Some(new_pin.clone());
                            self.emisor_config.pin = Some(new_pin.clone());
                            self.configuracion_state.pin = new_pin;
                            let _ = guardar_configuracion(&self.emisor_config);
                            self.is_unlocked = true;
                            self.show_toast(
                                ToastType::Success,
                                "PIN Configurado",
                                "PIN de seguridad creado exitosamente.",
                            );
                        }
                    }
                }
                Task::none()
            }
            Message::Facturador(msg) => match msg {
                FacturadorMessage::IdentificacionChanged(val) => {
                    self.facturador_state
                        .update(FacturadorMessage::IdentificacionChanged(val.clone()));
                    if val.len() >= 10 {
                        let pool = self.pool.clone();
                        Task::perform(
                            async move {
                                buscar_cliente_por_identificacion(&pool, &val)
                                    .await
                                    .unwrap_or(None)
                            },
                            Message::ClientFound,
                        )
                    } else {
                        self.facturador_state.cliente_encontrado = false;
                        Task::none()
                    }
                }
                FacturadorMessage::CopiarClave(clave) => {
                    self.show_toast(
                        ToastType::Info,
                        "Copiado",
                        "Clave de acceso de 49 dígitos copiada.",
                    );
                    clipboard::write(clave)
                }
                FacturadorMessage::GuardarPdf => {
                    let maybe_datos = if let (Some(resp), Some(req)) = (
                        &self.facturador_state.respuesta_sri,
                        &self.facturador_state.ultimo_request,
                    ) {
                        Some((
                            DatosRide::from_request(req, resp, &self.emisor_config),
                            req.secuencial.clone(),
                        ))
                    } else {
                        None
                    };

                    if let Some((datos, sec)) = maybe_datos {
                        match generar_ride_pdf(&datos, &self.emisor_config.plantilla_pdf) {
                            Ok(pdf_bytes) => {
                                let filename = format!(
                                    "FACT-{}-{}.pdf",
                                    self.emisor_config.cod_establecimiento, sec
                                );
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_file_name(&filename)
                                    .add_filter("Documento PDF (*.pdf)", &["pdf"])
                                    .set_title("Guardar Factura RIDE en PDF")
                                    .save_file()
                                {
                                    match std::fs::write(&path, &pdf_bytes) {
                                        Ok(_) => self.show_toast(
                                            ToastType::Success,
                                            "PDF Guardado",
                                            format!("Archivo guardado en: {}", path.display()),
                                        ),
                                        Err(e) => self.show_toast(
                                            ToastType::Error,
                                            "Error al Guardar",
                                            format!("No se pudo escribir el archivo: {}", e),
                                        ),
                                    }
                                }
                            }
                            Err(e) => self.show_toast(ToastType::Error, "Error Generando PDF", e),
                        }
                    } else {
                        self.show_toast(
                            ToastType::Warning,
                            "Exportar PDF",
                            "No hay factura emitida recientemente en pantalla.",
                        );
                    }
                    Task::none()
                }
                FacturadorMessage::GuardarXml => {
                    let maybe_xml = if let Some(resp) = &self.facturador_state.respuesta_sri {
                        resp.xml_autorizado
                            .clone()
                            .or_else(|| resp.xml_firmado.clone())
                    } else {
                        None
                    };
                    let sec = self
                        .facturador_state
                        .ultimo_request
                        .as_ref()
                        .map(|r| r.secuencial.clone())
                        .unwrap_or_else(|| "000000001".to_string());

                    if let Some(xml_content) = maybe_xml {
                        let filename = format!(
                            "FACT-{}-{}.xml",
                            self.emisor_config.cod_establecimiento, sec
                        );
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name(&filename)
                            .add_filter("Comprobante XML (*.xml)", &["xml"])
                            .set_title("Guardar Comprobante XML SRI")
                            .save_file()
                        {
                            match std::fs::write(
                                &path,
                                crate::sri::xml_builder::unescape_xml(&xml_content).as_bytes(),
                            ) {
                                Ok(_) => self.show_toast(
                                    ToastType::Success,
                                    "XML Guardado",
                                    format!("Archivo guardado en: {}", path.display()),
                                ),
                                Err(e) => self.show_toast(
                                    ToastType::Error,
                                    "Error al Guardar",
                                    format!("No se pudo escribir el archivo: {}", e),
                                ),
                            }
                        }
                    } else if self.facturador_state.respuesta_sri.is_some() {
                        self.show_toast(
                            ToastType::Warning,
                            "Exportar XML",
                            "El comprobante no tiene XML firmado disponible.",
                        );
                    } else {
                        self.show_toast(
                            ToastType::Warning,
                            "Exportar XML",
                            "No hay factura emitida recientemente en pantalla.",
                        );
                    }
                    Task::none()
                }
                FacturadorMessage::GuardarAmbos => {
                    let maybe_data = if let (Some(resp), Some(req)) = (
                        &self.facturador_state.respuesta_sri,
                        &self.facturador_state.ultimo_request,
                    ) {
                        let xml = resp
                            .xml_autorizado
                            .clone()
                            .or_else(|| resp.xml_firmado.clone());
                        let datos = DatosRide::from_request(req, resp, &self.emisor_config);
                        Some((datos, xml, req.secuencial.clone()))
                    } else {
                        None
                    };

                    if let Some((datos, xml, sec)) = maybe_data {
                        let pdf_res = generar_ride_pdf(&datos, &self.emisor_config.plantilla_pdf);

                        match (pdf_res, xml) {
                            (Ok(pdf_bytes), Some(xml_content)) => {
                                if let Some(folder) = rfd::FileDialog::new()
                                    .set_title("Seleccionar Carpeta para Guardar PDF y XML")
                                    .pick_folder()
                                {
                                    let base_name = format!(
                                        "FACT-{}-{}",
                                        self.emisor_config.cod_establecimiento, sec
                                    );
                                    let pdf_path = folder.join(format!("{}.pdf", base_name));
                                    let xml_path = folder.join(format!("{}.xml", base_name));
                                    let mut ok = true;
                                    if let Err(e) = std::fs::write(&pdf_path, &pdf_bytes) {
                                        self.show_toast(
                                            ToastType::Error,
                                            "Error PDF",
                                            format!("{}", e),
                                        );
                                        ok = false;
                                    }
                                    let xml_clean =
                                        crate::sri::xml_builder::unescape_xml(&xml_content);
                                    if let Err(e) = std::fs::write(&xml_path, xml_clean.as_bytes())
                                    {
                                        self.show_toast(
                                            ToastType::Error,
                                            "Error XML",
                                            format!("{}", e),
                                        );
                                        ok = false;
                                    }
                                    if ok {
                                        self.show_toast(
                                            ToastType::Success,
                                            "Archivos Guardados",
                                            format!("PDF y XML guardados en: {}", folder.display()),
                                        );
                                    }
                                }
                            }
                            (Err(e), _) => {
                                self.show_toast(ToastType::Error, "Error Generando PDF", e)
                            }
                            (_, None) => self.show_toast(
                                ToastType::Warning,
                                "Exportar XML",
                                "No se encontró contenido XML.",
                            ),
                        }
                    } else {
                        self.show_toast(
                            ToastType::Warning,
                            "Guardar",
                            "No hay factura emitida recientemente en pantalla.",
                        );
                    }
                    Task::none()
                }
                FacturadorMessage::EmitirFactura => {
                    let (_sub15, _sub0, _, total) = self.facturador_state.calcular_totales();
                    let total_pagos = self.facturador_state.calcular_total_pagos();

                    if self.facturador_state.items.is_empty() {
                        self.show_toast(
                            ToastType::Warning,
                            "Factura Vacía",
                            "Debes agregar al menos un ítem a la factura.",
                        );
                        return Task::none();
                    }

                    if (total - total_pagos).abs() >= 0.01 {
                        self.show_toast(
                                ToastType::Warning,
                                "Descuadre en Pagos",
                                format!("La suma de las formas de pago (${:.2}) debe ser igual al total de la factura (${:.2}).", total_pagos, total),
                            );
                        return Task::none();
                    }

                    let req = match self.facturador_state.construir_factura_request() {
                        Ok(r) => r,
                        Err(e) => {
                            self.show_toast(ToastType::Error, "Error de Validación", e);
                            return Task::none();
                        }
                    };

                    self.facturador_state
                        .update(FacturadorMessage::EmitirFactura);
                    self.facturador_state.ultimo_request = Some(req.clone());

                    let emisor = self.emisor_config.clone();
                    let password = if !self.facturador_state.password_p12.trim().is_empty() {
                        Some(self.facturador_state.password_p12.trim().to_string())
                    } else {
                        emisor.p12_password.clone()
                    };
                    let pool = self.pool.clone();

                    Task::perform(
                        async move { emitir_factura_proceso(emisor, req, password, pool).await },
                        Message::InvoiceEmitted,
                    )
                }
                other => {
                    self.facturador_state.update(other);
                    Task::none()
                }
            },
            Message::ClientFound(cliente) => {
                if cliente.is_some() {
                    self.show_toast(
                        ToastType::Info,
                        "Autocompletado",
                        "Cliente frecuente encontrado.",
                    );
                }
                self.facturador_state.autocompletar_cliente(cliente);
                Task::none()
            }
            Message::InvoiceEmitted(res) => {
                self.facturador_state.is_emitiendo = false;
                match res {
                    Ok(resp) => {
                        let is_ok = resp.estado == "AUTORIZADO"
                            || resp.estado.contains("RECIBIDA")
                            || resp.estado.contains("PROCESAMIENTO");
                        if is_ok {
                            self.show_toast(
                                ToastType::Success,
                                "Respuesta SRI",
                                format!("Comprobante {} por el SRI.", resp.estado),
                            );
                            // Auto-incrementar secuencial
                            if let Ok(num) = self.facturador_state.secuencial.parse::<i64>() {
                                self.facturador_state.secuencial = format!("{:0>9}", num + 1);
                            }
                        } else {
                            self.show_toast(
                                ToastType::Error,
                                "Respuesta SRI",
                                format!("Comprobante {}.", resp.estado),
                            );
                        }
                        self.facturador_state.respuesta_sri = Some(resp);

                        // Recargar historial
                        let pool = self.pool.clone();
                        Task::perform(
                            async move { listar_historial_facturas(&pool).await.unwrap_or_default() },
                            Message::InvoicesLoaded,
                        )
                    }
                    Err(err) => {
                        self.show_toast(ToastType::Error, "Fallo al Emitir", err);
                        Task::none()
                    }
                }
            }
            Message::Inventario(msg) => match msg {
                InventarioMessage::SaveForm => {
                    let editing_id = self
                        .inventario_state
                        .modal_form
                        .as_ref()
                        .and_then(|f| f.editing_id);
                    if let Some(req) = self.inventario_state.update(InventarioMessage::SaveForm) {
                        self.inventario_state.modal_form = None;
                        let pool = self.pool.clone();
                        Task::perform(
                            async move {
                                if let Some(id) = editing_id {
                                    let _ = actualizar_producto(&pool, id, &req).await;
                                } else {
                                    let _ = crear_producto(&pool, &req).await;
                                }
                                listar_productos(&pool).await.unwrap_or_default()
                            },
                            Message::ProductsLoaded,
                        )
                    } else {
                        Task::none()
                    }
                }
                InventarioMessage::ExecuteDelete(id) => {
                    self.inventario_state
                        .update(InventarioMessage::ExecuteDelete(id));
                    let pool = self.pool.clone();
                    self.show_toast(ToastType::Info, "Inventario", "Producto eliminado.");
                    Task::perform(
                        async move {
                            let _ = eliminar_producto(&pool, id).await;
                            listar_productos(&pool).await.unwrap_or_default()
                        },
                        Message::ProductsLoaded,
                    )
                }
                other => {
                    self.inventario_state.update(other);
                    Task::none()
                }
            },
            Message::Historial(msg) => match msg {
                HistorialMessage::CopiarClave(clave) => {
                    self.historial_state
                        .update(HistorialMessage::CopiarClave(clave.clone()));
                    self.show_toast(
                        ToastType::Info,
                        "Copiado",
                        "Clave de acceso de 49 dígitos copiada.",
                    );
                    clipboard::write(clave)
                }
                HistorialMessage::AnularEnSri(clave) => {
                    self.historial_state
                        .update(HistorialMessage::AnularEnSri(clave.clone()));
                    self.show_toast(
                        ToastType::Info,
                        "Anulación SRI",
                        "Clave copiada. Abriendo SRI en Línea en el navegador...",
                    );
                    clipboard::write(clave)
                }
                HistorialMessage::DescargarPdf(factura) => {
                    let xml = factura
                        .xml_autorizado
                        .as_deref()
                        .or(factura.xml_firmado.as_deref());
                    if let Some(xml_str) = xml {
                        match extraer_datos_ride_de_xml(xml_str, &self.emisor_config) {
                            Ok(datos) => {
                                match generar_ride_pdf(&datos, &self.emisor_config.plantilla_pdf) {
                                    Ok(pdf_bytes) => {
                                        let filename = format!(
                                            "FACT-{}-{}.pdf",
                                            datos.emisor.cod_establecimiento, factura.secuencial
                                        );
                                        if let Some(path) = rfd::FileDialog::new()
                                            .set_file_name(&filename)
                                            .add_filter("Documento PDF (*.pdf)", &["pdf"])
                                            .set_title("Guardar Factura RIDE en PDF")
                                            .save_file()
                                        {
                                            match std::fs::write(&path, &pdf_bytes) {
                                                Ok(_) => self.show_toast(
                                                    ToastType::Success,
                                                    "PDF Guardado",
                                                    format!(
                                                        "Archivo guardado en: {}",
                                                        path.display()
                                                    ),
                                                ),
                                                Err(e) => self.show_toast(
                                                    ToastType::Error,
                                                    "Error al Guardar",
                                                    format!(
                                                        "No se pudo escribir el archivo: {}",
                                                        e
                                                    ),
                                                ),
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        self.show_toast(ToastType::Error, "Error Generando PDF", e)
                                    }
                                }
                            }
                            Err(e) => self.show_toast(ToastType::Error, "Error Leyendo XML", e),
                        }
                    } else {
                        self.show_toast(
                            ToastType::Warning,
                            "Descargar PDF",
                            "Esta factura no contiene XML guardado en la base de datos.",
                        );
                    }
                    Task::none()
                }
                HistorialMessage::DescargarXml(factura) => {
                    let xml = factura
                        .xml_autorizado
                        .as_deref()
                        .or(factura.xml_firmado.as_deref());
                    if let Some(xml_content) = xml {
                        let filename = format!(
                            "FACT-{}-{}.xml",
                            self.emisor_config.cod_establecimiento, factura.secuencial
                        );
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name(&filename)
                            .add_filter("Comprobante XML (*.xml)", &["xml"])
                            .set_title("Guardar Comprobante XML SRI")
                            .save_file()
                        {
                            let xml_clean = crate::sri::xml_builder::unescape_xml(xml_content);
                            match std::fs::write(&path, xml_clean.as_bytes()) {
                                Ok(_) => self.show_toast(
                                    ToastType::Success,
                                    "XML Guardado",
                                    format!("Archivo guardado en: {}", path.display()),
                                ),
                                Err(e) => self.show_toast(
                                    ToastType::Error,
                                    "Error al Guardar",
                                    format!("No se pudo escribir el archivo: {}", e),
                                ),
                            }
                        }
                    } else {
                        self.show_toast(
                            ToastType::Warning,
                            "Descargar XML",
                            "Esta factura no contiene XML guardado en la base de datos.",
                        );
                    }
                    Task::none()
                }
                HistorialMessage::DescargarAmbos(factura) => {
                    let xml = factura
                        .xml_autorizado
                        .as_deref()
                        .or(factura.xml_firmado.as_deref());
                    if let Some(xml_str) = xml {
                        match extraer_datos_ride_de_xml(xml_str, &self.emisor_config) {
                            Ok(datos) => {
                                match generar_ride_pdf(&datos, &self.emisor_config.plantilla_pdf) {
                                    Ok(pdf_bytes) => {
                                        if let Some(folder) = rfd::FileDialog::new()
                                            .set_title("Seleccionar Carpeta para Guardar PDF y XML")
                                            .pick_folder()
                                        {
                                            let base_name = format!(
                                                "FACT-{}-{}",
                                                self.emisor_config.cod_establecimiento,
                                                factura.secuencial
                                            );
                                            let pdf_path =
                                                folder.join(format!("{}.pdf", base_name));
                                            let xml_path =
                                                folder.join(format!("{}.xml", base_name));
                                            let mut ok = true;
                                            if let Err(e) = std::fs::write(&pdf_path, &pdf_bytes) {
                                                self.show_toast(
                                                    ToastType::Error,
                                                    "Error PDF",
                                                    format!("{}", e),
                                                );
                                                ok = false;
                                            }
                                            let xml_clean =
                                                crate::sri::xml_builder::unescape_xml(xml_str);
                                            if let Err(e) =
                                                std::fs::write(&xml_path, xml_clean.as_bytes())
                                            {
                                                self.show_toast(
                                                    ToastType::Error,
                                                    "Error XML",
                                                    format!("{}", e),
                                                );
                                                ok = false;
                                            }
                                            if ok {
                                                self.show_toast(
                                                    ToastType::Success,
                                                    "Archivos Guardados",
                                                    format!(
                                                        "PDF y XML guardados en: {}",
                                                        folder.display()
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        self.show_toast(ToastType::Error, "Error Generando PDF", e)
                                    }
                                }
                            }
                            Err(e) => self.show_toast(ToastType::Error, "Error Leyendo XML", e),
                        }
                    } else {
                        self.show_toast(
                            ToastType::Warning,
                            "Descargar",
                            "Esta factura no contiene XML guardado en la base de datos.",
                        );
                    }
                    Task::none()
                }
                other => {
                    self.historial_state.update(other);
                    Task::none()
                }
            },
            Message::Configuracion(msg) => match msg {
                ConfiguracionMessage::Guardar => {
                    if let Some(cfg) = self
                        .configuracion_state
                        .update(ConfiguracionMessage::Guardar)
                    {
                        if let Err(e) = guardar_configuracion(&cfg) {
                            self.show_toast(ToastType::Error, "Configuración", e);
                        } else {
                            self.stored_pin = cfg.pin.clone();
                            self.emisor_config = cfg;
                            self.show_toast(
                                ToastType::Success,
                                "Configuración",
                                "Datos tributarios y PIN guardados correctamente.",
                            );
                        }
                    }
                    Task::none()
                }
                other => {
                    self.configuracion_state.update(other);
                    Task::none()
                }
            },
            Message::CloseToast => {
                self.toast = None;
                Task::none()
            }
            Message::ToastTick => {
                if let Some(toast) = &self.toast
                    && toast.created_at.elapsed() >= std::time::Duration::from_secs(6)
                {
                    self.toast = None;
                }
                Task::none()
            }
            Message::ProductsLoaded(prods) => {
                self.inventario_state.productos = prods;
                Task::none()
            }
            Message::InvoicesLoaded(invs) => {
                self.historial_state.facturas = invs;
                Task::none()
            }
            Message::OpenCredits => {
                let _ = open::that("https://cacaoscript.com");
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if !self.is_unlocked {
            let header = header_view(self.mode, false, Message::Header);
            let pin_content = self.pin_state.view(self.mode, Message::Pin);
            let app_layout = column![header, pin_content]
                .width(Length::Fill)
                .height(Length::Fill);

            let base_container = container(app_layout)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(primary_container_style(self.mode));

            if let Some(toast) = &self.toast {
                let toast_element = toast.view(self.mode, Message::CloseToast);
                let overlay = container(toast_element)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::End)
                    .align_y(Alignment::End)
                    .padding(iced::Padding {
                        top: 0.0,
                        right: 20.0,
                        bottom: 20.0,
                        left: 0.0,
                    });
                return stack![base_container, overlay].into();
            } else {
                return base_container.into();
            }
        }

        let header = header_view(self.mode, self.is_unlocked, Message::Header);
        let sidebar = sidebar_view(
            self.mode,
            self.active_tab,
            Message::Navigate,
            Message::OpenCredits,
        );

        let main_content: Element<Message> = match self.active_tab {
            NavigationTab::Dashboard => dashboard_view(
                self.mode,
                self.historial_state.facturas.first(),
                self.inventario_state.productos.first(),
                &self.emisor_config,
                Message::Navigate,
            ),
            NavigationTab::Facturador => self.facturador_state.view(
                self.mode,
                &self.inventario_state.productos,
                Message::Facturador,
            ),
            NavigationTab::Inventario => self.inventario_state.view(self.mode, Message::Inventario),
            NavigationTab::Historial => self.historial_state.view(self.mode, Message::Historial),
            NavigationTab::Configuracion => self
                .configuracion_state
                .view(self.mode, Message::Configuracion),
        };

        let body = row![
            sidebar,
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill),
        ];

        let app_layout = column![header, body]
            .width(Length::Fill)
            .height(Length::Fill);

        let base_container = container(app_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(primary_container_style(self.mode));

        if let Some(toast) = &self.toast {
            let toast_element = toast.view(self.mode, Message::CloseToast);
            let overlay = container(toast_element)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::End)
                .align_y(Alignment::End)
                .padding(iced::Padding {
                    top: 0.0,
                    right: 20.0,
                    bottom: 20.0,
                    left: 0.0,
                });
            stack![base_container, overlay].into()
        } else {
            base_container.into()
        }
    }
}

async fn emitir_factura_proceso(
    emisor: EmisorConfig,
    req: FacturaRequest,
    password: Option<String>,
    pool: SqlitePool,
) -> Result<RespuestaSRI, String> {
    let fecha_limpia: String = req
        .fecha_emision
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    let cod_num = format!("{:0>8}", rand::random::<u32>() % 100_000_000);

    let clave_acceso = generar_clave_acceso(
        &fecha_limpia,
        "01",
        &emisor.ruc,
        &emisor.ambiente,
        &emisor.cod_establecimiento,
        &emisor.pto_emision,
        &req.secuencial,
        Some(&cod_num),
    )
    .map_err(|e| format!("Error generando clave de acceso: {}", e))?;

    let xml_sin_firma = construir_xml_factura(&emisor, &req, &clave_acceso)
        .map_err(|e| format!("Error construyendo XML: {}", e))?;

    let p12_path = emisor.p12_path.as_deref().ok_or_else(|| {
        "No se ha configurado el certificado digital .p12 en la sección Configuración".to_string()
    })?;

    let p12_bytes = std::fs::read(p12_path)
        .map_err(|e| format!("Error leyendo certificado '{}': {}", p12_path, e))?;

    let pass = password.as_deref().unwrap_or("");

    let xml_firmado = firmar_xml(&p12_bytes, pass, &xml_sin_firma)
        .map_err(|e| format!("Error firmando comprobante XAdES-BES: {}", e))?;

    let client = SriClient::new(&emisor.ambiente);
    let mut mensajes = Vec::new();

    let recepcion = client.enviar_recepcion(&xml_firmado).await.map_err(|e| {
        format!(
            "Error conectando con WebService de Recepción del SRI: {}",
            e
        )
    })?;

    for c in recepcion.comprobantes {
        for m in c.mensajes {
            mensajes.push(format!("[{}] {}: {}", m.tipo, m.identificador, m.mensaje));
        }
    }

    let (estado_final, num_aut, fecha_aut, xml_aut) = if recepcion.estado == "RECIBIDA" {
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        match client.consultar_autorizacion(&clave_acceso).await {
            Ok(aut) => {
                for m in aut.mensajes {
                    mensajes.push(format!("[{}] {}: {}", m.tipo, m.identificador, m.mensaje));
                }
                (
                    aut.estado,
                    aut.numero_autorizacion,
                    aut.fecha_autorizacion,
                    aut.xml_autorizado,
                )
            }
            Err(_) => ("RECIBIDA".to_string(), None, None, None),
        }
    } else {
        (recepcion.estado, None, None, None)
    };

    let total_sin_imp = req.detalles.iter().map(|d| d.subtotal()).sum();
    let total_iva = req.detalles.iter().map(|d| d.valor_iva()).sum();
    let total_fac = total_sin_imp + total_iva;

    let _ = guardar_factura_db(
        &pool,
        &clave_acceso,
        &req.secuencial,
        &req.fecha_emision,
        &req.cliente.identificacion,
        &req.cliente.razon_social,
        total_sin_imp,
        total_iva,
        total_fac,
        &estado_final,
        Some(&xml_firmado),
        xml_aut.as_deref(),
    )
    .await;

    if req.cliente.tipo_identificacion != "07" && !req.cliente.identificacion.is_empty() {
        let _ = crate::db::clientes::guardar_o_actualizar_cliente(
            &pool,
            &req.cliente.tipo_identificacion,
            &req.cliente.identificacion,
            &req.cliente.razon_social,
            req.cliente.direccion.as_deref(),
            req.cliente.email.as_deref(),
            req.cliente.telefono.as_deref(),
        )
        .await;
    }

    Ok(RespuestaSRI {
        estado: estado_final,
        clave_acceso,
        numero_autorizacion: num_aut,
        fecha_autorizacion: fecha_aut,
        ambiente: emisor.ambiente.clone(),
        mensajes,
        xml_firmado: Some(xml_firmado),
        xml_autorizado: xml_aut,
    })
}

pub fn run(pool: SqlitePool) -> iced::Result {
    let pool_app = pool.clone();
    iced::application(
        move || CacaoApp::new(pool_app.clone()),
        CacaoApp::update,
        CacaoApp::view,
    )
    .title(CacaoApp::title)
    .subscription(CacaoApp::subscription)
    .window(iced::window::Settings {
        size: Size::new(1150.0, 780.0),
        min_size: Some(Size::new(950.0, 650.0)),
        position: iced::window::Position::Centered,
        decorations: false,
        ..Default::default()
    })
    .run()
}
