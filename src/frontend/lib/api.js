const API_BASE = (typeof window !== 'undefined' && window.location.port === '8080')
  ? ''
  : 'http://127.0.0.1:8080';

export async function fetchConfig() {
  const res = await fetch(`${API_BASE}/api/config`);
  if (!res.ok) throw new Error('Error al cargar configuración');
  return res.json();
}

export async function saveConfig(cfg) {
  const res = await fetch(`${API_BASE}/api/config`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(cfg),
  });
  if (!res.ok) throw new Error('Error al guardar configuración');
  return res.json();
}

export async function uploadP12(file) {
  const formData = new FormData();
  formData.append('file', file);
  const res = await fetch(`${API_BASE}/api/upload-p12`, {
    method: 'POST',
    body: formData,
  });
  if (!res.ok) throw new Error('Error al subir firma .p12');
  return res.json();
}

export async function deleteP12() {
  const res = await fetch(`${API_BASE}/api/delete-p12`, { method: 'DELETE' });
  if (!res.ok) throw new Error('Error al eliminar firma .p12');
  return res.json();
}

export async function fetchProductos() {
  const res = await fetch(`${API_BASE}/api/productos`);
  if (!res.ok) throw new Error('Error al cargar productos');
  return res.json();
}

export async function saveProducto(producto, id = null) {
  const url = id ? `${API_BASE}/api/productos/${id}` : `${API_BASE}/api/productos`;
  const method = id ? 'PUT' : 'POST';
  const res = await fetch(url, {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(producto),
  });
  if (!res.ok) {
    const errText = await res.text();
    throw new Error(errText);
  }
  return res.json();
}

export async function deleteProducto(id) {
  const res = await fetch(`${API_BASE}/api/productos/${id}`, { method: 'DELETE' });
  if (!res.ok) {
    const errText = await res.text();
    throw new Error(errText);
  }
  return res.json();
}

export async function emitirFactura(payload) {
  const res = await fetch(`${API_BASE}/api/facturar`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  if (!res.ok) {
    const errText = await res.text();
    throw new Error(errText);
  }
  return res.json();
}

export async function fetchFacturas() {
  const res = await fetch(`${API_BASE}/api/facturas`);
  if (!res.ok) throw new Error('Error al cargar historial de facturas');
  return res.json();
}

export async function fetchCliente(identificacion) {
  if (!identificacion || !identificacion.trim()) return null;
  const res = await fetch(`${API_BASE}/api/clientes/${encodeURIComponent(identificacion.trim())}`);
  if (!res.ok) return null;
  return res.json();
}

export async function fetchClientes() {
  const res = await fetch(`${API_BASE}/api/clientes`);
  if (!res.ok) throw new Error('Error al cargar clientes');
  return res.json();
}

export async function saveCliente(cliente) {
  const res = await fetch(`${API_BASE}/api/clientes`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(cliente),
  });
  if (!res.ok) throw new Error('Error al guardar cliente');
  return res.json();
}
