//! Persistencia e Historial de Facturas emitidas al SRI en SQLite con SQLx.

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FacturaGuardada {
    pub id: i64,
    pub clave_acceso: String,
    pub secuencial: String,
    pub fecha_emision: String,
    pub cliente_identificacion: String,
    pub cliente_razon: String,
    pub total_sin_impuestos: f64,
    pub total_iva: f64,
    pub importe_total: f64,
    pub estado: String,
    pub xml_firmado: Option<String>,
    pub xml_autorizado: Option<String>,
    pub created_at: String,
}

pub async fn guardar_factura_db(
    pool: &SqlitePool,
    clave_acceso: &str,
    secuencial: &str,
    fecha_emision: &str,
    cliente_identificacion: &str,
    cliente_razon: &str,
    total_sin_impuestos: f64,
    total_iva: f64,
    importe_total: f64,
    estado: &str,
    xml_firmado: Option<&str>,
    xml_autorizado: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let id = sqlx::query(
        r#"
        INSERT INTO facturas (
            clave_acceso, secuencial, fecha_emision, cliente_identificacion, cliente_razon,
            total_sin_impuestos, total_iva, importe_total, estado, xml_firmado, xml_autorizado
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(clave_acceso) DO UPDATE SET
            estado = excluded.estado,
            xml_autorizado = excluded.xml_autorizado
        "#,
    )
    .bind(clave_acceso)
    .bind(secuencial)
    .bind(fecha_emision)
    .bind(cliente_identificacion)
    .bind(cliente_razon)
    .bind(total_sin_impuestos)
    .bind(total_iva)
    .bind(importe_total)
    .bind(estado)
    .bind(xml_firmado)
    .bind(xml_autorizado)
    .execute(pool)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn listar_historial_facturas(
    pool: &SqlitePool,
) -> Result<Vec<FacturaGuardada>, sqlx::Error> {
    sqlx::query_as::<_, FacturaGuardada>("SELECT * FROM facturas ORDER BY id DESC LIMIT 50")
        .fetch_all(pool)
        .await
}
