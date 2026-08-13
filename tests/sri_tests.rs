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
