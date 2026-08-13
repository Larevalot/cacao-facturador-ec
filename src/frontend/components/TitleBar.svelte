<script>
  import logoImg from '../assets/LOGO.png';
  import Icon from './Icon.svelte';
  import { toastStore } from '../lib/toast.svelte.js';

  let { theme = $bindable('dark'), isUnlocked = false, onLock = () => {} } = $props();

  async function handleMinimize() {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('minimize_window');
      }
    } catch(e) { console.error('Error minimizando ventana:', e); }
  }

  async function handleMaximize() {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('toggle_maximize_window');
      }
    } catch(e) { console.error('Error maximizando ventana:', e); }
  }

  async function handleClose() {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('close_window');
      }
    } catch(e) { console.error('Error cerrando ventana:', e); }
  }

  async function handleStartDrag(e) {
    if (e.button !== 0 || e.target.closest('button')) return;
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('start_drag_window');
      }
    } catch(err) {
      console.error('Error al arrastrar ventana:', err);
    }
  }

  function toggleTheme() {
    theme = theme === 'light' ? 'dark' : 'light';
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('cacao-theme', theme);
    toastStore.info(`Modo ${theme === 'light' ? 'Claro' : 'Oscuro'} activado.`, 'Tema Cambiado');
  }

  async function buscarActualizaciones() {
    toastStore.info('Verificando actualizaciones con el servidor CacaoApps...', 'Actualizaciones');
    try {
      const res = await fetch('https://api.github.com/repos/cacaoscript/cacaofacturador-ec/releases/latest', { cache: 'no-cache' });
      if (res.ok) {
        const data = await res.json();
        const latestTag = data.tag_name || data.name || '';
        if (latestTag && latestTag !== 'v1.0.0' && latestTag !== '1.0.0') {
          toastStore.info(`¡Nueva versión ${latestTag} disponible! Redirigiendo a descargas...`, 'Actualización Disponible');
          if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
            const { invoke } = await import('@tauri-apps/api/core');
            await invoke('open_external_url', { url: data.html_url || 'https://cacaoscript.com' });
          } else {
            window.open(data.html_url || 'https://cacaoscript.com', '_blank');
          }
          return;
        }
      }
    } catch (err) {
      console.log('Finalizado chequeo de actualización:', err);
    }
    toastStore.success('Cacao Facturador v1.0.0 - Tu aplicación está en la versión más reciente.', 'Sistema Actualizado');
  }
</script>

<div class="custom-titlebar" data-tauri-drag-region onmousedown={handleStartDrag} role="toolbar" tabindex="-1">
  <div class="titlebar-left" data-tauri-drag-region onmousedown={handleStartDrag} role="presentation">
    <img src={logoImg} alt="Logo Cacao" class="titlebar-logo" data-tauri-drag-region />
    <span class="titlebar-brand" data-tauri-drag-region>Cacao Facturador</span>
    <span class="titlebar-version-badge" data-tauri-drag-region>v1.0.0</span>
  </div>

  <div class="titlebar-drag-space" data-tauri-drag-region onmousedown={handleStartDrag} role="presentation"></div>

  <div class="titlebar-right">
    <button type="button" class="titlebar-action-btn" onclick={toggleTheme} aria-label="Modo claro u oscuro" title={theme === 'light' ? 'Modo Claro' : 'Modo Oscuro'}>
      <Icon name={theme === 'light' ? 'sun' : 'moon'} size="1.1em" />
    </button>

    <button type="button" class="titlebar-action-btn" onclick={buscarActualizaciones} aria-label="Buscar actualizaciones" title="Buscar actualizaciones">
      <Icon name="refresh" size="1.1em" />
    </button>

    {#if isUnlocked}
      <button type="button" class="titlebar-action-btn" onclick={onLock} title="Bloquear App">
        <Icon name="lock" size="1.05em" />
      </button>
    {/if}

    <div class="titlebar-divider"></div>

    <button type="button" class="titlebar-window-btn" onclick={handleMinimize} title="Minimizar">
      <Icon name="minus" size="0.9em" />
    </button>

    <button type="button" class="titlebar-window-btn" onclick={handleMaximize} title="Maximizar / Restaurar">
      <Icon name="square" size="0.85em" />
    </button>

    <button type="button" class="titlebar-window-btn btn-close" onclick={handleClose} title="Cerrar">
      <Icon name="x" size="1em" />
    </button>
  </div>
</div>
