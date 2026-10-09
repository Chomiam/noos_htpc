/* ==============================================================================
   Noos HTPC - Main Installer Wizard Logic
   Coordination des étapes, appels Tauri et gestion des événements
   ============================================================================== */

(function () {
  let currentStep = 1;
  let selectedDisk = null;
  let selectedGpuProfile = "amd";
  let enableHDR = true;
  let selectedWifiSsid = null;

  // Accès sécurisé à l'API Tauri v2 avec fallback de test
  const tauriInvoke = window.__TAURI__ && window.__TAURI__.core
    ? window.__TAURI__.core.invoke
    : async (cmd, args) => {
        console.warn(`[Tauri Mock] Commande invoquée : ${cmd}`, args);
        if (cmd === "detect_gpu") {
          return {
            detected_name: "AMD Radeon 780M (Ryzen 7)",
            vendor: "AMD",
            recommended_profile: "amd",
            hdr_supported: true
          };
        }
        if (cmd === "list_disks") {
          return [
            { name: "nvme0n1", path: "/dev/nvme0n1", size_human: "512.0 Go", model: "Samsung 980 NVMe SSD", bus_type: "NVME", is_rotational: false },
            { name: "sda", path: "/dev/sda", size_human: "1.0 To", model: "Crucial MX500 SATA SSD", bus_type: "SATA", is_rotational: false }
          ];
        }
        if (cmd === "get_network_status") {
          return { is_connected: true, connection_type: "Ethernet Filaire", ip_address: "192.168.1.50" };
        }
        if (cmd === "scan_wifi") {
          return [
            { ssid: "Noos_Salon_5G", signal_percent: 92, security: "WPA2", is_connected: false },
            { ssid: "Box_Internet_Fibres", signal_percent: 78, security: "WPA2", is_connected: false }
          ];
        }
        return true;
      };

  const tauriListen = window.__TAURI__ && window.__TAURI__.event
    ? window.__TAURI__.event.listen
    : (event, callback) => {
        console.warn(`[Tauri Mock] Écoute de l'événement : ${event}`);
      };

  function setStep(stepNum) {
    document.querySelectorAll(".wizard-step").forEach((el) => el.classList.remove("active"));
    document.querySelectorAll(".step-pill").forEach((el) => el.classList.remove("active"));

    const targetStep = document.getElementById(`step-${stepNum}`);
    const targetPill = document.getElementById(`pill-step-${stepNum}`);

    if (targetStep) targetStep.classList.add("active");
    if (targetPill) targetPill.classList.add("active");

    currentStep = stepNum;

    // Réinitialiser le focus manette
    setTimeout(() => {
      if (window.GamepadEngine) window.GamepadEngine.refreshFocus();
    }, 150);
  }

  // Initialisation Étape 1 : Matériel
  async function initStep1() {
    try {
      const gpu = await tauriInvoke("detect_gpu");
      const name = gpu.detected_name || gpu.detectedName || "Carte graphique détectée";
      const profile = gpu.recommended_profile || gpu.recommendedProfile || "generic";
      const hdr = gpu.hdr_supported !== undefined ? gpu.hdr_supported : (gpu.hdrSupported !== undefined ? gpu.hdrSupported : false);

      document.getElementById("gpu-info-text").textContent = name;
      const badge = document.getElementById("gpu-badge");
      if (badge) badge.textContent = `Profil recommandé : ${profile.toUpperCase()}`;

      selectedGpuProfile = profile;
      enableHDR = hdr;

      // Présélectionner dans l'étape 4
      updateGpuSelection(selectedGpuProfile);
      const hdrBtn = document.getElementById("btn-toggle-hdr");
      if (hdrBtn) {
        hdrBtn.textContent = enableHDR ? "HDR : Activé" : "HDR : Désactivé";
        hdrBtn.className = enableHDR ? "tv-btn toggle active" : "tv-btn toggle";
      }
    } catch (err) {
      document.getElementById("gpu-info-text").textContent = "Détection générique standard";
    }
  }

  // Initialisation Étape 2 : Réseau
  async function initStep2() {
    const statusText = document.getElementById("current-net-status");
    try {
      const status = await tauriInvoke("get_network_status");
      if (status.is_connected) {
        statusText.innerHTML = `<span style="color: var(--ctp-green);">Connecté via ${status.connection_type} (${status.ip_address})</span>`;
      } else {
        statusText.innerHTML = `<span style="color: var(--ctp-yellow);">Non connecté. Veuillez choisir un réseau Wi-Fi.</span>`;
      }

      // Scanner le Wi-Fi
      const wifis = await tauriInvoke("scan_wifi");
      const grid = document.getElementById("wifi-grid");
      grid.innerHTML = "";

      if (wifis.length === 0) {
        grid.innerHTML = "<div>Aucun réseau Wi-Fi trouvé ou interface non disponible.</div>";
        return;
      }

      wifis.forEach((w) => {
        const card = document.createElement("div");
        card.className = "wifi-card";
        card.setAttribute("data-focusable", "");
        card.innerHTML = `<strong>📶 ${w.ssid}</strong> (Signal: ${w.signal_percent}%)`;
        card.addEventListener("click", () => {
          selectedWifiSsid = w.ssid;
          document.getElementById("selected-ssid-name").textContent = w.ssid;
          const box = document.getElementById("wifi-password-box");
          box.style.display = "block";
          const input = document.getElementById("wifi-password-input");
          input.value = "";
          if (window.OSK) window.OSK.open(input);
        });
        grid.appendChild(card);
      });
    } catch (e) {
      statusText.textContent = "Erreur lors de l'analyse réseau";
    }
  }

  // Initialisation Étape 3 : Disques
  async function initStep3() {
    const grid = document.getElementById("disks-grid");
    const confirmBtn = document.getElementById("btn-goto-step-4");
    grid.innerHTML = '<div class="loading-spinner">Détection des disques de stockage...</div>';

    try {
      const disks = await tauriInvoke("list_disks");
      grid.innerHTML = "";

      if (disks.length === 0) {
        grid.innerHTML = "<div>Aucun disque dur ou SSD détecté.</div>";
        confirmBtn.disabled = true;
        return;
      }

      disks.forEach((d) => {
        const card = document.createElement("div");
        card.className = "disk-card";
        card.setAttribute("data-focusable", "");
        card.innerHTML = `
          <h4>💾 ${d.model}</h4>
          <p><strong>Taille :</strong> ${d.size_human} | <strong>Bus :</strong> ${d.bus_type}</p>
          <p style="font-size: 16px; color: var(--ctp-subtext);">Périphérique : ${d.path}</p>
        `;
        card.addEventListener("click", () => {
          document.querySelectorAll(".disk-card").forEach((c) => c.classList.remove("selected"));
          card.classList.add("selected");
          selectedDisk = d.path;
          confirmBtn.disabled = false;
        });
        grid.appendChild(card);
      });

      // Sélection automatique du premier disque si disponible
      if (disks.length > 0) {
        const first = grid.querySelector(".disk-card");
        if (first) first.click();
      }
    } catch (e) {
      grid.innerHTML = "<div>Erreur lors de la détection des disques.</div>";
    }
  }

  // Options & Profil GPU
  function updateGpuSelection(profile) {
    document.querySelectorAll(".gpu-card").forEach((card) => {
      if (card.dataset.profile === profile) {
        card.classList.add("selected");
      } else {
        card.classList.remove("selected");
      }
    });
  }

  function setupEventListeners() {
    // Navigation Étapes
    document.getElementById("btn-goto-step-2").addEventListener("click", () => { setStep(2); initStep2(); });
    document.getElementById("btn-back-step-1").addEventListener("click", () => setStep(1));
    document.getElementById("btn-goto-step-3").addEventListener("click", () => { setStep(3); initStep3(); });
    document.getElementById("btn-back-step-2").addEventListener("click", () => setStep(2));
    document.getElementById("btn-goto-step-4").addEventListener("click", () => setStep(4));
    document.getElementById("btn-back-step-3").addEventListener("click", () => setStep(3));

    // Sélection GPU
    document.querySelectorAll(".gpu-card").forEach((card) => {
      card.addEventListener("click", () => {
        selectedGpuProfile = card.dataset.profile;
        updateGpuSelection(selectedGpuProfile);
        if (selectedGpuProfile === "vm" || selectedGpuProfile === "generic" || selectedGpuProfile === "nvidia-legacy") {
          enableHDR = false;
        } else {
          enableHDR = true;
        }
        const hdrBtn = document.getElementById("btn-toggle-hdr");
        if (hdrBtn) {
          hdrBtn.textContent = enableHDR ? "HDR : Activé" : "HDR : Désactivé";
          hdrBtn.className = enableHDR ? "tv-btn toggle active" : "tv-btn toggle";
        }
      });
    });

    // Toggle HDR
    const hdrBtn = document.getElementById("btn-toggle-hdr");
    hdrBtn.addEventListener("click", () => {
      enableHDR = !enableHDR;
      hdrBtn.textContent = enableHDR ? "HDR : Activé" : "HDR : Désactivé";
      hdrBtn.className = enableHDR ? "tv-btn toggle active" : "tv-btn toggle";
    });

    // Bouton Clavier Virtuel
    document.getElementById("btn-open-osk").addEventListener("click", () => {
      const input = document.getElementById("wifi-password-input");
      if (window.OSK) window.OSK.open(input);
    });

    // Connexion Wi-Fi
    document.getElementById("btn-submit-wifi").addEventListener("click", async () => {
      const pwd = document.getElementById("wifi-password-input").value;
      if (!selectedWifiSsid) return;
      alert(`Connexion au réseau ${selectedWifiSsid}...`);
      try {
        await tauriInvoke("connect_wifi", { ssid: selectedWifiSsid, password: pwd });
        initStep2();
      } catch (e) {
        alert(`Échec de connexion : ${e}`);
      }
    });

    // Fonction centralisée de rafraîchissement UI d'installation
    function updateInstallUI(payload) {
      if (!payload) return;
      const stepTitle = document.getElementById("install-current-step");
      const progressFill = document.getElementById("progress-fill");
      const progressText = document.getElementById("progress-text");
      const progressPhase = document.getElementById("progress-phase");
      const packageCounter = document.getElementById("package-counter");
      const packageNumbers = document.getElementById("package-numbers");
      const logs = document.getElementById("terminal-logs");

      if (stepTitle && payload.step_name) stepTitle.textContent = payload.step_name;

      if (progressPhase && payload.step && payload.total_steps) {
        progressPhase.textContent = `Étape ${payload.step} / ${payload.total_steps} : ${payload.step_name}`;
      }

      if (packageCounter && packageNumbers) {
        if (payload.packages_total > 0) {
          packageCounter.style.display = "inline-flex";
          const pct = Math.min(100, Math.round((payload.packages_done / payload.packages_total) * 100));
          packageNumbers.textContent = `${payload.packages_done} / ${payload.packages_total} (${pct}%)`;
        } else if (payload.step === 6) {
          packageCounter.style.display = "inline-flex";
          packageNumbers.textContent = "Calcul de l'arbre Nix...";
        } else {
          packageCounter.style.display = "none";
        }
      }

      if (progressFill && payload.percent !== undefined) {
        progressFill.style.width = `${payload.percent}%`;
        if (payload.step_name && payload.step_name.toLowerCase().includes("erreur")) {
          progressFill.style.background = "var(--ctp-red)";
        } else {
          progressFill.style.background = "linear-gradient(90deg, #74c7ec, #cba6f7, #a6e3a1)";
        }
      }
      if (progressText && payload.percent !== undefined) progressText.textContent = `${payload.percent}%`;

      if (logs && payload.log_line) {
        const line = document.createElement("div");
        const lower = payload.log_line.toLowerCase();
        let typeClass = "normal";

        if (lower.includes("error:") || lower.includes("failed") || (payload.step_name && payload.step_name.toLowerCase().includes("erreur"))) {
          typeClass = "error";
        } else if (lower.includes("copying path") || lower.includes("fetching path") || lower.includes("fetching")) {
          typeClass = "fetch";
        } else if (lower.includes("building") || lower.includes("compilation")) {
          typeClass = "build";
        } else if (lower.includes("terminé") || lower.includes("succès")) {
          typeClass = "success";
        } else if (lower.includes("étape") || lower.includes("préparation") || lower.includes("partitionnement") || lower.includes("formatage")) {
          typeClass = "notice";
        }

        line.className = `log-line ${typeClass}`;
        const timeStr = new Date().toLocaleTimeString("fr-FR", { hour12: false });
        line.innerHTML = `<span class="log-time">[${timeStr}]</span> <span class="log-tag">[${payload.step}/${payload.total_steps}]</span> <span class="log-msg">${payload.log_line}</span>`;
        logs.appendChild(line);

        // Limite pour préserver les performances WebKit
        if (logs.childNodes.length > 500) {
          logs.removeChild(logs.firstChild);
        }

        // Défilement automatique fluide vers le bas
        logs.scrollTop = logs.scrollHeight;
      }

      if (payload.percent >= 100) {
        setTimeout(() => setStep(6), 1800);
      }
    }

    // Hook global direct pour appel depuis Rust via Webview eval
    window.onInstallProgress = updateInstallUI;

    // Écoute des événements de progression en direct (Tauri events)
    tauriListen("install_progress", (event) => {
      updateInstallUI(event.payload);
    });

    // Lancement de l'Installation
    document.getElementById("btn-start-install").addEventListener("click", async () => {
      if (!selectedDisk) {
        alert("Veuillez sélectionner un disque de destination.");
        setStep(3);
        return;
      }

      setStep(5);
      const hostname = "noos-htpc"; // Forcé et déclaratif

      // Polling actif toutes les 400ms pour garantir un rafraîchissement temps réel
      const pollTimer = setInterval(async () => {
        try {
          const status = await tauriInvoke("get_install_progress");
          if (status) {
            updateInstallUI(status);
            if (status.percent >= 100) clearInterval(pollTimer);
          }
        } catch (_) {}
      }, 400);

      try {
        await tauriInvoke("start_installation", {
          req: {
            target_disk: selectedDisk,
            gpu_profile: selectedGpuProfile,
            enable_hdr: enableHDR,
            hostname: hostname
          }
        });
      } catch (err) {
        clearInterval(pollTimer);
        const logs = document.getElementById("terminal-logs");
        if (logs) {
          const line = document.createElement("div");
          line.className = "log-line error";
          line.textContent = `[ERREUR CRITIQUE] ${err}`;
          logs.appendChild(line);
        }
      }
    });

    // Redémarrage final
    document.getElementById("btn-reboot-now").addEventListener("click", async () => {
      await tauriInvoke("reboot_system");
    });
  }

  document.addEventListener("DOMContentLoaded", () => {
    setupEventListeners();
    initStep1();
  });
})();
