#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Facturador Electrónico SRI Ecuador e Inventario (Rust + SQLite + SQLx + Iced GUI)
//!
//! Soporta modo Servidor Web (Axum), modo CLI por Consola y Aplicación de Escritorio Nativa (Iced).

mod config;
mod db;
pub mod gui;
mod sri;
mod web;

use crate::config::cargar_configuracion;
use crate::db::inicializar_db;
use crate::sri::clave_acceso::generar_clave_acceso;
use crate::sri::client::SriClient;
use crate::sri::models::FacturaRequest;
use crate::sri::xades_signer::firmar_xml;
use crate::sri::xml_builder::construir_xml_factura;
use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar y mantener vivos los proveedores Legacy y Default de OpenSSL 3.0 para PKCS12KDF, RC2, 3DES, etc.
    openssl::init();
    let _legacy_prov = openssl::provider::Provider::load(None, "legacy");
    let _default_prov = openssl::provider::Provider::load(None, "default");

    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--cli" {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(ejecutar_modo_cli(&args))?;
    } else if args.len() > 1 && args[1] == "--serve" {
        let rt = tokio::runtime::Runtime::new()?;
        println!("====================================================");
        println!("  CACAO FACTURADOR & INVENTARIO (SERVIDOR WEB)");
        println!("====================================================");
        rt.block_on(async {
            let pool = inicializar_db().await.expect("Error conectando a SQLite");
            web::iniciar_servidor_web(8080, pool).await
        })?;
    } else {
        // Modo Aplicación de Escritorio Nativa (Iced)
        println!("====================================================");
        println!("  CACAO FACTURADOR & INVENTARIO (DESKTOP APP - ICED)");
        println!("====================================================");

        let rt = tokio::runtime::Runtime::new()?;
        let pool =
            rt.block_on(async { inicializar_db().await.expect("Error conectando a SQLite") });

        // Iniciar servidor web Axum local en segundo plano (para integraciones o APIs)
        let pool_server = pool.clone();
        std::thread::spawn(move || {
            let rt_server = tokio::runtime::Runtime::new().unwrap();
            rt_server.block_on(async {
                let _ = web::iniciar_servidor_web(8080, pool_server).await;
            });
        });

        // Lanzar interfaz nativa Iced
        gui::run(pool)?;
    }

    Ok(())
}

async fn ejecutar_modo_cli(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Modo CLI seleccionado.");

    if args.len() < 3 {
        println!("Uso: cacaofacturador-ec --cli <ruta_factura.json> [password_p12]");
        return Ok(());
    }

    let json_path = &args[2];
    let password_override = args.get(3).cloned();

    let cfg = cargar_configuracion();
    println!("Emisor RUC: {}", cfg.ruc);
    println!("Razón Social: {}", cfg.razon_social);
    println!(
        "Ambiente SRI: {}",
        if cfg.ambiente == "2" {
            "Producción"
        } else {
            "Pruebas"
        }
    );

    let json_content = fs::read_to_string(json_path)
        .map_err(|e| format!("Error al abrir archivo {}: {}", json_path, e))?;

    let factura: FacturaRequest = serde_json::from_str(&json_content)
        .map_err(|e| format!("Error en formato JSON de la factura: {}", e))?;

    let clave = generar_clave_acceso(
        &factura.fecha_emision.replace('/', ""),
        "01",
        &cfg.ruc,
        &cfg.ambiente,
        &cfg.cod_establecimiento,
        &cfg.pto_emision,
        &factura.secuencial,
        None,
    )?;

    println!("Clave de Acceso Generada: {}", clave);

    let xml_unsigned = construir_xml_factura(&cfg, &factura, &clave)?;
    fs::write("factura_unsigned.xml", &xml_unsigned)?;
    println!("XML sin firmar guardado en 'factura_unsigned.xml'");

    let p12_path = cfg
        .p12_path
        .ok_or("Firma electrónica .p12 no configurada en config.json")?;
    let p12_password = password_override
        .or(cfg.p12_password)
        .ok_or("Contraseña de la firma .p12 requerida")?;

    let p12_bytes = fs::read(&p12_path)?;
    println!("Firmando XML con XAdES-BES...");
    let xml_firmado = firmar_xml(&p12_bytes, &p12_password, &xml_unsigned)?;
    fs::write("factura_firmada.xml", &xml_firmado)?;
    println!("XML firmado guardado en 'factura_firmada.xml'");

    println!("Enviando comprobante al SRI...");
    let client = SriClient::new(&cfg.ambiente);
    let recepcion = client.enviar_recepcion(&xml_firmado).await?;

    println!("Estado Recepción SRI: {}", recepcion.estado);
    if recepcion.estado == "RECIBIDA" {
        println!("Esperando procesamiento de autorización...");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let aut = client.consultar_autorizacion(&clave).await?;
        println!("Estado Autorización SRI: {}", aut.estado);
        if let Some(num) = aut.numero_autorizacion {
            println!("N° Autorización: {}", num);
        }
        if let Some(fec) = aut.fecha_autorizacion {
            println!("Fecha Autorización: {}", fec);
        }
        if let Some(xml_aut) = aut.xml_autorizado {
            fs::write("factura_autorizada.xml", &xml_aut)?;
            println!("XML Autorizado guardado en 'factura_autorizada.xml'");
        }
    } else {
        println!("Comprobante Devuelto o con Errores:");
        for comp in recepcion.comprobantes {
            for m in comp.mensajes {
                println!(
                    "   - [{}] {} ({:?})",
                    m.tipo, m.mensaje, m.informacion_adicional
                );
            }
        }
    }

    Ok(())
}
