//! Operaciones CRUD de Inventario de Productos y Servicios en SQLite con SQLx.

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

fn default_tipo() -> String {
    "PRODUCTO".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Producto {
    pub id: i64,
    pub codigo: String,
    pub codigo_auxiliar: Option<String>,
    pub descripcion: String,
    pub precio_unitario: f64,
    pub stock: f64,
    pub codigo_iva: String, // '4' (15%), '0' (0%)
    pub tarifa_iva: f64,
    #[serde(default = "default_tipo")]
    pub tipo: String, // 'PRODUCTO' o 'SERVICIO'
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevoProductoRequest {
    pub codigo: String,
    pub codigo_auxiliar: Option<String>,
    pub descripcion: String,
    pub precio_unitario: f64,
    pub stock: f64,
    pub codigo_iva: String,
    pub tarifa_iva: f64,
    pub tipo: Option<String>,
}

pub async fn listar_productos(pool: &SqlitePool) -> Result<Vec<Producto>, sqlx::Error> {
    sqlx::query_as::<_, Producto>(
        "SELECT id, codigo, codigo_auxiliar, descripcion, precio_unitario, stock, codigo_iva, tarifa_iva, COALESCE(tipo, 'PRODUCTO') AS tipo, created_at FROM productos ORDER BY id DESC"
    )
    .fetch_all(pool)
    .await
}

#[allow(dead_code)]
pub async fn buscar_producto_por_codigo(
    pool: &SqlitePool,
    codigo: &str,
) -> Result<Option<Producto>, sqlx::Error> {
    sqlx::query_as::<_, Producto>(
        "SELECT id, codigo, codigo_auxiliar, descripcion, precio_unitario, stock, codigo_iva, tarifa_iva, COALESCE(tipo, 'PRODUCTO') AS tipo, created_at FROM productos WHERE codigo = ?"
    )
    .bind(codigo)
    .fetch_optional(pool)
    .await
}

pub async fn crear_producto(
    pool: &SqlitePool,
    req: &NuevoProductoRequest,
) -> Result<Producto, sqlx::Error> {
    let tipo = req.tipo.as_deref().unwrap_or("PRODUCTO");
    let id = sqlx::query(
        r#"
        INSERT INTO productos (codigo, codigo_auxiliar, descripcion, precio_unitario, stock, codigo_iva, tarifa_iva, tipo)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&req.codigo)
    .bind(&req.codigo_auxiliar)
    .bind(&req.descripcion)
    .bind(req.precio_unitario)
    .bind(req.stock)
    .bind(&req.codigo_iva)
    .bind(req.tarifa_iva)
    .bind(tipo)
    .execute(pool)
    .await?
    .last_insert_rowid();

    sqlx::query_as::<_, Producto>(
        "SELECT id, codigo, codigo_auxiliar, descripcion, precio_unitario, stock, codigo_iva, tarifa_iva, COALESCE(tipo, 'PRODUCTO') AS tipo, created_at FROM productos WHERE id = ?"
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn actualizar_producto(
    pool: &SqlitePool,
    id: i64,
    req: &NuevoProductoRequest,
) -> Result<Producto, sqlx::Error> {
    let tipo = req.tipo.as_deref().unwrap_or("PRODUCTO");
    sqlx::query(
        r#"
        UPDATE productos 
        SET codigo = ?, codigo_auxiliar = ?, descripcion = ?, precio_unitario = ?, stock = ?, codigo_iva = ?, tarifa_iva = ?, tipo = ?
        WHERE id = ?
        "#,
    )
    .bind(&req.codigo)
    .bind(&req.codigo_auxiliar)
    .bind(&req.descripcion)
    .bind(req.precio_unitario)
    .bind(req.stock)
    .bind(&req.codigo_iva)
    .bind(req.tarifa_iva)
    .bind(tipo)
    .bind(id)
    .execute(pool)
    .await?;

    sqlx::query_as::<_, Producto>(
        "SELECT id, codigo, codigo_auxiliar, descripcion, precio_unitario, stock, codigo_iva, tarifa_iva, COALESCE(tipo, 'PRODUCTO') AS tipo, created_at FROM productos WHERE id = ?"
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn eliminar_producto(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM productos WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Disminuye el stock solo si es de tipo 'PRODUCTO'.
pub async fn descontar_stock(
    pool: &SqlitePool,
    codigo_producto: &str,
    cantidad: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE productos SET stock = MAX(0, stock - ?) WHERE codigo = ? AND (tipo IS NULL OR tipo = 'PRODUCTO')"
    )
    .bind(cantidad)
    .bind(codigo_producto)
    .execute(pool)
    .await?;
    Ok(())
}
