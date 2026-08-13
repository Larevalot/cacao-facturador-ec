<script>
  import Icon from './Icon.svelte';
  let { onNavigate = (sec) => {} } = $props();
</script>

<div class="dashboard-header">
  <h1>Panel de Control</h1>
  <p>Selecciona una opción para gestionar tu facturación e inventario</p>
</div>

<div class="dashboard-grid">
  <!-- CAJA 1: FACTURADOR -->
  <button type="button" class="dash-card" onclick={() => onNavigate('facturador')}>
    <div class="dash-icon-box">
      <Icon name="receipt" size="2.2rem" />
    </div>
    <h3>Facturador</h3>
    <p>Generar, firmar y autorizar comprobantes electrónicos ante el SRI</p>
  </button>

  <!-- CAJA 2: INVENTARIO -->
  <button type="button" class="dash-card" onclick={() => onNavigate('inventario')}>
    <div class="dash-icon-box">
      <Icon name="package" size="2.2rem" />
    </div>
    <h3>Inventario</h3>
    <p>Administrar productos, precios, tarifas de IVA y stock en tiempo real</p>
  </button>

  <!-- CAJA 3: HISTORIAL -->
  <button type="button" class="dash-card" onclick={() => onNavigate('historial')}>
    <div class="dash-icon-box">
      <Icon name="history" size="2.2rem" />
    </div>
    <h3>Historial</h3>
    <p>Consultar facturas emitidas, claves de acceso y respuestas del SRI</p>
  </button>

  <!-- CAJA 4: CONFIGURACIÓN -->
  <button type="button" class="dash-card" onclick={() => onNavigate('configuracion')}>
    <div class="dash-icon-box">
      <Icon name="settings" size="2.2rem" />
    </div>
    <h3>Configuración</h3>
    <p>Configurar RUC del emisor, certificado digital .p12 y ambiente</p>
  </button>
</div>
