//! Vista de Configuración del Emisor y Certificado Digital en Iced.

use crate::gui::icons::{icon_folder, icon_image, icon_save, icon_trash};
use crate::gui::theme::{
    ThemeMode, card_style, danger_button_style, pick_list_style, primary_button_style,
    secondary_button_style, text_input_style,
};
use crate::sri::models::EmisorConfig;
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbienteOption {
    Pruebas,
    Produccion,
}

impl std::fmt::Display for AmbienteOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AmbienteOption::Pruebas => write!(f, "1 - Pruebas (SRI Test)"),
            AmbienteOption::Produccion => write!(f, "2 - Producción (SRI Oficial)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObligadoContabilidadOption {
    No,
    Si,
}

impl std::fmt::Display for ObligadoContabilidadOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObligadoContabilidadOption::No => write!(f, "NO"),
            ObligadoContabilidadOption::Si => write!(f, "SI"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RimpeOption {
    Emprendedor,
    NegocioPopular,
    Ninguno,
}

impl std::fmt::Display for RimpeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RimpeOption::Emprendedor => write!(f, "CONTRIBUYENTE RÉGIMEN RIMPE"),
            RimpeOption::NegocioPopular => {
                write!(f, "CONTRIBUYENTE NEGOCIO POPULAR - RÉGIMEN RIMPE")
            }
            RimpeOption::Ninguno => write!(f, "NINGUNO (Régimen General)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlantillaPdfOption {
    Clasica,
    Moderna,
    Compacta,
}

impl std::fmt::Display for PlantillaPdfOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlantillaPdfOption::Clasica => write!(f, "1. Clásica (RIDE Oficial SRI)"),
            PlantillaPdfOption::Moderna => write!(f, "2. Moderna (Corporativa Cacao)"),
            PlantillaPdfOption::Compacta => write!(f, "3. Compacta (Minimalista Limpia)"),
        }
    }
}

impl PlantillaPdfOption {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlantillaPdfOption::Clasica => "clasica",
            PlantillaPdfOption::Moderna => "moderna",
            PlantillaPdfOption::Compacta => "compacta",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "moderna" => PlantillaPdfOption::Moderna,
            "compacta" => PlantillaPdfOption::Compacta,
            _ => PlantillaPdfOption::Clasica,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfiguracionState {
    pub ruc: String,
    pub razon_social: String,
    pub nombre_comercial: String,
    pub dir_matriz: String,
    pub dir_establecimiento: String,
    pub cod_establecimiento: String,
    pub pto_emision: String,
    pub obligado_contabilidad: Option<ObligadoContabilidadOption>,
    pub regimen_rimpe: Option<RimpeOption>,
    pub ambiente: Option<AmbienteOption>,
    pub p12_path: Option<String>,
    pub p12_password: String,
    pub pin: String,
    pub logo_path: Option<String>,
    pub plantilla_pdf: Option<PlantillaPdfOption>,
}

#[derive(Debug, Clone)]
pub enum ConfiguracionMessage {
    RucChanged(String),
    RazonSocialChanged(String),
    NombreComercialChanged(String),
    DirMatrizChanged(String),
    DirEstablecimientoChanged(String),
    CodEstablecimientoChanged(String),
    PtoEmisionChanged(String),
    ObligadoChanged(ObligadoContabilidadOption),
    RimpeChanged(RimpeOption),
    AmbienteChanged(AmbienteOption),
    SeleccionarP12,
    EliminarP12,
    P12PasswordChanged(String),
    PinChanged(String),
    SeleccionarLogo,
    EliminarLogo,
    PlantillaChanged(PlantillaPdfOption),
    Guardar,
}

impl ConfiguracionState {
    pub fn from_config(config: &EmisorConfig) -> Self {
        let obligado = if config.obligado_contabilidad.to_uppercase() == "SI" {
            Some(ObligadoContabilidadOption::Si)
        } else {
            Some(ObligadoContabilidadOption::No)
        };

        let rimpe = match config.regimen_rimpe.as_deref() {
            Some(r) if r.contains("POPULAR") => Some(RimpeOption::NegocioPopular),
            Some(r) if r.contains("RIMPE") => Some(RimpeOption::Emprendedor),
            _ => Some(RimpeOption::Ninguno),
        };

        let ambiente = if config.ambiente == "2" {
            Some(AmbienteOption::Produccion)
        } else {
            Some(AmbienteOption::Pruebas)
        };

        Self {
            ruc: config.ruc.clone(),
            razon_social: config.razon_social.clone(),
            nombre_comercial: config.nombre_comercial.clone().unwrap_or_default(),
            dir_matriz: config.dir_matriz.clone(),
            dir_establecimiento: config.dir_establecimiento.clone(),
            cod_establecimiento: config.cod_establecimiento.clone(),
            pto_emision: config.pto_emision.clone(),
            obligado_contabilidad: obligado,
            regimen_rimpe: rimpe,
            ambiente,
            p12_path: config.p12_path.clone(),
            p12_password: config.p12_password.clone().unwrap_or_default(),
            pin: config.pin.clone().unwrap_or_default(),
            logo_path: config.logo_path.clone(),
            plantilla_pdf: Some(PlantillaPdfOption::from_str(&config.plantilla_pdf)),
        }
    }

    pub fn to_emisor_config(&self) -> EmisorConfig {
        let obligado = match self.obligado_contabilidad {
            Some(ObligadoContabilidadOption::Si) => "SI".to_string(),
            _ => "NO".to_string(),
        };

        let rimpe = match self.regimen_rimpe {
            Some(RimpeOption::NegocioPopular) => {
                Some("CONTRIBUYENTE NEGOCIO POPULAR - RÉGIMEN RIMPE".to_string())
            }
            Some(RimpeOption::Emprendedor) => Some("CONTRIBUYENTE RÉGIMEN RIMPE".to_string()),
            _ => None,
        };

        let ambiente = match self.ambiente {
            Some(AmbienteOption::Produccion) => "2".to_string(),
            _ => "1".to_string(),
        };

        EmisorConfig {
            ruc: self.ruc.trim().to_string(),
            razon_social: self.razon_social.trim().to_string(),
            nombre_comercial: if self.nombre_comercial.trim().is_empty() {
                None
            } else {
                Some(self.nombre_comercial.trim().to_string())
            },
            dir_matriz: self.dir_matriz.trim().to_string(),
            dir_establecimiento: self.dir_establecimiento.trim().to_string(),
            cod_establecimiento: if self.cod_establecimiento.trim().is_empty() {
                "001".to_string()
            } else {
                format!("{:0>3}", self.cod_establecimiento.trim())
            },
            pto_emision: if self.pto_emision.trim().is_empty() {
                "001".to_string()
            } else {
                format!("{:0>3}", self.pto_emision.trim())
            },
            obligado_contabilidad: obligado,
            contribuyente_especial: None,
            regimen_microempresas: None,
            regimen_rimpe: rimpe,
            ambiente,
            p12_path: self.p12_path.clone(),
            p12_password: if self.p12_password.trim().is_empty() {
                None
            } else {
                Some(self.p12_password.trim().to_string())
            },
            pin: if self.pin.trim().is_empty() {
                None
            } else {
                Some(self.pin.trim().to_string())
            },
            logo_path: self.logo_path.clone(),
            plantilla_pdf: self
                .plantilla_pdf
                .unwrap_or(PlantillaPdfOption::Clasica)
                .as_str()
                .to_string(),
        }
    }

    pub fn update(&mut self, message: ConfiguracionMessage) -> Option<EmisorConfig> {
        match message {
            ConfiguracionMessage::RucChanged(val) => {
                self.ruc = val;
                None
            }
            ConfiguracionMessage::RazonSocialChanged(val) => {
                self.razon_social = val;
                None
            }
            ConfiguracionMessage::NombreComercialChanged(val) => {
                self.nombre_comercial = val;
                None
            }
            ConfiguracionMessage::DirMatrizChanged(val) => {
                self.dir_matriz = val;
                None
            }
            ConfiguracionMessage::DirEstablecimientoChanged(val) => {
                self.dir_establecimiento = val;
                None
            }
            ConfiguracionMessage::CodEstablecimientoChanged(val) => {
                self.cod_establecimiento = val;
                None
            }
            ConfiguracionMessage::PtoEmisionChanged(val) => {
                self.pto_emision = val;
                None
            }
            ConfiguracionMessage::ObligadoChanged(val) => {
                self.obligado_contabilidad = Some(val);
                None
            }
            ConfiguracionMessage::RimpeChanged(val) => {
                self.regimen_rimpe = Some(val);
                None
            }
            ConfiguracionMessage::AmbienteChanged(val) => {
                self.ambiente = Some(val);
                None
            }
            ConfiguracionMessage::SeleccionarP12 => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Certificado Digital PKCS#12", &["p12", "pfx"])
                    .pick_file()
                {
                    self.p12_path = Some(path.display().to_string());
                }
                None
            }
            ConfiguracionMessage::EliminarP12 => {
                self.p12_path = None;
                None
            }
            ConfiguracionMessage::P12PasswordChanged(val) => {
                self.p12_password = val;
                None
            }
            ConfiguracionMessage::PinChanged(val) => {
                let digits: String = val.chars().filter(|c| c.is_ascii_digit()).take(4).collect();
                self.pin = digits;
                None
            }
            ConfiguracionMessage::SeleccionarLogo => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(
                        "Imágenes (*.png, *.jpg, *.jpeg)",
                        &["png", "jpg", "jpeg", "PNG", "JPG", "JPEG"],
                    )
                    .set_title("Seleccionar Logotipo del Negocio")
                    .pick_file()
                {
                    self.logo_path = Some(path.display().to_string());
                }
                None
            }
            ConfiguracionMessage::EliminarLogo => {
                self.logo_path = None;
                None
            }
            ConfiguracionMessage::PlantillaChanged(val) => {
                self.plantilla_pdf = Some(val);
                None
            }
            ConfiguracionMessage::Guardar => Some(self.to_emisor_config()),
        }
    }

    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        mode: ThemeMode,
        on_message: fn(ConfiguracionMessage) -> Message,
    ) -> Element<'a, Message> {
        let p = mode.palette();

        let header = column![
            text("Configuración del Emisor SRI")
                .size(20)
                .color(p.text_main),
            text("Datos tributarios de tu negocio y certificado digital de firma electrónica")
                .size(13)
                .color(p.text_muted),
        ]
        .spacing(4);

        // Campos Emisor
        let row_emisor1 = row![
            column![
                text("RUC Emisor (13 dígitos):")
                    .size(12)
                    .color(p.text_muted),
                text_input("1790000000001", &self.ruc)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(ConfiguracionMessage::RucChanged(v)))
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text("Razón Social (según RUC):")
                    .size(12)
                    .color(p.text_muted),
                text_input("MI EMPRESA S.A.S.", &self.razon_social)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(ConfiguracionMessage::RazonSocialChanged(v)))
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12);

        let row_emisor2 = row![
            column![
                text("Nombre Comercial (Opcional):")
                    .size(12)
                    .color(p.text_muted),
                text_input("NOMBRE FANTASÍA", &self.nombre_comercial)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(ConfiguracionMessage::NombreComercialChanged(v)))
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text("Ambiente SRI:").size(12).color(p.text_muted),
                pick_list(
                    vec![AmbienteOption::Pruebas, AmbienteOption::Produccion],
                    self.ambiente,
                    move |v| on_message(ConfiguracionMessage::AmbienteChanged(v))
                )
                .style(pick_list_style(mode))
                .width(Length::Fill)
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12);

        let row_emisor3 = row![
            column![
                text("Dirección Matriz:").size(12).color(p.text_muted),
                text_input("Quito, Ecuador", &self.dir_matriz)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(ConfiguracionMessage::DirMatrizChanged(v)))
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text("Dirección Establecimiento:")
                    .size(12)
                    .color(p.text_muted),
                text_input("Quito, Ecuador", &self.dir_establecimiento)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(
                        ConfiguracionMessage::DirEstablecimientoChanged(v)
                    ))
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12);

        let row_emisor4 = row![
            column![
                text("Establecimiento:").size(12).color(p.text_muted),
                text_input("001", &self.cod_establecimiento)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(
                        ConfiguracionMessage::CodEstablecimientoChanged(v)
                    ))
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text("Punto Emisión:").size(12).color(p.text_muted),
                text_input("001", &self.pto_emision)
                    .style(text_input_style(mode))
                    .on_input(move |v| on_message(ConfiguracionMessage::PtoEmisionChanged(v)))
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text("Obligado Contabilidad:").size(12).color(p.text_muted),
                pick_list(
                    vec![
                        ObligadoContabilidadOption::No,
                        ObligadoContabilidadOption::Si
                    ],
                    self.obligado_contabilidad,
                    move |v| on_message(ConfiguracionMessage::ObligadoChanged(v))
                )
                .style(pick_list_style(mode))
                .width(Length::Fill)
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12);

        let row_emisor5 = column![
            text("Régimen RIMPE:").size(12).color(p.text_muted),
            pick_list(
                vec![
                    RimpeOption::Emprendedor,
                    RimpeOption::NegocioPopular,
                    RimpeOption::Ninguno
                ],
                self.regimen_rimpe,
                move |v| on_message(ConfiguracionMessage::RimpeChanged(v))
            )
            .style(pick_list_style(mode))
            .width(Length::Fill),
        ]
        .spacing(4);

        // Sección Certificado P12
        let p12_display = self
            .p12_path
            .as_deref()
            .unwrap_or("Ningún certificado cargado");
        let p12_actions = if self.p12_path.is_some() {
            row![
                button(
                    row![
                        icon_folder(p.text_main, 14.0),
                        text("Cambiar .p12").size(12),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .style(secondary_button_style(mode))
                .padding([6, 12])
                .on_press(on_message(ConfiguracionMessage::SeleccionarP12)),
                button(
                    row![
                        icon_trash(p.danger, 14.0),
                        text("Quitar").size(12).color(p.danger),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .style(danger_button_style(mode))
                .padding([6, 12])
                .on_press(on_message(ConfiguracionMessage::EliminarP12)),
            ]
            .spacing(8)
        } else {
            row![
                button(
                    row![
                        icon_folder(Color::WHITE, 14.0),
                        text("Seleccionar Archivo .p12").size(12),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .style(primary_button_style(mode))
                .padding([6, 14])
                .on_press(on_message(ConfiguracionMessage::SeleccionarP12)),
            ]
        };

        let cert_box = container(
            column![
                text("CERTIFICADO DIGITAL DE FIRMA ELECTRÓNICA (.p12 / .pfx)")
                    .size(13)
                    .color(p.accent),
                row![
                    column![
                        text("Archivo de Firma:").size(12).color(p.text_muted),
                        text(p12_display).size(12).color(p.text_main),
                    ]
                    .spacing(4)
                    .width(Length::Fill),
                    p12_actions,
                ]
                .align_y(Alignment::Center),
                column![
                    text("Contraseña del Certificado (Opcional si deseas guardarla):")
                        .size(12)
                        .color(p.text_muted),
                    text_input("••••••••", &self.p12_password)
                        .secure(true)
                        .style(text_input_style(mode))
                        .on_input(move |v| on_message(ConfiguracionMessage::P12PasswordChanged(v)))
                ]
                .spacing(4),
            ]
            .spacing(12),
        )
        .padding(16)
        .style(card_style(mode));

        // Sección Logo y Plantillas de Factura PDF
        let logo_display = self
            .logo_path
            .as_deref()
            .unwrap_or("Ningún logo cargado (Se usará identificación en texto)");
        let logo_actions = if self.logo_path.is_some() {
            row![
                button(
                    row![icon_folder(p.text_main, 14.0), text("Cambiar").size(12),]
                        .spacing(6)
                        .align_y(Alignment::Center),
                )
                .style(secondary_button_style(mode))
                .padding([6, 12])
                .on_press(on_message(ConfiguracionMessage::SeleccionarLogo)),
                button(
                    row![
                        icon_trash(p.danger, 14.0),
                        text("Quitar").size(12).color(p.danger),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .style(danger_button_style(mode))
                .padding([6, 12])
                .on_press(on_message(ConfiguracionMessage::EliminarLogo)),
            ]
            .spacing(8)
        } else {
            row![
                button(
                    row![
                        icon_image(Color::WHITE, 14.0),
                        text("Seleccionar Logo (PNG / JPG)").size(12),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .style(primary_button_style(mode))
                .padding([6, 14])
                .on_press(on_message(ConfiguracionMessage::SeleccionarLogo)),
            ]
        };

        let plantilla_desc = match self.plantilla_pdf {
            Some(PlantillaPdfOption::Moderna) => {
                "Plantilla Moderna: Cabecera con franja Cacao, logo destacado, filas alternadas elegantes y tarjeta de total destacada."
            }
            Some(PlantillaPdfOption::Compacta) => {
                "Plantilla Compacta: Formato minimalista de ancho completo y tipografía eficiente. Ideal para ahorro de papel y tinta."
            }
            _ => {
                "Plantilla Clásica: Formato oficial reglamentario del SRI en dos columnas con recuadros estándar y código de barras."
            }
        };

        let plantilla_box = container(
            column![
                text("PLANTILLAS Y PERSONALIZACIÓN DE FACTURA (PDF)")
                    .size(13)
                    .color(p.accent),
                row![
                    column![
                        text("Logotipo del Negocio (Encabezado de Comprobantes):")
                            .size(12)
                            .color(p.text_muted),
                        text(logo_display).size(12).color(p.text_main),
                    ]
                    .spacing(4)
                    .width(Length::Fill),
                    logo_actions,
                ]
                .align_y(Alignment::Center),
                column![
                    text("Modelo de Plantilla PDF (RIDE):")
                        .size(12)
                        .color(p.text_muted),
                    pick_list(
                        vec![
                            PlantillaPdfOption::Clasica,
                            PlantillaPdfOption::Moderna,
                            PlantillaPdfOption::Compacta,
                        ],
                        self.plantilla_pdf,
                        move |v| on_message(ConfiguracionMessage::PlantillaChanged(v))
                    )
                    .style(pick_list_style(mode))
                    .width(Length::Fill),
                    text(plantilla_desc).size(11).color(p.text_muted),
                ]
                .spacing(6),
            ]
            .spacing(14),
        )
        .padding(16)
        .style(card_style(mode));

        let pin_box = container(
            column![
                text("SEGURIDAD Y CONTROL DE ACCESO (PIN)")
                    .size(13)
                    .color(p.accent),
                column![
                    text("PIN de Bloqueo de la App (4 dígitos numéricos):")
                        .size(12)
                        .color(p.text_muted),
                    text_input("Ej: 1234", &self.pin)
                        .secure(true)
                        .style(text_input_style(mode))
                        .on_input(move |v| on_message(ConfiguracionMessage::PinChanged(v)))
                ]
                .spacing(4),
            ]
            .spacing(12),
        )
        .padding(16)
        .style(card_style(mode));

        let save_button = button(
            row![
                icon_save(Color::WHITE, 16.0),
                text("Guardar Configuración").size(14),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .style(primary_button_style(mode))
        .padding([10, 24])
        .on_press(on_message(ConfiguracionMessage::Guardar));

        let content = column![
            header,
            container(
                column![
                    text("DATOS TRIBUTARIOS DEL EMISOR")
                        .size(13)
                        .color(p.accent),
                    row_emisor1,
                    row_emisor2,
                    row_emisor3,
                    row_emisor4,
                    row_emisor5,
                ]
                .spacing(12),
            )
            .padding(16)
            .style(card_style(mode)),
            cert_box,
            plantilla_box,
            pin_box,
            row![save_button].align_y(Alignment::Center),
        ]
        .spacing(16);

        scrollable(container(content).padding(24).width(Length::Fill)).into()
    }
}
