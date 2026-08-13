<script>
  import Icon from './Icon.svelte';
  let { activeSection = 'facturador', onSelectSection = (sec) => {}, onGoHome = () => {} } = $props();
</script>

<div style="display: flex; flex-direction: column; gap: 1rem;">
  <div class="sidebar-rounded">
    <button type="button" class="sidebar-header-btn" onclick={onGoHome}>
      <Icon name="home" size="1.1em" /> Inicio / Dashboard
    </button>

    <button type="button" class="sidebar-nav-btn {activeSection === 'facturador' ? 'active' : ''}" onclick={() => onSelectSection('facturador')}>
      <Icon name="receipt" size="1.1em" /> Facturador
    </button>

    <button type="button" class="sidebar-nav-btn {activeSection === 'inventario' ? 'active' : ''}" onclick={() => onSelectSection('inventario')}>
      <Icon name="package" size="1.1em" /> Inventario
    </button>

    <button type="button" class="sidebar-nav-btn {activeSection === 'historial' ? 'active' : ''}" onclick={() => onSelectSection('historial')}>
      <Icon name="history" size="1.1em" /> Historial
    </button>

    <button type="button" class="sidebar-nav-btn {activeSection === 'configuracion' ? 'active' : ''}" onclick={() => onSelectSection('configuracion')}>
      <Icon name="settings" size="1.1em" /> Configuración
    </button>
  </div>
</div>
