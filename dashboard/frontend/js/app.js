// ==============================================================================
// NOOS TV DASHBOARD - CONTRÔLEUR JS & NAVIGATION MANETTE 10-FOOT
// ==============================================================================

(function() {
  'use strict';

  // Sélecteurs DOM
  const cards = Array.from(document.querySelectorAll('.app-card'));
  const ambientGlow = document.getElementById('ambient-glow');
  const heroBadge = document.getElementById('hero-badge');
  const heroTitle = document.getElementById('hero-title');
  const heroDesc = document.getElementById('hero-description');
  const heroMeta = document.getElementById('hero-meta');
  const btnLaunchHero = document.getElementById('btn-launch-hero');
  const clockDisplay = document.getElementById('clock-display');
  const modalSettings = document.getElementById('modal-settings');
  const btnCloseModal = document.getElementById('modal-close');
  const btnPower = document.getElementById('btn-power');
  const toggleHdrBtn = document.getElementById('toggle-hdr-btn');
  const hdrPill = document.getElementById('hdr-pill');
  const wifiName = document.getElementById('wifi-name');
  const settingsStorageStatus = document.getElementById('settings-storage-status');

  let currentIndex = 0;
  let isModalOpen = false;
  let hdrEnabled = true;

  // 1. Synthétiseur Audio Web Audio API (Sons de navigation feutrés & discrets)
  let audioCtx = null;
  function getAudioContext() {
    if (!audioCtx) {
      const AudioContext = window.AudioContext || window.webkitAudioContext;
      audioCtx = new AudioContext();
    }
    if (audioCtx.state === 'suspended') {
      audioCtx.resume();
    }
    return audioCtx;
  }

  function playTickSound() {
    try {
      const ctx = getAudioContext();
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.type = 'sine';
      osc.frequency.setValueAtTime(440, ctx.currentTime);
      osc.frequency.exponentialRampToValueAtTime(220, ctx.currentTime + 0.04);
      gain.gain.setValueAtTime(0.04, ctx.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.04);
      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.start();
      osc.stop(ctx.currentTime + 0.04);
    } catch (_) {}
  }

  function playConfirmSound() {
    try {
      const ctx = getAudioContext();
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.type = 'sine';
      osc.frequency.setValueAtTime(523.25, ctx.currentTime); // C5
      osc.frequency.exponentialRampToValueAtTime(659.25, ctx.currentTime + 0.1); // E5
      gain.gain.setValueAtTime(0.08, ctx.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.15);
      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.start();
      osc.stop(ctx.currentTime + 0.15);
    } catch (_) {}
  }

  // 2. Horloge temps réel
  function updateClock() {
    const now = new Date();
    const hours = String(now.getHours()).padStart(2, '0');
    const minutes = String(now.getMinutes()).padStart(2, '0');
    clockDisplay.textContent = `${hours}:${minutes}`;
  }
  setInterval(updateClock, 1000);
  updateClock();

  // 3. Mise à jour de la vitrine Hero et de l'ambiance lumineuse
  function updateHero(card) {
    if (!card) return;
    const title = card.getAttribute('data-title') || '';
    const badge = card.getAttribute('data-badge') || '';
    const desc = card.getAttribute('data-desc') || '';
    const meta = card.getAttribute('data-meta') || '';
    const color = card.getAttribute('data-color') || '#38bdf8';

    heroTitle.textContent = title;
    heroBadge.textContent = badge;
    heroDesc.textContent = desc;
    heroMeta.textContent = meta;

    // Dégradé d'ambiance avec couleur réactive feutrée
    ambientGlow.style.background = `radial-gradient(circle, ${color}22 0%, rgba(7, 8, 11, 0) 70%)`;
  }

  // 4. Sélection et focus d'une carte
  function selectCard(index, playAudio = true) {
    if (cards.length === 0) return;
    if (index < 0) index = 0;
    if (index >= cards.length) index = cards.length - 1;

    cards.forEach((c, idx) => {
      if (idx === index) {
        c.classList.add('focused');
        c.focus();
        updateHero(c);
        c.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' });
      } else {
        c.classList.remove('focused');
      }
    });

    currentIndex = index;
    if (playAudio) playTickSound();
  }

  // 5. Lancement d'application
  async function launchApplication(appId) {
    playConfirmSound();

    if (appId === 'settings-modal') {
      openSettings();
      return;
    }

    console.log(`[Noos TV] Lancement de l'application : ${appId}`);

    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        await window.__TAURI__.core.invoke('launch_app', { appId });
      } catch (err) {
        console.error('Erreur au lancement :', err);
      }
    } else {
      console.log(`[Mock] Application lancée : ${appId}`);
    }
  }

  // 6. Gestion du Modal Paramètres
  function openSettings() {
    isModalOpen = true;
    modalSettings.classList.remove('hidden');
    refreshSystemInfo();
  }

  function closeSettings() {
    isModalOpen = false;
    modalSettings.classList.add('hidden');
    selectCard(currentIndex, false);
  }

  // 7. Bascule HDR
  async function toggleHdr() {
    hdrEnabled = !hdrEnabled;
    toggleHdrBtn.classList.toggle('active', hdrEnabled);
    toggleHdrBtn.textContent = hdrEnabled ? 'Activé' : 'Désactivé';
    hdrPill.classList.toggle('hdr-active', hdrEnabled);

    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        await window.__TAURI__.core.invoke('toggle_hdr', { enable: hdrEnabled });
      } catch (err) {
        console.error('Erreur toggle HDR :', err);
      }
    }
  }

  // 8. Actions d'alimentation
  async function handlePowerAction(action) {
    playConfirmSound();
    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        await window.__TAURI__.core.invoke('power_action', { action });
      } catch (err) {
        console.error('Erreur power action :', err);
      }
    } else {
      console.log(`[Mock] Power action : ${action}`);
    }
  }

  // 9. Rafraîchissement des informations système
  async function refreshSystemInfo() {
    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        const info = await window.__TAURI__.core.invoke('get_system_info');
        if (info) {
          if (info.wifi_connected && info.wifi_ssid) {
            wifiName.textContent = info.wifi_ssid;
          }
          if (info.storage_free_gb && info.storage_total_gb) {
            settingsStorageStatus.textContent = `${info.storage_free_gb} Go libres sur ${info.storage_total_gb} Go`;
          }
          hdrEnabled = info.hdr_enabled;
          toggleHdrBtn.classList.toggle('active', hdrEnabled);
          toggleHdrBtn.textContent = hdrEnabled ? 'Activé' : 'Désactivé';
          hdrPill.classList.toggle('hdr-active', hdrEnabled);
        }
      } catch (err) {
        console.warn('Impossible de charger les infos système :', err);
      }
    }
  }

  // 10. Événements DOM (Souris / Clavier)
  cards.forEach((card, index) => {
    card.addEventListener('mouseenter', () => {
      if (!isModalOpen) selectCard(index);
    });
    card.addEventListener('click', () => {
      launchApplication(card.getAttribute('data-id'));
    });
  });

  btnLaunchHero.addEventListener('click', () => {
    const activeCard = cards[currentIndex];
    if (activeCard) launchApplication(activeCard.getAttribute('data-id'));
  });

  btnPower.addEventListener('click', openSettings);
  btnCloseModal.addEventListener('click', closeSettings);
  toggleHdrBtn.addEventListener('click', toggleHdr);

  document.querySelectorAll('.power-action-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      handlePowerAction(btn.getAttribute('data-action'));
    });
  });

  // Clavier physique
  window.addEventListener('keydown', (e) => {
    if (isModalOpen) {
      if (e.key === 'Escape' || e.key === 'Backspace') {
        closeSettings();
      }
      return;
    }

    if (e.key === 'ArrowRight') {
      selectCard(currentIndex + 1);
    } else if (e.key === 'ArrowLeft') {
      selectCard(currentIndex - 1);
    } else if (e.key === 'Enter' || e.key === ' ') {
      const activeCard = cards[currentIndex];
      if (activeCard) launchApplication(activeCard.getAttribute('data-id'));
    } else if (e.key === 'Escape') {
      openSettings();
    }
  });

  // ==============================================================================
  // 11. GESTION DE LA MANETTE DE JEU (GAMEPAD API)
  // ==============================================================================
  let lastNavTime = 0;
  const NAV_COOLDOWN = 180; // ms entre deux pas de navigation
  const prevButtonsState = {};

  function pollGamepad() {
    const gamepads = navigator.getGamepads ? navigator.getGamepads() : [];
    const gp = gamepads[0] || gamepads[1] || gamepads[2] || gamepads[3];

    if (gp) {
      const now = performance.now();

      // Boutons principaux
      const btnA = gp.buttons[0]?.pressed;       // Croix / A (Validation)
      const btnB = gp.buttons[1]?.pressed;       // Rond / B (Retour)
      const btnX = gp.buttons[2]?.pressed;       // Carré / X (Options)
      const btnY = gp.buttons[3]?.pressed;       // Triangle / Y (Alimentation)
      const dpadUp = gp.buttons[12]?.pressed;
      const dpadDown = gp.buttons[13]?.pressed;
      const dpadLeft = gp.buttons[14]?.pressed;
      const dpadRight = gp.buttons[15]?.pressed;

      // Sticks analogiques avec zone morte 0.45
      const axisX = gp.axes[0] || 0;
      const axisY = gp.axes[1] || 0;

      const stickLeft = axisX < -0.45;
      const stickRight = axisX > 0.45;
      const stickUp = axisY < -0.45;
      const stickDown = axisY > 0.45;

      // Navigation Horizontale D-Pad / Stick
      if (now - lastNavTime > NAV_COOLDOWN) {
        if (!isModalOpen) {
          if (dpadRight || stickRight) {
            selectCard(currentIndex + 1);
            lastNavTime = now;
          } else if (dpadLeft || stickLeft) {
            selectCard(currentIndex - 1);
            lastNavTime = now;
          }
        }
      }

      // Action A (Ouvrir) - Déclenchement sur front montant
      if (btnA && !prevButtonsState['A']) {
        if (isModalOpen) {
          // Si dans modal, valide le bouton actif
          const activeEl = document.activeElement;
          if (activeEl && activeEl.click) activeEl.click();
        } else {
          const activeCard = cards[currentIndex];
          if (activeCard) launchApplication(activeCard.getAttribute('data-id'));
        }
      }

      // Action B (Retour / Fermer modal)
      if (btnB && !prevButtonsState['B']) {
        if (isModalOpen) closeSettings();
      }

      // Action X (Paramètres)
      if (btnX && !prevButtonsState['X']) {
        if (!isModalOpen) openSettings(); else closeSettings();
      }

      // Action Y (Alimentation rapide)
      if (btnY && !prevButtonsState['Y']) {
        openSettings();
      }

      // Mémorisation de l'état
      prevButtonsState['A'] = btnA;
      prevButtonsState['B'] = btnB;
      prevButtonsState['X'] = btnX;
      prevButtonsState['Y'] = btnY;
    }

    requestAnimationFrame(pollGamepad);
  }

  // Initialisation au chargement
  window.addEventListener('DOMContentLoaded', () => {
    selectCard(0, false);
    refreshSystemInfo();
    requestAnimationFrame(pollGamepad);
  });

})();
