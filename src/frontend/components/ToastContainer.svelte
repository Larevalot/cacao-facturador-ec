<script>
  import { toastStore } from '../lib/toast.svelte.js';
  import Icon from './Icon.svelte';
</script>

<div class="toast-container">
  {#each toastStore.toasts as toast (toast.id)}
    <div class="toast toast-{toast.type}">
      <div class="toast-icon">
        {#if toast.type === 'success'}
          <Icon name="check" size="1.25em" style="color: var(--success-color);" />
        {:else if toast.type === 'error'}
          <Icon name="alert-triangle" size="1.25em" style="color: var(--danger-color);" />
        {:else if toast.type === 'warning'}
          <Icon name="alert-circle" size="1.25em" style="color: #f59e0b;" />
        {:else}
          <Icon name="info" size="1.25em" style="color: var(--accent-color);" />
        {/if}
      </div>
      <div class="toast-content">
        {#if toast.title}
          <div class="toast-title">{toast.title}</div>
        {/if}
        <div class="toast-message">{toast.message}</div>
      </div>
      <button type="button" class="toast-close" onclick={() => toastStore.remove(toast.id)} aria-label="Cerrar notificación">
        <Icon name="x" size="0.9em" />
      </button>
    </div>
  {/each}
</div>
