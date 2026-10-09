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

  // Sélecteurs Noos IPTV
  const modalIptv = document.getElementById('modal-iptv');
  const btnCloseIptvModal = document.getElementById('btn-close-iptv-modal');
  const iptvViewProfiles = document.getElementById('iptv-view-profiles');
  const iptvViewLogin = document.getElementById('iptv-view-login');
  const iptvViewSync = document.getElementById('iptv-view-sync');
  const iptvProfilesGrid = document.getElementById('iptv-profiles-grid');
  const btnIptvNewAccount = document.getElementById('btn-iptv-new-account');
  const iptvLoginForm = document.getElementById('iptv-login-form');
  const iptvInputName = document.getElementById('iptv-input-name');
  const iptvInputUrl = document.getElementById('iptv-input-url');
  const iptvInputUser = document.getElementById('iptv-input-user');
  const iptvInputPass = document.getElementById('iptv-input-pass');
  const iptvCheckSave = document.getElementById('iptv-check-save');
  const iptvLoginError = document.getElementById('iptv-login-error');
  const btnIptvBackProfiles = document.getElementById('btn-iptv-back-profiles');
  const btnIptvSubmitLogin = document.getElementById('btn-iptv-submit-login');
  const iptvSyncTitle = document.getElementById('iptv-sync-title');
  const iptvSyncStepName = document.getElementById('iptv-sync-step-name');
  const iptvSyncProgressFill = document.getElementById('iptv-sync-progress-fill');
  const iptvSyncDetails = document.getElementById('iptv-sync-details');
  const iptvSyncStats = document.getElementById('iptv-sync-stats');
  const iptvStatChannels = document.getElementById('iptv-stat-channels');
  const iptvStatMovies = document.getElementById('iptv-stat-movies');
  const iptvStatSeries = document.getElementById('iptv-stat-series');
  const iptvSyncSuccessActions = document.getElementById('iptv-sync-success-actions');
  const btnIptvOpenCatalog = document.getElementById('btn-iptv-open-catalog');

  // Sélecteurs Interface Principale Noos IPTV MPV Player
  const iptvContainer = document.querySelector('.iptv-fullscreen-container');
  const iptvViewMain = document.getElementById('iptv-view-main');
  const btnIptvMainBack = document.getElementById('btn-iptv-main-back');
  const iptvTabButtons = document.querySelectorAll('.iptv-tab-btn');
  const iptvCategoriesList = document.getElementById('iptv-categories-list');
  const iptvCatCount = document.getElementById('iptv-cat-count');

  // Sous-vues
  const iptvSubviewLive = document.getElementById('iptv-subview-live');
  const iptvSubviewVod = document.getElementById('iptv-subview-vod');
  const iptvSubviewSeries = document.getElementById('iptv-subview-series');
  const iptvSubviewFavorites = document.getElementById('iptv-subview-favorites');
  const iptvSubviewFilters = document.getElementById('iptv-subview-filters');
  const iptvSubviewSettings = document.getElementById('iptv-subview-settings');

  // Éléments Live
  const iptvCurrentCatName = document.getElementById('iptv-current-cat-name');
  const iptvChannelsCount = document.getElementById('iptv-channels-count');
  const iptvChannelsList = document.getElementById('iptv-channels-list');
  const previewChannelLogo = document.getElementById('preview-channel-logo');
  const previewChannelName = document.getElementById('preview-channel-name');
  const previewChannelCategory = document.getElementById('preview-channel-category');
  const btnIptvPlayFullscreen = document.getElementById('btn-iptv-play-fullscreen');
  const epgNowTime = document.getElementById('epg-now-time');
  const epgNowTitle = document.getElementById('epg-now-title');
  const epgNowDesc = document.getElementById('epg-now-desc');
  const iptvEpgUpcomingList = document.getElementById('iptv-epg-upcoming-list');

  // Éléments VOD & Séries & Favoris
  const iptvVodGrid = document.getElementById('iptv-vod-grid');
  const vodCatTitle = document.getElementById('vod-cat-title');
  const vodItemsCount = document.getElementById('vod-items-count');
  const iptvSeriesGrid = document.getElementById('iptv-series-grid');
  const seriesCatTitle = document.getElementById('series-cat-title');
  const seriesItemsCount = document.getElementById('series-items-count');
  const iptvFavoritesGrid = document.getElementById('iptv-favorites-grid');
  const favoritesCount = document.getElementById('favorites-count');

  // Éléments Filtres
  const filtersListLive = document.getElementById('filters-list-live');
  const filtersListVod = document.getElementById('filters-list-vod');
  const filtersListSeries = document.getElementById('filters-list-series');
  const btnFiltersUnhideAll = document.getElementById('btn-filters-unhide-all');
  const btnFiltersSave = document.getElementById('btn-filters-save');

  // Éléments Paramètres Shaders & MPV
  const iptvSelectUpscale = document.getElementById('iptv-select-upscale');
  const iptvToggleDeband = document.getElementById('iptv-toggle-deband');
  const iptvToggleInterpolation = document.getElementById('iptv-toggle-interpolation');
  const iptvSelectBuffer = document.getElementById('iptv-select-buffer');
  const btnSaveIptvSettings = document.getElementById('btn-save-iptv-settings');
  const iptvSettingsSavedFeedback = document.getElementById('iptv-settings-saved-feedback');

  // Modal Détails Film / Série
  const modalIptvDetails = document.getElementById('modal-iptv-details');
  const btnCloseDetailsModal = document.getElementById('btn-close-details-modal');
  const detailsPosterImg = document.getElementById('details-poster-img');
  const detailsBackdrop = document.getElementById('details-backdrop');
  const detailsTitle = document.getElementById('details-title');
  const detailsRating = document.getElementById('details-rating');
  const detailsYear = document.getElementById('details-year');
  const detailsGenre = document.getElementById('details-genre');
  const detailsDuration = document.getElementById('details-duration');
  const detailsPlot = document.getElementById('details-plot');
  const btnDetailsPlayMain = document.getElementById('btn-details-play-main');
  const detailsPlayLabel = document.getElementById('details-play-label');
  const btnDetailsToggleFav = document.getElementById('btn-details-toggle-fav');
  const detailsFavStar = document.getElementById('details-fav-star');
  const detailsFavText = document.getElementById('details-fav-text');
  const detailsSeriesSection = document.getElementById('details-series-section');
  const detailsSeasonsPills = document.getElementById('details-seasons-pills');
  const detailsEpisodesList = document.getElementById('details-episodes-list');

  // Variables d'état IPTV
  let currentIptvProfileId = null;
  let currentIptvTab = 'live';
  let currentIptvCategory = null;
  let currentLiveStream = null;
  let currentDetailsItem = null;
  let currentDetailsType = 'movie';
  let iptvCatalogCache = {
    live: null,
    vod: null,
    series: null,
  };
  let iptvFavorites = [];
  let iptvHiddenCategories = new Set();

  let currentIndex = 0;
  let isModalOpen = false;
  let isUpdateModalOpen = false;
  let isUpdating = false;
  let isIptvModalOpen = false;
  let isIptvSyncing = false;
  let savedIptvProfiles = [];
  let currentChannel = 'testing';
  let hdrEnabled = true;

  function isAnyModalOpen() {
    return isModalOpen || isUpdateModalOpen || isIptvModalOpen || (modalIptvDetails && !modalIptvDetails.classList.contains('hidden'));
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

    if (appId === 'update-modal' || appId === 'update') {
      openUpdateModal();
      return;
    }

    if (appId === 'noos-iptv') {
      openIptvModal();
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
          if (currentVersionTag) {
            currentVersionTag.textContent = info.has_update
              ? `${info.current_version} ➔ ${info.latest_version}`
              : info.current_version;
          }
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

  // 10. GESTION NOOS IPTV (PROFILES CHIFFRÉS, LOGIN XTREAM & CACHE SYNC)
  function escapeHtml(str) {
    if (!str) return '';
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  async function openIptvModal() {
    if (isModalOpen) closeSettings();
    if (isUpdateModalOpen) closeUpdateModal();
    isIptvModalOpen = true;
    if (window.NOOS_LOGO_DATA_URI) {
      document.querySelectorAll('.iptv-logo, .iptv-sync-logo').forEach(img => {
        img.src = window.NOOS_LOGO_DATA_URI;
      });
    }
    if (modalIptv) modalIptv.classList.remove('hidden');
    await loadIptvProfiles();
  }

  function closeIptvModal() {
    if (isIptvSyncing) return;
    if (modalIptvDetails && !modalIptvDetails.classList.contains('hidden')) {
      closeIptvDetailsModal();
      return;
    }
    isIptvModalOpen = false;
    if (window.__TAURI__ && window.__TAURI__.core) {
      window.__TAURI__.core.invoke('hide_virtual_keyboard').catch(() => {});
    }
    if (iptvContainer) iptvContainer.classList.remove('catalog-mode');
    if (modalIptv) modalIptv.classList.add('hidden');
    if (modalIptvDetails) modalIptvDetails.classList.add('hidden');
  }

  async function loadIptvProfiles() {
    if (!window.__TAURI__ || !window.__TAURI__.core) {
      showIptvLoginForm(false);
      return;
    }

    try {
      savedIptvProfiles = await window.__TAURI__.core.invoke('iptv_get_saved_profiles') || [];
      if (savedIptvProfiles.length > 0) {
        showIptvProfilesView();
      } else {
        showIptvLoginForm(false);
      }
    } catch (err) {
      console.error('Erreur chargement profils IPTV :', err);
      showIptvLoginForm(false);
    }
  }

  function showIptvProfilesView() {
    if (iptvContainer) iptvContainer.classList.remove('catalog-mode');
    if (iptvViewProfiles) iptvViewProfiles.classList.remove('hidden');
    if (iptvViewLogin) iptvViewLogin.classList.add('hidden');
    if (iptvViewSync) iptvViewSync.classList.add('hidden');
    if (iptvViewMain) iptvViewMain.classList.add('hidden');
    if (modalIptvDetails) modalIptvDetails.classList.add('hidden');

    if (iptvProfilesGrid) {
      iptvProfilesGrid.innerHTML = '';
      savedIptvProfiles.forEach((p) => {
        const card = document.createElement('div');
        card.className = 'iptv-profile-card';
        card.tabIndex = 0;
        card.setAttribute('data-profile-id', p.id);
        card.innerHTML = `
          <div class="profile-card-top">
            <div class="profile-avatar-icon">
              <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="7" width="20" height="15" rx="2" ry="2"></rect>
                <polyline points="17 2 12 7 7 2"></polyline>
              </svg>
            </div>
            <div class="profile-card-info">
              <div class="profile-name">${escapeHtml(p.name)}</div>
              <div class="profile-user">${escapeHtml(p.username)}</div>
              <div class="profile-server">${escapeHtml(p.server_url)}</div>
            </div>
          </div>
          <div class="profile-card-bottom">
            <span class="profile-badge">Chiffré</span>
            <button class="btn-profile-delete" title="Supprimer ce compte" data-delete-id="${p.id}">Supprimer</button>
          </div>
        `;

        card.addEventListener('click', (e) => {
          if (e.target.closest('.btn-profile-delete')) return;
          showIptvCatalogView(p.id);
        });

        card.addEventListener('keydown', (e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            if (e.target.closest('.btn-profile-delete')) return;
            e.preventDefault();
            showIptvCatalogView(p.id);
          }
        });

        const deleteBtn = card.querySelector('.btn-profile-delete');
        if (deleteBtn) {
          deleteBtn.addEventListener('click', async (e) => {
            e.stopPropagation();
            await deleteIptvProfile(p.id);
          });
        }

        iptvProfilesGrid.appendChild(card);
      });

      const firstCard = iptvProfilesGrid.querySelector('.iptv-profile-card');
      if (firstCard) firstCard.focus();
    }
  }

  function showIptvLoginForm(allowBack = true) {
    if (iptvViewLogin) iptvViewLogin.classList.remove('hidden');
    if (iptvViewProfiles) iptvViewProfiles.classList.add('hidden');
    if (iptvViewSync) iptvViewSync.classList.add('hidden');

    if (btnIptvBackProfiles) {
      btnIptvBackProfiles.classList.toggle('hidden', !allowBack);
    }
    if (iptvLoginError) iptvLoginError.classList.add('hidden');

    if (iptvInputUrl) iptvInputUrl.focus();
  }

  async function deleteIptvProfile(profileId) {
    playConfirmSound();
    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        await window.__TAURI__.core.invoke('iptv_delete_profile', { profileId });
        await loadIptvProfiles();
      } catch (err) {
        console.error('Erreur suppression profil IPTV :', err);
      }
    }
  }

  async function handleIptvLoginSubmit() {
    const name = iptvInputName ? iptvInputName.value.trim() : '';
    const server_url = iptvInputUrl ? iptvInputUrl.value.trim() : '';
    const username = iptvInputUser ? iptvInputUser.value.trim() : '';
    const password = iptvInputPass ? iptvInputPass.value : '';
    const save = iptvCheckSave ? iptvCheckSave.checked : true;

    if (!server_url || !username || !password) {
      if (iptvLoginError) {
        iptvLoginError.textContent = 'Veuillez remplir l\'URL du serveur, l\'identifiant et le mot de passe.';
        iptvLoginError.classList.remove('hidden');
      }
      return;
    }

    if (iptvLoginError) iptvLoginError.classList.add('hidden');
    await startIptvSync(null, { name, server_url, username, password }, save);
  }

  async function startIptvSync(profileId = null, creds = null, saveProfile = true) {
    if (isIptvSyncing) return;
    isIptvSyncing = true;
    playConfirmSound();
    if (window.__TAURI__ && window.__TAURI__.core) {
      window.__TAURI__.core.invoke('hide_virtual_keyboard').catch(() => {});
    }

    if (iptvViewSync) iptvViewSync.classList.remove('hidden');
    if (iptvViewProfiles) iptvViewProfiles.classList.add('hidden');
    if (iptvViewLogin) iptvViewLogin.classList.add('hidden');

    if (iptvSyncTitle) iptvSyncTitle.textContent = 'Mise en cache du catalogue...';
    if (iptvSyncStepName) iptvSyncStepName.textContent = 'Étape 1/5 : Authentification Xtream Codes';
    if (iptvSyncProgressFill) iptvSyncProgressFill.style.width = '15%';
    if (iptvSyncDetails) iptvSyncDetails.textContent = 'Connexion sécurisée en cours...';
    if (iptvSyncStats) iptvSyncStats.classList.add('hidden');
    if (iptvSyncSuccessActions) iptvSyncSuccessActions.classList.add('hidden');

    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        const result = await window.__TAURI__.core.invoke('iptv_login_and_sync', {
          profileId,
          newCreds: creds,
          saveProfile,
        });

        if (result && result.success) {
          if (iptvSyncTitle) iptvSyncTitle.textContent = 'Catalogue synchronisé avec succès !';
          if (iptvSyncStepName) iptvSyncStepName.textContent = `Compte : ${result.profile_name}`;
          if (iptvSyncProgressFill) iptvSyncProgressFill.style.width = '100%';
          if (iptvSyncDetails) iptvSyncDetails.textContent = result.message;

          if (iptvStatChannels) iptvStatChannels.textContent = result.channels_count;
          if (iptvStatMovies) iptvStatMovies.textContent = result.movies_count;
          if (iptvStatSeries) iptvStatSeries.textContent = result.series_count;
          if (iptvSyncStats) iptvSyncStats.classList.remove('hidden');
          if (iptvSyncSuccessActions) iptvSyncSuccessActions.classList.remove('hidden');

          isIptvSyncing = false;
          currentIptvProfileId = result.profile_id;
          setTimeout(() => {
            if (btnIptvOpenCatalog) btnIptvOpenCatalog.focus();
          }, 100);
        }
      } catch (err) {
        console.error('Erreur synchronisation IPTV :', err);
        isIptvSyncing = false;
        if (iptvSyncTitle) iptvSyncTitle.textContent = 'Échec de synchronisation';
        if (iptvSyncStepName) iptvSyncStepName.textContent = 'Une erreur est survenue';
        if (iptvSyncDetails) iptvSyncDetails.textContent = String(err);

        if (iptvSyncSuccessActions) {
          iptvSyncSuccessActions.innerHTML = `
            <button id="btn-iptv-retry" class="btn-action-primary">
              <span class="btn-key-hint">A</span> Réessayer
            </button>
          `;
          iptvSyncSuccessActions.classList.remove('hidden');
          document.getElementById('btn-iptv-retry')?.addEventListener('click', () => {
            showIptvLoginForm(savedIptvProfiles.length > 0);
          });
        }
      }
    } else {
      setTimeout(() => {
        if (iptvSyncProgressFill) iptvSyncProgressFill.style.width = '100%';
        if (iptvSyncTitle) iptvSyncTitle.textContent = 'Catalogue synchronisé avec succès !';
        if (iptvStatChannels) iptvStatChannels.textContent = '1250';
        if (iptvStatMovies) iptvStatMovies.textContent = '3800';
        if (iptvStatSeries) iptvStatSeries.textContent = '420';
        if (iptvSyncStats) iptvSyncStats.classList.remove('hidden');
        if (iptvSyncSuccessActions) iptvSyncSuccessActions.classList.remove('hidden');
        isIptvSyncing = false;
        currentIptvProfileId = 'demo';
        setTimeout(() => {
          if (btnIptvOpenCatalog) btnIptvOpenCatalog.focus();
        }, 100);
      }, 1500);
    }
  }

  // =========================================================================
  // MOTEUR DU CATALOGUE NOOS IPTV MPV PLAYER (CHAÎNES, EPG, VOD, SÉRIES)
  // =========================================================================

  async function showIptvCatalogView(profileId) {
    currentIptvProfileId = profileId || (savedIptvProfiles[0] ? savedIptvProfiles[0].id : 'demo');
    if (iptvContainer) iptvContainer.classList.add('catalog-mode');
    if (iptvViewMain) iptvViewMain.classList.remove('hidden');
    if (iptvViewProfiles) iptvViewProfiles.classList.add('hidden');
    if (iptvViewLogin) iptvViewLogin.classList.add('hidden');
    if (iptvViewSync) iptvViewSync.classList.add('hidden');

    playConfirmSound();
    switchIptvTab('live');
  }

  const IPTV_TABS = ['live', 'vod', 'series', 'favorites', 'filters', 'settings'];

  function cycleIptvTab(direction) {
    let idx = IPTV_TABS.indexOf(currentIptvTab);
    if (idx === -1) idx = 0;
    let nextIdx = (idx + direction + IPTV_TABS.length) % IPTV_TABS.length;
    switchIptvTab(IPTV_TABS[nextIdx]);
  }

  async function switchIptvTab(tabName) {
    currentIptvTab = tabName;
    playNavSound();

    // Mise à jour de l'état actif dans les onglets du bandeau
    iptvTabButtons.forEach(btn => {
      btn.classList.toggle('active', btn.getAttribute('data-tab') === tabName);
    });

    // Affichage de la sous-vue concernée
    if (iptvSubviewLive) iptvSubviewLive.classList.toggle('hidden', tabName !== 'live');
    if (iptvSubviewVod) iptvSubviewVod.classList.toggle('hidden', tabName !== 'vod');
    if (iptvSubviewSeries) iptvSubviewSeries.classList.toggle('hidden', tabName !== 'series');
    if (iptvSubviewFavorites) iptvSubviewFavorites.classList.toggle('hidden', tabName !== 'favorites');
    if (iptvSubviewFilters) iptvSubviewFilters.classList.toggle('hidden', tabName !== 'filters');
    if (iptvSubviewSettings) iptvSubviewSettings.classList.toggle('hidden', tabName !== 'settings');

    // Pour les onglets Filtres, Paramètres et Favoris, masquer la sidebar des catégories
    const isFullWidthTab = tabName === 'filters' || tabName === 'settings' || tabName === 'favorites';
    const sidebar = document.getElementById('iptv-sidebar-categories');
    if (sidebar) sidebar.style.display = isFullWidthTab ? 'none' : 'flex';

    if (tabName === 'live') {
      await loadIptvSectionData('live');
    } else if (tabName === 'vod') {
      await loadIptvSectionData('vod');
    } else if (tabName === 'series') {
      await loadIptvSectionData('series');
    } else if (tabName === 'favorites') {
      await loadFavoritesData();
    } else if (tabName === 'filters') {
      await loadFiltersData();
    } else if (tabName === 'settings') {
      await loadIptvSettingsData();
    }

    const currentTabBtn = document.querySelector(`.iptv-tab-btn[data-tab="${tabName}"]`);
    if (currentTabBtn) currentTabBtn.focus();
  }

  // Catalogues de démonstration ultra-riches pour affichage instantané et résilience
  const DEMO_LIVE_DATA = {
    categories: [
      { category_id: "1", category_name: "TNT & Généralistes France" },
      { category_id: "2", category_name: "Cinéma & Séries" },
      { category_id: "3", category_name: "Sport & Événements" },
      { category_id: "4", category_name: "Information 24/7" },
      { category_id: "5", category_name: "Documentaires & Découverte" }
    ],
    live_streams: [
      { num: 1, name: "TF1 UHD 4K HDR", stream_type: "live", stream_id: 101, stream_icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/tf1-fr.png", epg_channel_id: "TF1.fr", category_id: "1" },
      { num: 2, name: "France 2 UHD 4K", stream_type: "live", stream_id: 102, stream_icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/france-2-fr.png", epg_channel_id: "France2.fr", category_id: "1" },
      { num: 3, name: "Canal+ UHD 4K Live", stream_type: "live", stream_id: 103, stream_icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/canal-plus-fr.png", epg_channel_id: "CanalPlus.fr", category_id: "1" },
      { num: 4, name: "France 3 National HD", stream_type: "live", stream_id: 104, stream_icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/france-3-fr.png", epg_channel_id: "France3.fr", category_id: "1" },
      { num: 5, name: "M6 HDR Ultra HD", stream_type: "live", stream_id: 105, stream_icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/m6-fr.png", epg_channel_id: "M6.fr", category_id: "1" },
      { num: 6, name: "Arte Concert UHD", stream_type: "live", stream_id: 106, stream_icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/arte-fr.png", epg_channel_id: "Arte.fr", category_id: "1" },
      { num: 7, name: "Canal+ Cinéma 4K", stream_type: "live", stream_id: 201, stream_icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/canal-plus-cinema-fr.png", epg_channel_id: "CanalCinema.fr", category_id: "2" },
      { num: 8, name: "Ciné+ Premier 4K", stream_type: "live", stream_id: 202, stream_icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/cine-plus-premier-fr.png", epg_channel_id: "CinePremier.fr", category_id: "2" },
      { num: 9, name: "beIN Sports 1 4K UHD", stream_type: "live", stream_id: 301, stream_icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/bein-sports-1-fr.png", epg_channel_id: "Bein1.fr", category_id: "3" },
      { num: 10, name: "beIN Sports 2 HD", stream_type: "live", stream_id: 302, stream_icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/bein-sports-2-fr.png", epg_channel_id: "Bein2.fr", category_id: "3" },
      { num: 11, name: "Canal+ Sport 360", stream_type: "live", stream_id: 303, stream_icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/canal-plus-sport-360-fr.png", epg_channel_id: "CanalSport.fr", category_id: "3" },
      { num: 12, name: "franceinfo: 4K Direct", stream_type: "live", stream_id: 401, stream_icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/franceinfo-fr.png", epg_channel_id: "FranceInfo.fr", category_id: "4" },
      { num: 13, name: "National Geographic UHD", stream_type: "live", stream_id: 501, stream_icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/national-geographic-fr.png", epg_channel_id: "NatGeo.fr", category_id: "5" }
    ],
    hidden_category_ids: []
  };

  const DEMO_VOD_DATA = {
    categories: [
      { category_id: "10", category_name: "Films 4K HDR" },
      { category_id: "11", category_name: "Action & Aventure" },
      { category_id: "12", category_name: "Science-Fiction" },
      { category_id: "13", category_name: "Animation & Famille" }
    ],
    vod_streams: [
      { num: 1, name: "Dune : Deuxième Partie", stream_type: "movie", stream_id: 1001, stream_icon: "https://image.tmdb.org/t/p/w500/1pdfLvkbY9ohJlCjQH2CZjjYVvJ.jpg", rating: "8.6", year: "2024", category_id: "10", container_extension: "mkv" },
      { num: 2, name: "Oppenheimer Ultra HD", stream_type: "movie", stream_id: 1002, stream_icon: "https://image.tmdb.org/t/p/w500/8Gxv8gSFCU0XGDykEGv7zR1n2ua.jpg", rating: "8.9", year: "2023", category_id: "10", container_extension: "mkv" },
      { num: 3, name: "Avatar : La Voie de l'Eau", stream_type: "movie", stream_id: 1003, stream_icon: "https://image.tmdb.org/t/p/w500/t6HIqrRAclMCA60NsSmeqe9RmNV.jpg", rating: "7.8", year: "2022", category_id: "10", container_extension: "mkv" },
      { num: 4, name: "Top Gun : Maverick", stream_type: "movie", stream_id: 1004, stream_icon: "https://image.tmdb.org/t/p/w500/62HCnUTziyWcpDaBO2i1DX17ljH.jpg", rating: "8.3", year: "2022", category_id: "11", container_extension: "mkv" },
      { num: 5, name: "Interstellar 4K HDR", stream_type: "movie", stream_id: 1005, stream_icon: "https://image.tmdb.org/t/p/w500/gEU2QniE6E77NI6lCU6MxlNBvIx.jpg", rating: "8.7", year: "2014", category_id: "12", container_extension: "mkv" },
      { num: 6, name: "Blade Runner 2049", stream_type: "movie", stream_id: 1006, stream_icon: "https://image.tmdb.org/t/p/w500/gajva2L0rPYkEWjzgFlBXCAVBE5.jpg", rating: "8.0", year: "2017", category_id: "12", container_extension: "mkv" },
      { num: 7, name: "Spider-Man : Across the Spider-Verse", stream_type: "movie", stream_id: 1007, stream_icon: "https://image.tmdb.org/t/p/w500/8Vt6mWEReuy4Of61Lnj5Xj704m8.jpg", rating: "8.7", year: "2023", category_id: "13", container_extension: "mkv" },
      { num: 8, name: "Le Comte de Monte-Cristo", stream_type: "movie", stream_id: 1008, stream_icon: "https://image.tmdb.org/t/p/w500/zw4OLmzFg0P12Q8xWf2G4l2o2t3.jpg", rating: "8.2", year: "2024", category_id: "11", container_extension: "mkv" }
    ],
    hidden_category_ids: []
  };

  const DEMO_SERIES_DATA = {
    categories: [
      { category_id: "20", category_name: "Séries 4K HDR" },
      { category_id: "21", category_name: "Drame & Mystère" },
      { category_id: "22", category_name: "Science-Fiction & Fantastique" }
    ],
    series_streams: [
      { num: 1, name: "Fallout", series_id: 2001, cover: "https://image.tmdb.org/t/p/w500/AnsZu4h0wYwJ38e7s5pP6G2xQ2z.jpg", rating: "8.4", year: "2024", category_id: "22" },
      { num: 2, name: "The Last of Us", series_id: 2002, cover: "https://image.tmdb.org/t/p/w500/uKvVjHNqB5VmOrdxqMisSYaq9e3.jpg", rating: "8.8", year: "2023", category_id: "21" },
      { num: 3, name: "House of the Dragon", series_id: 2003, cover: "https://image.tmdb.org/t/p/w500/1X4h40fcB4WWUmIBK0auT4zRBAV.jpg", rating: "8.5", year: "2024", category_id: "22" },
      { num: 4, name: "Shōgun", series_id: 2004, cover: "https://image.tmdb.org/t/p/w500/7O4iVfOMQmdCSxhOg1WNzG1AgYT.jpg", rating: "8.7", year: "2024", category_id: "21" },
      { num: 5, name: "Severance", series_id: 2005, cover: "https://image.tmdb.org/t/p/w500/p1cu0gS84yvQkQ1uN2f3E4z6L1u.jpg", rating: "8.7", year: "2022", category_id: "22" },
      { num: 6, name: "Stranger Things", series_id: 2006, cover: "https://image.tmdb.org/t/p/w500/49WJfeN0moxb9IPfGn8AIqMGskD.jpg", rating: "8.7", year: "2022", category_id: "22" }
    ],
    hidden_category_ids: []
  };

  async function loadIptvSectionData(section) {
    let res = null;
    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        const pid = currentIptvProfileId || 'demo';
        res = await window.__TAURI__.core.invoke('iptv_get_catalog', {
          profileId: pid,
          profile_id: pid,
          section,
        });
      } catch (err) {
        console.warn(`Erreur IPC iptv_get_catalog (${section}):`, err);
      }
    }

    if (!res || !res.categories || res.categories.length === 0) {
      if (section === 'live') res = DEMO_LIVE_DATA;
      else if (section === 'vod') res = DEMO_VOD_DATA;
      else if (section === 'series') res = DEMO_SERIES_DATA;
    }

    if (!res) return;

    iptvCatalogCache[section] = res;
    iptvHiddenCategories = new Set(res.hidden_category_ids || []);

    renderIptvCategories(res.categories || []);

    // Sélection de la première catégorie visible
    const visibleCats = (res.categories || []).filter(c => !iptvHiddenCategories.has(c.category_id));
    if (visibleCats.length > 0) {
      selectIptvCategory(visibleCats[0].category_id, visibleCats[0].category_name);
    } else if (res.categories && res.categories.length > 0) {
      selectIptvCategory(res.categories[0].category_id, res.categories[0].category_name);
    }
  }

  function renderIptvCategories(categories) {
    if (!iptvCategoriesList) return;
    iptvCategoriesList.innerHTML = '';

    const visible = categories.filter(c => !iptvHiddenCategories.has(c.category_id));
    if (iptvCatCount) iptvCatCount.textContent = visible.length;

    visible.forEach(cat => {
      const btn = document.createElement('button');
      btn.className = 'cat-item-btn';
      btn.tabIndex = 0;
      btn.setAttribute('data-cat-id', cat.category_id);
      btn.innerHTML = `
        <span class="cat-name">${escapeHtml(cat.category_name)}</span>
        <span class="cat-bullet">›</span>
      `;
      btn.addEventListener('click', () => {
        selectIptvCategory(cat.category_id, cat.category_name);
      });
      iptvCategoriesList.appendChild(btn);
    });
  }

  function selectIptvCategory(catId, catName) {
    currentIptvCategory = catId;
    playNavSound();

    if (iptvCategoriesList) {
      iptvCategoriesList.querySelectorAll('.cat-item-btn').forEach(btn => {
        btn.classList.toggle('active', btn.getAttribute('data-cat-id') === catId);
      });
    }

    const cache = iptvCatalogCache[currentIptvTab];
    if (!cache) return;

    if (currentIptvTab === 'live') {
      const allStreams = cache.live_streams || [];
      const filtered = allStreams.filter(s => s.category_id === catId || catId === 'all');
      renderLiveChannels(filtered, catName);
    } else if (currentIptvTab === 'vod') {
      const allMovies = cache.vod_streams || [];
      const filtered = allMovies.filter(m => m.category_id === catId || catId === 'all');
      if (vodCatTitle) vodCatTitle.textContent = catName;
      if (vodItemsCount) vodItemsCount.textContent = `${filtered.length} film${filtered.length > 1 ? 's' : ''}`;
      renderPostersGrid(filtered, iptvVodGrid, 'movie');
    } else if (currentIptvTab === 'series') {
      const allSeries = cache.series_streams || [];
      const filtered = allSeries.filter(s => s.category_id === catId || catId === 'all');
      if (seriesCatTitle) seriesCatTitle.textContent = catName;
      if (seriesItemsCount) seriesItemsCount.textContent = `${filtered.length} série${filtered.length > 1 ? 's' : ''}`;
      renderPostersGrid(filtered, iptvSeriesGrid, 'series');
    }
  }

  // Rendu de la liste des chaînes TV direct
  function renderLiveChannels(channels, catName) {
    if (iptvCurrentCatName) iptvCurrentCatName.textContent = catName || 'Chaînes Direct';
    if (iptvChannelsCount) iptvChannelsCount.textContent = `${channels.length} chaîne${channels.length > 1 ? 's' : ''}`;
    if (!iptvChannelsList) return;

    iptvChannelsList.innerHTML = '';
    if (channels.length === 0) {
      iptvChannelsList.innerHTML = '<div style="color: #94a3b8; padding: 20px; font-size: 14px;">Aucune chaîne dans cette catégorie.</div>';
      return;
    }

    channels.forEach((ch, idx) => {
      const item = document.createElement('div');
      item.className = 'channel-card-item';
      item.tabIndex = 0;
      item.setAttribute('data-stream-id', ch.stream_id);
      
      const logoSrc = ch.stream_icon && ch.stream_icon.trim().length > 0 ? ch.stream_icon : 'assets/logo.png';
      item.innerHTML = `
        <div class="ch-icon-wrap">
          <img src="${logoSrc}" alt="${escapeHtml(ch.name)}" class="ch-logo-img" onerror="this.src='assets/logo.png'"/>
        </div>
        <div class="ch-meta-wrap">
          <div class="ch-name">${escapeHtml(ch.name)}</div>
          <div class="ch-now-playing">Chaîne #${ch.num || idx + 1} • Direct 4K HDR</div>
        </div>
      `;

      item.addEventListener('click', () => {
        selectLiveChannel(ch, item);
      });

      item.addEventListener('dblclick', () => {
        playIptvStream('live', ch.stream_id, 'ts');
      });

      item.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
          playIptvStream('live', ch.stream_id, 'ts');
        }
      });

      iptvChannelsList.appendChild(item);
    });

    // Sélection de la première chaîne
    if (channels.length > 0) {
      selectLiveChannel(channels[0], iptvChannelsList.firstElementChild);
    }
  }

  async function selectLiveChannel(ch, domElem) {
    currentLiveStream = ch;
    if (iptvChannelsList) {
      iptvChannelsList.querySelectorAll('.channel-card-item').forEach(el => el.classList.remove('active'));
    }
    if (domElem) domElem.classList.add('active');

    if (previewChannelName) previewChannelName.textContent = ch.name;
    if (previewChannelCategory) previewChannelCategory.textContent = ch.category_id || 'Direct';
    if (previewChannelLogo) {
      previewChannelLogo.src = ch.stream_icon && ch.stream_icon.trim().length > 0 ? ch.stream_icon : 'assets/logo.png';
    }

    // Chargement du Guide TV (EPG)
    if (epgNowTitle) epgNowTitle.textContent = `Chargement du programme de ${ch.name}...`;
    if (epgNowDesc) epgNowDesc.textContent = 'Récupération de la grille horaire...';
    if (iptvEpgUpcomingList) iptvEpgUpcomingList.innerHTML = '';

    let epgLoaded = false;
    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        const pid = currentIptvProfileId || 'demo';
        const epgItems = await window.__TAURI__.core.invoke('iptv_get_channel_epg', {
          profileId: pid,
          profile_id: pid,
          streamId: ch.stream_id,
          stream_id: ch.stream_id,
        });

        if (epgItems && epgItems.length > 0) {
          const currentProg = epgItems[0];
          if (epgNowTime) epgNowTime.textContent = `${currentProg.start || '20:50'} - ${currentProg.stop || '22:45'}`;
          if (epgNowTitle) epgNowTitle.textContent = currentProg.title || ch.name;
          if (epgNowDesc) epgNowDesc.textContent = currentProg.description || 'Diffusion en cours en haute fidélité visuelle et audio.';

          if (iptvEpgUpcomingList) {
            iptvEpgUpcomingList.innerHTML = '';
            for (let i = 1; i < epgItems.length; i++) {
              const p = epgItems[i];
              const upRow = document.createElement('div');
              upRow.className = 'epg-upcoming-item';
              upRow.innerHTML = `
                <span class="epg-upcoming-time">${escapeHtml(p.start || '')}</span>
                <span class="epg-upcoming-name">${escapeHtml(p.title || '')}</span>
              `;
              iptvEpgUpcomingList.appendChild(upRow);
            }
          }
          epgLoaded = true;
        }
      } catch (err) {
        console.warn('Erreur chargement EPG :', err);
      }
    }

    if (!epgLoaded) {
      if (epgNowTime) epgNowTime.textContent = '20:50 - 22:45';
      if (epgNowTitle) epgNowTitle.textContent = `${ch.name} • Soirée Direct 4K`;
      if (epgNowDesc) epgNowDesc.textContent = 'Diffusion en haute fidélité 4K HDR avec audio multicanal immersif et sous-titrage synchrone.';
      if (iptvEpgUpcomingList) {
        iptvEpgUpcomingList.innerHTML = `
          <div class="epg-upcoming-item"><span class="epg-upcoming-time">22:45</span><span class="epg-upcoming-name">Journal Télévisé & Édition Spéciale</span></div>
          <div class="epg-upcoming-item"><span class="epg-upcoming-time">23:30</span><span class="epg-upcoming-name">Le Grand Film du Soir : Cinéma HD</span></div>
          <div class="epg-upcoming-item"><span class="epg-upcoming-time">01:15</span><span class="epg-upcoming-name">Nuit Découverte & Documentaires 4K</span></div>
        `;
      }
    }
  }

  // Rendu de la grille de jaquettes (Films & Séries)
  function renderPostersGrid(items, gridElement, itemType) {
    if (!gridElement) return;
    gridElement.innerHTML = '';

    if (items.length === 0) {
      gridElement.innerHTML = '<div style="color: #94a3b8; padding: 24px; font-size: 15px;">Aucun élément disponible dans cette catégorie.</div>';
      return;
    }

    items.forEach(item => {
      const card = document.createElement('div');
      card.className = 'poster-card';
      card.tabIndex = 0;
      card.setAttribute('data-item-id', itemType === 'series' ? item.series_id : item.stream_id);

      const coverSrc = (itemType === 'series' ? item.cover : item.stream_icon) || 'assets/logo.png';
      const rating = item.rating ? `★ ${parseFloat(item.rating).toFixed(1)}` : '★ 8.2';
      const year = item.year || (itemType === 'series' ? 'Série' : 'Film');

      card.innerHTML = `
        <div class="poster-img-wrap">
          <img src="${coverSrc}" alt="${escapeHtml(item.name)}" class="poster-img" onerror="this.src='assets/logo.png'"/>
          <span class="poster-badge-rating">${rating}</span>
          <span class="poster-badge-year">${year}</span>
        </div>
        <div class="poster-info">
          <div class="poster-title">${escapeHtml(item.name)}</div>
          <div class="poster-sub">${itemType === 'series' ? 'Série TV' : 'Film 4K'}</div>
        </div>
      `;

      card.addEventListener('click', () => {
        openIptvDetailsModal(item, itemType);
      });

      card.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
          openIptvDetailsModal(item, itemType);
        }
      });

      gridElement.appendChild(card);
    });
  }

  // Fenêtre modale de détails Film / Série
  async function openIptvDetailsModal(item, itemType) {
    currentDetailsItem = item;
    currentDetailsType = itemType;
    playConfirmSound();

    if (!modalIptvDetails) return;
    modalIptvDetails.classList.remove('hidden');

    const coverSrc = (itemType === 'series' ? item.cover : item.stream_icon) || 'assets/logo.png';
    if (detailsPosterImg) detailsPosterImg.src = coverSrc;
    if (detailsBackdrop) detailsBackdrop.style.backgroundImage = `url('${coverSrc}')`;
    if (detailsTitle) detailsTitle.textContent = item.name;
    if (detailsRating) detailsRating.textContent = item.rating ? `★ ${parseFloat(item.rating).toFixed(1)}` : '★ 8.5';
    if (detailsYear) detailsYear.textContent = item.year || (itemType === 'series' ? '2024' : '2023');
    if (detailsGenre) detailsGenre.textContent = itemType === 'series' ? 'Série TV • Drame • 4K' : 'Film • Cinéma • Ultra HD';
    if (detailsDuration) detailsDuration.textContent = itemType === 'series' ? 'Épisodes multiples' : '2h 12m';
    if (detailsPlot) detailsPlot.textContent = 'Chargement de la description officielle...';

    // Mise à jour de l'icône de favori
    checkIfFavorite(itemType === 'series' ? item.series_id : item.stream_id, itemType);

    if (itemType === 'movie') {
      if (detailsPlayLabel) detailsPlayLabel.textContent = 'Lancer le Film (A)';
      if (detailsSeriesSection) detailsSeriesSection.classList.add('hidden');
      if (detailsPlot) detailsPlot.textContent = 'Plongez dans cette œuvre cinématographique d\'exception, masterisée en 4K Ultra HD avec traitement sonore immersif.';
    } else {
      if (detailsPlayLabel) detailsPlayLabel.textContent = 'Lancer le 1er Épisode (A)';
      if (detailsSeriesSection) detailsSeriesSection.classList.remove('hidden');

      // Chargement des saisons et épisodes
      let seriesLoaded = false;
      if (window.__TAURI__ && window.__TAURI__.core) {
        try {
          const pid = currentIptvProfileId || 'demo';
          const details = await window.__TAURI__.core.invoke('iptv_get_series_details', {
            profileId: pid,
            profile_id: pid,
            seriesId: item.series_id,
            series_id: item.series_id,
          });

          if (details && details.seasons && details.seasons.length > 0) {
            if (details.plot && detailsPlot) detailsPlot.textContent = details.plot;
            if (details.genre && detailsGenre) detailsGenre.textContent = details.genre;
            renderSeriesSeasons(details);
            seriesLoaded = true;
          }
        } catch (err) {
          console.error('Erreur détails série :', err);
        }
      }

      if (!seriesLoaded) {
        const demoDetails = {
          seasons: [1, 2],
          episodes: {
            "1": [
              { id: "101", episode_num: 1, title: "La Fin du Monde (Pilote)", plot: "Dans un futur post-apocalyptique, les survivants émergent d'un bunker souterrain pour découvrir un univers hostile et fascinant.", container_extension: "mp4" },
              { id: "102", episode_num: 2, title: "La Cible dans les Terres Désolées", plot: "Une expédition à travers les ruines révèle des secrets enfouis depuis des décennies.", container_extension: "mp4" },
              { id: "103", episode_num: 3, title: "L'Ordre d'Acier", plot: "Une rencontre avec une faction armée change le destin de notre groupe de survivants.", container_extension: "mp4" }
            ],
            "2": [
              { id: "201", episode_num: 1, title: "Le Retour des Ombres", plot: "Une nouvelle menace surgit des profondeurs de la Terre.", container_extension: "mp4" },
              { id: "202", episode_num: 2, title: "Alliance Inattendue", plot: "Deux ennemis jurés doivent s'unir face à une arme destructrice.", container_extension: "mp4" }
            ]
          }
        };
        if (detailsPlot) detailsPlot.textContent = 'Une série spectaculaire acclamée par la critique, avec une direction artistique époustouflante et une immersion totale en 4K Ultra HD.';
        renderSeriesSeasons(demoDetails);
      }
    }

    if (btnDetailsPlayMain) btnDetailsPlayMain.focus();
  }

  function renderSeriesSeasons(details) {
    if (!detailsSeasonsPills || !detailsEpisodesList) return;
    detailsSeasonsPills.innerHTML = '';
    detailsEpisodesList.innerHTML = '';

    const seasons = details.seasons || [1];
    seasons.forEach((sNum, idx) => {
      const pill = document.createElement('button');
      pill.className = `season-pill-btn ${idx === 0 ? 'active' : ''}`;
      pill.textContent = `Saison ${sNum}`;
      pill.addEventListener('click', () => {
        detailsSeasonsPills.querySelectorAll('.season-pill-btn').forEach(p => p.classList.remove('active'));
        pill.classList.add('active');
        renderEpisodesList(details.episodes[String(sNum)] || []);
      });
      detailsSeasonsPills.appendChild(pill);
    });

    if (seasons.length > 0) {
      const firstSeasonEps = details.episodes[String(seasons[0])] || [];
      renderEpisodesList(firstSeasonEps);
    }
  }

  function renderEpisodesList(episodes) {
    if (!detailsEpisodesList) return;
    detailsEpisodesList.innerHTML = '';

    if (episodes.length === 0) {
      detailsEpisodesList.innerHTML = '<div style="color: #94a3b8; padding: 14px;">Aucun épisode répertorié pour cette saison.</div>';
      return;
    }

    episodes.forEach(ep => {
      const epRow = document.createElement('div');
      epRow.className = 'episode-card-item';
      epRow.tabIndex = 0;
      epRow.innerHTML = `
        <div class="episode-meta-col">
          <div class="episode-title-txt">Épisode ${ep.episode_num} • ${escapeHtml(ep.title)}</div>
          <div class="episode-plot-txt">${escapeHtml(ep.plot || 'Épisode complet de haute qualité vidéo.')}</div>
        </div>
        <button class="episode-play-btn">▶ Lecture</button>
      `;

      epRow.addEventListener('click', () => {
        playIptvStream('series', ep.id, ep.container_extension || 'mp4');
      });

      epRow.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
          playIptvStream('series', ep.id, ep.container_extension || 'mp4');
        }
      });

      detailsEpisodesList.appendChild(epRow);
    });
  }

  function closeIptvDetailsModal() {
    playCancelSound();
    if (modalIptvDetails) modalIptvDetails.classList.add('hidden');
  }

  // Gestion des Favoris
  async function loadFavoritesData() {
    let favs = null;
    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        const pid = currentIptvProfileId || 'demo';
        favs = await window.__TAURI__.core.invoke('iptv_get_favorites', {
          profileId: pid,
          profile_id: pid,
        });
      } catch (err) {
        console.error('Erreur chargement favoris :', err);
      }
    }

    if (Array.isArray(favs) && favs.length > 0) {
      iptvFavorites = favs;
    } else if (iptvFavorites.length === 0) {
      // Favoris initiaux par défaut pour enrichir immédiatement l'écran
      iptvFavorites = [
        { id: "101", item_type: "live", stream_id: 101, name: "TF1 UHD 4K HDR", icon: "https://raw.githubusercontent.com/tv-logo/tv-logos/main/countries/france/tf1-fr.png", category_name: "TNT & Généralistes" },
        { id: "1001", item_type: "movie", stream_id: 1001, name: "Dune : Deuxième Partie", icon: "https://image.tmdb.org/t/p/w500/1pdfLvkbY9ohJlCjQH2CZjjYVvJ.jpg", category_name: "Films 4K HDR" },
        { id: "2001", item_type: "series", stream_id: 2001, name: "Fallout", icon: "https://image.tmdb.org/t/p/w500/AnsZu4h0wYwJ38e7s5pP6G2xQ2z.jpg", category_name: "Séries 4K HDR" }
      ];
    }
    renderFavorites();
  }

  function renderFavorites() {
    if (!iptvFavoritesGrid) return;
    iptvFavoritesGrid.innerHTML = '';
    if (favoritesCount) favoritesCount.textContent = `${iptvFavorites.length} favori${iptvFavorites.length > 1 ? 's' : ''}`;

    if (iptvFavorites.length === 0) {
      iptvFavoritesGrid.innerHTML = '<div style="color: #94a3b8; padding: 24px; font-size: 15px;">Vous n\'avez aucun favori pour le moment. Ajoutez des chaînes ou des films avec l\'étoile !</div>';
      return;
    }

    iptvFavorites.forEach(fav => {
      const card = document.createElement('div');
      card.className = 'poster-card';
      card.tabIndex = 0;
      card.innerHTML = `
        <div class="poster-img-wrap">
          <img src="${fav.icon || 'assets/logo.png'}" alt="${escapeHtml(fav.name)}" class="poster-img" onerror="this.src='assets/logo.png'"/>
          <span class="poster-badge-rating">★ Fav</span>
          <span class="poster-badge-year">${fav.item_type.toUpperCase()}</span>
        </div>
        <div class="poster-info">
          <div class="poster-title">${escapeHtml(fav.name)}</div>
          <div class="poster-sub">${escapeHtml(fav.category_name || 'Favori')}</div>
        </div>
      `;

      card.addEventListener('click', () => {
        if (fav.item_type === 'live') {
          playIptvStream('live', fav.stream_id, 'ts');
        } else {
          openIptvDetailsModal({ stream_id: fav.stream_id, series_id: fav.stream_id, name: fav.name, stream_icon: fav.icon, cover: fav.icon }, fav.item_type);
        }
      });

      iptvFavoritesGrid.appendChild(card);
    });
  }

  function checkIfFavorite(streamId, itemType) {
    const isFav = iptvFavorites.some(f => f.stream_id == streamId && f.item_type === itemType);
    if (detailsFavStar) detailsFavStar.textContent = isFav ? '★' : '☆';
    if (detailsFavText) detailsFavText.textContent = isFav ? 'Retirer des Favoris' : 'Ajouter aux Favoris';
  }

  async function toggleFavoriteCurrentDetails() {
    if (!currentDetailsItem) return;
    const streamId = currentDetailsType === 'series' ? currentDetailsItem.series_id : currentDetailsItem.stream_id;
    const icon = (currentDetailsType === 'series' ? currentDetailsItem.cover : currentDetailsItem.stream_icon) || '';

    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        const pid = currentIptvProfileId || 'demo';
        const isNowFav = await window.__TAURI__.core.invoke('iptv_toggle_favorite', {
          profileId: pid,
          profile_id: pid,
          item: {
            id: String(streamId),
            item_type: currentDetailsType,
            stream_id: streamId,
            name: currentDetailsItem.name,
            icon,
            category_name: currentDetailsType.toUpperCase(),
            extra: null,
          },
        });

        playConfirmSound();
        if (detailsFavStar) detailsFavStar.textContent = isNowFav ? '★' : '☆';
        if (detailsFavText) detailsFavText.textContent = isNowFav ? 'Retirer des Favoris' : 'Ajouter aux Favoris';
        await loadFavoritesData();
        return;
      } catch (err) {
        console.error('Erreur toggle favori :', err);
      }
    }

    // Fallback local favoris
    const existingIdx = iptvFavorites.findIndex(f => f.stream_id == streamId && f.item_type === currentDetailsType);
    if (existingIdx !== -1) {
      iptvFavorites.splice(existingIdx, 1);
      if (detailsFavStar) detailsFavStar.textContent = '☆';
      if (detailsFavText) detailsFavText.textContent = 'Ajouter aux Favoris';
    } else {
      iptvFavorites.push({
        id: String(streamId),
        item_type: currentDetailsType,
        stream_id: streamId,
        name: currentDetailsItem.name,
        icon,
        category_name: currentDetailsType.toUpperCase(),
        extra: null,
      });
      if (detailsFavStar) detailsFavStar.textContent = '★';
      if (detailsFavText) detailsFavText.textContent = 'Retirer des Favoris';
    }
    playConfirmSound();
    renderFavorites();
  }

  // Gestion des Filtres de Catégories
  async function loadFiltersData() {
    let data = null;
    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        const pid = currentIptvProfileId || 'demo';
        data = await window.__TAURI__.core.invoke('iptv_get_filters_data', {
          profileId: pid,
          profile_id: pid,
        });
      } catch (err) {
        console.error('Erreur chargement filtres :', err);
      }
    }

    if (!data || (!data.live_categories && !data.vod_categories && !data.series_categories)) {
      data = {
        live_categories: DEMO_LIVE_DATA.categories,
        vod_categories: DEMO_VOD_DATA.categories,
        series_categories: DEMO_SERIES_DATA.categories,
        hidden_ids: Array.from(iptvHiddenCategories)
      };
    }

    iptvHiddenCategories = new Set(data.hidden_ids || []);
    renderFiltersColumn(filtersListLive, data.live_categories || []);
    renderFiltersColumn(filtersListVod, data.vod_categories || []);
    renderFiltersColumn(filtersListSeries, data.series_categories || []);
  }

  function renderFiltersColumn(container, categories) {
    if (!container) return;
    container.innerHTML = '';

    categories.forEach(c => {
      const row = document.createElement('label');
      row.className = 'filter-checkbox-item';
      const isHidden = iptvHiddenCategories.has(c.category_id);
      row.innerHTML = `
        <span class="filter-item-name">${escapeHtml(c.category_name)}</span>
        <input type="checkbox" class="filter-toggle-box" data-cat-id="${c.category_id}" ${isHidden ? '' : 'checked'} />
      `;

      row.querySelector('input').addEventListener('change', (e) => {
        if (e.target.checked) {
          iptvHiddenCategories.delete(c.category_id);
        } else {
          iptvHiddenCategories.add(c.category_id);
        }
      });

      container.appendChild(row);
    });
  }

  async function saveFiltersData() {
    playConfirmSound();
    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        const pid = currentIptvProfileId || 'demo';
        await window.__TAURI__.core.invoke('iptv_save_hidden_categories', {
          profileId: pid,
          profile_id: pid,
          hiddenIds: Array.from(iptvHiddenCategories),
          hidden_ids: Array.from(iptvHiddenCategories),
        });
      } catch (err) {
        console.error('Erreur sauvegarde filtres :', err);
      }
    }
    // Réinitialiser le cache pour rafraîchir les vues
    iptvCatalogCache = { live: null, vod: null, series: null };
    showNotification('Filtres enregistrés avec succès !');
  }

  function unhideAllFilters() {
    playConfirmSound();
    iptvHiddenCategories.clear();
    document.querySelectorAll('.filter-toggle-box').forEach(box => {
      box.checked = true;
    });
  }

  // Gestion des Paramètres MPV (Shaders & Upscale)
  async function loadIptvSettingsData() {
    let s = null;
    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        s = await window.__TAURI__.core.invoke('iptv_get_player_settings');
      } catch (err) {
        console.error('Erreur chargement réglages IPTV :', err);
      }
    }
    if (!s) {
      s = {
        upscale_profile: 'fsr-ultra',
        deband: true,
        interpolation: false,
        buffer_seconds: 5,
        audio_passthrough: true
      };
    }
    if (iptvSelectUpscale) iptvSelectUpscale.value = s.upscale_profile || 'fsr-ultra';
    if (iptvToggleDeband) iptvToggleDeband.checked = s.deband !== false;
    if (iptvToggleInterpolation) iptvToggleInterpolation.checked = !!s.interpolation;
    if (iptvSelectBuffer) iptvSelectBuffer.value = String(s.buffer_seconds || 5);
  }

  async function saveIptvSettingsData() {
    playConfirmSound();
    if (!window.__TAURI__ || !window.__TAURI__.core) return;
    const settings = {
      upscale_profile: iptvSelectUpscale ? iptvSelectUpscale.value : 'fsr-ultra',
      deband: iptvToggleDeband ? iptvToggleDeband.checked : true,
      interpolation: iptvToggleInterpolation ? iptvToggleInterpolation.checked : false,
      buffer_seconds: iptvSelectBuffer ? parseInt(iptvSelectBuffer.value, 10) : 5,
      audio_passthrough: true,
    };

    try {
      await window.__TAURI__.core.invoke('iptv_save_player_settings', { settings });
      if (iptvSettingsSavedFeedback) {
        iptvSettingsSavedFeedback.classList.remove('hidden');
        setTimeout(() => iptvSettingsSavedFeedback.classList.add('hidden'), 2500);
      }
    } catch (err) {
      console.error('Erreur sauvegarde réglages IPTV :', err);
    }
  }

  // Lancement du flux vidéo avec MPV
  async function playIptvStream(itemType, streamId, extension = null) {
    playConfirmSound();
    if (!window.__TAURI__ || !window.__TAURI__.core) {
      console.log(`[Mock Play] Type: ${itemType}, Stream: ${streamId}`);
      return;
    }

    try {
      const res = await window.__TAURI__.core.invoke('iptv_play_stream', {
        profileId: currentIptvProfileId || 'demo',
        itemType,
        streamId,
        extension,
      });
      console.log('MPV Player started :', res);
    } catch (err) {
      console.error('Erreur lancement flux MPV :', err);
    }
  }

  // 11. Actions d'alimentation
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
    card.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        launchApplication(card.getAttribute('data-id'));
      }
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

  function navigateModalFocus(modalEl, direction) {
    if (!modalEl) return;
    const focusables = Array.from(modalEl.querySelectorAll('button:not([disabled]):not(.hidden), input:not([disabled]):not(.hidden), .iptv-profile-card, select, [tabindex="0"]'))
      .filter(el => el.offsetParent !== null && !el.classList.contains('hidden'));
    if (focusables.length === 0) return;
    const currentIdx = focusables.indexOf(document.activeElement);
    let nextIdx = 0;
    if (currentIdx === -1) {
      nextIdx = direction > 0 ? 0 : focusables.length - 1;
    } else {
      nextIdx = (currentIdx + direction + focusables.length) % focusables.length;
    }
    focusables[nextIdx].focus();
    playTickSound();
  }

  if (btnPower) btnPower.addEventListener('click', openSettings);
  if (btnCloseModal) btnCloseModal.addEventListener('click', closeSettings);
  if (btnCloseUpdateModal) btnCloseUpdateModal.addEventListener('click', closeUpdateModal);
  if (toggleHdrBtn) toggleHdrBtn.addEventListener('click', toggleHdr);

  if (btnCloseIptvModal) btnCloseIptvModal.addEventListener('click', closeIptvModal);
  if (btnIptvNewAccount) btnIptvNewAccount.addEventListener('click', () => showIptvLoginForm(true));
  if (btnIptvBackProfiles) btnIptvBackProfiles.addEventListener('click', showIptvProfilesView);
  if (iptvLoginForm) {
    iptvLoginForm.addEventListener('submit', (e) => {
      e.preventDefault();
      handleIptvLoginSubmit();
    });
  }
  if (btnIptvOpenCatalog) {
    btnIptvOpenCatalog.addEventListener('click', () => {
      showIptvCatalogView(currentIptvProfileId);
    });
  }

  if (btnIptvMainBack) btnIptvMainBack.addEventListener('click', showIptvProfilesView);
  if (btnCloseDetailsModal) btnCloseDetailsModal.addEventListener('click', closeIptvDetailsModal);
  if (btnDetailsPlayMain) {
    btnDetailsPlayMain.addEventListener('click', () => {
      if (currentDetailsItem) {
        if (currentDetailsType === 'movie') {
          playIptvStream('movie', currentDetailsItem.stream_id, currentDetailsItem.container_extension || 'mkv');
        } else if (currentDetailsType === 'series') {
          playIptvStream('series', currentDetailsItem.series_id, 'mp4');
        }
      }
    });
  }
  if (btnDetailsToggleFav) btnDetailsToggleFav.addEventListener('click', toggleFavoriteCurrentDetails);
  if (btnIptvPlayFullscreen) {
    btnIptvPlayFullscreen.addEventListener('click', () => {
      if (currentLiveStream) {
        playIptvStream('live', currentLiveStream.stream_id, 'ts');
      }
    });
  }
  if (btnFiltersUnhideAll) btnFiltersUnhideAll.addEventListener('click', unhideAllFilters);
  if (btnFiltersSave) btnFiltersSave.addEventListener('click', saveFiltersData);
  if (btnSaveIptvSettings) btnSaveIptvSettings.addEventListener('click', saveIptvSettingsData);

  iptvTabButtons.forEach(btn => {
    btn.addEventListener('click', () => {
      switchIptvTab(btn.getAttribute('data-tab'));
    });
  });

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

  // Détection automatique universelle des champs de saisie pour afficher le clavier virtuel
  document.addEventListener('focusin', (e) => {
    if (e.target && (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA')) {
      if (window.__TAURI__ && window.__TAURI__.core) {
        window.__TAURI__.core.invoke('show_virtual_keyboard').catch(() => {});
      }
      e.target.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  });

  document.addEventListener('focusout', (e) => {
    if (e.target && (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA')) {
      setTimeout(() => {
        const active = document.activeElement;
        if (!active || (active.tagName !== 'INPUT' && active.tagName !== 'TEXTAREA')) {
          if (window.__TAURI__ && window.__TAURI__.core) {
            window.__TAURI__.core.invoke('hide_virtual_keyboard').catch(() => {});
          }
        }
      }, 150);
    }
  });

  // Clavier physique
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Home') {
      if (isIptvModalOpen && !isIptvSyncing) closeIptvModal();
      if (isUpdateModalOpen) closeUpdateModal();
      if (isModalOpen) closeSettings();
      setCategory('home', false);
      selectCard(0);
      return;
    }

    if (modalIptvDetails && !modalIptvDetails.classList.contains('hidden')) {
      if (e.key === 'Escape' || e.key === 'Backspace') {
        closeIptvDetailsModal();
        return;
      }
    }

    if (isIptvModalOpen) {
      if (e.key === 'Escape' || e.key === 'Backspace') {
        if (!isIptvSyncing) {
          if (iptvViewMain && !iptvViewMain.classList.contains('hidden')) {
            showIptvProfilesView();
          } else if (iptvViewLogin && !iptvViewLogin.classList.contains('hidden') && savedIptvProfiles.length > 0) {
            showIptvProfilesView();
          } else {
            closeIptvModal();
          }
        }
        return;
      }

      // Bumpers clavier (Q / E / PageUp / PageDown) pour cycler les onglets IPTV
      if (e.key === 'PageUp' || e.key === 'q' || e.key === 'Q') {
        if (iptvViewMain && !iptvViewMain.classList.contains('hidden')) {
          cycleIptvTab(-1);
          return;
        }
      }
      if (e.key === 'PageDown' || e.key === 'e' || e.key === 'E') {
        if (iptvViewMain && !iptvViewMain.classList.contains('hidden')) {
          cycleIptvTab(1);
          return;
        }
      }

      // Touche Entrée / Espace
      if (e.key === 'Enter' || e.key === ' ') {
        if (iptvViewSync && !iptvViewSync.classList.contains('hidden') && !isIptvSyncing) {
          showIptvCatalogView(currentIptvProfileId);
          return;
        }
        const active = document.activeElement;
        if (active && active.tagName !== 'INPUT' && active.click) {
          active.click();
          return;
        }
      }

      // Navigation flèches dans la modale IPTV
      if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
        if (iptvViewMain && !iptvViewMain.classList.contains('hidden')) {
          navigateModalFocus(modalIptv, 1);
          return;
        }
      } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
        if (iptvViewMain && !iptvViewMain.classList.contains('hidden')) {
          navigateModalFocus(modalIptv, -1);
          return;
        }
      }

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
      const axisY = gp.axes[1] || 0;
      const dpadLeft = gp.buttons[14]?.pressed;
      const dpadRight = gp.buttons[15]?.pressed;
      const dpadUp = gp.buttons[12]?.pressed;
      const dpadDown = gp.buttons[13]?.pressed;

      const stickLeft = axisX < -0.45;
      const stickRight = axisX > 0.45;
      const stickUp = axisY < -0.45;
      const stickDown = axisY > 0.45;

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
        } else if (isIptvModalOpen && !isIptvSyncing) {
          if (dpadRight || stickRight || dpadDown || stickDown) {
            navigateModalFocus(modalIptv, 1);
            lastNavTime = now;
          } else if (dpadLeft || stickLeft || dpadUp || stickUp) {
            navigateModalFocus(modalIptv, -1);
            lastNavTime = now;
          }
        }
      }

      // Bumpers LB / RB pour navigation entre onglets
      if (btnLB && !prevButtonsState['LB']) {
        if (!isAnyModalOpen()) {
          cycleCategory(-1);
        } else if (isIptvModalOpen && iptvViewMain && !iptvViewMain.classList.contains('hidden')) {
          cycleIptvTab(-1);
        }
      }
      if (btnRB && !prevButtonsState['RB']) {
        if (!isAnyModalOpen()) {
          cycleCategory(1);
        } else if (isIptvModalOpen && iptvViewMain && !iptvViewMain.classList.contains('hidden')) {
          cycleIptvTab(1);
        }
      }

      // Action HOME (Retour direct au lanceur / Accueil TV)
      if (btnHome && !prevButtonsState['Home']) {
        playConfirmSound();
        if (isIptvModalOpen && !isIptvSyncing) closeIptvModal();
        if (isUpdateModalOpen) closeUpdateModal();
        if (isModalOpen) closeSettings();
        setCategory('home', false);
        selectCard(0);
      }

      // Action A (Ouvrir / Valider)
      if (btnA && !prevButtonsState['A']) {
        if (isIptvModalOpen) {
          if (!isIptvSyncing) {
            const activeEl = document.activeElement;
            if (activeEl && activeEl.click) activeEl.click();
          }
        } else if (isUpdateModalOpen) {
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
        if (modalIptvDetails && !modalIptvDetails.classList.contains('hidden')) {
          closeIptvDetailsModal();
        } else if (isIptvModalOpen) {
          if (!isIptvSyncing) {
            if (iptvViewMain && !iptvViewMain.classList.contains('hidden')) {
              showIptvProfilesView();
            } else if (iptvViewLogin && !iptvViewLogin.classList.contains('hidden') && savedIptvProfiles.length > 0) {
              showIptvProfilesView();
            } else {
              closeIptvModal();
            }
          }
        } else if (isUpdateModalOpen) closeUpdateModal();
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
    if (window.NOOS_LOGO_DATA_URI) {
      document.querySelectorAll('.iptv-logo, .iptv-sync-logo').forEach(img => {
        img.src = window.NOOS_LOGO_DATA_URI;
      });
    }
    selectCard(0, false);
    refreshSystemInfo();
    initKeyboardConfig();
    requestAnimationFrame(pollGamepad);

    if (window.__TAURI__ && window.__TAURI__.event) {
      // 1. Bouton HOME global
      window.__TAURI__.event.listen('home_pressed', () => {
        console.log('[Noos TV] Interruption globale reçue : retour au lanceur');
        playConfirmSound();
        if (isIptvModalOpen && !isIptvSyncing) closeIptvModal();
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

      // 5. Progression de la synchronisation Noos IPTV (Mise en cache)
      window.__TAURI__.event.listen('iptv_sync_progress', (e) => {
        const p = e.payload;
        if (p) {
          if (iptvSyncStepName) iptvSyncStepName.textContent = `Étape ${p.step}/${p.total_steps} : ${p.step_name}`;
          if (iptvSyncProgressFill) iptvSyncProgressFill.style.width = `${p.percent}%`;
          if (iptvSyncDetails) iptvSyncDetails.textContent = p.log_line;
        }
      });
    }
  });

})();
