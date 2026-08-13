//! Modelos de datos para el Emisor, Facturas del SRI y respuestas SOAP.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmisorConfig {
    pub ruc: String,
    pub razon_social: String,
    pub nombre_comercial: Option<String>,
    pub dir_matriz: String,
    pub dir_establecimiento: String,
    pub cod_establecimiento: String, // ej: "001"
    pub pto_emision: String,         // ej: "001"
    pub obligado_contabilidad: String, // "SI" o "NO"
    pub contribuyente_especial: Option<String>,
    pub regimen_microempresas: Option<String>,
    pub regimen_rimpe: Option<String>, // "CONTRIBUYENTE REGIMEN RIMPE" o "CONTRIBUYENTE NEGOCIO POPULAR - REGIMEN RIMPE"
    pub ambiente: String,            // "1" (Pruebas), "2" (Producción)
    pub p12_path: Option<String>,
    pub p12_password: Option<String>,
}

impl Default for EmisorConfig {
    fn default() -> Self {
        Self {
            ruc: "1790000000001".to_string(),
            razon_social: "MI EMPRESA S.A.".to_string(),
            nombre_comercial: Some("MI EMPRESA".to_string()),
            dir_matriz: "Quito, Ecuador".to_string(),
            dir_establecimiento: "Quito, Ecuador".to_string(),
            cod_establecimiento: "001".to_string(),
            pto_emision: "001".to_string(),
            obligado_contabilidad: "NO".to_string(),
            contribuyente_especial: None,
            regimen_microempresas: None,
            regimen_rimpe: Some("CONTRIBUYENTE RÉGIMEN RIMPE".to_string()),
            ambiente: "1".to_string(), // Pruebas por defecto
            p12_path: None,
            p12_password: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClienteInfo {
    pub tipo_identificacion: String, // "04" RUC, "05" Cédula, "06" Pasaporte, "07" Consumidor Final, "08" Identificación Exterior
    pub razon_social: String,
    pub identificacion: String,
    pub direccion: Option<String>,
    pub email: Option<String>,
    pub telefono: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetalleFactura {
    pub codigo_principal: String,
    pub codigo_auxiliar: Option<String>,
    pub descripcion: String,
    pub cantidad: f64,
    pub precio_unitario: f64,
    pub descuento: f64,
    pub codigo_porcentaje_iva: String, // "0" (0%), "2" (12%), "4" (15%), "10" (13%), "6" (No Objeto), "7" (Exento)
    pub tarifa_iva: f64,               // 15.0, 0.0, etc.
}

impl DetalleFactura {
    pub fn subtotal(&self) -> f64 {
        (self.cantidad * self.precio_unitario) - self.descuento
    }

    pub fn valor_iva(&self) -> f64 {
        self.subtotal() * (self.tarifa_iva / 100.0)
    }

    #[allow(dead_code)]
    pub fn total(&self) -> f64 {
        self.subtotal() + self.valor_iva()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormaPago {
    pub forma_pago: String, // "01" Sin utilización del sistema financiero, "19" Tarjeta de crédito, "20" Otros
    pub total: f64,
    pub plazo: Option<u32>,
    pub unidad_tiempo: Option<String>, // "dias", "meses"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacturaRequest {
    pub secuencial: String, // ej: "000000001"
    pub fecha_emision: String, // "DD/MM/AAAA"
    pub cliente: ClienteInfo,
    pub detalles: Vec<DetalleFactura>,
    pub formas_pago: Vec<FormaPago>,
    pub propina: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespuestaSRI {
    pub estado: String, // "RECIBIDA", "DEVUELTA", "AUTORIZADO", "NO AUTORIZADO", "ERROR"
    pub clave_acceso: String,
    pub numero_autorizacion: Option<String>,
    pub fecha_autorizacion: Option<String>,
    pub ambiente: String,
    pub mensajes: Vec<String>,
    pub xml_firmado: Option<String>,
    pub xml_autorizado: Option<String>,
}
