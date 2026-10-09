// ==============================================================================
// NOOS TV DASHBOARD - CONTRÔLEUR JS & NAVIGATION MANETTE 10-FOOT
// ==============================================================================

(function() {
  'use strict';

  // Sélecteurs DOM Principaux
  const allCards = Array.from(document.querySelectorAll('.app-card'));
  const navItems = Array.from(document.querySelectorAll('.nav-item'));
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

  // Sélecteurs Clavier Virtuel
  const btnLayoutAzerty = document.getElementById('btn-layout-azerty');
  const btnLayoutQwerty = document.getElementById('btn-layout-qwerty');

  // Sélecteurs Moteur d'Upscaling MPV
  const upscaleScreenRes = document.getElementById('upscale-screen-res');
  const upscaleGpuBadge = document.getElementById('upscale-gpu-badge');
  const btnGpuAmd = document.getElementById('btn-gpu-amd');
  const btnGpuNvidia = document.getElementById('btn-gpu-nvidia');
  const btnGpuIntel = document.getElementById('btn-gpu-intel');
  const upscaleProfilesContainer = document.getElementById('upscale-profiles-container');
  let activeUpscaleBrand = null;
  let activeUpscaleProfileId = null;

  // Sélecteurs Modal Mise à Jour
  const modalUpdate = document.getElementById('modal-update');
  const btnCloseUpdateModal = document.getElementById('modal-update-close');
  const updateBadge = document.getElementById('update-badge');
  const updateTileSubtitle = document.getElementById('update-tile-subtitle');
  const btnChannelStable = document.getElementById('btn-channel-stable');
  const btnChannelTesting = document.getElementById('btn-channel-testing');
  const currentVersionTag = document.getElementById('current-version-tag');
  const statusBadge = document.getElementById('status-badge');
  const updateStatusMsg = document.getElementById('update-status-msg');
  const btnCheckUpdates = document.getElementById('btn-check-updates');
  const updateChangelogSection = document.getElementById('update-changelog-section');
  const updateCommitsList = document.getElementById('update-commits-list');
  const btnApplyUpdate = document.getElementById('btn-apply-update');
  const updateProgressContainer = document.getElementById('update-progress-container');
  const progressStepName = document.getElementById('progress-step-name');
  const progressPercent = document.getElementById('progress-percent');
  const progressBarFill = document.getElementById('progress-bar-fill');
  const progressLogLine = document.getElementById('progress-log-line');

  // Sélecteurs Splash Screen Redémarrage
  const restartSplash = document.getElementById('restart-splash');
  const splashStatusText = document.getElementById('splash-status-text');

  let currentIndex = 0;
  let isModalOpen = false;
  let isUpdateModalOpen = false;
  let isUpdating = false;
  let currentChannel = 'testing';
  let hdrEnabled = true;

  function isAnyModalOpen() {
    return isModalOpen || isUpdateModalOpen;
  }

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
    if (clockDisplay) clockDisplay.textContent = `${hours}:${minutes}`;
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

    if (heroTitle) heroTitle.textContent = title;
    if (heroBadge) heroBadge.textContent = badge;
    if (heroDesc) heroDesc.textContent = desc;
    if (heroMeta) heroMeta.textContent = meta;

    if (ambientGlow) {
      ambientGlow.style.background = `radial-gradient(circle, ${color}22 0%, rgba(7, 8, 11, 0) 70%)`;
    }
  }

  // 4. Catégories & Sélection / focus d'une carte
  const CATEGORIES = ['home', 'media', 'games', 'settings'];
  let currentCategory = 'home';

  function getVisibleCards() {
    if (currentCategory === 'home') {
      return allCards;
    }
    return allCards.filter(c => c.getAttribute('data-category') === currentCategory);
  }

  function getActiveCard() {
    const visible = getVisibleCards();
    return visible[currentIndex] || visible[0];
  }

  function selectCard(index, playAudio = true) {
    const visibleCards = getVisibleCards();
    if (visibleCards.length === 0) return;
    if (index < 0) index = 0;
    if (index >= visibleCards.length) index = visibleCards.length - 1;

    allCards.forEach(c => c.classList.remove('focused'));

    const targetCard = visibleCards[index];
    if (targetCard) {
      targetCard.classList.add('focused');
      targetCard.focus();
      updateHero(targetCard);
      targetCard.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' });
    }

    currentIndex = index;
    if (playAudio) playTickSound();
  }

  function setCategory(category, playAudio = true) {
    if (!CATEGORIES.includes(category)) return;
    currentCategory = category;

    navItems.forEach(item => {
      if (item.getAttribute('data-category') === category) {
        item.classList.add('active');
      } else {
        item.classList.remove('active');
      }
    });

    allCards.forEach(card => {
      const cardCat = card.getAttribute('data-category');
      if (currentCategory === 'home' || cardCat === currentCategory) {
        card.classList.remove('category-hidden');
      } else {
        card.classList.remove('focused');
        card.classList.add('category-hidden');
      }
    });

    selectCard(0, playAudio);
  }

  function cycleCategory(direction) {
    let idx = CATEGORIES.indexOf(currentCategory);
    if (idx === -1) idx = 0;
    idx = (idx + direction + CATEGORIES.length) % CATEGORIES.length;
    setCategory(CATEGORIES[idx], true);
  }

  // 5. Lancement d'application
  async function launchApplication(appId) {
    playConfirmSound();

    if (appId === 'settings-modal') {
      openSettings();
      return;
    }

    if (appId === 'update-modal') {
      openUpdateModal();
      return;
    }

    if (appId === 'disc_player') {
      console.log('[Noos TV] Lancement de la lecture DVD / Blu-ray');
      if (window.__TAURI__ && window.__TAURI__.core) {
        try {
          await window.__TAURI__.core.invoke('play_disc', { device: null });
        } catch (err) {
          console.error('Erreur lecture disque :', err);
        }
      }
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
    if (isUpdateModalOpen) closeUpdateModal();
    isModalOpen = true;
    modalSettings.classList.remove('hidden');
    refreshSystemInfo();
    initKeyboardConfig();
    loadUpscaleProfiles();
  }

  function closeSettings() {
    isModalOpen = false;
    modalSettings.classList.add('hidden');
    selectCard(currentIndex, false);
  }

  // 7. Gestion du Modal Mise à Jour
  function openUpdateModal() {
    if (isModalOpen) closeSettings();
    isUpdateModalOpen = true;
    modalUpdate.classList.remove('hidden');
    checkForUpdates(currentChannel);
  }

  function closeUpdateModal() {
    if (isUpdating) return;
    isUpdateModalOpen = false;
    modalUpdate.classList.add('hidden');
    selectCard(currentIndex, false);
  }

  function setChannel(channel) {
    currentChannel = channel;
    if (channel === 'stable') {
      btnChannelStable.classList.add('active');
      btnChannelTesting.classList.remove('active');
    } else {
      btnChannelTesting.classList.add('active');
      btnChannelStable.classList.remove('active');
    }
    checkForUpdates(currentChannel);
  }

  async function checkForUpdates(channel) {
    if (btnCheckUpdates) btnCheckUpdates.disabled = true;
    if (updateStatusMsg) updateStatusMsg.textContent = `Recherche de mises à jour sur le canal '${channel}'...`;
    if (statusBadge) {
      statusBadge.textContent = 'Vérification...';
      statusBadge.className = 'badge-pill';
    }

    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        const info = await window.__TAURI__.core.invoke('check_for_updates', { channel });
        if (info) {
          if (currentVersionTag) currentVersionTag.textContent = info.latest_version;
          if (updateStatusMsg) updateStatusMsg.textContent = info.message;

          if (info.has_update) {
            if (statusBadge) {
              statusBadge.textContent = 'Mise à jour disponible';
              statusBadge.className = 'badge-pill hdr-active';
            }
            if (updateBadge) updateBadge.style.display = 'block';
            if (updateTileSubtitle) updateTileSubtitle.textContent = 'MàJ disponible !';

            if (updateCommitsList) {
              updateCommitsList.innerHTML = '';
              info.commits.forEach(c => {
                const item = document.createElement('div');
                item.className = 'commit-item';
                const parts = c.split(' • ');
                const hash = parts[0] || '';
                const msg = parts[1] || c;
                item.innerHTML = `<span class="commit-hash">${hash}</span><span class="commit-msg">${msg}</span>`;
                updateCommitsList.appendChild(item);
              });
            }
            if (updateChangelogSection) updateChangelogSection.classList.remove('hidden');
            if (btnApplyUpdate) btnApplyUpdate.disabled = false;
          } else {
            if (statusBadge) {
              statusBadge.textContent = 'À jour';
              statusBadge.className = 'badge-pill';
            }
            if (updateBadge) updateBadge.style.display = 'none';
            if (updateTileSubtitle) updateTileSubtitle.textContent = 'Système à jour';
            if (updateChangelogSection) updateChangelogSection.classList.add('hidden');
          }
        }
      } catch (err) {
        console.error('Erreur check updates :', err);
        if (updateStatusMsg) updateStatusMsg.textContent = 'Erreur vérification : ' + err;
        if (statusBadge) statusBadge.textContent = 'Erreur';
      } finally {
        if (btnCheckUpdates) btnCheckUpdates.disabled = false;
      }
    } else {
      setTimeout(() => {
        if (statusBadge) statusBadge.textContent = 'À jour';
        if (updateStatusMsg) updateStatusMsg.textContent = `Votre système Noos HTPC est à jour sur le canal '${channel}'.`;
        if (btnCheckUpdates) btnCheckUpdates.disabled = false;
      }, 500);
    }
  }

  async function applyUpdate() {
    if (isUpdating) return;
    isUpdating = true;
    playConfirmSound();

    if (btnApplyUpdate) btnApplyUpdate.disabled = true;
    if (btnCheckUpdates) btnCheckUpdates.disabled = true;
    if (btnChannelStable) btnChannelStable.disabled = true;
    if (btnChannelTesting) btnChannelTesting.disabled = true;
    if (updateProgressContainer) updateProgressContainer.classList.remove('hidden');

    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        await window.__TAURI__.core.invoke('apply_system_update', { channel: currentChannel });
      } catch (err) {
        console.error('Erreur application mise à jour :', err);
        if (progressStepName) progressStepName.textContent = 'Erreur lors de la mise à jour';
        if (progressLogLine) progressLogLine.textContent = String(err);
        isUpdating = false;
        if (btnApplyUpdate) btnApplyUpdate.disabled = false;
        if (btnCheckUpdates) btnCheckUpdates.disabled = false;
        if (btnChannelStable) btnChannelStable.disabled = false;
        if (btnChannelTesting) btnChannelTesting.disabled = false;
      }
    }
  }

  // 8. Clavier Virtuel : Disposition AZERTY / QWERTY
  async function initKeyboardConfig() {
    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        const cfg = await window.__TAURI__.core.invoke('get_keyboard_config');
        if (cfg && cfg.layout) {
          setKeyboardLayoutUI(cfg.layout);
        }
      } catch (err) {
        console.warn('Erreur chargement disposition clavier :', err);
      }
    }
  }

  function setKeyboardLayoutUI(layout) {
    const isAzerty = layout.toLowerCase() === 'azerty';
    if (btnLayoutAzerty) btnLayoutAzerty.classList.toggle('active', isAzerty);
    if (btnLayoutQwerty) btnLayoutQwerty.classList.toggle('active', !isAzerty);
  }

  async function setKeyboardLayout(layout) {
    setKeyboardLayoutUI(layout);
    playConfirmSound();
    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        await window.__TAURI__.core.invoke('set_keyboard_config', { config: { layout } });
      } catch (err) {
        console.error('Erreur configuration disposition clavier :', err);
      }
    }
  }

  // 8c. Gestion des profils d'upscaling MPV (AMD, NVIDIA, Intel)
  async function loadUpscaleProfiles(requestedBrand = null) {
    if (!upscaleProfilesContainer) return;

    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        const info = await window.__TAURI__.core.invoke('get_upscale_info', { brand: requestedBrand });
        if (info) {
          activeUpscaleBrand = info.active_brand;
          activeUpscaleProfileId = info.current_profile_id;

          if (upscaleScreenRes) {
            upscaleScreenRes.textContent = `Écran : ${info.detected_max_res}`;
          }
          if (upscaleGpuBadge) {
            const brandNames = { amd: 'AMD Radeon', nvidia: 'NVIDIA GeForce', intel: 'Intel Arc' };
            upscaleGpuBadge.textContent = `GPU : ${brandNames[info.detected_brand] || info.detected_brand.toUpperCase()}`;
          }

          // Mise à jour de la pilule de marque active
          if (btnGpuAmd) btnGpuAmd.classList.toggle('active', info.active_brand === 'amd');
          if (btnGpuNvidia) btnGpuNvidia.classList.toggle('active', info.active_brand === 'nvidia');
          if (btnGpuIntel) btnGpuIntel.classList.toggle('active', info.active_brand === 'intel');

          // Rendu des 3 cartes de profils
          renderUpscaleProfiles(info.profiles, info.current_profile_id);
        }
      } catch (err) {
        console.error('Erreur chargement profils d\'upscale :', err);
      }
    } else {
      // Mock data pour tests hors Tauri
      const mockProfiles = [
        {
          id: 'amd_simple',
          name: 'FidelityFX CAS',
          brand: 'amd',
          level: 'simple',
          level_label: 'Simple',
          tech_tag: 'FidelityFX CAS',
          description: 'Accentuation adaptative des contrastes AMD FidelityFX CAS. Traitement ultraléger.',
          scale_method: 'spline36',
          max_res_target: '3840×2160 (4K UHD)'
        },
        {
          id: 'amd_moyen',
          name: 'AMD FSR Équilibré',
          brand: 'amd',
          level: 'moyen',
          level_label: 'Moyen',
          tech_tag: 'FSR (EASU + RCAS)',
          description: 'Super-résolution spatiale AMD FidelityFX Super Resolution (FSR).',
          scale_method: 'ewa_lanczossharp',
          max_res_target: '3840×2160 (4K UHD)'
        },
        {
          id: 'amd_eleve',
          name: 'AMD FSR Ultra Neuronal',
          brand: 'amd',
          level: 'eleve',
          level_label: 'Élevé',
          tech_tag: 'FSRCNNX 16 + FSR Ultra',
          description: 'Réseau de neurones convolutifs FSRCNNX 16 passes couplé au shader FSR.',
          scale_method: 'ewa_lanczossharp',
          max_res_target: '3840×2160 (4K UHD)'
        }
      ];
      renderUpscaleProfiles(mockProfiles, 'amd_moyen');
    }
  }

  function renderUpscaleProfiles(profiles, currentProfileId) {
    if (!upscaleProfilesContainer) return;
    upscaleProfilesContainer.innerHTML = '';

    profiles.forEach(p => {
      const card = document.createElement('div');
      const isActive = p.id === currentProfileId;
      card.className = `upscale-card ${isActive ? 'active-profile' : ''}`;
      card.setAttribute('data-profile-id', p.id);
      card.setAttribute('tabindex', '0');

      card.innerHTML = `
        <div class="upscale-card-header">
          <span class="upscale-level-badge level-${p.level}">${p.level_label}</span>
          ${isActive ? '<span class="upscale-status-indicator"><span class="indicator-dot"></span>ACTIF</span>' : ''}
        </div>
        <div class="upscale-card-title">${p.name}</div>
        <div class="upscale-card-tag">${p.tech_tag}</div>
        <div class="upscale-card-desc">${p.description}</div>
        <div class="upscale-card-footer">
          <span>Cible : ${p.max_res_target}</span>
          <span>${p.scale_method}</span>
        </div>
      `;

      card.addEventListener('click', () => {
        applyUpscaleProfile(p.id);
      });

      card.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          applyUpscaleProfile(p.id);
        }
      });

      upscaleProfilesContainer.appendChild(card);
    });
  }

  async function applyUpscaleProfile(profileId) {
    playConfirmSound();
    activeUpscaleProfileId = profileId;

    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        await window.__TAURI__.core.invoke('set_upscale_profile', { profileId });
        await loadUpscaleProfiles(activeUpscaleBrand);
      } catch (err) {
        console.error('Erreur application profil d\'upscale :', err);
      }
    } else {
      // Mock update
      document.querySelectorAll('.upscale-card').forEach(c => {
        const isTarget = c.getAttribute('data-profile-id') === profileId;
        c.classList.toggle('active-profile', isTarget);
        const header = c.querySelector('.upscale-card-header');
        if (header) {
          const oldInd = header.querySelector('.upscale-status-indicator');
          if (oldInd) oldInd.remove();
          if (isTarget) {
            header.insertAdjacentHTML('beforeend', '<span class="upscale-status-indicator"><span class="indicator-dot"></span>ACTIF</span>');
          }
        }
      });
    }
  }

  // 9. Bascule HDR
  async function toggleHdr() {
    hdrEnabled = !hdrEnabled;
    applyHdrState(hdrEnabled);

    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        await window.__TAURI__.core.invoke('toggle_hdr', { enable: hdrEnabled });
      } catch (err) {
        console.error('Erreur toggle HDR :', err);
      }
    }
  }

  function applyHdrState(enabled) {
    if (toggleHdrBtn) {
      toggleHdrBtn.classList.toggle('active', enabled);
      toggleHdrBtn.textContent = enabled ? 'Activé (Rec.2020)' : 'Désactivé (SDR)';
    }
    if (hdrPill) {
      if (enabled) {
        hdrPill.classList.add('hdr-active');
        hdrPill.classList.remove('sdr-active');
        hdrPill.innerHTML = '<span class="status-dot"></span><span>4K HDR</span>';
      } else {
        hdrPill.classList.remove('hdr-active');
        hdrPill.classList.add('sdr-active');
        hdrPill.innerHTML = '<span class="status-dot sdr"></span><span>SDR • Tonemap Auto</span>';
      }
    }
  }

  // 10. Actions d'alimentation
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

  // 11. Rafraîchissement des informations système
  async function refreshSystemInfo() {
    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        const info = await window.__TAURI__.core.invoke('get_system_info');
        if (info) {
          if (info.wifi_connected && info.wifi_ssid && wifiName) {
            wifiName.textContent = info.wifi_ssid;
          }
          if (info.storage_free_gb && info.storage_total_gb && settingsStorageStatus) {
            settingsStorageStatus.textContent = `${info.storage_free_gb} Go libres sur ${info.storage_total_gb} Go`;
          }
          hdrEnabled = info.hdr_enabled;
          applyHdrState(hdrEnabled);
        }
      } catch (err) {
        console.warn('Impossible de charger les infos système :', err);
      }
    }
  }

  // 12. Surveillance et détection en direct des lecteurs DVD/Blu-ray (USB & SATA)
  let currentOpticalDrive = null;
  const cardDiscPlayer = document.getElementById('card-disc-player');
  const discCardBadge = document.getElementById('disc-card-badge');
  const discCardTitle = document.getElementById('disc-card-title');
  const discCardSubtitle = document.getElementById('disc-card-subtitle');

  async function checkOpticalDrive() {
    if (window.__TAURI__ && window.__TAURI__.core && cardDiscPlayer) {
      try {
        const drive = await window.__TAURI__.core.invoke('get_optical_drive');
        currentOpticalDrive = drive;

        if (!drive) {
          cardDiscPlayer.classList.remove('has-disc');
          if (discCardBadge) discCardBadge.textContent = 'OPTIQUE';
          if (discCardTitle) discCardTitle.textContent = 'Lecteur Disque';
          if (discCardSubtitle) discCardSubtitle.textContent = 'Aucun lecteur connecté';
          cardDiscPlayer.setAttribute('data-title', 'Lecteur DVD / Blu-ray');
          cardDiscPlayer.setAttribute('data-badge', 'DVD • BLU-RAY (NON DÉTECTÉ)');
          cardDiscPlayer.setAttribute('data-desc', 'Connectez un lecteur DVD ou Blu-ray en USB ou SATA pour lire vos disques physiques en direct.');
          cardDiscPlayer.setAttribute('data-meta', 'Détection USB & SATA');
        } else if (!drive.disc_inserted) {
          cardDiscPlayer.classList.remove('has-disc');
          if (discCardBadge) discCardBadge.textContent = `${drive.transport} • PRÊT`;
          if (discCardTitle) discCardTitle.textContent = 'Lecteur ' + drive.transport;
          if (discCardSubtitle) discCardSubtitle.textContent = 'Tiroir vide • Insérez un disque';
          cardDiscPlayer.setAttribute('data-title', `Lecteur ${drive.transport} (${drive.name})`);
          cardDiscPlayer.setAttribute('data-badge', `LECTEUR ${drive.transport} CONNECTÉ`);
          cardDiscPlayer.setAttribute('data-desc', `Lecteur optique ${drive.name} détecté en ${drive.transport}. Insérez un film DVD ou Blu-ray pour démarrer.`);
          cardDiscPlayer.setAttribute('data-meta', `Interface ${drive.transport} • Tiroir Vide`);
        } else {
          cardDiscPlayer.classList.add('has-disc');
          if (discCardBadge) discCardBadge.textContent = `${drive.disc_type.toUpperCase()} (${drive.transport})`;
          if (discCardTitle) discCardTitle.textContent = drive.disc_label || drive.disc_type;
          if (discCardSubtitle) discCardSubtitle.textContent = 'Film prêt • Appuyez sur [A]';
          cardDiscPlayer.setAttribute('data-title', drive.disc_label || "Film " + drive.disc_type);
          cardDiscPlayer.setAttribute('data-badge', `${drive.disc_type.toUpperCase()} • ${drive.transport}`);
          cardDiscPlayer.setAttribute('data-desc', `Disque ${drive.disc_type} « ${drive.disc_label} » prêt dans le lecteur ${drive.transport}. Lecture cinéma directe avec décodage matériel.`);
          cardDiscPlayer.setAttribute('data-meta', `Lecteur ${drive.transport} • Menu & Chapitres`);
        }

        if (getActiveCard() === cardDiscPlayer) {
          updateHero(cardDiscPlayer);
        }
      } catch (err) {
        console.warn('Erreur vérification lecteur optique :', err);
      }
    }
  }

  setInterval(checkOpticalDrive, 2500);
  checkOpticalDrive();

  // 13. Événements DOM (Souris / Clavier)
  allCards.forEach(card => {
    card.addEventListener('mouseenter', () => {
      if (!isAnyModalOpen()) {
        const visible = getVisibleCards();
        const idx = visible.indexOf(card);
        if (idx !== -1) selectCard(idx);
      }
    });
    card.addEventListener('click', () => {
      launchApplication(card.getAttribute('data-id'));
    });
  });

  navItems.forEach(btn => {
    btn.addEventListener('click', () => {
      setCategory(btn.getAttribute('data-category'));
    });
  });

  if (btnLaunchHero) {
    btnLaunchHero.addEventListener('click', () => {
      const activeCard = getActiveCard();
      if (activeCard) launchApplication(activeCard.getAttribute('data-id'));
    });
  }

  if (btnPower) btnPower.addEventListener('click', openSettings);
  if (btnCloseModal) btnCloseModal.addEventListener('click', closeSettings);
  if (btnCloseUpdateModal) btnCloseUpdateModal.addEventListener('click', closeUpdateModal);
  if (toggleHdrBtn) toggleHdrBtn.addEventListener('click', toggleHdr);

  if (btnChannelStable) btnChannelStable.addEventListener('click', () => setChannel('stable'));
  if (btnChannelTesting) btnChannelTesting.addEventListener('click', () => setChannel('testing'));
  if (btnCheckUpdates) btnCheckUpdates.addEventListener('click', () => checkForUpdates(currentChannel));
  if (btnApplyUpdate) btnApplyUpdate.addEventListener('click', applyUpdate);

  if (btnLayoutAzerty) btnLayoutAzerty.addEventListener('click', () => setKeyboardLayout('azerty'));
  if (btnLayoutQwerty) btnLayoutQwerty.addEventListener('click', () => setKeyboardLayout('qwerty'));

  if (btnGpuAmd) btnGpuAmd.addEventListener('click', () => loadUpscaleProfiles('amd'));
  if (btnGpuNvidia) btnGpuNvidia.addEventListener('click', () => loadUpscaleProfiles('nvidia'));
  if (btnGpuIntel) btnGpuIntel.addEventListener('click', () => loadUpscaleProfiles('intel'));

  document.querySelectorAll('.power-action-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      handlePowerAction(btn.getAttribute('data-action'));
    });
  });

  // Clavier physique
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Home') {
      if (isUpdateModalOpen) closeUpdateModal();
      if (isModalOpen) closeSettings();
      selectCard(0);
      return;
    }

    if (isUpdateModalOpen) {
      if (e.key === 'Escape' || e.key === 'Backspace') {
        closeUpdateModal();
      } else if (e.key === 'Enter' || e.key === ' ') {
        if (!isUpdating && btnApplyUpdate && !btnApplyUpdate.disabled && !updateChangelogSection.classList.contains('hidden')) {
          applyUpdate();
        }
      }
      return;
    }

    if (isModalOpen) {
      if (e.key === 'Escape' || e.key === 'Backspace') {
        closeSettings();
      }
      return;
    }

    if (e.key === 'PageUp' || e.key === 'q' || e.key === 'Q') {
      cycleCategory(-1);
      return;
    } else if (e.key === 'PageDown' || e.key === 'e' || e.key === 'E') {
      cycleCategory(1);
      return;
    }

    if (e.key === 'ArrowRight') {
      selectCard(currentIndex + 1);
    } else if (e.key === 'ArrowLeft') {
      selectCard(currentIndex - 1);
    } else if (e.key === 'Enter' || e.key === ' ') {
      const activeCard = getActiveCard();
      if (activeCard) launchApplication(activeCard.getAttribute('data-id'));
    } else if (e.key === 'Escape') {
      openSettings();
    }
  });

  // ==============================================================================
  // 14. GESTION DE LA MANETTE DE JEU (GAMEPAD API)
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
      const btnLB = gp.buttons[4]?.pressed;      // Bumper Gauche (LB / L1) -> Onglet précédent
      const btnRB = gp.buttons[5]?.pressed;      // Bumper Droit (RB / R1) -> Onglet suivant
      const btnHome = gp.buttons[16]?.pressed;   // Guide / Xbox / PS / Home (Bouton HOME)

      // Sticks analogiques avec zone morte 0.45
      const axisX = gp.axes[0] || 0;
      const dpadLeft = gp.buttons[14]?.pressed;
      const dpadRight = gp.buttons[15]?.pressed;

      const stickLeft = axisX < -0.45;
      const stickRight = axisX > 0.45;

      // Navigation Horizontale D-Pad / Stick
      if (now - lastNavTime > NAV_COOLDOWN) {
        if (!isAnyModalOpen()) {
          if (dpadRight || stickRight) {
            selectCard(currentIndex + 1);
            lastNavTime = now;
          } else if (dpadLeft || stickLeft) {
            selectCard(currentIndex - 1);
            lastNavTime = now;
          }
        }
      }

      // Bumpers LB / RB pour navigation entre onglets
      if (btnLB && !prevButtonsState['LB']) {
        if (!isAnyModalOpen()) cycleCategory(-1);
      }
      if (btnRB && !prevButtonsState['RB']) {
        if (!isAnyModalOpen()) cycleCategory(1);
      }

      // Action HOME (Retour direct au lanceur / Accueil TV)
      if (btnHome && !prevButtonsState['Home']) {
        playConfirmSound();
        if (isUpdateModalOpen) closeUpdateModal();
        if (isModalOpen) closeSettings();
        setCategory('home', false);
        selectCard(0);
      }

      // Action A (Ouvrir / Valider)
      if (btnA && !prevButtonsState['A']) {
        if (isUpdateModalOpen) {
          if (!isUpdating && btnApplyUpdate && !btnApplyUpdate.disabled && !updateChangelogSection.classList.contains('hidden')) {
            applyUpdate();
          }
        } else if (isModalOpen) {
          const activeEl = document.activeElement;
          if (activeEl && activeEl.click) activeEl.click();
        } else {
          const activeCard = getActiveCard();
          if (activeCard) launchApplication(activeCard.getAttribute('data-id'));
        }
      }

      // Action B (Retour / Fermer modal)
      if (btnB && !prevButtonsState['B']) {
        if (isUpdateModalOpen) closeUpdateModal();
        else if (isModalOpen) closeSettings();
      }

      // Action X (Paramètres ou Vérifier MàJ si modal ouvert)
      if (btnX && !prevButtonsState['X']) {
        if (isUpdateModalOpen) {
          checkForUpdates(currentChannel);
        } else if (!isModalOpen) {
          openSettings();
        } else {
          closeSettings();
        }
      }

      // Action Y (Éjection si sur le lecteur disque, sinon alimentation)
      if (btnY && !prevButtonsState['Y']) {
        if (!isAnyModalOpen() && getActiveCard() === cardDiscPlayer) {
          playConfirmSound();
          if (window.__TAURI__ && window.__TAURI__.core) {
            window.__TAURI__.core.invoke('eject_disc', { device: null });
          }
        } else if (!isAnyModalOpen()) {
          openSettings();
        }
      }

      // Mémorisation de l'état
      prevButtonsState['A'] = btnA;
      prevButtonsState['B'] = btnB;
      prevButtonsState['X'] = btnX;
      prevButtonsState['Y'] = btnY;
      prevButtonsState['LB'] = btnLB;
      prevButtonsState['RB'] = btnRB;
      prevButtonsState['Home'] = btnHome;
    }

    requestAnimationFrame(pollGamepad);
  }

  // 15. Initialisation au chargement & écoute des événements système Tauri
  window.addEventListener('DOMContentLoaded', () => {
    selectCard(0, false);
    refreshSystemInfo();
    initKeyboardConfig();
    requestAnimationFrame(pollGamepad);

    if (window.__TAURI__ && window.__TAURI__.event) {
      // 1. Bouton HOME global
      window.__TAURI__.event.listen('home_pressed', () => {
        console.log('[Noos TV] Interruption globale reçue : retour au lanceur');
        playConfirmSound();
        if (isUpdateModalOpen) closeUpdateModal();
        if (isModalOpen) closeSettings();
        selectCard(0);
      });

      // 2. Progression de la mise à jour
      window.__TAURI__.event.listen('update_progress', (e) => {
        const p = e.payload;
        if (p) {
          if (progressStepName) progressStepName.textContent = `Étape ${p.step}/${p.total_steps} : ${p.step_name}`;
          if (progressPercent) progressPercent.textContent = `${p.percent}%`;
          if (progressBarFill) progressBarFill.style.width = `${p.percent}%`;
          if (progressLogLine) progressLogLine.textContent = p.log_line;
        }
      });

      // 3. Mise à jour terminée : Affichage du Splash Screen Stylisé (Logo Infini + Blur) puis Redémarrage
      window.__TAURI__.event.listen('update_completed', async () => {
        console.log('[Noos TV] Mise à jour terminée avec succès !');
        if (restartSplash) {
          restartSplash.classList.remove('hidden');
        }
        if (splashStatusText) {
          splashStatusText.textContent = 'Mise à jour appliquée avec succès !';
        }
        setTimeout(async () => {
          try {
            await window.__TAURI__.core.invoke('restart_dashboard');
          } catch (err) {
            console.error('Erreur relance dashboard :', err);
          }
        }, 2600);
      });

      // 4. Pastille rouge dynamique sur la tuile Mise à jour
      window.__TAURI__.event.listen('update_badge_status', (e) => {
        const hasUpdate = !!e.payload;
        if (updateBadge) {
          updateBadge.style.display = hasUpdate ? 'block' : 'none';
        }
        if (updateTileSubtitle) {
          updateTileSubtitle.textContent = hasUpdate ? 'MàJ disponible !' : 'Système à jour';
        }
      });
    }
  });

})();
