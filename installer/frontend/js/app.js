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
      document.getElementById("gpu-info-text").textContent = gpu.detected_name;
      const badge = document.getElementById("gpu-badge");
      badge.textContent = `Profil recommandé : ${gpu.recommended_profile.toUpperCase()}`;

      selectedGpuProfile = gpu.recommended_profile;
      enableHDR = gpu.hdr_supported;

      // Présélectionner dans l'étape 4
      updateGpuSelection(selectedGpuProfile);
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

    // Lancement de l'Installation
    document.getElementById("btn-start-install").addEventListener("click", async () => {
      if (!selectedDisk) {
        alert("Veuillez sélectionner un disque de destination.");
        setStep(3);
        return;
      }

      setStep(5);
      const hostname = document.getElementById("input-hostname").value || "noos-htpc";

      // Écoute des événements de progression en direct
      tauriListen("install_progress", (event) => {
        const payload = event.payload;
        document.getElementById("install-current-step").textContent = payload.step_name;
        document.getElementById("progress-fill").style.width = `${payload.percent}%`;
        document.getElementById("progress-text").textContent = `${payload.percent}%`;

        const logs = document.getElementById("terminal-logs");
        const line = document.createElement("div");
        line.className = "log-line";
        line.textContent = `[${payload.step}/${payload.total_steps}] ${payload.log_line}`;
        logs.appendChild(line);
        logs.scrollTop = logs.scrollHeight;

        if (payload.percent >= 100) {
          setTimeout(() => setStep(6), 1500);
        }
      });

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
        alert(`Erreur d'installation : ${err}`);
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
