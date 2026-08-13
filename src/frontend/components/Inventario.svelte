<script>
  import { fetchProductos, saveProducto, deleteProducto } from '../lib/api.js';
  import { toastStore } from '../lib/toast.svelte.js';
  import ModalProducto from './ModalProducto.svelte';
  import Icon from './Icon.svelte';

  let productos = $state([]);
  let busqueda = $state('');
  let tabFiltro = $state('TODOS'); // 'TODOS' | 'PRODUCTO' | 'SERVICIO'
  let isModalOpen = $state(false);
  let productoEditar = $state(null);
  let productoEliminar = $state(null);

  let productosFiltrados = $derived(
    productos.filter(p => {
      const matchTipo = tabFiltro === 'TODOS' || (p.tipo || 'PRODUCTO') === tabFiltro;
      const matchText = p.codigo.toLowerCase().includes(busqueda.toLowerCase()) ||
                        p.descripcion.toLowerCase().includes(busqueda.toLowerCase());
      return matchTipo && matchText;
    })
  );

  async function cargar() {
    try {
      productos = await fetchProductos();
    } catch(e) {
      console.error('Error cargando inventario:', e);
    }
  }

  cargar();

  function abrirNuevo(tipoInicial = 'PRODUCTO') {
    productoEditar = tipoInicial ? { tipo: tipoInicial } : null;
    isModalOpen = true;
  }

  function abrirEditar(p) {
    productoEditar = p;
    isModalOpen = true;
  }

  async function handleSave(payload, id) {
    try {
      await saveProducto(payload, id);
      isModalOpen = false;
      await cargar();
      toastStore.success(
        id ? 'Registro actualizado.' : (payload.tipo === 'SERVICIO' ? 'Servicio agregado exitosamente.' : 'Producto agregado al inventario.'),
        'Gestión'
      );
    } catch(err) {
      toastStore.error('Error al guardar: ' + err.message, 'Gestión');
    }
  }

  function pedirConfirmarBorrar(p, e) {
    if (e) {
      e.preventDefault();
      e.stopPropagation();
    }
    productoEliminar = p;
  }

  async function ejecutarBorrar() {
    if (!productoEliminar) return;
    const id = productoEliminar.id;
    productoEliminar = null;
    try {
      await deleteProducto(id);
      await cargar();
      toastStore.success('Registro eliminado.', 'Gestión');
    } catch(err) {
      toastStore.error('Error al eliminar: ' + err.message, 'Gestión');
    }
  }

  function getPrecioFinal(p) {
    const factor = p.codigo_iva === '4' ? 1.15 : 1.0;
    return (p.precio_unitario * factor).toFixed(2);
  }
</script>

<div class="card">
  <h2>
    <span style="display: inline-flex; align-items: center; gap: 0.5rem;">
      <Icon name="package" size="1.2em" /> Gestor de Productos y Servicios
    </span>
    <div style="display: flex; gap: 0.5rem;">
      <button type="button" class="btn btn-sm" onclick={() => abrirNuevo('PRODUCTO')}>
        <Icon name="plus" size="1em" /> Nuevo Producto
      </button>
      <button type="button" class="btn btn-secondary btn-sm" onclick={() => abrirNuevo('SERVICIO')}>
        <Icon name="plus" size="1em" /> Nuevo Servicio
      </button>
    </div>
  </h2>

  <!-- BARRA DE PESTAÑAS (TODOS / PRODUCTOS / SERVICIOS) Y BÚSQUEDA -->
  <div style="display: flex; gap: 1rem; margin-bottom: 1.2rem; align-items: center; flex-wrap: wrap;">
    <div style="display: flex; background: var(--bg-secondary); padding: 0.25rem; border-radius: var(--radius-input); border: 1px solid var(--border-color);">
      <button
        type="button"
        class={tabFiltro === 'TODOS' ? 'btn btn-sm' : 'btn btn-secondary btn-sm'}
        style="border: none; padding: 0.35rem 0.8rem; font-size: 0.8rem;"
        onclick={() => tabFiltro = 'TODOS'}
      >
        Todos ({productos.length})
      </button>
      <button
        type="button"
        class={tabFiltro === 'PRODUCTO' ? 'btn btn-sm' : 'btn btn-secondary btn-sm'}
        style="border: none; padding: 0.35rem 0.8rem; font-size: 0.8rem;"
        onclick={() => tabFiltro = 'PRODUCTO'}
      >
        📦 Productos ({productos.filter(p => (p.tipo || 'PRODUCTO') === 'PRODUCTO').length})
      </button>
      <button
        type="button"
        class={tabFiltro === 'SERVICIO' ? 'btn btn-sm' : 'btn btn-secondary btn-sm'}
        style="border: none; padding: 0.35rem 0.8rem; font-size: 0.8rem;"
        onclick={() => tabFiltro = 'SERVICIO'}
      >
        🛠️ Servicios ({productos.filter(p => p.tipo === 'SERVICIO').length})
      </button>
    </div>

    <div style="flex: 1; position: relative; min-width: 200px;">
      <div style="position: absolute; left: 0.9rem; top: 50%; transform: translateY(-50%); color: var(--text-muted); display: flex; align-items: center; pointer-events: none;">
        <Icon name="search" size="1.1em" />
      </div>
      <input
        type="text"
        bind:value={busqueda}
        placeholder="Buscar por código o descripción..."
        style="padding-left: 2.5rem;"
      />
    </div>
  </div>

  <div class="table-container">
    <table>
      <thead>
        <tr>
          <th style="width: 80px;">Tipo</th>
          <th>Código</th>
          <th>Descripción</th>
          <th>P. Unit ($)</th>
          <th style="color: var(--accent-color);">P. Final ($)</th>
          <th>Stock</th>
          <th>IVA</th>
          <th style="width: 110px; text-align: center;">Acciones</th>
        </tr>
      </thead>
      <tbody>
        {#if productosFiltrados.length === 0}
          <tr>
            <td colspan="8" style="text-align: center; color: var(--text-muted);">No hay ítems registrados en esta categoría</td>
          </tr>
        {:else}
          {#each productosFiltrados as p (p.id)}
            <tr>
              <td>
                {#if p.tipo === 'SERVICIO'}
                  <span class="status-badge" style="background: rgba(59, 130, 246, 0.15); color: #60a5fa; border: 1px solid #3b82f6;">Servicio</span>
                {:else}
                  <span class="status-badge status-RECIBIDA">Producto</span>
                {/if}
              </td>
              <td style="white-space: nowrap;"><strong>{p.codigo}</strong></td>
              <td>{p.descripcion}</td>
              <td style="white-space: nowrap;">${p.precio_unitario.toFixed(2)}</td>
              <td style="white-space: nowrap; font-weight: 700; color: var(--accent-color);">${getPrecioFinal(p)}</td>
              <td style="white-space: nowrap;">
                {#if p.tipo === 'SERVICIO'}
                  <span style="color: var(--text-muted); font-size: 0.8rem;">Sin Stock</span>
                {:else}
                  <span class="status-badge {p.stock > 10 ? 'status-AUTORIZADO' : 'status-DEVUELTA'}">{p.stock} unids</span>
                {/if}
              </td>
              <td style="white-space: nowrap;">{p.codigo_iva === '4' ? '15%' : '0%'}</td>
              <td style="vertical-align: middle; text-align: center;">
                <div style="display: flex; gap: 0.4rem; justify-content: center; align-items: center;">
                  <button type="button" class="btn btn-secondary btn-sm" onclick={(e) => abrirEditar(p)} title="Editar" aria-label="Editar">
                    <Icon name="pencil" size="1em" />
                  </button>
                  <button type="button" class="btn btn-danger btn-sm" onclick={(e) => pedirConfirmarBorrar(p, e)} title="Eliminar" aria-label="Eliminar">
                    <Icon name="trash" size="1em" />
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</div>

<!-- MODAL CREAR / EDITAR -->
<ModalProducto
  isOpen={isModalOpen}
  producto={productoEditar}
  onClose={() => isModalOpen = false}
  onSave={handleSave}
/>

<!-- MODAL CONFIRMAR ELIMINACIÓN -->
{#if productoEliminar}
  <div class="modal-overlay active">
    <div class="modal-box" style="max-width: 400px; text-align: center;">
      <div style="color: var(--danger-color); margin-bottom: 0.5rem; display: flex; justify-content: center;">
        <Icon name="trash" size="3rem" />
      </div>
      <h2 style="justify-content: center; border: none; padding: 0; margin-bottom: 0.5rem;">¿Eliminar Registro?</h2>
      <p style="color: var(--text-muted); font-size: 0.9rem; margin-bottom: 1.5rem;">
        ¿Seguro deseas eliminar <strong>"{productoEliminar.codigo} - {productoEliminar.descripcion}"</strong>?
      </p>
      <div style="display: flex; gap: 0.75rem;">
        <button type="button" class="btn btn-danger" onclick={ejecutarBorrar}>Sí, Eliminar</button>
        <button type="button" class="btn btn-secondary" onclick={() => productoEliminar = null}>Cancelar</button>
      </div>
    </div>
  </div>
{/if}
