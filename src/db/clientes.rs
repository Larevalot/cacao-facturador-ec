//! Módulo de Gestión de Clientes en SQLite (SQLx) para Cacao Facturador.

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Cliente {
    pub id: Option<i64>,
    pub tipo_identificacion: String,
    pub identificacion: String,
    pub razon_social: String,
    pub direccion: Option<String>,
    pub email: Option<String>,
    pub telefono: Option<String>,
    pub created_at: Option<String>,
}

pub async fn guardar_o_actualizar_cliente(
    pool: &SqlitePool,
    tipo_identificacion: &str,
    identificacion: &str,
    razon_social: &str,
    direccion: Option<&str>,
    email: Option<&str>,
    telefono: Option<&str>,
) -> Result<Cliente, String> {
    sqlx::query(
        r#"
        INSERT INTO clientes (tipo_identificacion, identificacion, razon_social, direccion, email, telefono)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(identificacion) DO UPDATE SET
            tipo_identificacion = excluded.tipo_identificacion,
            razon_social = excluded.razon_social,
            direccion = excluded.direccion,
            email = excluded.email,
            telefono = excluded.telefono
        "#,
    )
    .bind(tipo_identificacion)
    .bind(identificacion)
    .bind(razon_social)
    .bind(direccion)
    .bind(email)
    .bind(telefono)
    .execute(pool)
    .await
    .map_err(|e| format!("Error guardando cliente: {}", e))?;

    buscar_cliente_por_identificacion(pool, identificacion)
        .await?
        .ok_or_else(|| "Error al recuperar cliente guardado".to_string())
}

pub async fn buscar_cliente_por_identificacion(
    pool: &SqlitePool,
    identificacion: &str,
) -> Result<Option<Cliente>, String> {
    sqlx::query_as::<_, Cliente>(
        "SELECT id, tipo_identificacion, identificacion, razon_social, direccion, email, telefono, created_at FROM clientes WHERE identificacion = ?1"
    )
    .bind(identificacion)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Error buscando cliente: {}", e))
}

pub async fn listar_clientes(pool: &SqlitePool) -> Result<Vec<Cliente>, String> {
    sqlx::query_as::<_, Cliente>(
        "SELECT id, tipo_identificacion, identificacion, razon_social, direccion, email, telefono, created_at FROM clientes ORDER BY razon_social ASC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Error listando clientes: {}", e))
}
