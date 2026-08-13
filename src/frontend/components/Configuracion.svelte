<script>
  import { fetchConfig, saveConfig, uploadP12, deleteP12 } from '../lib/api.js';
  import { toastStore } from '../lib/toast.svelte.js';
  import Icon from './Icon.svelte';

  let ruc = $state('');
  let razonSocial = $state('');
  let nombreComercial = $state('');
  let dirMatriz = $state('');
  let dirEstablecimiento = $state('');
  let codEstablecimiento = $state('001');
  let ptoEmision = $state('001');
  let obligadoContabilidad = $state('NO');
  let ambiente = $state('1');
  let regimenRimpe = $state('CONTRIBUYENTE RÉGIMEN RIMPE');
  let p12Password = $state('');
  let p12Status = $state('Sin firma subida');
  let p12Path = $state(null);

  async function cargar() {
    try {
      const cfg = await fetchConfig();
      ruc = cfg.ruc || '';
      razonSocial = cfg.razon_social || '';
      nombreComercial = cfg.nombre_comercial || '';
      dirMatriz = cfg.dir_matriz || '';
      dirEstablecimiento = cfg.dir_establecimiento || '';
      codEstablecimiento = cfg.cod_establecimiento || '001';
      ptoEmision = cfg.pto_emision || '001';
      obligadoContabilidad = cfg.obligado_contabilidad || 'NO';
      ambiente = cfg.ambiente || '1';
      regimenRimpe = cfg.regimen_rimpe || 'CONTRIBUYENTE RÉGIMEN RIMPE';
      p12Password = cfg.p12_password || '';

      if (cfg.p12_path) {
        p12Path = cfg.p12_path;
        p12Status = `Firma guardada: ${cfg.p12_path}`;
      }
    } catch(e) { console.error('Error cargando configuración:', e); }
  }

  cargar();

  async function handleSave(e) {
    if (e) {
      e.preventDefault();
      e.stopPropagation();
    }
    const cfg = {
      ruc,
      razon_social: razonSocial,
      nombre_comercial: nombreComercial,
      dir_matriz: dirMatriz,
      dir_establecimiento: dirEstablecimiento,
      cod_establecimiento: codEstablecimiento,
      pto_emision: ptoEmision,
      obligado_contabilidad: obligadoContabilidad,
      ambiente,
      regimen_rimpe: regimenRimpe,
      p12_password: p12Password,
      p12_path: p12Path,
    };

    try {
      await saveConfig(cfg);
      toastStore.success('Configuración guardada exitosamente.', 'Configuración');
    } catch(err) {
      toastStore.error('Error guardando configuración: ' + err.message, 'Configuración');
    }
  }

  async function handleFileChange(e) {
    const file = e.target.files[0];
    if (!file) return;
    try {
      const res = await uploadP12(file);
      if (res.path) {
        p12Path = res.path;
        p12Status = `Firma guardada: ${res.path}`;
        toastStore.success('Archivo de firma .p12 subido y reemplazado correctamente.', 'Firma Electrónica');
      }
    } catch(err) {
      toastStore.error('Error al subir firma .p12: ' + err.message, 'Firma Electrónica');
    }
  }

  async function handleDeleteSignature() {
    try {
      await deleteP12();
      p12Path = null;
      p12Password = '';
      p12Status = 'Sin firma subida';
      toastStore.success('Firma electrónica eliminada correctamente.', 'Firma Electrónica');
    } catch (err) {
      toastStore.error('Error al eliminar firma: ' + err.message, 'Firma Electrónica');
    }
  }
</script>

<div class="card">
  <h2>
    <span style="display: inline-flex; align-items: center; gap: 0.5rem;">
      <Icon name="settings" size="1.2em" /> Configuración Emisor
    </span>
  </h2>

  <form onsubmit={(e) => { e.preventDefault(); e.stopPropagation(); handleSave(e); }}>
    <div class="form-group">
      <label for="cfg-ruc-i">RUC (13 dígitos)</label>
      <input id="cfg-ruc-i" type="text" maxlength="13" bind:value={ruc} required />
    </div>
    <div class="form-group">
      <label for="cfg-rz-i">Razón Social</label>
      <input id="cfg-rz-i" type="text" bind:value={razonSocial} required />
    </div>
    <div class="form-group">
      <label for="cfg-nc-i">Nombre Comercial</label>
      <input id="cfg-nc-i" type="text" bind:value={nombreComercial} />
    </div>
    <div class="form-group">
      <label for="cfg-dm-i">Dirección Matriz</label>
      <input id="cfg-dm-i" type="text" bind:value={dirMatriz} required />
    </div>
    <div class="form-group">
      <label for="cfg-de-i">Dirección Establecimiento</label>
      <input id="cfg-de-i" type="text" bind:value={dirEstablecimiento} required />
    </div>
    <div class="row">
      <div class="form-group">
        <label for="cfg-ce-i">Establecimiento</label>
        <input id="cfg-ce-i" type="text" maxlength="3" bind:value={codEstablecimiento} required />
      </div>
      <div class="form-group">
        <label for="cfg-pe-i">Pto. Emisión</label>
        <input id="cfg-pe-i" type="text" maxlength="3" bind:value={ptoEmision} required />
      </div>
    </div>
    <div class="row">
      <div class="form-group">
        <label for="cfg-oc-i">Obligado Contab.</label>
        <select id="cfg-oc-i" bind:value={obligadoContabilidad}>
          <option value="NO">NO</option>
          <option value="SI">SI</option>
        </select>
      </div>
      <div class="form-group">
        <label for="cfg-amb-i">Ambiente SRI</label>
        <select id="cfg-amb-i" bind:value={ambiente}>
          <option value="1">1 - Pruebas</option>
          <option value="2">2 - Producción</option>
        </select>
      </div>
    </div>
    <div class="form-group">
      <label for="cfg-rim-i">Régimen RIMPE</label>
      <select id="cfg-rim-i" bind:value={regimenRimpe}>
        <option value="CONTRIBUYENTE RÉGIMEN RIMPE">CONTRIBUYENTE RÉGIMEN RIMPE (Emprendedor)</option>
        <option value="CONTRIBUYENTE NEGOCIO POPULAR - RÉGIMEN RIMPE">CONTRIBUYENTE NEGOCIO POPULAR - RÉGIMEN RIMPE</option>
        <option value="">Ninguno (Régimen General)</option>
      </select>
    </div>
    <hr style="border-color: var(--border-color); margin: 1.2rem 0;" />
    <div class="form-group">
      <label for="cfg-file-i">Firma Electrónica (.p12 / .pfx)</label>
      <div style="display: flex; gap: 0.5rem; align-items: center;">
        <input id="cfg-file-i" type="file" accept=".p12,.pfx" onchange={handleFileChange} style="flex: 1;" />
        {#if p12Path}
          <button
            type="button"
            class="btn btn-sm"
            onclick={handleDeleteSignature}
            title="Eliminar Firma Electrónica"
            style="background: #991b1b; color: #ffffff; padding: 0.4rem 0.75rem; border-radius: 6px; border: none; font-weight: 500; font-size: 0.8rem; display: inline-flex; align-items: center; gap: 0.3rem; cursor: pointer;"
          >
            <Icon name="trash" size="1em" /> Eliminar
          </button>
        {/if}
      </div>
      <span style="font-size: 0.78rem; color: {p12Path ? 'var(--success-color)' : 'var(--text-muted)'}; display: block; margin-top: 0.3rem;">
        {p12Status}
      </span>
    </div>
    <div class="form-group">
      <label for="cfg-pw-i">Contraseña Firma .p12</label>
      <input id="cfg-pw-i" type="password" bind:value={p12Password} placeholder="••••••••" />
    </div>
    <button type="submit" class="btn btn-secondary">
      <Icon name="save" size="1.1em" /> Guardar Configuración
    </button>
  </form>
</div>
