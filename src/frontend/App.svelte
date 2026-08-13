<script>
  import TitleBar from './components/TitleBar.svelte';
  import PinScreen from './components/PinScreen.svelte';
  import Dashboard from './components/Dashboard.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import Facturador from './components/Facturador.svelte';
  import Inventario from './components/Inventario.svelte';
  import Historial from './components/Historial.svelte';
  import Configuracion from './components/Configuracion.svelte';
  import ToastContainer from './components/ToastContainer.svelte';

  let theme = $state('dark');
  let isUnlocked = $state(false);
  let currentView = $state('dashboard'); // 'dashboard' | 'section'
  let activeSection = $state('facturador'); // 'facturador' | 'inventario' | 'historial' | 'configuracion'

  $effect(() => {
    if (typeof localStorage !== 'undefined') {
      const saved = localStorage.getItem('cacao-theme') || 'dark';
      theme = saved;
      document.documentElement.setAttribute('data-theme', saved);
    }
  });

  function handleUnlock() {
    isUnlocked = true;
    currentView = 'dashboard';
  }

  function handleLock() {
    isUnlocked = false;
    currentView = 'dashboard';
  }

  function handleNavigate(sec) {
    activeSection = sec;
    currentView = 'section';
  }

  function handleGoHome() {
    currentView = 'dashboard';
  }

  async function handleOpenUrl(url) {
    if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
      try {
        const { invoke } = window.__TAURI_INTERNALS__;
        await invoke('open_external_url', { url });
        return;
      } catch (err) {
        console.error('Error invocando open_external_url:', err);
      }
    }
    if (typeof window !== 'undefined') {
      window.open(url, '_blank', 'noopener,noreferrer');
    }
  }
</script>

<TitleBar bind:theme {isUnlocked} onLock={handleLock} />
<ToastContainer />

<div class="container">

  {#if !isUnlocked}
    <PinScreen onUnlock={handleUnlock} />
  {:else if currentView === 'dashboard'}
    <Dashboard onNavigate={handleNavigate} />
  {:else}
    <div class="section-layout">
      <Sidebar
        {activeSection}
        onSelectSection={(sec) => activeSection = sec}
        onGoHome={handleGoHome}
      />

      <div>
        {#if activeSection === 'facturador'}
          <Facturador />
        {:else if activeSection === 'inventario'}
          <Inventario />
        {:else if activeSection === 'historial'}
          <Historial />
        {:else if activeSection === 'configuracion'}
          <Configuracion />
        {/if}
      </div>
    </div>
  {/if}
</div>

<!-- PIE DE CRÉDITOS FIJO EN LA ESQUINA INFERIOR IZQUIERDA DE LA VENTANA GENERAL (SIN CAJA) -->
{#if isUnlocked}
  <div class="fixed-pixel-art-credits">
    desarrollado por{' '}
    <a
      href="https://cacaoscript.com"
      onclick={(e) => { e.preventDefault(); handleOpenUrl('https://cacaoscript.com'); }}
      title="Abrir cacaoscript.com en el navegador"
    >
      cacaoscript
    </a>
  </div>
{/if}
