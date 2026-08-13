//! Cliente HTTP SOAP para comunicación directa con los Webservices del SRI Ecuador.

use base64::Engine;
use reqwest::Client;
use std::time::Duration;

pub const SRI_RECEPCION_PRUEBAS: &str =
    "https://celcer.sri.gob.ec/comprobantes-electronicos-ws/RecepcionComprobantesOffline";
pub const SRI_AUTORIZACION_PRUEBAS: &str =
    "https://celcer.sri.gob.ec/comprobantes-electronicos-ws/AutorizacionComprobantesOffline";

pub const SRI_RECEPCION_PRODUCCION: &str =
    "https://cel.sri.gob.ec/comprobantes-electronicos-ws/RecepcionComprobantesOffline";
pub const SRI_AUTORIZACION_PRODUCCION: &str =
    "https://cel.sri.gob.ec/comprobantes-electronicos-ws/AutorizacionComprobantesOffline";

#[derive(Debug, Clone)]
pub struct SriClient {
    client: Client,
    pub ambiente: String, // "1" Pruebas, "2" Producción
}

#[derive(Debug, Clone)]
pub struct ResultadoRecepcion {
    pub estado: String, // "DEVUELTA", "RECIBIDA", "ERROR"
    pub comprobantes: Vec<ComprobanteRespuesta>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ComprobanteRespuesta {
    pub clave_acceso: String,
    pub mensajes: Vec<MensajeSRI>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MensajeSRI {
    pub identificador: String,
    pub mensaje: String,
    pub informacion_adicional: Option<String>,
    pub tipo: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ResultadoAutorizacion {
    pub estado: String, // "AUTORIZADO", "NO AUTORIZADO", "EN PROCESO", "ERROR"
    pub numero_autorizacion: Option<String>,
    pub fecha_autorizacion: Option<String>,
    pub ambiente: String,
    pub mensajes: Vec<MensajeSRI>,
    pub xml_autorizado: Option<String>,
}

impl SriClient {
    pub fn new(ambiente: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            client,
            ambiente: ambiente.to_string(),
        }
    }

    fn url_recepcion(&self) -> &'static str {
        if self.ambiente == "2" {
            SRI_RECEPCION_PRODUCCION
        } else {
            SRI_RECEPCION_PRUEBAS
        }
    }

    fn url_autorizacion(&self) -> &'static str {
        if self.ambiente == "2" {
            SRI_AUTORIZACION_PRODUCCION
        } else {
            SRI_AUTORIZACION_PRUEBAS
        }
    }

    /// Envía el XML firmado al WebService de Recepción del SRI.
    pub async fn enviar_recepcion(&self, xml_firmado: &str) -> Result<ResultadoRecepcion, String> {
        let xml_b64 = base64::engine::general_purpose::STANDARD.encode(xml_firmado.as_bytes());

        let soap_payload = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
            <soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:ec=\"http://ec.gob.sri.ws.recepcion\">\n\
            <soapenv:Header/>\n\
            <soapenv:Body>\n\
            <ec:validarComprobante>\n\
            <xml>{}</xml>\n\
            </ec:validarComprobante>\n\
            </soapenv:Body>\n\
            </soapenv:Envelope>",
            xml_b64
        );

        let res = self
            .client
            .post(self.url_recepcion())
            .header("Content-Type", "text/xml;charset=UTF-8")
            .body(soap_payload)
            .send()
            .await
            .map_err(|e| format!("Error en comunicación con Recepción SRI: {}", e))?;

        let res_text = res
            .text()
            .await
            .map_err(|e| format!("Error leyendo respuesta del SRI: {}", e))?;

        parse_respuesta_recepcion(&res_text)
    }

    /// Consulta el estado del comprobante en el WebService de Autorización del SRI por su Clave de Acceso.
    pub async fn consultar_autorizacion(&self, clave_acceso: &str) -> Result<ResultadoAutorizacion, String> {
        let soap_payload = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
            <soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:ec=\"http://ec.gob.sri.ws.autorizacion\">\n\
            <soapenv:Header/>\n\
            <soapenv:Body>\n\
            <ec:autorizacionComprobante>\n\
            <claveAccesoComprobante>{}</claveAccesoComprobante>\n\
            </ec:autorizacionComprobante>\n\
            </soapenv:Body>\n\
            </soapenv:Envelope>",
            clave_acceso
        );

        let res = self
            .client
            .post(self.url_autorizacion())
            .header("Content-Type", "text/xml;charset=UTF-8")
            .body(soap_payload)
            .send()
            .await
            .map_err(|e| format!("Error en comunicación con Autorización SRI: {}", e))?;

        let res_text = res
            .text()
            .await
            .map_err(|e| format!("Error leyendo respuesta del SRI: {}", e))?;

        parse_respuesta_autorizacion(&res_text)
    }
}

fn parse_respuesta_recepcion(xml_response: &str) -> Result<ResultadoRecepcion, String> {
    let estado = if xml_response.contains("<estado>RECIBIDA</estado>") {
        "RECIBIDA".to_string()
    } else if xml_response.contains("<estado>DEVUELTA</estado>") {
        "DEVUELTA".to_string()
    } else {
        "ERROR".to_string()
    };

    let clave_acceso = extract_tag(xml_response, "claveAcceso").unwrap_or_default();
    let mut mensajes = parse_sri_mensajes(xml_response);

    if mensajes.is_empty() && (estado == "DEVUELTA" || estado == "ERROR") {
        if let Some(fault) = extract_tag(xml_response, "faultstring") {
            mensajes.push(MensajeSRI {
                identificador: "SOAP_FAULT".to_string(),
                mensaje: fault,
                informacion_adicional: None,
                tipo: "ERROR".to_string(),
            });
        } else if let Some(msg) = extract_tag(xml_response, "mensaje") {
            mensajes.push(MensajeSRI {
                identificador: "SRI_MSG".to_string(),
                mensaje: msg,
                informacion_adicional: None,
                tipo: "ERROR".to_string(),
            });
        } else {
            mensajes.push(MensajeSRI {
                identificador: "DESCONOCIDO".to_string(),
                mensaje: format!("Respuesta del SRI (Estado {}): {}", estado, xml_response),
                informacion_adicional: None,
                tipo: "ERROR".to_string(),
            });
        }
    }

    Ok(ResultadoRecepcion {
        estado,
        comprobantes: vec![ComprobanteRespuesta {
            clave_acceso,
            mensajes,
        }],
    })
}

fn parse_respuesta_autorizacion(xml_response: &str) -> Result<ResultadoAutorizacion, String> {
    let estado = if xml_response.contains("<estado>AUTORIZADO</estado>") {
        "AUTORIZADO".to_string()
    } else if xml_response.contains("<estado>NO AUTORIZADO</estado>") {
        "NO AUTORIZADO".to_string()
    } else if xml_response.contains("<estado>EN PROCESO</estado>") {
        "EN PROCESO".to_string()
    } else if xml_response.contains("<numeroComprobantes>0</numeroComprobantes>") || xml_response.contains("<autorizaciones/>") {
        "EN PROCESO".to_string()
    } else {
        "ERROR".to_string()
    };

    let numero_autorizacion = extract_tag(xml_response, "numeroAutorizacion");
    let fecha_autorizacion = extract_tag(xml_response, "fechaAutorizacion");
    let ambiente = extract_tag(xml_response, "ambiente").unwrap_or_else(|| "1".to_string());
    let mut mensajes = parse_sri_mensajes(xml_response);

    if mensajes.is_empty() && (estado == "NO AUTORIZADO" || estado == "ERROR") {
        if let Some(fault) = extract_tag(xml_response, "faultstring") {
            mensajes.push(MensajeSRI {
                identificador: "SOAP_FAULT".to_string(),
                mensaje: fault,
                informacion_adicional: None,
                tipo: "ERROR".to_string(),
            });
        } else {
            mensajes.push(MensajeSRI {
                identificador: "DESCONOCIDO".to_string(),
                mensaje: format!("Respuesta de Autorización SRI (Estado {}): {}", estado, xml_response),
                informacion_adicional: None,
                tipo: "ERROR".to_string(),
            });
        }
    }

    let xml_autorizado = extract_tag(xml_response, "comprobante");

    Ok(ResultadoAutorizacion {
        estado,
        numero_autorizacion,
        fecha_autorizacion,
        ambiente,
        mensajes,
        xml_autorizado,
    })
}

fn parse_sri_mensajes(xml: &str) -> Vec<MensajeSRI> {
    let mut mensajes = Vec::new();
    let mut pos = 0;

    while let Some(start) = xml[pos..].find("<identificador>") {
        let abs_start = pos + start;
        if let Some(end) = xml[abs_start..].find("</tipo>") {
            let snippet = &xml[abs_start..abs_start + end + 7];
            let id = extract_tag(snippet, "identificador").unwrap_or_default();
            let msg = extract_tag(snippet, "mensaje").unwrap_or_default();
            let info = extract_tag(snippet, "informacionAdicional");
            let tipo = extract_tag(snippet, "tipo").unwrap_or_else(|| "ERROR".to_string());

            if !msg.is_empty() || !id.is_empty() {
                mensajes.push(MensajeSRI {
                    identificador: id,
                    mensaje: msg,
                    informacion_adicional: info,
                    tipo,
                });
            }
            pos = abs_start + end + 7;
        } else {
            pos = abs_start + 15;
        }
    }

    mensajes
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag);
    let close_tag = format!("</{}>", tag);

    if let Some(start) = xml.find(&open_tag) {
        let content_start = start + open_tag.len();
        if let Some(end) = xml[content_start..].find(&close_tag) {
            return Some(xml[content_start..content_start + end].trim().to_string());
        }
    }
    None
}
