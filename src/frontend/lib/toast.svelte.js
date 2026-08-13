let toastsList = $state([]);

export const toastStore = {
  get toasts() {
    return toastsList;
  },
  show(message, type = 'info', title = '', duration = 4000) {
    const id = Date.now() + Math.random();
    const item = { id, type, title, message };
    toastsList = [...toastsList, item];
    if (duration > 0) {
      setTimeout(() => {
        this.remove(id);
      }, duration);
    }
  },
  success(message, title = 'Éxito') {
    this.show(message, 'success', title);
  },
  error(message, title = 'Error') {
    this.show(message, 'error', title);
  },
  info(message, title = 'Información') {
    this.show(message, 'info', title);
  },
  warning(message, title = 'Advertencia') {
    this.show(message, 'warning', title);
  },
  remove(id) {
    toastsList = toastsList.filter(t => t.id !== id);
  }
};
