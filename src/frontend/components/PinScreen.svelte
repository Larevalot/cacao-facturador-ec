<script>
  import { onMount, onDestroy } from 'svelte';
  import { toastStore } from '../lib/toast.svelte.js';
  import logoImg from '../assets/LOGO.png';
  import Icon from './Icon.svelte';

  let { onUnlock = () => {} } = $props();

  let p1 = $state('');
  let p2 = $state('');
  let p3 = $state('');
  let p4 = $state('');

  let pinGuardado = $state(typeof localStorage !== 'undefined' ? localStorage.getItem('cacao_pin') : null);

  let canvasEl;
  let animId;

  onMount(() => {
    if (!canvasEl) return;
    const ctx = canvasEl.getContext('2d');
    let width = 0;
    let height = 0;
    let dots = [];
    let mouse = { x: -1000, y: -1000 };
    let time = 0;

    const spacing = 26;

    function initDots() {
      const parent = canvasEl.parentElement;
      width = canvasEl.width = parent.clientWidth;
      height = canvasEl.height = parent.clientHeight;
      dots = [];

      const cols = Math.ceil(width / spacing) + 1;
      const rows = Math.ceil(height / spacing) + 1;

      for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
          const baseX = c * spacing;
          const baseY = r * spacing;
          dots.push({
            r,
            c,
            baseX,
            baseY,
            x: baseX,
            y: baseY,
            vx: 0,
            vy: 0,
            size: 2.2
          });
        }
      }
    }

    initDots();

    const resizeObserver = new ResizeObserver(() => {
      initDots();
    });
    resizeObserver.observe(canvasEl.parentElement);

    function handleMouseMove(e) {
      const rect = canvasEl.getBoundingClientRect();
      mouse.x = e.clientX - rect.left;
      mouse.y = e.clientY - rect.top;
    }

    function handleMouseLeave() {
      mouse.x = -1000;
      mouse.y = -1000;
    }

    const parentEl = canvasEl.parentElement;
    parentEl.addEventListener('mousemove', handleMouseMove);
    parentEl.addEventListener('mouseleave', handleMouseLeave);

    function animate() {
      time += 0.025;
      ctx.clearRect(0, 0, width, height);

      const maxDist = 135;

      // Update positions
      for (let i = 0; i < dots.length; i++) {
        const dot = dots[i];

        // Wave motion
        const waveX = Math.sin(dot.baseY * 0.04 + time * 1.5) * 7;
        const waveY = Math.cos(dot.baseX * 0.04 + time * 1.5) * 7;
        let targetX = dot.baseX + waveX;
        let targetY = dot.baseY + waveY;

        // Repulsion from mouse
        const dx = dot.x - mouse.x;
        const dy = dot.y - mouse.y;
        const dist = Math.sqrt(dx * dx + dy * dy);

        if (dist < maxDist && dist > 0) {
          const force = Math.pow((maxDist - dist) / maxDist, 2);
          const angle = Math.atan2(dy, dx);
          const repelDist = force * 55;
          targetX += Math.cos(angle) * repelDist;
          targetY += Math.sin(angle) * repelDist;
        }

        // Spring physics
        dot.vx = (dot.vx + (targetX - dot.x) * 0.12) * 0.8;
        dot.vy = (dot.vy + (targetY - dot.y) * 0.12) * 0.8;
        dot.x += dot.vx;
        dot.y += dot.vy;
      }

      // Draw grid lines
      const cols = Math.ceil(width / spacing) + 1;
      ctx.lineWidth = 1;

      for (let i = 0; i < dots.length; i++) {
        const dot = dots[i];

        // Connect to right neighbor
        if (dot.c + 1 < cols && i + 1 < dots.length) {
          const rightDot = dots[i + 1];
          if (rightDot.r === dot.r) {
            const dLine = Math.hypot(dot.x - rightDot.x, dot.y - rightDot.y);
            if (dLine < spacing * 1.6) {
              const alpha = (1 - dLine / (spacing * 1.6)) * 0.15;
              ctx.strokeStyle = `rgba(210, 125, 45, ${alpha})`;
              ctx.beginPath();
              ctx.moveTo(dot.x, dot.y);
              ctx.lineTo(rightDot.x, rightDot.y);
              ctx.stroke();
            }
          }
        }

        // Connect to bottom neighbor
        const bottomIdx = i + cols;
        if (bottomIdx < dots.length) {
          const bottomDot = dots[bottomIdx];
          const dLine = Math.hypot(dot.x - bottomDot.x, dot.y - bottomDot.y);
          if (dLine < spacing * 1.6) {
            const alpha = (1 - dLine / (spacing * 1.6)) * 0.15;
            ctx.strokeStyle = `rgba(210, 125, 45, ${alpha})`;
            ctx.beginPath();
            ctx.moveTo(dot.x, dot.y);
            ctx.lineTo(bottomDot.x, bottomDot.y);
            ctx.stroke();
          }
        }
      }

      // Draw dots
      for (let i = 0; i < dots.length; i++) {
        const dot = dots[i];
        const distFromBase = Math.hypot(dot.x - dot.baseX, dot.y - dot.baseY);
        const intensity = Math.min(1, distFromBase / 30);
        const alpha = 0.25 + intensity * 0.6;

        ctx.beginPath();
        ctx.arc(dot.x, dot.y, dot.size + intensity * 1.2, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(${210 + Math.floor(intensity * 45)}, ${125 + Math.floor(intensity * 60)}, 45, ${alpha})`;
        ctx.fill();
      }

      animId = requestAnimationFrame(animate);
    }

    animate();

    return () => {
      cancelAnimationFrame(animId);
      resizeObserver.disconnect();
      parentEl.removeEventListener('mousemove', handleMouseMove);
      parentEl.removeEventListener('mouseleave', handleMouseLeave);
    };
  });

  onDestroy(() => {
    if (animId) cancelAnimationFrame(animId);
  });

  function handleInput(e, nextId) {
    if (e.target.value.length === 1 && nextId) {
      document.getElementById(nextId)?.focus();
    }
  }

  function handleKeyDown(e, prevId) {
    if (e.key === 'Backspace' && !e.target.value && prevId) {
      document.getElementById(prevId)?.focus();
    }
  }

  function submitPin(e) {
    e.preventDefault();
    const pinIngresado = `${p1}${p2}${p3}${p4}`;

    if (pinIngresado.length !== 4) {
      toastStore.warning('Por favor ingresa los 4 dígitos del PIN.', 'PIN Incompleto');
      return;
    }

    if (!pinGuardado) {
      localStorage.setItem('cacao_pin', pinIngresado);
      toastStore.success('PIN creado exitosamente. Bienvenido.', 'Seguridad');
      onUnlock();
    } else {
      if (pinIngresado === pinGuardado) {
        toastStore.success('Acceso concedido.', 'Bienvenido');
        onUnlock();
      } else {
        toastStore.error('PIN incorrecto. Inténtalo nuevamente.', 'Acceso Denegado');
        p1 = p2 = p3 = p4 = '';
        document.getElementById('pin-1')?.focus();
      }
    }
  }
</script>

<div class="pin-container">
  <div class="pin-split-card">
    <!-- COLUMNA IZQUIERDA: LOGO + ANIMACIÓN DE RED DE PUNTOS CON OLA DE REPULSIÓN (CAFÉ CHOCOLATE) -->
    <div class="pin-left-panel">
      <canvas bind:this={canvasEl} class="pin-canvas-bg"></canvas>
      
      <div class="pin-left-content">
        <img src={logoImg} alt="Logo Cacao Facturador" class="pin-logo-hero" />
        <h1 class="pin-left-title">Cacao Facturador</h1>
        <p class="pin-left-subtitle">Facturador + Manager de Inventario</p>

        <div class="pin-badge-tag">
          <Icon name="lock" size="0.9em" />
          <span>Acceso Seguro Encriptado</span>
        </div>
      </div>
    </div>

    <!-- COLUMNA DERECHA: FORMULARIO PIN -->
    <div class="pin-right-panel">
      <div class="pin-lock-badge">
        <Icon name="lock" size="1.4em" />
      </div>

      <h2>{!pinGuardado ? 'Bienvenido a Cacao Facturador' : 'Aplicación Bloqueada'}</h2>
      <p>{!pinGuardado ? 'Crea un PIN de 4 dígitos para proteger tu información' : 'Ingresa tu PIN de 4 dígitos para continuar'}</p>
      
      <form onsubmit={submitPin}>
        <div class="pin-input-group">
          <!-- svelte-ignore a11y_autofocus -->
          <input id="pin-1" type="password" maxlength="1" class="pin-digit" bind:value={p1} oninput={(e) => handleInput(e, 'pin-2')} required />
          <input id="pin-2" type="password" maxlength="1" class="pin-digit" bind:value={p2} oninput={(e) => handleInput(e, 'pin-3')} onkeydown={(e) => handleKeyDown(e, 'pin-1')} required />
          <input id="pin-3" type="password" maxlength="1" class="pin-digit" bind:value={p3} oninput={(e) => handleInput(e, 'pin-4')} onkeydown={(e) => handleKeyDown(e, 'pin-2')} required />
          <input id="pin-4" type="password" maxlength="1" class="pin-digit" bind:value={p4} onkeydown={(e) => handleKeyDown(e, 'pin-3')} required />
        </div>

        <button type="submit" class="btn">
          {!pinGuardado ? 'Crear PIN e Ingresar' : 'Desbloquear App'}
        </button>
      </form>
    </div>
  </div>
</div>
