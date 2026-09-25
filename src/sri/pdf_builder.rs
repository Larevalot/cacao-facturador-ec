//! Motor de generación de Comprobantes RIDE en PDF (3 plantillas) para Facturación Electrónica SRI.
//!
//! Soporta:
//! - 3 Modelos visuales: Clásica (RIDE Oficial SRI), Moderna (Corporativa Cacao) y Compacta (Minimalista).
//! - Inserción y escalado de logo corporativo (PNG / JPG / JPEG).
//! - Código de barras Code 128 vectorial matemáticamente exacto y nítido.
//! - Desglose de impuestos (IVA 15%, 0%, no objeto, exento), descuentos, formas de pago y datos de autorización.

use crate::sri::models::{
    ClienteInfo, DetalleFactura, EmisorConfig, FacturaRequest, FormaPago, RespuestaSRI,
};
use barcoders::sym::code128::Code128;
use printpdf::*;
use std::fs;
use std::path::Path;

/// Estructura unificada con todos los datos necesarios para renderizar el RIDE.
#[derive(Debug, Clone)]
pub struct DatosRide {
    pub emisor: EmisorConfig,
    pub secuencial: String,
    pub fecha_emision: String,
    pub clave_acceso: String,
    pub numero_autorizacion: String,
    pub fecha_autorizacion: String,
    pub ambiente: String, // "PRUEBAS" o "PRODUCCIÓN"
    pub emision: String,  // "NORMAL"
    pub cliente: ClienteInfo,
    pub detalles: Vec<DetalleFactura>,
    pub formas_pago: Vec<FormaPago>,
    pub propina: f64,
    pub guia_remision: Option<String>,
}

impl DatosRide {
    pub fn from_request(req: &FacturaRequest, resp: &RespuestaSRI, emisor: &EmisorConfig) -> Self {
        let ambiente_str = if emisor.ambiente == "2" {
            "PRODUCCIÓN".to_string()
        } else {
            "PRUEBAS".to_string()
        };

        let num_auto = resp
            .numero_autorizacion
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| resp.clave_acceso.clone());

        let fecha_auto = resp
            .fecha_autorizacion
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| req.fecha_emision.clone());

        Self {
            emisor: emisor.clone(),
            secuencial: req.secuencial.clone(),
            fecha_emision: req.fecha_emision.clone(),
            clave_acceso: resp.clave_acceso.clone(),
            numero_autorizacion: num_auto,
            fecha_autorizacion: fecha_auto,
            ambiente: ambiente_str,
            emision: "NORMAL".to_string(),
            cliente: req.cliente.clone(),
            detalles: req.detalles.clone(),
            formas_pago: req.formas_pago.clone(),
            propina: req.propina,
            guia_remision: req.guia_remision.clone(),
        }
    }

    /// Calcula totales desglosados para el RIDE.
    pub fn calcular_totales(&self) -> TotalesRide {
        let mut subtotal_15 = 0.0;
        let mut subtotal_0 = 0.0;
        let mut subtotal_no_objeto = 0.0;
        let mut subtotal_exento = 0.0;
        let mut total_descuento = 0.0;
        let mut total_iva = 0.0;

        for d in &self.detalles {
            let sub = d.subtotal();
            total_descuento += d.descuento;

            match d.codigo_porcentaje_iva.as_str() {
                "0" => subtotal_0 += sub,
                "6" => subtotal_no_objeto += sub,
                "7" => subtotal_exento += sub,
                _ => {
                    subtotal_15 += sub;
                    total_iva += d.valor_iva();
                }
            }
        }

        let subtotal_sin_impuestos =
            subtotal_15 + subtotal_0 + subtotal_no_objeto + subtotal_exento;
        let valor_total = subtotal_sin_impuestos + total_iva + self.propina;

        TotalesRide {
            subtotal_15,
            subtotal_0,
            subtotal_no_objeto,
            subtotal_exento,
            subtotal_sin_impuestos,
            total_descuento,
            total_iva,
            propina: self.propina,
            valor_total,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TotalesRide {
    pub subtotal_15: f64,
    pub subtotal_0: f64,
    pub subtotal_no_objeto: f64,
    pub subtotal_exento: f64,
    pub subtotal_sin_impuestos: f64,
    pub total_descuento: f64,
    pub total_iva: f64,
    pub propina: f64,
    pub valor_total: f64,
}

/// Extrae un campo de la sección <infoAdicional> por uno o más posibles nombres.
pub fn extraer_campo_adicional(xml: &str, nombres: &[&str]) -> Option<String> {
    let mut rest = xml;
    while let Some(start) = rest.find("<campoAdicional") {
        rest = &rest[start + 15..];
        let tag_end = rest.find('>')?;
        let tag_header = &rest[..tag_end];
        rest = &rest[tag_end + 1..];
        let val_end = rest.find("</campoAdicional>")?;
        let val = &rest[..val_end];
        rest = &rest[val_end + 17..];

        for &nom in nombres {
            let p1 = format!("nombre=\"{}\"", nom).to_lowercase();
            let p2 = format!("nombre='{}'", nom).to_lowercase();
            let h_lower = tag_header.to_lowercase();
            if h_lower.contains(&p1) || h_lower.contains(&p2) {
                let clean = val.trim();
                if !clean.is_empty() {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

/// Extrae los datos necesarios para generar el RIDE a partir de un archivo XML del SRI.
pub fn extraer_datos_ride_de_xml(
    xml: &str,
    emisor_default: &EmisorConfig,
) -> Result<DatosRide, String> {
    // Si el XML viene escapado (&lt; y &gt;) desde el WebService de autorización, lo desescapamos.
    let xml_clean = if xml.contains("&lt;") || xml.contains("&gt;") {
        crate::sri::xml_builder::unescape_xml(xml)
    } else {
        xml.to_string()
    };
    let xml = xml_clean.as_str();

    let mut emisor = emisor_default.clone();

    if let Some(ruc) = extraer_tag(xml, "ruc") {
        emisor.ruc = ruc;
    }
    if let Some(razon) = extraer_tag(xml, "razonSocial") {
        emisor.razon_social = razon;
    }
    if let Some(nom_com) = extraer_tag(xml, "nombreComercial") {
        emisor.nombre_comercial = Some(nom_com);
    }
    if let Some(dir_matriz) = extraer_tag(xml, "dirMatriz") {
        emisor.dir_matriz = dir_matriz;
    }
    if let Some(dir_est) = extraer_tag(xml, "dirEstablecimiento") {
        emisor.dir_establecimiento = dir_est;
    }
    if let Some(estab) = extraer_tag(xml, "estab") {
        emisor.cod_establecimiento = estab;
    }
    if let Some(pto) = extraer_tag(xml, "ptoEmi") {
        emisor.pto_emision = pto;
    }
    if let Some(obl) = extraer_tag(xml, "obligadoContabilidad") {
        emisor.obligado_contabilidad = obl;
    }
    if let Some(contrib) = extraer_tag(xml, "contribuyenteEspecial") {
        emisor.contribuyente_especial = Some(contrib);
    }
    if let Some(rimpe) = extraer_tag(xml, "regimenRimpe") {
        emisor.regimen_rimpe = Some(rimpe);
    }

    let secuencial = extraer_tag(xml, "secuencial").unwrap_or_else(|| "000000001".to_string());
    let fecha_emision =
        extraer_tag(xml, "fechaEmision").unwrap_or_else(|| "01/01/2026".to_string());
    let clave_acceso = extraer_tag(xml, "claveAcceso").unwrap_or_default();
    let num_auto = extraer_tag(xml, "numeroAutorizacion")
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| clave_acceso.clone());
    let fecha_auto = extraer_tag(xml, "fechaAutorizacion")
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| fecha_emision.clone());
    let guia_remision = extraer_tag(xml, "guiaRemision").or_else(|| {
        extraer_campo_adicional(xml, &["GuiaRemision", "Guía de Remisión", "Guia", "guia"])
    });

    let amb_cod = extraer_tag(xml, "ambiente").unwrap_or_else(|| "1".to_string());
    let ambiente = if amb_cod == "2" {
        "PRODUCCIÓN".to_string()
    } else {
        "PRUEBAS".to_string()
    };

    let cliente_razon =
        extraer_tag(xml, "razonSocialComprador").unwrap_or_else(|| "CONSUMIDOR FINAL".to_string());
    let cliente_id =
        extraer_tag(xml, "identificacionComprador").unwrap_or_else(|| "9999999999999".to_string());
    let cliente_tipo_id =
        extraer_tag(xml, "tipoIdentificacionComprador").unwrap_or_else(|| "07".to_string());
    let cliente_dir = extraer_tag(xml, "direccionComprador").or_else(|| {
        extraer_campo_adicional(
            xml,
            &[
                "Direccion",
                "Dirección",
                "DireccionComprador",
                "DirComprador",
                "direccion",
            ],
        )
    });

    let propina = extraer_tag(xml, "propina")
        .and_then(|p| p.parse::<f64>().ok())
        .unwrap_or(0.0);

    let email = extraer_tag(xml, "email").or_else(|| {
        extraer_campo_adicional(
            xml,
            &["Email", "Correo", "CorreoElectronico", "E-mail", "email"],
        )
    });
    let telefono = extraer_tag(xml, "telefono").or_else(|| {
        extraer_campo_adicional(
            xml,
            &["Telefono", "Teléfono", "Celular", "Telf", "telefono"],
        )
    });

    let cliente = ClienteInfo {
        tipo_identificacion: cliente_tipo_id,
        razon_social: cliente_razon,
        identificacion: cliente_id,
        direccion: cliente_dir,
        email,
        telefono,
    };

    // Extraer detalles
    let mut detalles = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<detalle>") {
        rest = &rest[start + 9..];
        let end = match rest.find("</detalle>") {
            Some(e) => e,
            None => break,
        };
        let det_str = &rest[..end];
        rest = &rest[end + 10..];

        let cod_principal =
            extraer_tag(det_str, "codigoPrincipal").unwrap_or_else(|| "ITEM".to_string());
        let descripcion =
            extraer_tag(det_str, "descripcion").unwrap_or_else(|| "Producto".to_string());
        let cantidad = extraer_tag(det_str, "cantidad")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(1.0);
        let precio_unitario = extraer_tag(det_str, "precioUnitario")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);
        let descuento = extraer_tag(det_str, "descuento")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);
        let codigo_porcentaje_iva =
            extraer_tag(det_str, "codigoPorcentaje").unwrap_or_else(|| "4".to_string());
        let tarifa_iva = extraer_tag(det_str, "tarifa")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(15.0);

        detalles.push(DetalleFactura {
            codigo_principal: cod_principal,
            codigo_auxiliar: None,
            descripcion,
            cantidad,
            precio_unitario,
            descuento,
            codigo_porcentaje_iva,
            tarifa_iva,
        });
    }

    // Extraer formas de pago
    let mut formas_pago = Vec::new();
    let mut rest_pagos = xml;
    while let Some(start) = rest_pagos.find("<pago>") {
        rest_pagos = &rest_pagos[start + 6..];
        let end = match rest_pagos.find("</pago>") {
            Some(e) => e,
            None => break,
        };
        let pago_str = &rest_pagos[..end];
        rest_pagos = &rest_pagos[end + 7..];

        let forma_pago = extraer_tag(pago_str, "formaPago").unwrap_or_else(|| "01".to_string());
        let total = extraer_tag(pago_str, "total")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);
        let plazo = extraer_tag(pago_str, "plazo").and_then(|v| v.parse::<u32>().ok());
        let unidad_tiempo = extraer_tag(pago_str, "unidadTiempo");

        formas_pago.push(FormaPago {
            forma_pago,
            total,
            plazo,
            unidad_tiempo,
        });
    }

    if formas_pago.is_empty() {
        let total_factura = extraer_tag(xml, "importeTotal")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);
        formas_pago.push(FormaPago {
            forma_pago: "01".to_string(),
            total: total_factura,
            plazo: None,
            unidad_tiempo: None,
        });
    }

    Ok(DatosRide {
        emisor,
        secuencial,
        fecha_emision,
        clave_acceso,
        numero_autorizacion: num_auto,
        fecha_autorizacion: fecha_auto,
        ambiente,
        emision: "NORMAL".to_string(),
        cliente,
        detalles,
        formas_pago,
        propina,
        guia_remision,
    })
}

fn extraer_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].trim().to_string())
}

/// Genera los bytes del PDF del RIDE según la plantilla indicada ("clasica", "moderna", "compacta").
pub fn generar_ride_pdf(datos: &DatosRide, plantilla: &str) -> Result<Vec<u8>, String> {
    match plantilla.to_lowercase().as_str() {
        "moderna" => render_moderna(datos),
        "compacta" => render_compacta(datos),
        _ => render_clasica(datos),
    }
}

// ---------------------------------------------------------------------------
// Helpers de Dibujo y Geometría en printpdf 0.12.8
// ---------------------------------------------------------------------------

fn rgb_color(r: f32, g: f32, b: f32) -> Color {
    Color::Rgb(Rgb {
        r,
        g,
        b,
        icc_profile: None,
    })
}

fn set_fill(ops: &mut Vec<Op>, col: &Color) {
    ops.push(Op::SetFillColor { col: col.clone() });
}

fn push_text(ops: &mut Vec<Op>, font: &PdfFontHandle, size: f32, x: f32, y: f32, txt: &str) {
    if txt.trim().is_empty() {
        return;
    }
    ops.push(Op::StartTextSection);
    ops.push(Op::SetFont {
        font: font.clone(),
        size: Pt(size),
    });
    ops.push(Op::SetTextMatrix {
        matrix: TextMatrix::Translate(Pt(x), Pt(y)),
    });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(txt.to_string())],
    });
    ops.push(Op::EndTextSection);
}

fn push_rect_fill(ops: &mut Vec<Op>, x: f32, y: f32, w: f32, h: f32, col: &Color) {
    ops.push(Op::SetFillColor { col: col.clone() });
    let polygon = Polygon {
        rings: vec![PolygonRing {
            points: vec![
                LinePoint {
                    p: Point { x: Pt(x), y: Pt(y) },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: Pt(x + w),
                        y: Pt(y),
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: Pt(x + w),
                        y: Pt(y + h),
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: Pt(x),
                        y: Pt(y + h),
                    },
                    bezier: false,
                },
            ],
        }],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    };
    ops.push(Op::DrawPolygon { polygon });
}

fn push_rect_stroke(
    ops: &mut Vec<Op>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    col: &Color,
    thickness: f32,
) {
    ops.push(Op::SetOutlineColor { col: col.clone() });
    ops.push(Op::SetOutlineThickness { pt: Pt(thickness) });
    let polygon = Polygon {
        rings: vec![PolygonRing {
            points: vec![
                LinePoint {
                    p: Point { x: Pt(x), y: Pt(y) },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: Pt(x + w),
                        y: Pt(y),
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: Pt(x + w),
                        y: Pt(y + h),
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: Pt(x),
                        y: Pt(y + h),
                    },
                    bezier: false,
                },
            ],
        }],
        mode: PaintMode::Stroke,
        winding_order: WindingOrder::NonZero,
    };
    ops.push(Op::DrawPolygon { polygon });
}

fn push_line(ops: &mut Vec<Op>, x1: f32, y1: f32, x2: f32, y2: f32, col: &Color, thickness: f32) {
    ops.push(Op::SetOutlineColor { col: col.clone() });
    ops.push(Op::SetOutlineThickness { pt: Pt(thickness) });
    let line = Line {
        points: vec![
            LinePoint {
                p: Point {
                    x: Pt(x1),
                    y: Pt(y1),
                },
                bezier: false,
            },
            LinePoint {
                p: Point {
                    x: Pt(x2),
                    y: Pt(y2),
                },
                bezier: false,
            },
        ],
        is_closed: false,
    };
    ops.push(Op::DrawLine { line });
}

/// Dibuja el código de barras Code 128 vectorial exacto para la clave de acceso de 49 dígitos.
fn push_barcode(
    ops: &mut Vec<Op>,
    clave: &str,
    start_x: f32,
    start_y: f32,
    max_w: f32,
    height: f32,
) {
    if clave.is_empty() {
        return;
    }
    let barcode_input = format!("\u{0181}{}", clave);
    let barcode = match Code128::new(&barcode_input) {
        Ok(b) => b,
        Err(_) => return,
    };
    let encoded = barcode.encode();
    let total_modules = encoded.len() as f32;
    if total_modules == 0.0 {
        return;
    }

    let module_w = max_w / total_modules;
    let black = rgb_color(0.0, 0.0, 0.0);
    ops.push(Op::SetFillColor { col: black });

    for (i, bit) in encoded.iter().enumerate() {
        if *bit == 1 {
            let x = start_x + (i as f32 * module_w);
            let polygon = Polygon {
                rings: vec![PolygonRing {
                    points: vec![
                        LinePoint {
                            p: Point {
                                x: Pt(x),
                                y: Pt(start_y),
                            },
                            bezier: false,
                        },
                        LinePoint {
                            p: Point {
                                x: Pt(x + module_w),
                                y: Pt(start_y),
                            },
                            bezier: false,
                        },
                        LinePoint {
                            p: Point {
                                x: Pt(x + module_w),
                                y: Pt(start_y + height),
                            },
                            bezier: false,
                        },
                        LinePoint {
                            p: Point {
                                x: Pt(x),
                                y: Pt(start_y + height),
                            },
                            bezier: false,
                        },
                    ],
                }],
                mode: PaintMode::Fill,
                winding_order: WindingOrder::NonZero,
            };
            ops.push(Op::DrawPolygon { polygon });
        }
    }
}

/// Carga e incrusta el logo del emisor si existe en disco.
fn try_embed_logo(
    doc: &mut PdfDocument,
    ops: &mut Vec<Op>,
    logo_path: &Option<String>,
    box_x: f32,
    box_y: f32,
    max_w: f32,
    max_h: f32,
) -> bool {
    let path_str = match logo_path {
        Some(p) if !p.trim().is_empty() => p.trim(),
        _ => return false,
    };

    let path = Path::new(path_str);
    if !path.exists() {
        return false;
    }

    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let mut warnings = Vec::new();
    let raw_img = match RawImage::decode_from_bytes(&bytes, &mut warnings) {
        Ok(img) => img,
        Err(_) => return false,
    };

    let img_w = raw_img.width as f32;
    let img_h = raw_img.height as f32;
    if img_w <= 0.0 || img_h <= 0.0 {
        return false;
    }

    // Calcular escala proporcional (contain) para ocupar el máximo espacio disponible
    let scale = (max_w / img_w).min(max_h / img_h);
    let target_w = img_w * scale;
    let target_h = img_h * scale;

    // Centrar la imagen dentro del recuadro disponible
    let offset_x = (max_w - target_w) / 2.0;
    let offset_y = (max_h - target_h) / 2.0;
    let final_x = box_x + offset_x;
    let final_y = box_y + offset_y;

    // En printpdf 0.12, get_ctms() mapea inicialmente la imagen a su tamaño natural en pt
    // usando IMAGE_DPI (300.0 por defecto): natural_w_pt = img_w * 72.0 / IMAGE_DPI.
    // Para que la imagen ocupe exactamente target_w puntos en el PDF, debemos escalar por:
    //   scale_x = target_w / natural_w_pt.
    const IMAGE_DPI: f32 = 300.0;
    let natural_w_pt = (img_w * 72.0) / IMAGE_DPI;
    let natural_h_pt = (img_h * 72.0) / IMAGE_DPI;
    if natural_w_pt <= 0.0 || natural_h_pt <= 0.0 {
        return false;
    }

    let scale_x = target_w / natural_w_pt;
    let scale_y = target_h / natural_h_pt;

    let image_id = doc.add_image(&raw_img);

    ops.push(Op::UseXobject {
        id: image_id,
        transform: XObjectTransform {
            translate_x: Some(Pt(final_x)),
            translate_y: Some(Pt(final_y)),
            rotate: None,
            scale_x: Some(scale_x),
            scale_y: Some(scale_y),
            dpi: Some(IMAGE_DPI),
            no_auto_scale: false,
        },
    });

    true
}

fn trunc_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let mut res: String = s.chars().take(max_chars - 3).collect();
        res.push_str("...");
        res
    } else {
        s.to_string()
    }
}

// ---------------------------------------------------------------------------
// 1. MODELO 1: CLÁSICA (RIDE Oficial SRI Estándar en 2 Columnas)
// ---------------------------------------------------------------------------

fn render_clasica(datos: &DatosRide) -> Result<Vec<u8>, String> {
    let mut doc = PdfDocument::new("Factura SRI");
    let mut ops = Vec::new();

    let font_regular = PdfFontHandle::Builtin(BuiltinFont::Helvetica);
    let font_bold = PdfFontHandle::Builtin(BuiltinFont::HelveticaBold);

    let c_black = rgb_color(0.0, 0.0, 0.0);
    let c_gray_border = rgb_color(0.55, 0.55, 0.55);
    let c_gray_bg = rgb_color(0.94, 0.94, 0.94);
    let c_row_alt = rgb_color(0.98, 0.98, 0.98);

    let tot = datos.calcular_totales();

    // COLUMNA IZQUIERDA
    let col1_x = 35.0;
    let col1_w = 250.0;

    let logo_box_y = 730.0;
    let logo_box_h = 75.0;
    push_rect_stroke(
        &mut ops,
        col1_x,
        logo_box_y,
        col1_w,
        logo_box_h,
        &c_gray_border,
        0.7,
    );

    let logo_loaded = try_embed_logo(
        &mut doc,
        &mut ops,
        &datos.emisor.logo_path,
        col1_x + 5.0,
        logo_box_y + 5.0,
        col1_w - 10.0,
        logo_box_h - 10.0,
    );

    if !logo_loaded {
        let nombre_display = datos
            .emisor
            .nombre_comercial
            .as_deref()
            .unwrap_or(&datos.emisor.razon_social);
        set_fill(&mut ops, &c_black);
        push_text(
            &mut ops,
            &font_bold,
            13.0,
            col1_x + 15.0,
            logo_box_y + 40.0,
            &trunc_str(nombre_display, 28),
        );
        push_text(
            &mut ops,
            &font_regular,
            8.5,
            col1_x + 15.0,
            logo_box_y + 25.0,
            "LOGOTIPO / IDENTIFICACIÓN COMERCIAL",
        );
    }

    let emisor_box_y = 575.0;
    let emisor_box_h = 145.0;
    push_rect_stroke(
        &mut ops,
        col1_x,
        emisor_box_y,
        col1_w,
        emisor_box_h,
        &c_gray_border,
        0.7,
    );

    set_fill(&mut ops, &c_black);
    push_text(
        &mut ops,
        &font_bold,
        10.0,
        col1_x + 10.0,
        emisor_box_y + 128.0,
        &trunc_str(&datos.emisor.razon_social, 34),
    );

    if let Some(nc) = &datos.emisor.nombre_comercial {
        push_text(
            &mut ops,
            &font_regular,
            8.5,
            col1_x + 10.0,
            emisor_box_y + 114.0,
            &format!("Nombre Comercial: {}", trunc_str(nc, 30)),
        );
    }

    push_text(
        &mut ops,
        &font_bold,
        8.0,
        col1_x + 10.0,
        emisor_box_y + 98.0,
        "Dirección Matriz:",
    );
    push_text(
        &mut ops,
        &font_regular,
        8.0,
        col1_x + 10.0,
        emisor_box_y + 86.0,
        &trunc_str(&datos.emisor.dir_matriz, 42),
    );

    push_text(
        &mut ops,
        &font_bold,
        8.0,
        col1_x + 10.0,
        emisor_box_y + 72.0,
        "Dirección Sucursal:",
    );
    push_text(
        &mut ops,
        &font_regular,
        8.0,
        col1_x + 10.0,
        emisor_box_y + 60.0,
        &trunc_str(&datos.emisor.dir_establecimiento, 42),
    );

    push_text(
        &mut ops,
        &font_bold,
        8.0,
        col1_x + 10.0,
        emisor_box_y + 46.0,
        "OBLIGADO A LLEVAR CONTABILIDAD:",
    );
    push_text(
        &mut ops,
        &font_regular,
        8.0,
        col1_x + 180.0,
        emisor_box_y + 46.0,
        &datos.emisor.obligado_contabilidad,
    );

    if let Some(ce) = &datos.emisor.contribuyente_especial {
        push_text(
            &mut ops,
            &font_regular,
            8.0,
            col1_x + 10.0,
            emisor_box_y + 32.0,
            &format!("Contribuyente Especial Nro: {}", ce),
        );
    } else if let Some(rimpe) = &datos.emisor.regimen_rimpe {
        push_text(
            &mut ops,
            &font_bold,
            7.5,
            col1_x + 10.0,
            emisor_box_y + 20.0,
            &trunc_str(rimpe, 42),
        );
    }

    // COLUMNA DERECHA
    let col2_x = 295.0;
    let col2_w = 265.0;
    let col2_y = 575.0;
    let col2_h = 230.0;
    push_rect_stroke(
        &mut ops,
        col2_x,
        col2_y,
        col2_w,
        col2_h,
        &c_gray_border,
        0.7,
    );

    set_fill(&mut ops, &c_black);
    push_text(
        &mut ops,
        &font_bold,
        11.0,
        col2_x + 10.0,
        col2_y + 212.0,
        &format!("R.U.C.: {}", datos.emisor.ruc),
    );
    push_text(
        &mut ops,
        &font_bold,
        14.0,
        col2_x + 10.0,
        col2_y + 194.0,
        "F A C T U R A",
    );

    let num_factura = format!(
        "No. {}-{}-{}",
        datos.emisor.cod_establecimiento, datos.emisor.pto_emision, datos.secuencial
    );
    push_text(
        &mut ops,
        &font_regular,
        10.0,
        col2_x + 10.0,
        col2_y + 178.0,
        &num_factura,
    );

    push_text(
        &mut ops,
        &font_bold,
        8.0,
        col2_x + 10.0,
        col2_y + 162.0,
        "NÚMERO DE AUTORIZACIÓN:",
    );
    push_text(
        &mut ops,
        &font_regular,
        7.5,
        col2_x + 10.0,
        col2_y + 151.0,
        &datos.numero_autorizacion,
    );

    push_text(
        &mut ops,
        &font_bold,
        8.0,
        col2_x + 10.0,
        col2_y + 138.0,
        "FECHA Y HORA DE AUTORIZACIÓN:",
    );
    push_text(
        &mut ops,
        &font_regular,
        8.0,
        col2_x + 155.0,
        col2_y + 138.0,
        &datos.fecha_autorizacion,
    );

    push_text(
        &mut ops,
        &font_bold,
        8.0,
        col2_x + 10.0,
        col2_y + 124.0,
        "AMBIENTE:",
    );
    push_text(
        &mut ops,
        &font_regular,
        8.0,
        col2_x + 65.0,
        col2_y + 124.0,
        &datos.ambiente,
    );

    push_text(
        &mut ops,
        &font_bold,
        8.0,
        col2_x + 140.0,
        col2_y + 124.0,
        "EMISIÓN:",
    );
    push_text(
        &mut ops,
        &font_regular,
        8.0,
        col2_x + 190.0,
        col2_y + 124.0,
        &datos.emision,
    );

    push_text(
        &mut ops,
        &font_bold,
        8.0,
        col2_x + 10.0,
        col2_y + 108.0,
        "CLAVE DE ACCESO:",
    );

    let barcode_w = col2_w - 24.0;
    push_barcode(
        &mut ops,
        &datos.clave_acceso,
        col2_x + 12.0,
        col2_y + 50.0,
        barcode_w,
        42.0,
    );

    set_fill(&mut ops, &c_black);
    push_text(
        &mut ops,
        &font_regular,
        6.8,
        col2_x + 15.0,
        col2_y + 36.0,
        &datos.clave_acceso,
    );

    // DATOS DEL CLIENTE
    let cli_y = 505.0;
    let cli_h = 60.0;
    let full_w = 525.0;
    push_rect_stroke(&mut ops, col1_x, cli_y, full_w, cli_h, &c_gray_border, 0.7);

    set_fill(&mut ops, &c_black);
    push_text(
        &mut ops,
        &font_bold,
        8.5,
        col1_x + 8.0,
        cli_y + 44.0,
        "Razón Social / Nombres y Apellidos:",
    );
    push_text(
        &mut ops,
        &font_regular,
        8.5,
        col1_x + 165.0,
        cli_y + 44.0,
        &trunc_str(&datos.cliente.razon_social, 42),
    );

    push_text(
        &mut ops,
        &font_bold,
        8.5,
        col1_x + 370.0,
        cli_y + 44.0,
        "Identificación:",
    );
    push_text(
        &mut ops,
        &font_regular,
        8.5,
        col1_x + 435.0,
        cli_y + 44.0,
        &datos.cliente.identificacion,
    );

    push_text(
        &mut ops,
        &font_bold,
        8.5,
        col1_x + 8.0,
        cli_y + 26.0,
        "Fecha Emisión:",
    );
    push_text(
        &mut ops,
        &font_regular,
        8.5,
        col1_x + 75.0,
        cli_y + 26.0,
        &datos.fecha_emision,
    );

    push_text(
        &mut ops,
        &font_bold,
        8.5,
        col1_x + 200.0,
        cli_y + 26.0,
        "Guía Remisión:",
    );
    let guia_disp = datos.guia_remision.as_deref().unwrap_or("S/N");
    push_text(
        &mut ops,
        &font_regular,
        8.5,
        col1_x + 270.0,
        cli_y + 26.0,
        &trunc_str(guia_disp, 35),
    );

    let dir_cli = datos.cliente.direccion.as_deref().unwrap_or("S/N");
    push_text(
        &mut ops,
        &font_bold,
        8.5,
        col1_x + 8.0,
        cli_y + 10.0,
        "Dirección:",
    );
    push_text(
        &mut ops,
        &font_regular,
        8.5,
        col1_x + 60.0,
        cli_y + 10.0,
        &trunc_str(dir_cli, 70),
    );

    // TABLA DE ÍTEMS
    let table_y = 485.0;
    let header_h = 16.0;
    push_rect_fill(
        &mut ops,
        col1_x,
        table_y - header_h,
        full_w,
        header_h,
        &c_gray_bg,
    );
    push_rect_stroke(
        &mut ops,
        col1_x,
        table_y - header_h,
        full_w,
        header_h,
        &c_gray_border,
        0.7,
    );

    set_fill(&mut ops, &c_black);
    push_text(
        &mut ops,
        &font_bold,
        7.5,
        col1_x + 5.0,
        table_y - 12.0,
        "Cod. Principal",
    );
    push_text(
        &mut ops,
        &font_bold,
        7.5,
        col1_x + 75.0,
        table_y - 12.0,
        "Cant.",
    );
    push_text(
        &mut ops,
        &font_bold,
        7.5,
        col1_x + 110.0,
        table_y - 12.0,
        "Descripción",
    );
    push_text(
        &mut ops,
        &font_bold,
        7.5,
        col1_x + 360.0,
        table_y - 12.0,
        "Precio Unit.",
    );
    push_text(
        &mut ops,
        &font_bold,
        7.5,
        col1_x + 420.0,
        table_y - 12.0,
        "Descuento",
    );
    push_text(
        &mut ops,
        &font_bold,
        7.5,
        col1_x + 475.0,
        table_y - 12.0,
        "Precio Total",
    );

    let mut current_y = table_y - header_h;
    let row_h = 14.0;
    let max_rows = 15;

    for (idx, d) in datos.detalles.iter().take(max_rows).enumerate() {
        let y_row = current_y - row_h;
        if idx % 2 == 1 {
            push_rect_fill(&mut ops, col1_x, y_row, full_w, row_h, &c_row_alt);
        }
        push_line(
            &mut ops,
            col1_x,
            y_row,
            col1_x + full_w,
            y_row,
            &c_gray_border,
            0.3,
        );

        set_fill(&mut ops, &c_black);
        push_text(
            &mut ops,
            &font_regular,
            7.5,
            col1_x + 5.0,
            y_row + 4.0,
            &trunc_str(&d.codigo_principal, 14),
        );
        push_text(
            &mut ops,
            &font_regular,
            7.5,
            col1_x + 75.0,
            y_row + 4.0,
            &format!("{:.2}", d.cantidad),
        );
        push_text(
            &mut ops,
            &font_regular,
            7.5,
            col1_x + 110.0,
            y_row + 4.0,
            &trunc_str(&d.descripcion, 48),
        );
        push_text(
            &mut ops,
            &font_regular,
            7.5,
            col1_x + 360.0,
            y_row + 4.0,
            &format!("${:.2}", d.precio_unitario),
        );
        push_text(
            &mut ops,
            &font_regular,
            7.5,
            col1_x + 420.0,
            y_row + 4.0,
            &format!("${:.2}", d.descuento),
        );
        push_text(
            &mut ops,
            &font_bold,
            7.5,
            col1_x + 475.0,
            y_row + 4.0,
            &format!("${:.2}", d.subtotal()),
        );

        current_y = y_row;
    }
    push_rect_stroke(
        &mut ops,
        col1_x,
        current_y,
        full_w,
        (table_y - header_h) - current_y,
        &c_gray_border,
        0.5,
    );

    // PIE: INFO ADICIONAL Y TOTALES
    let footer_top_y = (current_y - 12.0).min(235.0);

    let info_w = 280.0;
    let info_h = 70.0;
    push_rect_stroke(
        &mut ops,
        col1_x,
        footer_top_y - info_h,
        info_w,
        info_h,
        &c_gray_border,
        0.7,
    );

    set_fill(&mut ops, &c_black);
    push_text(
        &mut ops,
        &font_bold,
        8.5,
        col1_x + 8.0,
        footer_top_y - 14.0,
        "Información Adicional:",
    );
    let email_val = datos.cliente.email.as_deref().unwrap_or("N/A");
    push_text(
        &mut ops,
        &font_regular,
        7.5,
        col1_x + 8.0,
        footer_top_y - 28.0,
        &format!("Email: {}", email_val),
    );
    let telf_val = datos.cliente.telefono.as_deref().unwrap_or("N/A");
    push_text(
        &mut ops,
        &font_regular,
        7.5,
        col1_x + 8.0,
        footer_top_y - 42.0,
        &format!("Teléfono: {}", telf_val),
    );
    push_text(
        &mut ops,
        &font_regular,
        7.0,
        col1_x + 8.0,
        footer_top_y - 58.0,
        "Generado por Cacao Facturador SRI - cacaoscript.com",
    );

    let fp_y = footer_top_y - info_h - 10.0;
    let fp_h = 55.0;
    push_rect_stroke(
        &mut ops,
        col1_x,
        fp_y - fp_h,
        info_w,
        fp_h,
        &c_gray_border,
        0.7,
    );
    push_rect_fill(&mut ops, col1_x, fp_y - 14.0, info_w, 14.0, &c_gray_bg);
    set_fill(&mut ops, &c_black);
    push_text(
        &mut ops,
        &font_bold,
        7.5,
        col1_x + 8.0,
        fp_y - 10.0,
        "Forma de Pago",
    );
    push_text(
        &mut ops,
        &font_bold,
        7.5,
        col1_x + 190.0,
        fp_y - 10.0,
        "Total",
    );
    push_text(
        &mut ops,
        &font_bold,
        7.5,
        col1_x + 235.0,
        fp_y - 10.0,
        "Plazo",
    );

    for (i, fp) in datos.formas_pago.iter().take(3).enumerate() {
        let row_y = fp_y - 26.0 - (i as f32 * 12.0);
        let desc = descripcion_pago_corta(&fp.forma_pago);
        set_fill(&mut ops, &c_black);
        push_text(
            &mut ops,
            &font_regular,
            7.0,
            col1_x + 8.0,
            row_y,
            &trunc_str(desc, 30),
        );
        push_text(
            &mut ops,
            &font_regular,
            7.0,
            col1_x + 190.0,
            row_y,
            &format!("${:.2}", fp.total),
        );
        let plazo_str = fp
            .plazo
            .map(|p| format!("{} {}", p, fp.unidad_tiempo.as_deref().unwrap_or("días")))
            .unwrap_or_else(|| "0".to_string());
        push_text(
            &mut ops,
            &font_regular,
            7.0,
            col1_x + 235.0,
            row_y,
            &plazo_str,
        );
    }

    let tot_x = 330.0;
    let tot_w = 230.0;
    let tot_h = 135.0;
    let tot_y = footer_top_y;
    push_rect_stroke(
        &mut ops,
        tot_x,
        tot_y - tot_h,
        tot_w,
        tot_h,
        &c_gray_border,
        0.7,
    );

    let labels_totales = [
        ("SUBTOTAL 15%", tot.subtotal_15),
        ("SUBTOTAL 0%", tot.subtotal_0),
        ("SUBTOTAL NO OBJETO DE IVA", tot.subtotal_no_objeto),
        ("SUBTOTAL EXENTO DE IVA", tot.subtotal_exento),
        ("SUBTOTAL SIN IMPUESTOS", tot.subtotal_sin_impuestos),
        ("TOTAL DESCUENTO", tot.total_descuento),
        ("IVA 15%", tot.total_iva),
        ("PROPINA", tot.propina),
    ];

    let mut line_y = tot_y - 14.0;
    for (label, val) in &labels_totales {
        set_fill(&mut ops, &c_black);
        push_text(&mut ops, &font_regular, 7.5, tot_x + 8.0, line_y, label);
        push_text(
            &mut ops,
            &font_regular,
            7.5,
            tot_x + 175.0,
            line_y,
            &format!("${:.2}", val),
        );
        push_line(
            &mut ops,
            tot_x,
            line_y - 3.0,
            tot_x + tot_w,
            line_y - 3.0,
            &c_gray_border,
            0.2,
        );
        line_y -= 13.0;
    }

    push_rect_fill(&mut ops, tot_x, tot_y - tot_h, tot_w, 18.0, &c_gray_bg);
    set_fill(&mut ops, &c_black);
    push_text(
        &mut ops,
        &font_bold,
        9.0,
        tot_x + 8.0,
        tot_y - tot_h + 5.0,
        "VALOR TOTAL:",
    );
    push_text(
        &mut ops,
        &font_bold,
        10.0,
        tot_x + 170.0,
        tot_y - tot_h + 5.0,
        &format!("${:.2}", tot.valor_total),
    );

    doc.pages.push(PdfPage::new(Mm(210.0), Mm(297.0), ops));

    let mut warnings = Vec::new();
    let bytes = doc.save(&PdfSaveOptions::default(), &mut warnings);
    Ok(bytes)
}

// ---------------------------------------------------------------------------
// 2. MODELO 2: MODERNA (Corporativa Ejecutiva con estilo Cacao)
// ---------------------------------------------------------------------------

fn render_moderna(datos: &DatosRide) -> Result<Vec<u8>, String> {
    let mut doc = PdfDocument::new("Factura SRI Moderna");
    let mut ops = Vec::new();

    let font_regular = PdfFontHandle::Builtin(BuiltinFont::Helvetica);
    let font_bold = PdfFontHandle::Builtin(BuiltinFont::HelveticaBold);

    let c_cacao = rgb_color(0.54, 0.31, 0.15); // #8A5025
    let c_cacao_dark = rgb_color(0.24, 0.13, 0.08); // #3D2214
    let c_cacao_light = rgb_color(0.97, 0.94, 0.91); // #F7F1EB
    let c_white = rgb_color(1.0, 1.0, 1.0);
    let c_dark_text = rgb_color(0.12, 0.12, 0.12);
    let c_muted_text = rgb_color(0.45, 0.45, 0.45);
    let c_border = rgb_color(0.85, 0.80, 0.75);

    let tot = datos.calcular_totales();
    let full_w = 525.0;
    let page_x = 35.0;

    // BANNER SUPERIOR
    push_rect_fill(&mut ops, 0.0, 835.0, 595.28, 7.0, &c_cacao);

    // Cabecera Emisor y Logo
    let header_y = 750.0;
    let logo_w = 140.0;
    let logo_h = 75.0;
    let logo_loaded = try_embed_logo(
        &mut doc,
        &mut ops,
        &datos.emisor.logo_path,
        page_x,
        header_y,
        logo_w,
        logo_h,
    );

    let text_x = if logo_loaded {
        page_x + logo_w + 14.0
    } else {
        page_x
    };
    set_fill(&mut ops, &c_cacao_dark);
    let nombre_disp = datos
        .emisor
        .nombre_comercial
        .as_deref()
        .unwrap_or(&datos.emisor.razon_social);
    push_text(
        &mut ops,
        &font_bold,
        13.0,
        text_x,
        header_y + 54.0,
        &trunc_str(nombre_disp, 26),
    );

    set_fill(&mut ops, &c_muted_text);
    push_text(
        &mut ops,
        &font_regular,
        8.5,
        text_x,
        header_y + 40.0,
        &format!("RUC: {}", datos.emisor.ruc),
    );
    push_text(
        &mut ops,
        &font_regular,
        7.5,
        text_x,
        header_y + 27.0,
        &trunc_str(&datos.emisor.dir_matriz, 36),
    );
    push_text(
        &mut ops,
        &font_regular,
        7.5,
        text_x,
        header_y + 14.0,
        &format!(
            "Obligado Contabilidad: {}",
            datos.emisor.obligado_contabilidad
        ),
    );
    if let Some(r) = &datos.emisor.regimen_rimpe {
        push_text(
            &mut ops,
            &font_regular,
            7.0,
            text_x,
            header_y + 2.0,
            &trunc_str(r, 38),
        );
    }

    // Tarjeta de Factura a la derecha
    let card_x = 370.0;
    let card_w = 190.0;
    let card_h = 75.0;
    push_rect_fill(&mut ops, card_x, header_y, card_w, card_h, &c_cacao_light);
    push_rect_stroke(&mut ops, card_x, header_y, card_w, card_h, &c_cacao, 0.8);

    push_rect_fill(
        &mut ops,
        card_x,
        header_y + card_h - 18.0,
        card_w,
        18.0,
        &c_cacao,
    );
    set_fill(&mut ops, &c_white);
    push_text(
        &mut ops,
        &font_bold,
        8.5,
        card_x + 22.0,
        header_y + card_h - 13.0,
        "FACTURA ELECTRÓNICA",
    );

    set_fill(&mut ops, &c_cacao_dark);
    let num_factura = format!(
        "{}-{}-{}",
        datos.emisor.cod_establecimiento, datos.emisor.pto_emision, datos.secuencial
    );
    push_text(
        &mut ops,
        &font_bold,
        11.0,
        card_x + 12.0,
        header_y + 38.0,
        &num_factura,
    );

    set_fill(&mut ops, &c_muted_text);
    push_text(
        &mut ops,
        &font_regular,
        8.0,
        card_x + 12.0,
        header_y + 24.0,
        &format!("Fecha: {}", datos.fecha_emision),
    );
    push_text(
        &mut ops,
        &font_regular,
        7.5,
        card_x + 12.0,
        header_y + 10.0,
        &format!("Ambiente: {}", datos.ambiente),
    );

    // AUTORIZACIÓN Y CÓDIGO DE BARRAS
    let auto_y = 675.0;
    let c_white_bg = rgb_color(0.99, 0.99, 0.99);
    push_rect_fill(&mut ops, page_x, auto_y, full_w, 60.0, &c_white_bg);
    push_rect_stroke(&mut ops, page_x, auto_y, full_w, 60.0, &c_border, 0.5);

    set_fill(&mut ops, &c_cacao_dark);
    push_text(
        &mut ops,
        &font_bold,
        8.0,
        page_x + 10.0,
        auto_y + 46.0,
        "CLAVE DE ACCESO Y AUTORIZACIÓN SRI:",
    );

    push_barcode(
        &mut ops,
        &datos.clave_acceso,
        page_x + 10.0,
        auto_y + 16.0,
        270.0,
        26.0,
    );
    set_fill(&mut ops, &c_dark_text);
    push_text(
        &mut ops,
        &font_regular,
        5.8,
        page_x + 10.0,
        auto_y + 6.0,
        &datos.clave_acceso,
    );

    set_fill(&mut ops, &c_muted_text);
    push_text(
        &mut ops,
        &font_bold,
        7.5,
        page_x + 295.0,
        auto_y + 46.0,
        "ESTADO: AUTORIZADO",
    );
    push_text(
        &mut ops,
        &font_regular,
        5.8,
        page_x + 295.0,
        auto_y + 34.0,
        &format!("No. Aut: {}", datos.numero_autorizacion),
    );
    push_text(
        &mut ops,
        &font_regular,
        6.5,
        page_x + 295.0,
        auto_y + 22.0,
        &format!("F. Aut: {}", trunc_str(&datos.fecha_autorizacion, 24)),
    );
    let guia_val = datos.guia_remision.as_deref().unwrap_or("S/N");
    push_text(
        &mut ops,
        &font_regular,
        6.5,
        page_x + 295.0,
        auto_y + 10.0,
        &format!("Guía Remisión: {}", trunc_str(guia_val, 26)),
    );

    // TARJETA DE CLIENTE
    let cli_y = 605.0;
    push_rect_fill(&mut ops, page_x, cli_y, full_w, 55.0, &c_cacao_light);
    push_rect_stroke(&mut ops, page_x, cli_y, full_w, 55.0, &c_border, 0.5);

    set_fill(&mut ops, &c_cacao_dark);
    push_text(
        &mut ops,
        &font_bold,
        8.5,
        page_x + 12.0,
        cli_y + 38.0,
        "FACTURADO A:",
    );
    push_text(
        &mut ops,
        &font_bold,
        10.0,
        page_x + 85.0,
        cli_y + 38.0,
        &trunc_str(&datos.cliente.razon_social, 40),
    );

    set_fill(&mut ops, &c_muted_text);
    push_text(
        &mut ops,
        &font_bold,
        8.0,
        page_x + 12.0,
        cli_y + 22.0,
        "RUC/C.I.:",
    );
    push_text(
        &mut ops,
        &font_regular,
        8.0,
        page_x + 60.0,
        cli_y + 22.0,
        &datos.cliente.identificacion,
    );

    push_text(
        &mut ops,
        &font_bold,
        8.0,
        page_x + 185.0,
        cli_y + 22.0,
        "Email:",
    );
    let em = datos.cliente.email.as_deref().unwrap_or("N/A");
    push_text(
        &mut ops,
        &font_regular,
        8.0,
        page_x + 220.0,
        cli_y + 22.0,
        &trunc_str(em, 24),
    );

    push_text(
        &mut ops,
        &font_bold,
        8.0,
        page_x + 375.0,
        cli_y + 22.0,
        "Telf:",
    );
    let telf = datos.cliente.telefono.as_deref().unwrap_or("N/A");
    push_text(
        &mut ops,
        &font_regular,
        8.0,
        page_x + 405.0,
        cli_y + 22.0,
        &trunc_str(telf, 18),
    );

    let dir = datos.cliente.direccion.as_deref().unwrap_or("S/N");
    push_text(
        &mut ops,
        &font_bold,
        8.0,
        page_x + 12.0,
        cli_y + 8.0,
        "Dirección:",
    );
    push_text(
        &mut ops,
        &font_regular,
        8.0,
        page_x + 60.0,
        cli_y + 8.0,
        &trunc_str(dir, 75),
    );

    // TABLA MODERNA
    let table_y = 585.0;
    let header_h = 18.0;
    push_rect_fill(
        &mut ops,
        page_x,
        table_y - header_h,
        full_w,
        header_h,
        &c_cacao,
    );

    set_fill(&mut ops, &c_white);
    push_text(
        &mut ops,
        &font_bold,
        8.0,
        page_x + 8.0,
        table_y - 13.0,
        "CÓDIGO",
    );
    push_text(
        &mut ops,
        &font_bold,
        8.0,
        page_x + 75.0,
        table_y - 13.0,
        "CANT.",
    );
    push_text(
        &mut ops,
        &font_bold,
        8.0,
        page_x + 115.0,
        table_y - 13.0,
        "DESCRIPCIÓN",
    );
    push_text(
        &mut ops,
        &font_bold,
        8.0,
        page_x + 360.0,
        table_y - 13.0,
        "PRECIO",
    );
    push_text(
        &mut ops,
        &font_bold,
        8.0,
        page_x + 420.0,
        table_y - 13.0,
        "DESC.",
    );
    push_text(
        &mut ops,
        &font_bold,
        8.0,
        page_x + 475.0,
        table_y - 13.0,
        "TOTAL",
    );

    let mut current_y = table_y - header_h;
    let row_h = 16.0;

    for (idx, d) in datos.detalles.iter().take(14).enumerate() {
        let y_row = current_y - row_h;
        if idx % 2 == 1 {
            push_rect_fill(&mut ops, page_x, y_row, full_w, row_h, &c_cacao_light);
        }
        push_line(
            &mut ops,
            page_x,
            y_row,
            page_x + full_w,
            y_row,
            &c_border,
            0.4,
        );

        set_fill(&mut ops, &c_dark_text);
        push_text(
            &mut ops,
            &font_regular,
            7.5,
            page_x + 8.0,
            y_row + 5.0,
            &trunc_str(&d.codigo_principal, 14),
        );
        push_text(
            &mut ops,
            &font_regular,
            7.5,
            page_x + 75.0,
            y_row + 5.0,
            &format!("{:.2}", d.cantidad),
        );
        push_text(
            &mut ops,
            &font_regular,
            7.5,
            page_x + 115.0,
            y_row + 5.0,
            &trunc_str(&d.descripcion, 48),
        );
        push_text(
            &mut ops,
            &font_regular,
            7.5,
            page_x + 360.0,
            y_row + 5.0,
            &format!("${:.2}", d.precio_unitario),
        );
        push_text(
            &mut ops,
            &font_regular,
            7.5,
            page_x + 420.0,
            y_row + 5.0,
            &format!("${:.2}", d.descuento),
        );
        push_text(
            &mut ops,
            &font_bold,
            7.5,
            page_x + 475.0,
            y_row + 5.0,
            &format!("${:.2}", d.subtotal()),
        );

        current_y = y_row;
    }

    // PIE: TARJETA DE TOTALES Y FORMAS DE PAGO MODERNA
    let footer_y = (current_y - 15.0).min(235.0);

    let pay_w = 260.0;
    let pay_h = 125.0;
    push_rect_fill(
        &mut ops,
        page_x,
        footer_y - pay_h,
        pay_w,
        pay_h,
        &c_white_bg,
    );
    push_rect_stroke(
        &mut ops,
        page_x,
        footer_y - pay_h,
        pay_w,
        pay_h,
        &c_border,
        0.6,
    );

    set_fill(&mut ops, &c_cacao);
    push_text(
        &mut ops,
        &font_bold,
        9.0,
        page_x + 10.0,
        footer_y - 18.0,
        "FORMAS DE PAGO",
    );

    set_fill(&mut ops, &c_dark_text);
    for (i, fp) in datos.formas_pago.iter().take(3).enumerate() {
        let py = footer_y - 36.0 - (i as f32 * 16.0);
        let desc = descripcion_pago_corta(&fp.forma_pago);
        push_text(
            &mut ops,
            &font_regular,
            7.5,
            page_x + 10.0,
            py,
            &trunc_str(desc, 32),
        );
        push_text(
            &mut ops,
            &font_bold,
            7.5,
            page_x + 200.0,
            py,
            &format!("${:.2}", fp.total),
        );
    }

    set_fill(&mut ops, &c_muted_text);
    push_text(
        &mut ops,
        &font_regular,
        7.0,
        page_x + 10.0,
        footer_y - pay_h + 12.0,
        "Gracias por su preferencia • Facturación Electrónica SRI",
    );

    let tot_x = 315.0;
    let tot_w = 245.0;
    let tot_h = 125.0;
    push_rect_fill(&mut ops, tot_x, footer_y - tot_h, tot_w, tot_h, &c_white_bg);
    push_rect_stroke(
        &mut ops,
        tot_x,
        footer_y - tot_h,
        tot_w,
        tot_h,
        &c_border,
        0.6,
    );

    let sub_labels = [
        ("Subtotal 15%:", tot.subtotal_15),
        ("Subtotal 0%:", tot.subtotal_0),
        ("Subtotal sin impuestos:", tot.subtotal_sin_impuestos),
        ("Descuento total:", tot.total_descuento),
        ("IVA 15%:", tot.total_iva),
    ];

    let mut ty = footer_y - 16.0;
    for (lbl, val) in &sub_labels {
        set_fill(&mut ops, &c_muted_text);
        push_text(&mut ops, &font_regular, 7.5, tot_x + 10.0, ty, lbl);
        set_fill(&mut ops, &c_dark_text);
        push_text(
            &mut ops,
            &font_regular,
            7.5,
            tot_x + 190.0,
            ty,
            &format!("${:.2}", val),
        );
        ty -= 13.0;
    }

    let grand_h = 28.0;
    push_rect_fill(
        &mut ops,
        tot_x,
        footer_y - tot_h,
        tot_w,
        grand_h,
        &c_cacao_dark,
    );
    set_fill(&mut ops, &c_white);
    push_text(
        &mut ops,
        &font_bold,
        9.5,
        tot_x + 12.0,
        footer_y - tot_h + 10.0,
        "TOTAL A PAGAR:",
    );
    push_text(
        &mut ops,
        &font_bold,
        13.0,
        tot_x + 160.0,
        footer_y - tot_h + 9.0,
        &format!("${:.2}", tot.valor_total),
    );

    doc.pages.push(PdfPage::new(Mm(210.0), Mm(297.0), ops));

    let mut warnings = Vec::new();
    let bytes = doc.save(&PdfSaveOptions::default(), &mut warnings);
    Ok(bytes)
}

// ---------------------------------------------------------------------------
// 3. MODELO 3: COMPACTA (Minimalista Limpia para ahorro de tinta/espacio)
// ---------------------------------------------------------------------------

fn render_compacta(datos: &DatosRide) -> Result<Vec<u8>, String> {
    let mut doc = PdfDocument::new("Factura SRI Compacta");
    let mut ops = Vec::new();

    let font_regular = PdfFontHandle::Builtin(BuiltinFont::Helvetica);
    let font_bold = PdfFontHandle::Builtin(BuiltinFont::HelveticaBold);

    let c_black = rgb_color(0.0, 0.0, 0.0);
    let c_dark = rgb_color(0.15, 0.15, 0.15);
    let c_gray_line = rgb_color(0.80, 0.80, 0.80);
    let c_muted = rgb_color(0.40, 0.40, 0.40);

    let tot = datos.calcular_totales();
    let full_w = 535.0;
    let page_x = 30.0;

    // ENCABEZADO COMPACTO
    let top_y = 752.0;
    let logo_w = 110.0;
    let logo_h = 66.0;
    let logo_loaded = try_embed_logo(
        &mut doc,
        &mut ops,
        &datos.emisor.logo_path,
        page_x,
        top_y,
        logo_w,
        logo_h,
    );

    let emisor_x = if logo_loaded {
        page_x + logo_w + 14.0
    } else {
        page_x
    };
    set_fill(&mut ops, &c_dark);
    let nombre_disp = datos
        .emisor
        .nombre_comercial
        .as_deref()
        .unwrap_or(&datos.emisor.razon_social);

    let right_x = 360.0;
    let barcode_w = 205.0;

    push_text(
        &mut ops,
        &font_bold,
        10.5,
        emisor_x,
        top_y + 54.0,
        &trunc_str(nombre_disp, 30),
    );
    push_text(
        &mut ops,
        &font_regular,
        7.5,
        emisor_x,
        top_y + 42.0,
        &format!("RUC: {}", datos.emisor.ruc),
    );
    push_text(
        &mut ops,
        &font_regular,
        7.0,
        emisor_x,
        top_y + 30.0,
        &format!("Matriz: {}", trunc_str(&datos.emisor.dir_matriz, 36)),
    );
    push_text(
        &mut ops,
        &font_regular,
        7.0,
        emisor_x,
        top_y + 18.0,
        &format!(
            "Obligado Contabilidad: {}",
            datos.emisor.obligado_contabilidad
        ),
    );

    let regimen_o_amb = if let Some(r) = &datos.emisor.regimen_rimpe {
        format!("{} | Amb: {}", trunc_str(r, 22), datos.ambiente)
    } else {
        format!("Ambiente: {}", datos.ambiente)
    };
    push_text(
        &mut ops,
        &font_regular,
        6.5,
        emisor_x,
        top_y + 7.0,
        &regimen_o_amb,
    );

    let num_factura = format!(
        "FACTURA: {}-{}-{}",
        datos.emisor.cod_establecimiento, datos.emisor.pto_emision, datos.secuencial
    );
    push_text(
        &mut ops,
        &font_bold,
        11.0,
        right_x,
        top_y + 54.0,
        &num_factura,
    );
    let guia_val = datos.guia_remision.as_deref().unwrap_or("S/N");
    push_text(
        &mut ops,
        &font_regular,
        7.0,
        right_x,
        top_y + 42.0,
        &format!(
            "Fecha: {} | Guía: {}",
            datos.fecha_emision,
            trunc_str(guia_val, 16)
        ),
    );
    push_text(
        &mut ops,
        &font_regular,
        5.6,
        right_x,
        top_y + 31.0,
        &format!("No. Aut: {}", datos.numero_autorizacion),
    );

    push_barcode(
        &mut ops,
        &datos.clave_acceso,
        right_x,
        top_y + 9.0,
        barcode_w,
        18.0,
    );
    set_fill(&mut ops, &c_dark);
    push_text(
        &mut ops,
        &font_regular,
        5.6,
        right_x,
        top_y + 1.0,
        &datos.clave_acceso,
    );

    push_line(
        &mut ops,
        page_x,
        top_y - 6.0,
        page_x + full_w,
        top_y - 6.0,
        &c_gray_line,
        0.6,
    );

    // CLIENTE COMPACTO
    let cli_y = top_y - 20.0;
    set_fill(&mut ops, &c_dark);
    push_text(&mut ops, &font_bold, 7.5, page_x, cli_y, "CLIENTE:");
    push_text(
        &mut ops,
        &font_regular,
        7.5,
        page_x + 45.0,
        cli_y,
        &trunc_str(&datos.cliente.razon_social, 35),
    );

    push_text(&mut ops, &font_bold, 7.5, page_x + 245.0, cli_y, "RUC/CI:");
    push_text(
        &mut ops,
        &font_regular,
        7.5,
        page_x + 285.0,
        cli_y,
        &datos.cliente.identificacion,
    );

    push_text(&mut ops, &font_bold, 7.5, page_x + 395.0, cli_y, "TELF:");
    let telf = datos.cliente.telefono.as_deref().unwrap_or("N/A");
    push_text(
        &mut ops,
        &font_regular,
        7.5,
        page_x + 425.0,
        cli_y,
        &trunc_str(telf, 18),
    );

    let cli_y2 = cli_y - 12.0;
    let dir_c = datos.cliente.direccion.as_deref().unwrap_or("S/N");
    push_text(&mut ops, &font_bold, 7.5, page_x, cli_y2, "DIR:");
    push_text(
        &mut ops,
        &font_regular,
        7.5,
        page_x + 25.0,
        cli_y2,
        &trunc_str(dir_c, 45),
    );

    push_text(&mut ops, &font_bold, 7.5, page_x + 285.0, cli_y2, "EMAIL:");
    let email_c = datos.cliente.email.as_deref().unwrap_or("N/A");
    push_text(
        &mut ops,
        &font_regular,
        7.5,
        page_x + 325.0,
        cli_y2,
        &trunc_str(email_c, 36),
    );

    push_line(
        &mut ops,
        page_x,
        cli_y2 - 6.0,
        page_x + full_w,
        cli_y2 - 6.0,
        &c_gray_line,
        0.6,
    );

    // TABLA COMPACTA
    let table_y = cli_y2 - 18.0;
    set_fill(&mut ops, &c_black);
    push_text(&mut ops, &font_bold, 7.0, page_x, table_y, "CÓDIGO");
    push_text(&mut ops, &font_bold, 7.0, page_x + 65.0, table_y, "CANT");
    push_text(
        &mut ops,
        &font_bold,
        7.0,
        page_x + 105.0,
        table_y,
        "DESCRIPCIÓN",
    );
    push_text(
        &mut ops,
        &font_bold,
        7.0,
        page_x + 355.0,
        table_y,
        "P. UNIT",
    );
    push_text(&mut ops, &font_bold, 7.0, page_x + 420.0, table_y, "DESC");
    push_text(&mut ops, &font_bold, 7.0, page_x + 480.0, table_y, "TOTAL");

    push_line(
        &mut ops,
        page_x,
        table_y - 4.0,
        page_x + full_w,
        table_y - 4.0,
        &c_black,
        0.8,
    );

    let mut current_y = table_y - 4.0;
    let row_h = 13.0;

    for d in datos.detalles.iter().take(16) {
        let y_row = current_y - row_h;
        set_fill(&mut ops, &c_dark);
        push_text(
            &mut ops,
            &font_regular,
            7.0,
            page_x,
            y_row + 3.0,
            &trunc_str(&d.codigo_principal, 12),
        );
        push_text(
            &mut ops,
            &font_regular,
            7.0,
            page_x + 65.0,
            y_row + 3.0,
            &format!("{:.1}", d.cantidad),
        );
        push_text(
            &mut ops,
            &font_regular,
            7.0,
            page_x + 105.0,
            y_row + 3.0,
            &trunc_str(&d.descripcion, 45),
        );
        push_text(
            &mut ops,
            &font_regular,
            7.0,
            page_x + 355.0,
            y_row + 3.0,
            &format!("${:.2}", d.precio_unitario),
        );
        push_text(
            &mut ops,
            &font_regular,
            7.0,
            page_x + 420.0,
            y_row + 3.0,
            &format!("${:.2}", d.descuento),
        );
        push_text(
            &mut ops,
            &font_bold,
            7.0,
            page_x + 480.0,
            y_row + 3.0,
            &format!("${:.2}", d.subtotal()),
        );

        push_line(
            &mut ops,
            page_x,
            y_row,
            page_x + full_w,
            y_row,
            &c_gray_line,
            0.2,
        );
        current_y = y_row;
    }

    // PIE COMPACTO
    let footer_y = (current_y - 15.0).min(200.0);
    push_line(
        &mut ops,
        page_x,
        footer_y,
        page_x + full_w,
        footer_y,
        &c_black,
        0.6,
    );

    set_fill(&mut ops, &c_muted);
    push_text(
        &mut ops,
        &font_bold,
        7.0,
        page_x,
        footer_y - 12.0,
        "Forma de Pago:",
    );
    if let Some(fp) = datos.formas_pago.first() {
        let desc = descripcion_pago_corta(&fp.forma_pago);
        push_text(
            &mut ops,
            &font_regular,
            7.0,
            page_x + 70.0,
            footer_y - 12.0,
            &format!("{} (${:.2})", desc, fp.total),
        );
    }
    push_text(
        &mut ops,
        &font_regular,
        6.5,
        page_x,
        footer_y - 25.0,
        "Comprobante electrónico emitido vía Cacao Facturador SRI",
    );

    let tot_x = 340.0;
    let lines = [
        ("Subtotal 15%:", tot.subtotal_15),
        ("Subtotal 0%:", tot.subtotal_0),
        ("Subtotal Sin Impuestos:", tot.subtotal_sin_impuestos),
        ("Total Descuento:", tot.total_descuento),
        ("IVA 15%:", tot.total_iva),
    ];

    let mut ty = footer_y - 12.0;
    for (lbl, val) in &lines {
        set_fill(&mut ops, &c_muted);
        push_text(&mut ops, &font_regular, 6.8, tot_x, ty, lbl);
        set_fill(&mut ops, &c_dark);
        push_text(
            &mut ops,
            &font_regular,
            6.8,
            tot_x + 140.0,
            ty,
            &format!("${:.2}", val),
        );
        ty -= 10.0;
    }

    push_line(
        &mut ops,
        tot_x,
        ty + 2.0,
        page_x + full_w,
        ty + 2.0,
        &c_black,
        0.6,
    );
    set_fill(&mut ops, &c_black);
    push_text(&mut ops, &font_bold, 8.5, tot_x, ty - 8.0, "TOTAL:");
    push_text(
        &mut ops,
        &font_bold,
        9.5,
        tot_x + 130.0,
        ty - 8.0,
        &format!("${:.2}", tot.valor_total),
    );

    doc.pages.push(PdfPage::new(Mm(210.0), Mm(297.0), ops));

    let mut warnings = Vec::new();
    let bytes = doc.save(&PdfSaveOptions::default(), &mut warnings);
    Ok(bytes)
}

fn descripcion_pago_corta(codigo: &str) -> &'static str {
    match codigo {
        "01" => "Sin utilización sist. financiero",
        "15" => "Compensación de deudas",
        "16" => "Tarjeta de débito",
        "17" => "Dinero electrónico",
        "18" => "Tarjeta prepago",
        "19" => "Tarjeta de crédito",
        "20" => "Otros con sist. financiero",
        "21" => "Endoso de títulos",
        _ => "Otros",
    }
}

// ---------------------------------------------------------------------------
// Tests Unitarios
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_datos() -> DatosRide {
        DatosRide {
            emisor: EmisorConfig::default(),
            secuencial: "000000042".to_string(),
            fecha_emision: "09/09/2026".to_string(),
            clave_acceso: "0909202601179000000000110010010000000421234567819".to_string(),
            numero_autorizacion: "0909202601179000000000110010010000000421234567819".to_string(),
            fecha_autorizacion: "09/09/2026 16:30:00".to_string(),
            ambiente: "PRUEBAS".to_string(),
            emision: "NORMAL".to_string(),
            cliente: ClienteInfo {
                tipo_identificacion: "05".to_string(),
                razon_social: "JUAN PEREZ".to_string(),
                identificacion: "1712345678".to_string(),
                direccion: Some("Av. Amazonas y Colón".to_string()),
                email: Some("juan@example.com".to_string()),
                telefono: Some("0991234567".to_string()),
            },
            detalles: vec![
                DetalleFactura {
                    codigo_principal: "PROD-01".to_string(),
                    codigo_auxiliar: None,
                    descripcion: "Servicio de Desarrollo de Software".to_string(),
                    cantidad: 2.0,
                    precio_unitario: 50.0,
                    descuento: 0.0,
                    codigo_porcentaje_iva: "4".to_string(),
                    tarifa_iva: 15.0,
                },
                DetalleFactura {
                    codigo_principal: "PROD-02".to_string(),
                    codigo_auxiliar: None,
                    descripcion: "Hosting Cloud Mensual".to_string(),
                    cantidad: 1.0,
                    precio_unitario: 20.0,
                    descuento: 2.0,
                    codigo_porcentaje_iva: "0".to_string(),
                    tarifa_iva: 0.0,
                },
            ],
            formas_pago: vec![FormaPago {
                forma_pago: "01".to_string(),
                total: 133.0,
                plazo: None,
                unidad_tiempo: None,
            }],
            propina: 0.0,
            guia_remision: Some("001-001-000000123".to_string()),
        }
    }

    #[test]
    fn test_generar_ride_clasico() {
        let datos = dummy_datos();
        let pdf = generar_ride_pdf(&datos, "clasica").expect("Fallo plantilla clasica");
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.len() > 1000);
    }

    #[test]
    fn test_generar_ride_moderno() {
        let datos = dummy_datos();
        let pdf = generar_ride_pdf(&datos, "moderna").expect("Fallo plantilla moderna");
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.len() > 1000);
    }

    #[test]
    fn test_generar_ride_compacto() {
        let datos = dummy_datos();
        let pdf = generar_ride_pdf(&datos, "compacta").expect("Fallo plantilla compacta");
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.len() > 1000);
    }

    #[test]
    fn test_extraer_datos_xml() {
        let xml = r#"
            <factura id="comprobante" version="1.1.0">
                <infoTributaria>
                    <ambiente>1</ambiente>
                    <tipoEmision>1</tipoEmision>
                    <razonSocial>ACME CORP</razonSocial>
                    <nombreComercial>ACME</nombreComercial>
                    <ruc>1790000000001</ruc>
                    <claveAcceso>0909202601179000000000110010010000000421234567819</claveAcceso>
                    <codDoc>01</codDoc>
                    <estab>001</estab>
                    <ptoEmi>001</ptoEmi>
                    <secuencial>000000042</secuencial>
                    <dirMatriz>Quito</dirMatriz>
                </infoTributaria>
                <infoFactura>
                    <fechaEmision>09/09/2026</fechaEmision>
                    <dirEstablecimiento>Quito</dirEstablecimiento>
                    <obligadoContabilidad>SI</obligadoContabilidad>
                    <tipoIdentificacionComprador>05</tipoIdentificacionComprador>
                    <razonSocialComprador>CLIENTE PRUEBA</razonSocialComprador>
                    <identificacionComprador>1712345678</identificacionComprador>
                    <totalSinImpuestos>100.00</totalSinImpuestos>
                    <totalDescuento>0.00</totalDescuento>
                    <importeTotal>115.00</importeTotal>
                    <guiaRemision>001-001-000000999</guiaRemision>
                    <pagos>
                        <pago>
                            <formaPago>01</formaPago>
                            <total>115.00</total>
                        </pago>
                    </pagos>
                </infoFactura>
                <detalles>
                    <detalle>
                        <codigoPrincipal>P01</codigoPrincipal>
                        <descripcion>Item 1</descripcion>
                        <cantidad>1.00</cantidad>
                        <precioUnitario>100.00</precioUnitario>
                        <descuento>0.00</descuento>
                        <precioTotalSinImpuesto>100.00</precioTotalSinImpuesto>
                        <impuestos>
                            <impuesto>
                                <codigo>2</codigo>
                                <codigoPorcentaje>4</codigoPorcentaje>
                                <tarifa>15.00</tarifa>
                                <baseImponible>100.00</baseImponible>
                                <valor>15.00</valor>
                            </impuesto>
                        </impuestos>
                    </detalle>
                </detalles>
                <infoAdicional>
                    <campoAdicional nombre="Email">cliente@correo.com</campoAdicional>
                    <campoAdicional nombre="Telefono">0987654321</campoAdicional>
                    <campoAdicional nombre="Direccion">Av. Amazonas y Colón</campoAdicional>
                </infoAdicional>
            </factura>
        "#;
        let emisor = EmisorConfig::default();
        let datos = extraer_datos_ride_de_xml(xml, &emisor).expect("Fallo parsear XML");
        assert_eq!(datos.secuencial, "000000042");
        assert_eq!(datos.cliente.razon_social, "CLIENTE PRUEBA");
        assert_eq!(datos.cliente.email.as_deref(), Some("cliente@correo.com"));
        assert_eq!(datos.cliente.telefono.as_deref(), Some("0987654321"));
        assert_eq!(
            datos.cliente.direccion.as_deref(),
            Some("Av. Amazonas y Colón")
        );
        assert_eq!(datos.guia_remision.as_deref(), Some("001-001-000000999"));
        assert_eq!(datos.detalles.len(), 1);
        assert_eq!(datos.detalles[0].precio_unitario, 100.0);
    }

    #[test]
    fn test_extraer_datos_xml_escapado() {
        let xml_escapado = "&lt;factura id=\"comprobante\"&gt;&lt;infoTributaria&gt;&lt;secuencial&gt;000000077&lt;/secuencial&gt;&lt;claveAcceso&gt;1234567890123456789012345678901234567890123456789&lt;/claveAcceso&gt;&lt;/infoTributaria&gt;&lt;infoFactura&gt;&lt;razonSocialComprador&gt;EMPRESA ESCAPADA&lt;/razonSocialComprador&gt;&lt;identificacionComprador&gt;0999999999001&lt;/identificacionComprador&gt;&lt;tipoIdentificacionComprador&gt;04&lt;/tipoIdentificacionComprador&gt;&lt;/infoFactura&gt;&lt;detalles&gt;&lt;detalle&gt;&lt;codigoPrincipal&gt;ESC-01&lt;/codigoPrincipal&gt;&lt;descripcion&gt;Prueba Escapada&lt;/descripcion&gt;&lt;cantidad&gt;2.00&lt;/cantidad&gt;&lt;precioUnitario&gt;15.00&lt;/precioUnitario&gt;&lt;descuento&gt;1.00&lt;/descuento&gt;&lt;codigoPorcentaje&gt;4&lt;/codigoPorcentaje&gt;&lt;/detalle&gt;&lt;/detalles&gt;&lt;/factura&gt;";
        let emisor = EmisorConfig::default();
        let datos =
            extraer_datos_ride_de_xml(xml_escapado, &emisor).expect("Fallo parsear XML escapado");
        assert_eq!(datos.secuencial, "000000077");
        assert_eq!(datos.cliente.razon_social, "EMPRESA ESCAPADA");
        assert_eq!(datos.detalles.len(), 1);
        assert_eq!(datos.detalles[0].codigo_principal, "ESC-01");
        assert_eq!(datos.detalles[0].descuento, 1.0);
    }
}
