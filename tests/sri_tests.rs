use cacaofacturador_ec::sri::clave_acceso::{calcular_modulo_11, generar_clave_acceso};

#[test]
fn test_calculo_modulo_11() {
    let clave_48 = "070820260117900000000011001001000000001123456781";
    let dv = calcular_modulo_11(clave_48);
    assert!(dv <= 9);
}

#[test]
fn test_generar_clave_acceso_longitud() {
    let clave = generar_clave_acceso(
        "07082026",
        "01",
        "1790000000001",
        "1",
        "001",
        "001",
        "000000001",
        Some("12345678"),
    ).unwrap();

    assert_eq!(clave.len(), 49);
    assert!(clave.starts_with("070820260117900000000011001001000000001123456781"));
}

#[test]
fn test_formas_pago_con_plazo_en_xml() {
    use cacaofacturador_ec::sri::models::{ClienteInfo, DetalleFactura, EmisorConfig, FacturaRequest, FormaPago};
    use cacaofacturador_ec::sri::xml_builder::construir_xml_factura;

    let emisor = EmisorConfig::default();

    let req = FacturaRequest {
        secuencial: "000000001".to_string(),
        fecha_emision: "18/08/2026".to_string(),
        cliente: ClienteInfo {
            tipo_identificacion: "05".to_string(),
            identificacion: "1712345678".to_string(),
            razon_social: "JUAN PEREZ".to_string(),
            direccion: Some("Quito".to_string()),
            email: Some("juan@test.com".to_string()),
            telefono: Some("0999999999".to_string()),
        },
        detalles: vec![DetalleFactura {
            codigo_principal: "PRD-01".to_string(),
            codigo_auxiliar: None,
            descripcion: "PRODUCTO PRUEBA".to_string(),
            cantidad: 1.0,
            precio_unitario: 100.0,
            descuento: 0.0,
            codigo_porcentaje_iva: "4".to_string(),
            tarifa_iva: 15.0,
        }],
        formas_pago: vec![
            FormaPago {
                forma_pago: "19".to_string(), // Tarjeta de Crédito
                total: 60.0,
                plazo: Some(30),
                unidad_tiempo: Some("dias".to_string()),
            },
            FormaPago {
                forma_pago: "01".to_string(), // Sin utilización del sistema financiero
                total: 55.0,
                plazo: None,
                unidad_tiempo: None,
            },
        ],
        propina: 0.0,
    };

    let xml = construir_xml_factura(&emisor, &req, "1808202601179000000000110010010000000011234567818").unwrap();
    
    assert!(xml.contains("<formaPago>19</formaPago>"));
    assert!(xml.contains("<total>60.00</total>"));
    assert!(xml.contains("<plazo>30</plazo>"));
    assert!(xml.contains("<unidadTiempo>dias</unidadTiempo>"));
    assert!(xml.contains("<formaPago>01</formaPago>"));
    assert!(xml.contains("<total>55.00</total>"));
}
