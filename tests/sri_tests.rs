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
    )
    .unwrap();

    assert_eq!(clave.len(), 49);
    assert!(clave.starts_with("070820260117900000000011001001000000001123456781"));
}

#[test]
fn test_formas_pago_con_plazo_en_xml() {
    use cacaofacturador_ec::sri::models::{
        ClienteInfo, DetalleFactura, EmisorConfig, FacturaRequest, FormaPago,
    };
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
        guia_remision: None,
    };

    let xml = construir_xml_factura(
        &emisor,
        &req,
        "1808202601179000000000110010010000000011234567818",
    )
    .unwrap();

    assert!(xml.contains("<formaPago>19</formaPago>"));
    assert!(xml.contains("<total>60.00</total>"));
    assert!(xml.contains("<plazo>30</plazo>"));
    assert!(xml.contains("<unidadTiempo>dias</unidadTiempo>"));
    assert!(xml.contains("<formaPago>01</formaPago>"));
    assert!(xml.contains("<total>55.00</total>"));
}

#[test]
fn test_generacion_pdf_3_modelos_con_campos_completos() {
    use cacaofacturador_ec::sri::models::EmisorConfig;
    use cacaofacturador_ec::sri::pdf_builder::{extraer_datos_ride_de_xml, generar_ride_pdf};

    let xml_ejemplo = r#"&lt;factura id="comprobante" version="1.1.0"&gt;
        &lt;infoTributaria&gt;
            &lt;ambiente&gt;1&lt;/ambiente&gt;
            &lt;tipoEmision&gt;1&lt;/tipoEmision&gt;
            &lt;razonSocial&gt;CACAO SCRIPT S.A.S.&lt;/razonSocial&gt;
            &lt;nombreComercial&gt;Cacao Software&lt;/nombreComercial&gt;
            &lt;ruc&gt;1793214567001&lt;/ruc&gt;
            &lt;claveAcceso&gt;1009202601179321456700110010010000000881234567812&lt;/claveAcceso&gt;
            &lt;codDoc&gt;01&lt;/codDoc&gt;
            &lt;estab&gt;001&lt;/estab&gt;
            &lt;ptoEmi&gt;001&lt;/ptoEmi&gt;
            &lt;secuencial&gt;000000088&lt;/secuencial&gt;
            &lt;dirMatriz&gt;Av. 10 de Agosto y Colón&lt;/dirMatriz&gt;
        &lt;/infoTributaria&gt;
        &lt;infoFactura&gt;
            &lt;fechaEmision&gt;10/09/2026&lt;/fechaEmision&gt;
            &lt;dirEstablecimiento&gt;Av. 10 de Agosto y Colón&lt;/dirEstablecimiento&gt;
            &lt;obligadoContabilidad&gt;SI&lt;/obligadoContabilidad&gt;
            &lt;tipoIdentificacionComprador&gt;04&lt;/tipoIdentificacionComprador&gt;
            &lt;guiaRemision&gt;001-001-000000555&lt;/guiaRemision&gt;
            &lt;razonSocialComprador&gt;ACME LOGISTICS CIA. LTDA.&lt;/razonSocialComprador&gt;
            &lt;identificacionComprador&gt;1790012345001&lt;/identificacionComprador&gt;
            &lt;direccionComprador&gt;Parque Industrial Sur Lote 4&lt;/direccionComprador&gt;
            &lt;totalSinImpuestos&gt;250.00&lt;/totalSinImpuestos&gt;
            &lt;totalDescuento&gt;15.00&lt;/totalDescuento&gt;
            &lt;totalConImpuestos&gt;
                &lt;totalImpuesto&gt;
                    &lt;codigo&gt;2&lt;/codigo&gt;
                    &lt;codigoPorcentaje&gt;4&lt;/codigoPorcentaje&gt;
                    &lt;baseImponible&gt;200.00&lt;/baseImponible&gt;
                    &lt;valor&gt;30.00&lt;/valor&gt;
                &lt;/totalImpuesto&gt;
            &lt;/totalConImpuestos&gt;
            &lt;propina&gt;0.00&lt;/propina&gt;
            &lt;importeTotal&gt;280.00&lt;/importeTotal&gt;
            &lt;pagos&gt;
                &lt;pago&gt;
                    &lt;formaPago&gt;20&lt;/formaPago&gt;
                    &lt;total&gt;280.00&lt;/total&gt;
                    &lt;plazo&gt;30&lt;/plazo&gt;
                    &lt;unidadTiempo&gt;dias&lt;/unidadTiempo&gt;
                &lt;/pago&gt;
            &lt;/pagos&gt;
        &lt;/infoFactura&gt;
        &lt;detalles&gt;
            &lt;detalle&gt;
                &lt;codigoPrincipal&gt;SRV-001&lt;/codigoPrincipal&gt;
                &lt;descripcion&gt;Mantenimiento de Servidores y Cloud&lt;/descripcion&gt;
                &lt;cantidad&gt;2.00&lt;/cantidad&gt;
                &lt;precioUnitario&gt;100.00&lt;/precioUnitario&gt;
                &lt;descuento&gt;10.00&lt;/descuento&gt;
                &lt;precioTotalSinImpuesto&gt;190.00&lt;/precioTotalSinImpuesto&gt;
                &lt;impuestos&gt;
                    &lt;impuesto&gt;
                        &lt;codigo&gt;2&lt;/codigo&gt;
                        &lt;codigoPorcentaje&gt;4&lt;/codigoPorcentaje&gt;
                        &lt;tarifa&gt;15.00&lt;/tarifa&gt;
                        &lt;baseImponible&gt;190.00&lt;/baseImponible&gt;
                        &lt;valor&gt;28.50&lt;/valor&gt;
                    &lt;/impuesto&gt;
                &lt;/impuestos&gt;
            &lt;/detalle&gt;
            &lt;detalle&gt;
                &lt;codigoPrincipal&gt;LIC-002&lt;/codigoPrincipal&gt;
                &lt;descripcion&gt;Licencia Software Facturador Anual&lt;/descripcion&gt;
                &lt;cantidad&gt;1.00&lt;/cantidad&gt;
                &lt;precioUnitario&gt;60.00&lt;/precioUnitario&gt;
                &lt;descuento&gt;5.00&lt;/descuento&gt;
                &lt;precioTotalSinImpuesto&gt;55.00&lt;/precioTotalSinImpuesto&gt;
                &lt;impuestos&gt;
                    &lt;impuesto&gt;
                        &lt;codigo&gt;2&lt;/codigo&gt;
                        &lt;codigoPorcentaje&gt;0&lt;/codigoPorcentaje&gt;
                        &lt;tarifa&gt;0.00&lt;/tarifa&gt;
                        &lt;baseImponible&gt;55.00&lt;/baseImponible&gt;
                        &lt;valor&gt;0.00&lt;/valor&gt;
                    &lt;/impuesto&gt;
                &lt;/impuestos&gt;
            &lt;/detalle&gt;
        &lt;/detalles&gt;
        &lt;infoAdicional&gt;
            &lt;campoAdicional nombre="Email"&gt;contabilidad@acmelogistics.ec&lt;/campoAdicional&gt;
            &lt;campoAdicional nombre="Telefono"&gt;022987654&lt;/campoAdicional&gt;
        &lt;/infoAdicional&gt;
    &lt;/factura&gt;"#;

    let mut emisor = EmisorConfig::default();
    let sample_logo = "/home/larevalo/Documents/AMD_Radeon_logo_2019.png";
    if std::path::Path::new(sample_logo).exists() {
        emisor.logo_path = Some(sample_logo.to_string());
    }
    emisor.razon_social = "LUIS FERNANDO AREVALO TORRES".to_string();
    emisor.nombre_comercial = Some("CCS EC".to_string());
    emisor.dir_matriz = "Santa Rosa, El Oro, Ecuador".to_string();
    emisor.dir_establecimiento = "Santa Rosa, El Oro, Ecuador".to_string();
    emisor.ruc = "0706497450001".to_string();
    emisor.cod_establecimiento = "001".to_string();
    emisor.pto_emision = "001".to_string();
    emisor.obligado_contabilidad = "NO".to_string();
    emisor.regimen_rimpe = Some("CONTRIBUYENTE RÉGIMEN RIMPE".to_string());
    emisor.ambiente = "1".to_string();

    let datos =
        extraer_datos_ride_de_xml(xml_ejemplo, &emisor).expect("Error al extraer datos RIDE");

    assert_eq!(datos.secuencial, "000000088");
    assert_eq!(
        datos.clave_acceso,
        "1009202601179321456700110010010000000881234567812"
    );
    assert_eq!(
        datos.numero_autorizacion,
        "1009202601179321456700110010010000000881234567812"
    );
    assert_eq!(datos.cliente.razon_social, "ACME LOGISTICS CIA. LTDA.");
    assert_eq!(datos.cliente.identificacion, "1790012345001");
    assert_eq!(
        datos.cliente.direccion.as_deref(),
        Some("Parque Industrial Sur Lote 4")
    );
    assert_eq!(
        datos.cliente.email.as_deref(),
        Some("contabilidad@acmelogistics.ec")
    );
    assert_eq!(datos.cliente.telefono.as_deref(), Some("022987654"));
    assert_eq!(datos.guia_remision.as_deref(), Some("001-001-000000555"));
    assert_eq!(datos.detalles.len(), 2);
    assert_eq!(datos.detalles[0].codigo_principal, "SRV-001");
    assert_eq!(datos.detalles[0].cantidad, 2.0);
    assert_eq!(datos.detalles[0].descuento, 10.0);
    assert_eq!(datos.detalles[1].codigo_principal, "LIC-002");
    assert_eq!(datos.detalles[1].descuento, 5.0);

    let tot = datos.calcular_totales();
    assert_eq!(tot.subtotal_15, 190.0);
    assert_eq!(tot.subtotal_0, 55.0);
    assert_eq!(tot.total_descuento, 15.0);

    // Generar y verificar bytes de las 3 plantillas
    for plantilla in &["clasica", "moderna", "compacta"] {
        let bytes = generar_ride_pdf(&datos, plantilla).expect("Fallo generando PDF");
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes.len() > 1000);

        let path = std::env::temp_dir().join(format!("ride_test_{}.pdf", plantilla));
        let _ = std::fs::write(&path, &bytes);
    }
}
