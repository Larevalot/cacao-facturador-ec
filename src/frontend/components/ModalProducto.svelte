<script>
  import { toastStore } from '../lib/toast.svelte.js';
  import Icon from './Icon.svelte';

  let { isOpen = false, producto = null, onClose = () => {}, onSave = () => {} } = $props();

  let id = $state('');
  let tipo = $state('PRODUCTO'); // 'PRODUCTO' | 'SERVICIO'
  let codigo = $state('');
  let descripcion = $state('');
  let precioUnitario = $state(0);
  let precioFinal = $state(0);
  let stock = $state(0);
  let codigoIva = $state('4'); // '4' = 15%, '0' = 0%

  function getFactorIva(code = codigoIva) {
    return code === '4' ? 1.15 : 1.0;
  }

  $effect(() => {
    if (producto) {
      id = producto.id || '';
      tipo = producto.tipo || 'PRODUCTO';
      codigo = producto.codigo || '';
      descripcion = producto.descripcion || '';
      const pu = Number((Number(producto.precio_unitario) || 0).toFixed(2));
      precioUnitario = pu;
      codigoIva = producto.codigo_iva || '4';
      stock = producto.stock || 0;
      precioFinal = Number((pu * getFactorIva(codigoIva)).toFixed(2));
    } else {
      id = '';
      tipo = 'PRODUCTO';
      codigo = '';
      descripcion = '';
      precioUnitario = 0;
      precioFinal = 0;
      stock = 0;
      codigoIva = '4';
    }
  });

  function handleUnitarioInput(e) {
    const val = parseFloat(e.target.value) || 0;
    precioUnitario = Number(val.toFixed(2));
    const factor = getFactorIva();
    precioFinal = Number((val * factor).toFixed(2));
  }

  function handleFinalInput(e) {
    const val = parseFloat(e.target.value) || 0;
    precioFinal = Number(val.toFixed(2));
    const factor = getFactorIva();
    precioUnitario = Number((val / factor).toFixed(2));
  }

  function handleIvaChange(e) {
    const newIva = e.target.value;
    codigoIva = newIva;
    const factor = getFactorIva(newIva);
    precioFinal = Number((precioUnitario * factor).toFixed(2));
  }

  async function handleSubmit(e) {
    e.preventDefault();
    if (!codigo.trim() || !descripcion.trim()) {
      toastStore.warning('Por favor ingresa el código y la descripción.', 'Campos Requeridos');
      return;
    }

    const payload = {
      tipo,
      codigo: codigo.trim(),
      codigo_auxiliar: null,
      descripcion: descripcion.trim(),
      precio_unitario: Number(Number(precioUnitario).toFixed(2)) || 0,
      stock: tipo === 'SERVICIO' ? 0 : (Number(stock) || 0),
      codigo_iva: codigoIva,
      tarifa_iva: codigoIva === '4' ? 15.0 : 0.0,
    };

    onSave(payload, id || null);
  }
</script>

{#if isOpen}
  <div class="modal-overlay active">
    <div class="modal-box">
      <h2 style="font-size: 1.1rem; margin-bottom: 1rem; color: var(--text-main);">
        {id ? (tipo === 'SERVICIO' ? 'Editar Servicio' : 'Editar Producto') : 'Nuevo Registro'}
      </h2>
      <form onsubmit={(e) => { e.preventDefault(); e.stopPropagation(); handleSubmit(e); }}>
        <!-- SELECTOR DE TIPO (PRODUCTO O SERVICIO) -->
        <div class="form-group">
          <span style="font-size: 0.82rem; font-weight: 600; color: var(--text-muted); display: block; margin-bottom: 0.35rem;">Tipo de Registro</span>
          <div style="display: flex; gap: 0.8rem; margin-top: 0.2rem;">
            <button
              type="button"
              class={tipo === 'PRODUCTO' ? 'btn btn-sm' : 'btn btn-secondary btn-sm'}
              style="flex: 1; padding: 0.6rem; font-size: 0.85rem;"
              onclick={() => tipo = 'PRODUCTO'}
            >
              <Icon name="package" size="1em" /> Producto (con Stock)
            </button>
            <button
              type="button"
              class={tipo === 'SERVICIO' ? 'btn btn-sm' : 'btn btn-secondary btn-sm'}
              style="flex: 1; padding: 0.6rem; font-size: 0.85rem;"
              onclick={() => { tipo = 'SERVICIO'; stock = 0; }}
            >
              <Icon name="wrench" size="1em" /> Servicio (sin Stock)
            </button>
          </div>
        </div>

        <div class="row">
          <div class="form-group">
            <label for="p-code">Código Principal</label>
            <input id="p-code" type="text" bind:value={codigo} required placeholder={tipo === 'SERVICIO' ? 'Ej: SRV-001' : 'Ej: PRD-001'} />
          </div>
          {#if tipo === 'PRODUCTO'}
            <div class="form-group">
              <label for="p-stock">Stock Inicial</label>
              <input id="p-stock" type="number" step="1" min="0" bind:value={stock} required />
            </div>
          {/if}
        </div>

        <div class="form-group">
          <label for="p-desc">Descripción</label>
          <input id="p-desc" type="text" bind:value={descripcion} required placeholder={tipo === 'SERVICIO' ? 'Nombre o detalle del servicio' : 'Nombre del producto'} />
        </div>

        <!-- CÁLCULO BI-DIRECCIONAL DE PRECIOS EXACTOS (2 DECIMALES) -->
        <div class="row">
          <div class="form-group">
            <label for="p-price-unit">Precio Unitario ($ Sin IVA)</label>
            <input
              id="p-price-unit"
              type="number"
              step="0.01"
              min="0"
              value={precioUnitario}
              oninput={handleUnitarioInput}
              required
            />
          </div>
          <div class="form-group">
            <label for="p-price-final" style="color: var(--accent-color);">Precio Final ($ Con IVA)</label>
            <input
              id="p-price-final"
              type="number"
              step="0.01"
              min="0"
              value={precioFinal}
              oninput={handleFinalInput}
              style="font-weight: 700; border-color: var(--accent-color);"
              required
            />
          </div>
        </div>

        <div class="form-group">
          <label for="p-iva">Porcentaje IVA</label>
          <select id="p-iva" value={codigoIva} onchange={handleIvaChange}>
            <option value="4">15% (Tarifa Actual IVA)</option>
            <option value="0">0% (Exento / Sin IVA)</option>
          </select>
        </div>

        <div style="display: flex; gap: 0.8rem; margin-top: 1.5rem;">
          <button type="submit" class="btn">
            <Icon name="save" size="1.1em" /> {tipo === 'SERVICIO' ? 'Guardar Servicio' : 'Guardar Producto'}
          </button>
          <button type="button" class="btn btn-secondary" onclick={onClose}>Cancelar</button>
        </div>
      </form>
    </div>
  </div>
{/if}
