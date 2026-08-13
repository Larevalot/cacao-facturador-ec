<script>
  import { toastStore } from '../lib/toast.svelte.js';
  import Icon from './Icon.svelte';
  import logoImg from '../assets/LOGO.png';

  let { theme = $bindable('dark'), isUnlocked = false, onLock = () => {} } = $props();

  function toggleTheme() {
    theme = theme === 'light' ? 'dark' : 'light';
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('cacao-theme', theme);
    toastStore.info(`Modo ${theme === 'light' ? 'Claro' : 'Oscuro'} activado.`, 'Tema Cambiado');
  }

  function buscarActualizaciones() {
    toastStore.success('Tu aplicación Cacao Facturador v0.1.0 está actualizada.', 'Actualizaciones');
  }
</script>

<div class="top-bar-persistent">
  <div class="top-bar-left-actions">
    <button type="button" class="top-btn" onclick={toggleTheme} aria-label="Modo claro u oscuro" title={theme === 'light' ? 'Modo Claro' : 'Modo Oscuro'}>
      <Icon name={theme === 'light' ? 'sun' : 'moon'} size="1.1em" />
    </button>

    <button type="button" class="top-btn" onclick={buscarActualizaciones} aria-label="Buscar actualizaciones" title="Buscar actualizaciones">
      <Icon name="refresh" size="1.1em" />
    </button>
  </div>

  <div class="brand-logo-top">
    <img src={logoImg} alt="Logo Cacao" class="logo-img" />
    <span>Cacao Facturador</span>
    {#if isUnlocked}
      <button type="button" class="top-btn" onclick={onLock} style="margin-left: 0.5rem;" title="Bloquear App">
        <Icon name="lock" size="1em" />
      </button>
    {/if}
  </div>
</div>
