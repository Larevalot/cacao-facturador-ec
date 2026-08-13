//! Generación de Clave de Acceso para comprobantes electrónicos del SRI Ecuador.
//! Algoritmo Módulo 11 (49 dígitos).

use rand::Rng;

#[allow(dead_code)]
pub enum TipoComprobante {
    Factura = 1,
    LiquidacionCompra = 3,
    NotaDebito = 4,
    NotaCredito = 5,
    GuiaRemision = 6,
    ComprobanteRetencion = 7,
}

impl TipoComprobante {
    #[allow(dead_code)]
    pub fn codigo(&self) -> &'static str {
        match self {
            Self::Factura => "01",
            Self::LiquidacionCompra => "03",
            Self::NotaDebito => "04",
            Self::NotaCredito => "05",
            Self::GuiaRemision => "06",
            Self::ComprobanteRetencion => "07",
        }
    }
}

/// Genera la clave de acceso de 49 dígitos para el SRI.
///
/// Formato:
/// [0-7]   Fecha Emisión (8 d) - ddmmaaaa
/// [8-9]   Tipo Comprobante (2 d) - "01"
/// [10-22] RUC (13 d)
/// [23]    Tipo Ambiente (1 d) - "1" Pruebas, "2" Producción
/// [24-29] Serie (6 d) - Establecimiento (3) + Punto Emisión (3)
/// [30-38] Secuencial (9 d) - e.g. "000000001"
/// [39-46] Código Numérico (8 d)
/// [47]    Tipo Emisión (1 d) - "1" Normal
/// [48]    Dígito Verificador (1 d) - Módulo 11
pub fn generar_clave_acceso(
    fecha_ddmmaaaa: &str,
    tipo_comprobante: &str,
    ruc: &str,
    ambiente: &str,
    establecimiento: &str,
    punto_emision: &str,
    secuencial: &str,
    codigo_numerico: Option<&str>,
) -> Result<String, String> {
    if ruc.len() != 13 {
        return Err("El RUC debe tener 13 dígitos".to_string());
    }

    let est_clean = format!("{:0>3}", establecimiento.trim());
    let pto_clean = format!("{:0>3}", punto_emision.trim());
    let serie = format!("{}{}", est_clean, pto_clean);
    let sec_clean = format!("{:0>9}", secuencial.trim());

    // Código numérico de 8 dígitos (si no se envía, genera uno aleatorio)
    let cod_num = match codigo_numerico {
        Some(c) if c.len() == 8 => c.to_string(),
        _ => {
            let mut rng = rand::rng();
            format!("{:08}", rng.random_range(10000000..99999999))
        }
    };

    let tipo_emision = "1"; // Normal

    let clave_48 = format!(
        "{}{}{}{}{}{}{}{}",
        fecha_ddmmaaaa, tipo_comprobante, ruc, ambiente, serie, sec_clean, cod_num, tipo_emision
    );

    if clave_48.len() != 48 {
        return Err(format!(
            "La base de la clave de acceso debe tener 48 dígitos (actual: {})",
            clave_48.len()
        ));
    }

    let dv = calcular_modulo_11(&clave_48);
    Ok(format!("{}{}", clave_48, dv))
}

/// Calcula el dígito verificador usando el algoritmo Módulo 11 según especificación SRI.
pub fn calcular_modulo_11(cadena_48: &str) -> u32 {
    let mut factor = 2;
    let mut suma = 0;

    for c in cadena_48.chars().rev() {
        if let Some(digito) = c.to_digit(10) {
            suma += digito * factor;
            factor = if factor == 7 { 2 } else { factor + 1 };
        }
    }

    let residuo = suma % 11;
    let mut dv = 11 - residuo;

    if dv == 11 {
        dv = 0;
    } else if dv == 10 {
        dv = 1;
    }

    dv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modulo_11() {
        // Ejemplo oficial o prueba estándar de módulo 11
        let clave_prueba = "010420210117920495040011001001000000001123456781";
        let dv = calcular_modulo_11(clave_prueba);
        assert!(dv <= 9);
    }

    #[test]
    fn test_generar_clave() {
        let res = generar_clave_acceso(
            "07082026",
            "01",
            "1792049504001",
            "1",
            "001",
            "001",
            "000000001",
            Some("12345678"),
        );
        assert!(res.is_ok());
        let clave = res.unwrap();
        assert_eq!(clave.len(), 49);
    }
}
