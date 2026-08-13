//! Servidor Web integrado con Axum para Cacao Facturador e Inventario (SQLite + SQLx).

use crate::config::{cargar_configuracion, guardar_configuracion};
use crate::db::clientes::{
    buscar_cliente_por_identificacion, guardar_o_actualizar_cliente, listar_clientes, Cliente,
};
use crate::db::facturas::{guardar_factura_db, listar_historial_facturas, FacturaGuardada};
use crate::db::productos::{
    actualizar_producto, crear_producto, descontar_stock, eliminar_producto, listar_productos,
    NuevoProductoRequest, Producto,
};
use crate::sri::clave_acceso::generar_clave_acceso;
use crate::sri::client::SriClient;
use crate::sri::models::{EmisorConfig, FacturaRequest, RespuestaSRI};
use crate::sri::xades_signer::firmar_xml;
use crate::sri::xml_builder::construir_xml_factura;
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::fs;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Mutex<EmisorConfig>>,
    pub db: SqlitePool,
}

#[derive(Debug, Deserialize)]
pub struct FacturarPayload {
    pub factura: FacturaRequest,
    pub password_p12: Option<String>,
}

use tower_http::services::{ServeDir, ServeFile};

pub async fn iniciar_servidor_web(puerto: u16, db_pool: SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let initial_config = cargar_configuracion();
    let state = AppState {
        config: Arc::new(Mutex::new(initial_config)),
        db: db_pool,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let serve_dir = ServeDir::new("dist").fallback(ServeFile::new("dist/index.html"));

    let app = Router::new()
        .fallback_service(serve_dir)
        .route("/api/config", get(get_config).post(update_config))
        .route("/api/upload-p12", post(upload_p12))
        .route("/api/delete-p12", delete(delete_p12))
        .route("/api/facturar", post(emitir_factura))
        // Rutas API de Inventario
        .route("/api/productos", get(get_productos).post(post_producto))
        .route("/api/productos/{id}", put(put_producto).delete(del_producto))
        // Rutas API de Clientes
        .route("/api/clientes", get(get_clientes).post(post_cliente))
        .route("/api/clientes/{identificacion}", get(get_cliente_by_id))
        // Ruta API de Historial Facturas
        .route("/api/facturas", get(get_facturas_historial))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], puerto));
    println!("Cacao Facturador & Inventario (Svelte 5 + Rust + SQLite) corriendo en: http://localhost:{}", puerto);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_config(State(state): State<AppState>) -> Json<EmisorConfig> {
    let cfg = state.config.lock().unwrap().clone();
    Json(cfg)
}

async fn update_config(
    State(state): State<AppState>,
    Json(new_cfg): Json<EmisorConfig>,
) -> Result<Json<EmisorConfig>, (StatusCode, String)> {
    guardar_configuracion(&new_cfg).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let mut cfg = state.config.lock().unwrap();
    *cfg = new_cfg.clone();
    Ok(Json(new_cfg))
}

async fn upload_p12(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut bytes_data = Vec::new();
    let mut filename = "firma.p12".to_string();

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        if field.name() == Some("file") {
            if let Some(name) = field.file_name() {
                filename = name.to_string();
            }
            bytes_data = field.bytes().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?.to_vec();
        }
    }

    if bytes_data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No se subió ningún archivo".to_string()));
    }

    let mut dir_path = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    dir_path.push(".cacaofacturador");
    let _ = fs::create_dir_all(&dir_path);
    dir_path.push(format!("uploads_{}", filename));
    let save_path_str = dir_path.to_string_lossy().to_string();

    fs::write(&dir_path, &bytes_data).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut cfg = state.config.lock().unwrap();
    cfg.p12_path = Some(save_path_str.clone());
    let _ = guardar_configuracion(&cfg);

    Ok(Json(serde_json::json!({
        "status": "ok",
        "path": save_path_str
    })))
}

async fn delete_p12(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cfg = state.config.lock().unwrap();
    if let Some(ref path) = cfg.p12_path {
        let _ = fs::remove_file(path);
    }
    cfg.p12_path = None;
    cfg.p12_password = None;
    let _ = guardar_configuracion(&cfg);

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

/* ==================== ENDPOINTS DE INVENTARIO (SQLite) ==================== */

async fn get_productos(State(state): State<AppState>) -> Result<Json<Vec<Producto>>, (StatusCode, String)> {
    let productos = listar_productos(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error al consultar inventario en SQLite: {}", e)))?;
    Ok(Json(productos))
}

async fn post_producto(
    State(state): State<AppState>,
    Json(req): Json<NuevoProductoRequest>,
) -> Result<Json<Producto>, (StatusCode, String)> {
    let prod = crear_producto(&state.db, &req)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Error al guardar producto en SQLite: {}", e)))?;
    Ok(Json(prod))
}

async fn put_producto(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<NuevoProductoRequest>,
) -> Result<Json<Producto>, (StatusCode, String)> {
    let prod = actualizar_producto(&state.db, id, &req)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Error al actualizar producto en SQLite: {}", e)))?;
    Ok(Json(prod))
}

async fn del_producto(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    eliminar_producto(&state.db, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error al eliminar producto: {}", e)))?;
    Ok(Json(serde_json::json!({ "status": "deleted", "id": id })))
}

async fn get_facturas_historial(
    State(state): State<AppState>,
) -> Result<Json<Vec<FacturaGuardada>>, (StatusCode, String)> {
    let facturas = listar_historial_facturas(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error consultando historial de facturas: {}", e)))?;
    Ok(Json(facturas))
}

/* ==================== ENDPOINTS DE CLIENTES ==================== */

async fn get_clientes(State(state): State<AppState>) -> Result<Json<Vec<Cliente>>, (StatusCode, String)> {
    listar_clientes(&state.db)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error consultando clientes: {}", e)))
}

async fn get_cliente_by_id(
    State(state): State<AppState>,
    Path(identificacion): Path<String>,
) -> Result<Json<Option<Cliente>>, (StatusCode, String)> {
    buscar_cliente_por_identificacion(&state.db, &identificacion)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error buscando cliente: {}", e)))
}

async fn post_cliente(
    State(state): State<AppState>,
    Json(c): Json<Cliente>,
) -> Result<Json<Cliente>, (StatusCode, String)> {
    guardar_o_actualizar_cliente(
        &state.db,
        &c.tipo_identificacion,
        &c.identificacion,
        &c.razon_social,
        c.direccion.as_deref(),
        c.email.as_deref(),
        c.telefono.as_deref(),
    )
    .await
    .map(Json)
    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Error guardando cliente: {}", e)))
}

/* ==================== EMISIÓN DE FACTURA + DESCUENTO DE INVENTARIO ==================== */

async fn emitir_factura(
    State(state): State<AppState>,
    Json(payload): Json<FacturarPayload>,
) -> Result<Json<RespuestaSRI>, (StatusCode, String)> {
    let cfg = state.config.lock().unwrap().clone();

    // Guardar o actualizar automáticamente la información del cliente en SQLite
    let _ = guardar_o_actualizar_cliente(
        &state.db,
        &payload.factura.cliente.tipo_identificacion,
        &payload.factura.cliente.identificacion,
        &payload.factura.cliente.razon_social,
        payload.factura.cliente.direccion.as_deref(),
        payload.factura.cliente.email.as_deref(),
        payload.factura.cliente.telefono.as_deref(),
    ).await;

    // 1. Clave de acceso
    let clave = generar_clave_acceso(
        &payload.factura.fecha_emision.replace('/', ""),
        "01",
        &cfg.ruc,
        &cfg.ambiente,
        &cfg.cod_establecimiento,
        &cfg.pto_emision,
        &payload.factura.secuencial,
        None,
    ).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // 2. XML Unsigned
    let xml_unsigned = construir_xml_factura(&cfg, &payload.factura, &clave)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // 3. Firmar XML
    let p12_path = cfg.p12_path.clone().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, "Firma electrónica .p12 no configurada. Por favor sube tu archivo .p12".to_string())
    })?;

    let p12_password = match payload.password_p12 {
        Some(ref p) if !p.trim().is_empty() => p.trim().to_string(),
        _ => match cfg.p12_password {
            Some(ref p) if !p.trim().is_empty() => p.trim().to_string(),
            _ => return Err((
                StatusCode::BAD_REQUEST,
                "Contraseña de firma .p12 requerida. Por favor ingrésala en Configuración.".to_string()
            )),
        },
    };

    let p12_bytes = fs::read(&p12_path).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Error al leer el archivo .p12 en '{}': {}", p12_path, e))
    })?;

    let xml_firmado = firmar_xml(&p12_bytes, &p12_password, &xml_unsigned)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Error en firmado XAdES-BES: {}", e)))?;

    // 4. Conectar con SRI
    let sri = SriClient::new(&cfg.ambiente);

    // Enviar a Recepción
    let recepcion = sri.enviar_recepcion(&xml_firmado).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Error en webservice de recepción SRI: {}", e))
    })?;

    let mut respuesta = RespuestaSRI {
        estado: recepcion.estado.clone(),
        clave_acceso: clave.clone(),
        numero_autorizacion: None,
        fecha_autorizacion: None,
        ambiente: if cfg.ambiente == "2" { "Producción".to_string() } else { "Pruebas".to_string() },
        mensajes: Vec::new(),
        xml_firmado: Some(xml_firmado.clone()),
        xml_autorizado: None,
    };

    if let Some(comp) = recepcion.comprobantes.first() {
        for m in &comp.mensajes {
            let info = match &m.informacion_adicional {
                Some(i) if !i.trim().is_empty() => format!(" ({})", i.trim()),
                _ => String::new(),
            };
            respuesta.mensajes.push(format!("[{}] {}{}", m.tipo, m.mensaje, info));
        }
    }

    let es_recibida_o_procesando = recepcion.estado == "RECIBIDA"
        || respuesta.estado == "EN PROCESO"
        || respuesta.mensajes.iter().any(|m| m.contains("EN PROCESAMIENTO") || m.contains("EN PROCESO"));

    if es_recibida_o_procesando {
        for intento in 0..6 {
            tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

            match sri.consultar_autorizacion(&clave).await {
                Ok(autorizacion) => {
                    respuesta.estado = autorizacion.estado.clone();
                    if autorizacion.numero_autorizacion.is_some() {
                        respuesta.numero_autorizacion = autorizacion.numero_autorizacion;
                    }
                    if autorizacion.fecha_autorizacion.is_some() {
                        respuesta.fecha_autorizacion = autorizacion.fecha_autorizacion;
                    }
                    if autorizacion.xml_autorizado.is_some() {
                        respuesta.xml_autorizado = autorizacion.xml_autorizado;
                    }

                    for m in autorizacion.mensajes {
                        let info = match &m.informacion_adicional {
                            Some(i) if !i.trim().is_empty() => format!(" ({})", i.trim()),
                            _ => String::new(),
                        };
                        let msg = format!("[{}] {}{}", m.tipo, m.mensaje, info);
                        if !respuesta.mensajes.contains(&msg) {
                            respuesta.mensajes.push(msg);
                        }
                    }

                    if respuesta.estado == "AUTORIZADO" || respuesta.estado == "RECHAZADO" || respuesta.estado == "NO AUTORIZADO" {
                        break;
                    }
                }
                Err(e) => {
                    respuesta.mensajes.push(format!("Error consultando autorización (intento {}): {}", intento + 1, e));
                }
            }
        }
    }

    // 5. Si fue RECIBIDA o AUTORIZADA, descontar stock en SQLite y guardar comprobante
    if respuesta.estado == "RECIBIDA" || respuesta.estado == "AUTORIZADO" || respuesta.estado == "EN PROCESAMIENTO" {
        for d in &payload.factura.detalles {
            let _ = descontar_stock(&state.db, &d.codigo_principal, d.cantidad).await;
        }

        // Calcular totales para guardar en la BD
        let mut total_sin_imp = 0.0;
        let mut total_iva = 0.0;
        for d in &payload.factura.detalles {
            total_sin_imp += d.subtotal();
            total_iva += d.valor_iva();
        }
        let total_factura = total_sin_imp + total_iva + payload.factura.propina;

        let _ = guardar_factura_db(
            &state.db,
            &clave,
            &payload.factura.secuencial,
            &payload.factura.fecha_emision,
            &payload.factura.cliente.identificacion,
            &payload.factura.cliente.razon_social,
            total_sin_imp,
            total_iva,
            total_factura,
            &respuesta.estado,
            respuesta.xml_firmado.as_deref(),
            respuesta.xml_autorizado.as_deref(),
        ).await;
    }

    Ok(Json(respuesta))
}
