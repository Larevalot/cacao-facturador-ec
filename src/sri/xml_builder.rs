//! Constructor de XML de Factura conforme a la Ficha Técnica del SRI de Ecuador (versión 1.1.0).

use crate::sri::models::{EmisorConfig, FacturaRequest};
use std::collections::HashMap;

pub fn construir_xml_factura(
    emisor: &EmisorConfig,
    request: &FacturaRequest,
    clave_acceso: &str,
) -> Result<String, String> {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<factura id=\"comprobante\" version=\"1.1.0\">\n");

    // 1. infoTributaria
    xml.push_str("  <infoTributaria>\n");
    xml.push_str(&format!("    <ambiente>{}</ambiente>\n", emisor.ambiente));
    xml.push_str("    <tipoEmision>1</tipoEmision>\n");
    xml.push_str(&format!("    <razonSocial>{}</razonSocial>\n", escape_xml(&emisor.razon_social)));

    if let Some(nc) = &emisor.nombre_comercial {
        if !nc.trim().is_empty() {
            xml.push_str(&format!("    <nombreComercial>{}</nombreComercial>\n", escape_xml(nc)));
        }
    }

    xml.push_str(&format!("    <ruc>{}</ruc>\n", emisor.ruc));
    xml.push_str(&format!("    <claveAcceso>{}</claveAcceso>\n", clave_acceso));
    xml.push_str("    <codDoc>01</codDoc>\n");
    xml.push_str(&format!("    <estab>{:0>3}</estab>\n", emisor.cod_establecimiento));
    xml.push_str(&format!("    <ptoEmi>{:0>3}</ptoEmi>\n", emisor.pto_emision));
    xml.push_str(&format!("    <secuencial>{:0>9}</secuencial>\n", request.secuencial));
    xml.push_str(&format!("    <dirMatriz>{}</dirMatriz>\n", escape_xml(&emisor.dir_matriz)));

    if let Some(micro) = &emisor.regimen_microempresas {
        if !micro.trim().is_empty() {
            xml.push_str(&format!("    <regimenMicroempresas>{}</regimenMicroempresas>\n", escape_xml(micro)));
        }
    }
    if let Some(rimpe) = &emisor.regimen_rimpe {
        let rimpe_str = rimpe.trim();
        if !rimpe_str.is_empty() {
            let rimpe_normalizado = if rimpe_str.to_uppercase().contains("NEGOCIO POPULAR") {
                "CONTRIBUYENTE NEGOCIO POPULAR - RÉGIMEN RIMPE"
            } else if rimpe_str.to_uppercase().contains("RIMPE") {
                "CONTRIBUYENTE RÉGIMEN RIMPE"
            } else {
                rimpe_str
            };
            xml.push_str(&format!("    <contribuyenteRimpe>{}</contribuyenteRimpe>\n", escape_xml(rimpe_normalizado)));
        }
    }
    xml.push_str("  </infoTributaria>\n");

    // Cálculos de la factura
    let mut total_sin_impuestos = 0.0;
    let mut total_descuento = 0.0;
    let mut resumen_iva: HashMap<String, (f64, f64, f64)> = HashMap::new(); // cod_porcentaje -> (base, tarifa, valor)

    for d in &request.detalles {
        let subtotal = d.subtotal();
        total_sin_impuestos += subtotal;
        total_descuento += d.descuento;

        let valor_iva = d.valor_iva();
        let entry = resumen_iva
            .entry(d.codigo_porcentaje_iva.clone())
            .or_insert((0.0, d.tarifa_iva, 0.0));
        entry.0 += subtotal;
        entry.2 += valor_iva;
    }

    let mut total_iva = 0.0;
    for (_, (_, _, val)) in &resumen_iva {
        total_iva += val;
    }

    let importe_total = total_sin_impuestos + total_iva + request.propina;

    // 2. infoFactura
    xml.push_str("  <infoFactura>\n");
    xml.push_str(&format!("    <fechaEmision>{}</fechaEmision>\n", request.fecha_emision));
    xml.push_str(&format!("    <dirEstablecimiento>{}</dirEstablecimiento>\n", escape_xml(&emisor.dir_establecimiento)));

    if let Some(ce) = &emisor.contribuyente_especial {
        if !ce.trim().is_empty() {
            xml.push_str(&format!("    <contribuyenteEspecial>{}</contribuyenteEspecial>\n", escape_xml(ce)));
        }
    }

    xml.push_str(&format!("    <obligadoContabilidad>{}</obligadoContabilidad>\n", emisor.obligado_contabilidad));
    xml.push_str(&format!("    <tipoIdentificacionComprador>{}</tipoIdentificacionComprador>\n", request.cliente.tipo_identificacion));
    xml.push_str(&format!("    <razonSocialComprador>{}</razonSocialComprador>\n", escape_xml(&request.cliente.razon_social)));
    xml.push_str(&format!("    <identificacionComprador>{}</identificacionComprador>\n", request.cliente.identificacion));

    if let Some(dir_c) = &request.cliente.direccion {
        if !dir_c.trim().is_empty() {
            xml.push_str(&format!("    <direccionComprador>{}</direccionComprador>\n", escape_xml(dir_c)));
        }
    }

    xml.push_str(&format!("    <totalSinImpuestos>{:.2}</totalSinImpuestos>\n", total_sin_impuestos));
    xml.push_str(&format!("    <totalDescuento>{:.2}</totalDescuento>\n", total_descuento));

    // totalConImpuestos
    xml.push_str("    <totalConImpuestos>\n");
    for (cod_pct, (base, tarifa, val)) in &resumen_iva {
        xml.push_str("      <totalImpuesto>\n");
        xml.push_str("        <codigo>2</codigo>\n");
        xml.push_str(&format!("        <codigoPorcentaje>{}</codigoPorcentaje>\n", cod_pct));
        xml.push_str(&format!("        <baseImponible>{:.2}</baseImponible>\n", base));
        xml.push_str(&format!("        <tarifa>{:.2}</tarifa>\n", tarifa));
        xml.push_str(&format!("        <valor>{:.2}</valor>\n", val));
        xml.push_str("      </totalImpuesto>\n");
    }
    xml.push_str("    </totalConImpuestos>\n");

    xml.push_str(&format!("    <propina>{:.2}</propina>\n", request.propina));
    xml.push_str(&format!("    <importeTotal>{:.2}</importeTotal>\n", importe_total));
    xml.push_str("    <moneda>DOLAR</moneda>\n");

    // pagos
    xml.push_str("    <pagos>\n");
    if request.formas_pago.is_empty() {
        // Pago por defecto si no se especificó
        xml.push_str("      <pago>\n");
        xml.push_str("        <formaPago>01</formaPago>\n");
        xml.push_str(&format!("        <total>{:.2}</total>\n", importe_total));
        xml.push_str("      </pago>\n");
    } else {
        for p in &request.formas_pago {
            xml.push_str("      <pago>\n");
            xml.push_str(&format!("        <formaPago>{}</formaPago>\n", p.forma_pago));
            xml.push_str(&format!("        <total>{:.2}</total>\n", p.total));
            if let Some(plazo) = p.plazo {
                if plazo > 0 {
                    xml.push_str(&format!("        <plazo>{}</plazo>\n", plazo));
                    if let Some(ut) = &p.unidad_tiempo {
                        let trimmed = ut.trim();
                        if !trimmed.is_empty() && trimmed != "none" {
                            xml.push_str(&format!("        <unidadTiempo>{}</unidadTiempo>\n", trimmed));
                        } else {
                            xml.push_str("        <unidadTiempo>dias</unidadTiempo>\n");
                        }
                    } else {
                        xml.push_str("        <unidadTiempo>dias</unidadTiempo>\n");
                    }
                }
            }
            xml.push_str("      </pago>\n");
        }
    }
    xml.push_str("    </pagos>\n");
    xml.push_str("  </infoFactura>\n");

    // 3. detalles
    xml.push_str("  <detalles>\n");
    for d in &request.detalles {
        xml.push_str("    <detalle>\n");
        xml.push_str(&format!("      <codigoPrincipal>{}</codigoPrincipal>\n", escape_xml(&d.codigo_principal)));
        if let Some(aux) = &d.codigo_auxiliar {
            if !aux.trim().is_empty() {
                xml.push_str(&format!("      <codigoAuxiliar>{}</codigoAuxiliar>\n", escape_xml(aux)));
            }
        }
        xml.push_str(&format!("      <descripcion>{}</descripcion>\n", escape_xml(&d.descripcion)));
        xml.push_str(&format!("      <cantidad>{:.2}</cantidad>\n", d.cantidad));
        xml.push_str(&format!("      <precioUnitario>{:.2}</precioUnitario>\n", d.precio_unitario));
        xml.push_str(&format!("      <descuento>{:.2}</descuento>\n", d.descuento));
        xml.push_str(&format!("      <precioTotalSinImpuesto>{:.2}</precioTotalSinImpuesto>\n", d.subtotal()));

        xml.push_str("      <impuestos>\n");
        xml.push_str("        <impuesto>\n");
        xml.push_str("          <codigo>2</codigo>\n");
        xml.push_str(&format!("          <codigoPorcentaje>{}</codigoPorcentaje>\n", d.codigo_porcentaje_iva));
        xml.push_str(&format!("          <tarifa>{:.2}</tarifa>\n", d.tarifa_iva));
        xml.push_str(&format!("          <baseImponible>{:.2}</baseImponible>\n", d.subtotal()));
        xml.push_str(&format!("          <valor>{:.2}</valor>\n", d.valor_iva()));
        xml.push_str("        </impuesto>\n");
        xml.push_str("      </impuestos>\n");
        xml.push_str("    </detalle>\n");
    }
    xml.push_str("  </detalles>\n");

    // 4. infoAdicional (opcional, ej: email, teléfono, plazo de pago)
    let mut campos_adicionales = Vec::new();
    if let Some(email) = &request.cliente.email {
        let e = email.trim();
        if !e.is_empty() {
            campos_adicionales.push(("Email", escape_xml(e)));
        }
    }
    if let Some(tel) = &request.cliente.telefono {
        let t = tel.trim();
        if !t.is_empty() {
            campos_adicionales.push(("Telefono", escape_xml(t)));
        }
    }

    // Agregar plazo y unidad de tiempo en infoAdicional para visualización en RIDE PDF
    for p in &request.formas_pago {
        if let Some(plazo) = p.plazo {
            if plazo > 0 {
                let ut_label = match p.unidad_tiempo.as_deref().unwrap_or("dias") {
                    "dias" | "días" => "Días",
                    "meses" => "Meses",
                    "anios" | "años" => "Años",
                    other => other,
                };
                let nombre_fp = match p.forma_pago.as_str() {
                    "01" => "Sin utilización del sistema financiero",
                    "15" => "Compensación de deudas",
                    "16" => "Tarjeta de débito",
                    "17" => "Dinero electrónico",
                    "18" => "Tarjeta prepago",
                    "19" => "Tarjeta de crédito",
                    "20" => "Otros con utilización del sistema financiero",
                    "21" => "Endoso de títulos",
                    _ => "Pago",
                };
                campos_adicionales.push((
                    "Plazo de Pago",
                    escape_xml(&format!("{}: {} {} (${:.2})", nombre_fp, plazo, ut_label, p.total)),
                ));
            }
        }
    }

    if !campos_adicionales.is_empty() {
        xml.push_str("  <infoAdicional>\n");
        for (nombre, val) in campos_adicionales {
            xml.push_str(&format!("    <campoAdicional nombre=\"{}\">{}</campoAdicional>\n", nombre, val));
        }
        xml.push_str("  </infoAdicional>\n");
    }

    xml.push_str("</factura>");

    Ok(xml)
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
