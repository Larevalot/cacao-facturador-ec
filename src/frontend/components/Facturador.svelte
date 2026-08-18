<script>
  import { fetchProductos, emitirFactura, fetchCliente, fetchFacturas } from '../lib/api.js';
  import { toastStore } from '../lib/toast.svelte.js';
  import Icon from './Icon.svelte';

  const FORMAS_PAGO_SRI = [
    { codigo: '01', descripcion: 'SIN UTILIZACION DEL SISTEMA FINANCIERO' },
    { codigo: '15', descripcion: 'COMPENSACIÓN DE DEUDAS' },
    { codigo: '16', descripcion: 'TARJETA DE DÉBITO' },
    { codigo: '17', descripcion: 'DINERO ELECTRÓNICO' },
    { codigo: '18', descripcion: 'TARJETA PREPAGO' },
    { codigo: '19', descripcion: 'TARJETA DE CRÉDITO' },
    { codigo: '20', descripcion: 'OTROS CON UTILIZACION DEL SISTEMA FINANCIERO' },
    { codigo: '21', descripcion: 'ENDOSO DE TÍTULOS' }
  ];

  const UNIDADES_TIEMPO = [
    { codigo: 'dias', label: 'Días' },
    { codigo: 'meses', label: 'Meses' },
    { codigo: 'anios', label: 'Años' }
  ];

  let secuencial = $state('000000001');
  let fechaEmision = $state('');
  let tipoId = $state('05');
  let identificacion = $state('');
  let razonSocial = $state('');
  let email = $state('');
  let telefono = $state('');
  let passwordP12 = $state('');
  let clienteEncontrado = $state(false);

  let productosInventario = $state([]);
  let items = $state([]);
  let isEmitiendo = $state(false);
  let respuestaSRI = $state(null);

  // Formas de pago dinámicas
  let formasPago = $state([
    {
      forma_pago: '01',
      total: 0.00,
      plazo: null,
      unidad_tiempo: 'dias'
    }
  ]);

  // Fecha actual
  const today = new Date();
  fechaEmision = `${String(today.getDate()).padStart(2, '0')}/${String(today.getMonth() + 1).padStart(2, '0')}/${today.getFullYear()}`;

  async function cargar() {
    try {
      productosInventario = await fetchProductos();
      const facturas = await fetchFacturas();
      if (facturas && facturas.length > 0) {
        let maxSec = 0;
        for (const f of facturas) {
          const s = parseInt(f.secuencial, 10);
          if (!isNaN(s) && s > maxSec) {
            maxSec = s;
          }
        }
        if (maxSec > 0) {
          secuencial = String(maxSec + 1).padStart(9, '0');
        }
      }

      if (items.length === 0 && productosInventario.length > 0) {
        items = [{
          codigo: productosInventario[0].codigo,
          descripcion: productosInventario[0].descripcion,
          cantidad: 1,
          precio: productosInventario[0].precio_unitario,
          descuento: 0,
          iva: productosInventario[0].codigo_iva,
          tarifa: productosInventario[0].tarifa_iva
        }];
      }
      if (identificacion && identificacion.length >= 10 && tipoId !== '07') {
        await buscarYAutocompletarCliente(identificacion);
      }
    } catch(e) { console.error('Error cargando inventario o facturas:', e); }
  }

  function setConsumidorFinal() {
    tipoId = '07';
    identificacion = '9999999999999';
    razonSocial = 'CONSUMIDOR FINAL';
    email = '';
    telefono = '';
    clienteEncontrado = false;
  }

  function handleTipoIdChange(e) {
    const val = e.target.value;
    tipoId = val;
    if (val === '07') {
      setConsumidorFinal();
    } else if (identificacion === '9999999999999') {
      identificacion = '';
      razonSocial = '';
    }
  }

  async function buscarYAutocompletarCliente(val) {
    if (tipoId === '07' || !val || val.trim().length < 10) {
      clienteEncontrado = false;
      return;
    }
    try {
      const cliente = await fetchCliente(val.trim());
      if (cliente) {
        if (cliente.tipo_identificacion) tipoId = cliente.tipo_identificacion;
        if (cliente.razon_social) razonSocial = cliente.razon_social;
        if (cliente.email) email = cliente.email;
        if (cliente.telefono) telefono = cliente.telefono;
        if (!clienteEncontrado) {
          clienteEncontrado = true;
          toastStore.info(`Cliente encontrado: ${cliente.razon_social}`, 'Autocompletado');
        }
      } else {
        clienteEncontrado = false;
      }
    } catch (err) {
      console.error('Error buscando cliente:', err);
    }
  }

  cargar();

  // Totales Reactivos con $derived
  let subtotal15 = $derived(
    items.reduce((acc, i) => acc + (i.iva === '4' ? (i.cantidad * i.precio - (i.descuento || 0)) : 0), 0)
  );

  let subtotal0 = $derived(
    items.reduce((acc, i) => acc + (i.iva === '0' ? (i.cantidad * i.precio - (i.descuento || 0)) : 0), 0)
  );

  let montoIva = $derived(subtotal15 * 0.15);
  let totalFactura = $derived(subtotal15 + subtotal0 + montoIva);

  // Cálculos reactivos de formas de pago
  let totalPagos = $derived(
    formasPago.reduce((acc, p) => acc + (Number(p.total) || 0), 0)
  );

  let diferenciaPagos = $derived(
    Number((totalFactura - totalPagos).toFixed(2))
  );

  // Auto-ajustar la forma de pago si solo hay una
  $effect(() => {
    if (formasPago.length === 1) {
      formasPago[0].total = Number(totalFactura.toFixed(2));
    }
  });

  function addItem() {
    items = [...items, { codigo: '', descripcion: '', cantidad: 1, precio: 0.00, descuento: 0, iva: '4', tarifa: 15.0 }];
  }

  function removeItem(index) {
    items = items.filter((_, idx) => idx !== index);
  }

  function seleccionarProducto(index, codigo) {
    const prod = productosInventario.find(p => p.codigo === codigo);
    if (prod) {
      items[index].codigo = prod.codigo;
      items[index].descripcion = prod.descripcion;
      items[index].precio = prod.precio_unitario;
      items[index].iva = prod.codigo_iva;
      items[index].tarifa = prod.tarifa_iva;
    }
  }

  function addFormaPago() {
    const restante = Math.max(0, Number((totalFactura - totalPagos).toFixed(2)));
    formasPago = [
      ...formasPago,
      {
        forma_pago: '01',
        total: restante,
        plazo: null,
        unidad_tiempo: 'dias'
      }
    ];
  }

  function removeFormaPago(index) {
    if (formasPago.length > 1) {
      formasPago = formasPago.filter((_, idx) => idx !== index);
    }
  }

  function ajustarAlTotal() {
    if (formasPago.length === 1) {
      formasPago[0].total = Number(totalFactura.toFixed(2));
    } else if (formasPago.length > 1) {
      const otros = formasPago.slice(0, -1).reduce((acc, p) => acc + (Number(p.total) || 0), 0);
      const restante = Math.max(0, Number((totalFactura - otros).toFixed(2)));
      formasPago[formasPago.length - 1].total = restante;
    }
  }

  async function handleEmitir(e) {
    e.preventDefault();
    if (items.length === 0) {
      toastStore.warning('Debes agregar al menos un ítem a la factura.', 'Factura Vacía');
      return;
    }

    if (formasPago.length === 0) {
      toastStore.warning('Debes agregar al menos una forma de pago.', 'Forma de Pago');
      return;
    }

    if (Math.abs(diferenciaPagos) >= 0.01) {
      toastStore.warning(
        `La suma de las formas de pago ($${totalPagos.toFixed(2)}) debe ser igual al total de la factura ($${totalFactura.toFixed(2)}).`,
        'Descuadre en Pagos'
      );
      return;
    }

    let finalTipoId = tipoId;
    let finalIdentificacion = identificacion.trim();
    let finalRazon = razonSocial.trim();

    if (finalTipoId === '07' || !finalIdentificacion || finalIdentificacion === '9999999999999') {
      finalTipoId = '07';
      finalIdentificacion = '9999999999999';
      finalRazon = 'CONSUMIDOR FINAL';
    }

    // Advertencia legal SRI para Consumidor Final > $50
    if (finalTipoId === '07' && totalFactura > 50.0) {
      toastStore.warning('Atención: El SRI requiere identificar al comprador (RUC/Cédula) para facturas mayores a $50.00 USD.', 'Regla SRI Consumidor Final');
    }

    isEmitiendo = true;
    respuestaSRI = null;

    const payload = {
      factura: {
        secuencial,
        fecha_emision: fechaEmision,
        cliente: {
          tipo_identificacion: finalTipoId,
          identificacion: finalIdentificacion,
          razon_social: finalRazon,
          direccion: 'Quito, Ecuador',
          email,
          telefono
        },
        detalles: items.map(i => ({
          codigo_principal: i.codigo || 'PRD-001',
          descripcion: i.descripcion,
          cantidad: Number(i.cantidad),
          precio_unitario: Number(Number(i.precio).toFixed(2)),
          descuento: 0.0,
          codigo_porcentaje_iva: i.iva,
          tarifa_iva: i.iva === '4' ? 15.0 : 0.0
        })),
        formas_pago: formasPago.map(p => ({
          forma_pago: p.forma_pago,
          total: Number(Number(p.total).toFixed(2)),
          plazo: p.plazo && Number(p.plazo) > 0 ? Math.round(Number(p.plazo)) : null,
          unidad_tiempo: p.plazo && Number(p.plazo) > 0 ? p.unidad_tiempo : null
        })),
        propina: 0.0
      },
      password_p12: passwordP12 && passwordP12.trim().length > 0 ? passwordP12.trim() : null
    };

    try {
      respuestaSRI = await emitirFactura(payload);
      await cargar();
      if (['AUTORIZADO', 'RECIBIDA', 'EN PROCESAMIENTO'].includes(respuestaSRI.estado)) {
        toastStore.success(`Comprobante ${respuestaSRI.estado} por el SRI.`, 'Respuesta SRI');
        const num = parseInt(secuencial, 10);
        if (!isNaN(num)) {
          secuencial = String(num + 1).padStart(9, '0');
        }
      } else {
        toastStore.error(`Comprobante ${respuestaSRI.estado}.`, 'Respuesta SRI');
      }
    } catch(err) {
      toastStore.error('Error emitiendo factura: ' + err.message, 'Fallo de Emisión');
    } finally {
      isEmitiendo = false;
    }
  }
</script>

<div class="card">
  <h2>
    <span style="display: inline-flex; align-items: center; gap: 0.5rem;">
      <Icon name="receipt" size="1.2em" /> Emitir Factura Electrónica
    </span>
  </h2>
  <form onsubmit={handleEmitir}>
    <div class="row">
      <div class="form-group">
        <label for="f-sec">Secuencial (9 dígitos)</label>
        <input id="f-sec" type="text" maxlength="9" bind:value={secuencial} required />
      </div>
      <div class="form-group">
        <label for="f-date">Fecha Emisión (DD/MM/AAAA)</label>
        <input id="f-date" type="text" bind:value={fechaEmision} required />
      </div>
    </div>

    <!-- DATOS DEL CLIENTE -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin: 1rem 0 0.5rem 0;">
      <h3 style="font-size: 0.85rem; color: var(--text-muted); text-transform: uppercase; margin: 0;">Datos del Cliente</h3>
      <button
        type="button"
        class="btn btn-secondary btn-sm"
        onclick={setConsumidorFinal}
        title="Llenar datos automáticos para Consumidor Final (9999999999999)"
      >
        <Icon name="user" size="1em" /> Consumidor Final
      </button>
    </div>

    <div class="row">
      <div class="form-group">
        <label for="c-tid">Tipo Identificación</label>
        <select id="c-tid" value={tipoId} onchange={handleTipoIdChange}>
          <option value="05">05 - Cédula</option>
          <option value="04">04 - RUC</option>
          <option value="06">06 - Pasaporte</option>
          <option value="07">07 - Consumidor Final</option>
        </select>
      </div>
      <div class="form-group">
        <label for="c-id" style="display: flex; justify-content: space-between; align-items: center;">
          <span>Identificación</span>
          {#if clienteEncontrado}
            <span style="font-size: 0.72rem; color: var(--success-color); font-weight: 600; display: inline-flex; align-items: center; gap: 0.25rem;">
              <Icon name="check" size="0.95em" /> Cliente Frecuente (Autocompletado)
            </span>
          {/if}
        </label>
        <input
          id="c-id"
          type="text"
          bind:value={identificacion}
          oninput={(e) => buscarYAutocompletarCliente(e.target.value)}
          onblur={(e) => buscarYAutocompletarCliente(e.target.value)}
          placeholder="Cédula / RUC / 9999999999999"
        />
      </div>
    </div>
    <div class="form-group">
      <label for="c-name">Nombres / Razón Social Cliente</label>
      <input id="c-name" type="text" bind:value={razonSocial} placeholder="CONSUMIDOR FINAL u Nombre del cliente" />
    </div>
    <div class="row">
      <div class="form-group">
        <label for="c-email">Email</label>
        <input id="c-email" type="email" bind:value={email} placeholder="correo@ejemplo.com (Opcional)" />
      </div>
      <div class="form-group">
        <label for="c-tel">Teléfono</label>
        <input id="c-tel" type="text" bind:value={telefono} placeholder="0991234567 (Opcional)" />
      </div>
    </div>

    <!-- ÍTEMS -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 1.5rem;">
      <h3 style="font-size: 0.85rem; color: var(--text-muted); text-transform: uppercase;">Ítems de la Factura</h3>
      <button type="button" class="btn btn-secondary btn-sm" onclick={addItem}>
        <Icon name="plus" size="1em" /> Agregar Ítem
      </button>
    </div>

    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>Seleccionar Producto / Servicio</th>
            <th>Descripción</th>
            <th style="width: 75px;">Cant.</th>
            <th style="width: 95px;">P.Unit ($)</th>
            <th style="width: 85px;">IVA</th>
            <th style="width: 40px;"></th>
          </tr>
        </thead>
        <tbody>
          {#each items as item, index}
            <tr>
              <td>
                <select value={item.codigo} onchange={(e) => seleccionarProducto(index, e.target.value)}>
                  <option value="">-- Seleccionar --</option>
                  {#each productosInventario as p}
                    <option value={p.codigo}>[{p.tipo === 'SERVICIO' ? 'SERVICIO' : 'PRODUCTO'}] {p.codigo} - {p.descripcion} (${p.precio_unitario.toFixed(2)})</option>
                  {/each}
                </select>
              </td>
              <td><input type="text" bind:value={item.descripcion} /></td>
              <td><input type="number" step="1" min="1" bind:value={item.cantidad} /></td>
              <td><input type="number" step="0.01" min="0" bind:value={item.precio} /></td>
              <td>
                <select bind:value={item.iva} onchange={() => item.tarifa = item.iva === '4' ? 15.0 : 0.0}>
                  <option value="4">15%</option>
                  <option value="0">0%</option>
                </select>
              </td>
              <td>
                <button type="button" class="btn btn-danger btn-sm" onclick={() => removeItem(index)} aria-label="Eliminar item">
                  <Icon name="x" size="1em" />
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- TOTALES REACTIVOS -->
    <div class="totales-box">
      <div class="totales-row"><span>Subtotal IVA 15%:</span> <span>${subtotal15.toFixed(2)}</span></div>
      <div class="totales-row"><span>Subtotal IVA 0%:</span> <span>${subtotal0.toFixed(2)}</span></div>
      <div class="totales-row"><span>Monto IVA (15%):</span> <span>${montoIva.toFixed(2)}</span></div>
      <div class="totales-row total-final"><span>TOTAL FACTURA:</span> <span>${totalFactura.toFixed(2)}</span></div>
    </div>

    <!-- FORMAS DE PAGO (SRI) -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 1.5rem;">
      <h3 style="font-size: 0.85rem; color: var(--text-muted); text-transform: uppercase;">
        💳 Formas de Pago (SRI)
      </h3>
      <button type="button" class="btn btn-secondary btn-sm" onclick={addFormaPago}>
        <Icon name="plus" size="1em" /> Agregar Forma de Pago
      </button>
    </div>

    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>Forma de Pago*</th>
            <th style="width: 120px;">Valor ($)*</th>
            <th style="width: 95px;">Plazo</th>
            <th style="width: 105px;">Tiempo</th>
            <th style="width: 40px;"></th>
          </tr>
        </thead>
        <tbody>
          {#each formasPago as pago, index}
            <tr>
              <td>
                <select bind:value={pago.forma_pago}>
                  {#each FORMAS_PAGO_SRI as fp}
                    <option value={fp.codigo}>{fp.codigo} - {fp.descripcion}</option>
                  {/each}
                </select>
              </td>
              <td>
                <input
                  type="number"
                  step="0.01"
                  min="0"
                  bind:value={pago.total}
                  required
                />
              </td>
              <td>
                <input
                  type="number"
                  step="1"
                  min="0"
                  placeholder="Ej: 30"
                  bind:value={pago.plazo}
                />
              </td>
              <td>
                <select bind:value={pago.unidad_tiempo}>
                  {#each UNIDADES_TIEMPO as ut}
                    <option value={ut.codigo}>{ut.label}</option>
                  {/each}
                </select>
              </td>
              <td>
                {#if formasPago.length > 1}
                  <button
                    type="button"
                    class="btn btn-danger btn-sm"
                    onclick={() => removeFormaPago(index)}
                    aria-label="Eliminar forma de pago"
                  >
                    <Icon name="x" size="1em" />
                  </button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- BALANCE / CUADRE DE FORMAS DE PAGO -->
    <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.5rem; margin-top: 0.4rem; padding: 0.5rem 0.8rem; background: var(--bg-input); border-radius: var(--radius-input); border: 1px solid var(--border-color); font-size: 0.82rem;">
      <div style="display: flex; align-items: center; gap: 0.6rem;">
        <span><strong>Total Pagos:</strong> ${totalPagos.toFixed(2)}</span>
        <span>|</span>
        <span><strong>Total Factura:</strong> ${totalFactura.toFixed(2)}</span>
      </div>
      <div>
        {#if Math.abs(diferenciaPagos) < 0.01}
          <span style="color: var(--success-color); font-weight: 600; display: inline-flex; align-items: center; gap: 0.25rem;">
            <Icon name="check" size="0.95em" /> Monto Cuadrado
          </span>
        {:else}
          <div style="display: inline-flex; align-items: center; gap: 0.5rem;">
            <span style="color: var(--danger-color); font-weight: 600;">
              Diferencia: ${Math.abs(diferenciaPagos).toFixed(2)} ({diferenciaPagos > 0 ? 'Falta asignar' : 'Excedido'})
            </span>
            <button
              type="button"
              class="btn btn-secondary btn-sm"
              style="padding: 0.2rem 0.5rem; font-size: 0.75rem;"
              onclick={ajustarAlTotal}
            >
              ⚡ Ajustar al Total
            </button>
          </div>
        {/if}
      </div>
    </div>

    <div class="form-group" style="margin-top: 1.2rem;">
      <label for="c-pass">Contraseña Firma .p12 (Opcional si ya se guardó en Configuración)</label>
      <input id="c-pass" type="password" bind:value={passwordP12} placeholder="••••••••" />
    </div>

    <div style="margin-top: 1.5rem;">
      <button type="submit" class="btn" disabled={isEmitiendo}>
        {#if isEmitiendo}
          <span>Procesando y Firmando...</span>
        {:else}
          <Icon name="send" size="1.2em" /> 🚀 Emitir Factura Electrónica SRI
        {/if}
      </button>
    </div>
  </form>

  <!-- TARJETA DE RESPUESTA DETALLADA DEL SRI -->
  {#if respuestaSRI}
    <div
      class="card"
      style="margin-top: 1.5rem; margin-bottom: 0; border-color: {respuestaSRI.estado === 'AUTORIZADO' ? 'var(--success-color)' : 'var(--danger-color)'};"
    >
      <h3 style="display: flex; align-items: center; gap: 0.6rem; font-size: 1rem; margin-bottom: 0.5rem;">
        <span>Estado SRI:</span>
        <span class="status-badge status-{respuestaSRI.estado.toLowerCase().replace(' ', '_')}">{respuestaSRI.estado}</span>
      </h3>
      
      <div style="margin-top: 0.6rem; display: flex; flex-direction: column; gap: 0.4rem; background: var(--bg-input); padding: 0.8rem; border-radius: var(--radius-input);">
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.5rem;">
          <span style="font-size: 0.82rem; color: var(--text-muted);">
            <strong>Número de Autorización / Clave de Acceso (49 dígitos):</strong>
          </span>
          <button
            type="button"
            class="btn btn-secondary btn-sm"
            style="padding: 0.25rem 0.6rem; font-size: 0.75rem;"
            onclick={() => {
              if (typeof navigator !== 'undefined' && navigator.clipboard) {
                navigator.clipboard.writeText(respuestaSRI.clave_acceso);
              }
              toastStore.info('Número de Autorización de 49 dígitos copiado al portapapeles.', 'Copiado');
            }}
          >
            📋 Copiar N° Autorización
          </button>
        </div>
        <code style="font-family: 'JetBrains Mono', monospace; font-size: 0.82rem; word-break: break-all; color: var(--accent-color);">
          {respuestaSRI.clave_acceso}
        </code>
      </div>

      {#if respuestaSRI.mensajes && respuestaSRI.mensajes.length > 0}
        <div style="margin-top: 0.8rem; background: var(--bg-input); padding: 0.85rem; border-radius: var(--radius-input); border: 1px solid var(--border-color);">
          <strong style="font-size: 0.78rem; text-transform: uppercase; color: var(--text-muted); display: block; margin-bottom: 0.4rem;">
            Detalles de Respuesta / Errores del SRI:
          </strong>
          <ul style="margin: 0; padding-left: 1.2rem; font-size: 0.85rem; display: flex; flex-direction: column; gap: 0.3rem;">
            {#each respuestaSRI.mensajes as m}
              <li style="color: {respuestaSRI.estado === 'AUTORIZADO' ? 'var(--success-color)' : 'var(--danger-color)'}; font-family: 'JetBrains Mono', monospace; font-size: 0.8rem;">
                {m}
              </li>
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  {/if}
</div>
