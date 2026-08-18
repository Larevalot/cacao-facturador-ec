<script>
  import { fetchFacturas } from '../lib/api.js';
  import { toastStore } from '../lib/toast.svelte.js';
  import Icon from './Icon.svelte';

  const FORMAS_PAGO_NOMBRES = {
    '01': 'SIN UTILIZACION DEL SISTEMA FINANCIERO',
    '15': 'COMPENSACIÓN DE DEUDAS',
    '16': 'TARJETA DE DÉBITO',
    '17': 'DINERO ELECTRÓNICO',
    '18': 'TARJETA PREPAGO',
    '19': 'TARJETA DE CRÉDITO',
    '20': 'OTROS CON UTILIZACION DEL SISTEMA FINANCIERO',
    '21': 'ENDOSO DE TÍTULOS'
  };

  let facturas = $state([]);
  let copiedClave = $state(null);
  let facturaSeleccionada = $state(null);

  async function cargar() {
    try {
      facturas = await fetchFacturas();
    } catch(e) {
      console.error('Error cargando historial:', e);
    }
  }

  function copiarClave(clave) {
    if (typeof navigator !== 'undefined' && navigator.clipboard) {
      navigator.clipboard.writeText(clave);
      copiedClave = clave;
      toastStore.info('Número de Autorización / Clave de Acceso copiado.', 'Copiado');
      setTimeout(() => {
        copiedClave = null;
      }, 2000);
    }
  }

  function anularEnSRI(clave) {
    copiarClave(clave);
    toastStore.info('Clave de Acceso de 49 dígitos copiada al portapapeles. Abriendo portal SRI en Línea...', 'Anulación SRI');
    if (typeof window !== 'undefined') {
      window.open('https://srienlinea.sri.gob.ec', '_blank');
    }
  }

  function formatClaveCorta(clave) {
    if (!clave || clave.length < 20) return clave || '';
    return `${clave.slice(0, 10)}...${clave.slice(-8)}`;
  }

  function extraerPagosDeXml(xmlStr) {
    if (!xmlStr) return [];
    try {
      const pagos = [];
      const pagoRegex = /<pago>([\s\S]*?)<\/pago>/g;
      let match;
      while ((match = pagoRegex.exec(xmlStr)) !== null) {
        const bloque = match[1];
        const fpMatch = /<formaPago>(.*?)<\/formaPago>/.exec(bloque);
        const totalMatch = /<total>(.*?)<\/total>/.exec(bloque);
        const plazoMatch = /<plazo>(.*?)<\/plazo>/.exec(bloque);
        const utMatch = /<unidadTiempo>(.*?)<\/unidadTiempo>/.exec(bloque);
        
        const codigoFp = fpMatch ? fpMatch[1].trim() : '01';
        pagos.push({
          forma_pago: codigoFp,
          nombre: FORMAS_PAGO_NOMBRES[codigoFp] || `Código ${codigoFp}`,
          total: totalMatch ? parseFloat(totalMatch[1]) : 0,
          plazo: plazoMatch ? plazoMatch[1].trim() : null,
          unidad_tiempo: utMatch ? utMatch[1].trim() : null
        });
      }
      return pagos;
    } catch (e) {
      console.error('Error parseando pagos del XML:', e);
      return [];
    }
  }

  function verDetalle(f) {
    facturaSeleccionada = f;
  }

  cargar();
</script>

<div class="card">
  <h2>
    <span style="display: inline-flex; align-items: center; gap: 0.5rem;">
      <Icon name="history" size="1.2em" /> Historial de Facturas Emitidas
    </span>
  </h2>
  <div class="table-container">
    <table>
      <thead>
        <tr>
          <th>Fecha</th>
          <th>Secuencial</th>
          <th>Cliente</th>
          <th>Identificación</th>
          <th>Total ($)</th>
          <th>Estado SRI</th>
          <th>Acciones</th>
        </tr>
      </thead>
      <tbody>
        {#if facturas.length === 0}
          <tr>
            <td colspan="7" style="text-align: center; color: var(--text-muted);">No se han emitido facturas aún</td>
          </tr>
        {:else}
          {#each facturas as f (f.id)}
            <tr>
              <td style="white-space: nowrap;">{f.fecha_emision}</td>
              <td style="white-space: nowrap;"><strong>{f.secuencial}</strong></td>
              <td style="max-width: 160px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title={f.cliente_razon}>{f.cliente_razon}</td>
              <td style="white-space: nowrap;">{f.cliente_identificacion}</td>
              <td style="font-weight: 700; color: var(--accent-color); white-space: nowrap;">${f.importe_total.toFixed(2)}</td>
              <td>
                <span class="status-badge status-{f.estado.toLowerCase().replace(' ', '_')}">{f.estado}</span>
              </td>
              <td>
                <div style="display: flex; align-items: center; gap: 0.4rem; flex-wrap: wrap;">
                  <button
                    type="button"
                    class="btn-secondary"
                    style="padding: 0.25rem 0.55rem; font-size: 0.74rem; white-space: nowrap;"
                    onclick={() => verDetalle(f)}
                    title="Ver detalle completo de productos, formas de pago, plazos y XML"
                  >
                    👁️ Ver Detalle
                  </button>
                  <button
                    type="button"
                    class="btn-secondary"
                    style="padding: 0.25rem 0.55rem; font-size: 0.74rem; white-space: nowrap;"
                    onclick={() => copiarClave(f.clave_acceso)}
                    title="Copiar Número de Autorización / Clave de Acceso Completa (49 dígitos)"
                  >
                    {copiedClave === f.clave_acceso ? '✓ Copiado' : '📋 Copiar N°'}
                  </button>
                  <button
                    type="button"
                    class="btn-secondary"
                    style="padding: 0.25rem 0.55rem; font-size: 0.74rem; white-space: nowrap; color: var(--danger-color); border-color: var(--danger-color);"
                    onclick={() => anularEnSRI(f.clave_acceso)}
                    title="Copiar clave de 49 dígitos y abrir portal SRI en Línea para anular"
                  >
                    🚫 Anular
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

<!-- MODAL DE DETALLE COMPLETO DE FACTURA -->
{#if facturaSeleccionada}
  <div class="modal-overlay active">
    <div class="modal-box" style="max-width: 650px;">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; border-bottom: 1px solid var(--border-color); padding-bottom: 0.75rem;">
        <h2 style="font-size: 1.15rem; margin: 0; display: flex; align-items: center; gap: 0.5rem; color: var(--text-main);">
          <Icon name="receipt" size="1.2em" /> Factura #{facturaSeleccionada.secuencial}
        </h2>
        <button
          type="button"
          class="btn-secondary btn-sm"
          style="padding: 0.2rem 0.5rem;"
          onclick={() => facturaSeleccionada = null}
          aria-label="Cerrar modal"
        >
          <Icon name="x" size="1.1em" />
        </button>
      </div>

      <!-- DATOS GENERALES -->
      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem; background: var(--bg-input); padding: 0.85rem; border-radius: var(--radius-input); margin-bottom: 1rem; font-size: 0.85rem;">
        <div><strong>Fecha Emisión:</strong> {facturaSeleccionada.fecha_emision}</div>
        <div><strong>Estado SRI:</strong> <span class="status-badge status-{facturaSeleccionada.estado.toLowerCase().replace(' ', '_')}">{facturaSeleccionada.estado}</span></div>
        <div><strong>Cliente:</strong> {facturaSeleccionada.cliente_razon}</div>
        <div><strong>Identificación:</strong> {facturaSeleccionada.cliente_identificacion}</div>
        <div><strong>Subtotal Sin Impuestos:</strong> ${facturaSeleccionada.total_sin_impuestos.toFixed(2)}</div>
        <div><strong>Monto IVA:</strong> ${facturaSeleccionada.total_iva.toFixed(2)}</div>
        <div style="grid-column: span 2; font-size: 1rem; color: var(--accent-color); font-weight: 700; border-top: 1px dashed var(--border-color); padding-top: 0.5rem;">
          TOTAL FACTURA: ${facturaSeleccionada.importe_total.toFixed(2)}
        </div>
      </div>

      <!-- FORMAS DE PAGO Y PLAZOS -->
      <h3 style="font-size: 0.85rem; color: var(--text-muted); text-transform: uppercase; margin: 1rem 0 0.5rem 0; display: flex; align-items: center; gap: 0.4rem;">
        💳 Formas de Pago, Plazos y Tiempo (SRI)
      </h3>
      <div class="table-container" style="margin-bottom: 1rem;">
        <table>
          <thead>
            <tr>
              <th>Forma de Pago</th>
              <th style="width: 110px;">Valor ($)</th>
              <th style="width: 140px;">Plazo / Tiempo</th>
            </tr>
          </thead>
          <tbody>
            {#each extraerPagosDeXml(facturaSeleccionada.xml_firmado || facturaSeleccionada.xml_autorizado) as p}
              <tr>
                <td><strong>{p.forma_pago}</strong> - {p.nombre}</td>
                <td style="font-weight: 600; color: var(--accent-color);">${p.total.toFixed(2)}</td>
                <td>
                  {#if p.plazo}
                    <span style="display: inline-block; padding: 0.2rem 0.5rem; background: var(--bg-card); border-radius: 4px; border: 1px solid var(--border-color); font-size: 0.8rem; font-weight: 600;">
                      ⏱️ {p.plazo} {p.unidad_tiempo ? p.unidad_tiempo.toUpperCase() : 'DÍAS'}
                    </span>
                  {:else}
                    <span style="color: var(--text-muted); font-size: 0.8rem;">Al contado (Sin plazo)</span>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <!-- CLAVE DE ACCESO -->
      <div style="background: var(--bg-input); padding: 0.75rem; border-radius: var(--radius-input); margin-bottom: 1.2rem;">
        <div style="font-size: 0.78rem; color: var(--text-muted); margin-bottom: 0.3rem;">
          <strong>Número de Autorización / Clave de Acceso (49 dígitos):</strong>
        </div>
        <code style="font-family: 'JetBrains Mono', monospace; font-size: 0.78rem; word-break: break-all; color: var(--accent-color);">
          {facturaSeleccionada.clave_acceso}
        </code>
      </div>

      <div style="display: flex; justify-content: flex-end; gap: 0.6rem;">
        <button type="button" class="btn btn-secondary" onclick={() => copiarClave(facturaSeleccionada.clave_acceso)}>
          📋 Copiar Clave
        </button>
        <button type="button" class="btn" onclick={() => facturaSeleccionada = null}>
          Cerrar
        </button>
      </div>
    </div>
  </div>
{/if}
