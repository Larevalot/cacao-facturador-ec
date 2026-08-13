<script>
  import { fetchFacturas } from '../lib/api.js';
  import { toastStore } from '../lib/toast.svelte.js';
  import Icon from './Icon.svelte';

  let facturas = $state([]);
  let copiedClave = $state(null);

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
          <th>N° Autorización / Clave de Acceso & Acciones</th>
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
                <div style="display: flex; align-items: center; gap: 0.4rem;">
                  <span
                    style="font-family: 'JetBrains Mono', monospace; font-size: 0.76rem; color: var(--text-muted);"
                    title={f.clave_acceso}
                  >
                    {formatClaveCorta(f.clave_acceso)}
                  </span>
                  <button
                    type="button"
                    class="btn-secondary"
                    style="padding: 0.2rem 0.45rem; font-size: 0.72rem; white-space: nowrap;"
                    onclick={() => copiarClave(f.clave_acceso)}
                    title="Copiar Número de Autorización / Clave de Acceso Completa (49 dígitos)"
                  >
                    {copiedClave === f.clave_acceso ? '✓ Copiado' : '📋 Copiar N° Autorización'}
                  </button>
                  <button
                    type="button"
                    class="btn-secondary"
                    style="padding: 0.2rem 0.45rem; font-size: 0.72rem; white-space: nowrap; color: var(--danger-color); border-color: var(--danger-color);"
                    onclick={() => anularEnSRI(f.clave_acceso)}
                    title="Copiar clave de 49 dígitos y abrir portal SRI en Línea para anular"
                  >
                    🚫 Anular en SRI
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
