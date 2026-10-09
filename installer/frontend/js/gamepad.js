/* ==============================================================================
   Noos HTPC - Gamepad Navigation Engine
   Gestion complète des manettes (Xbox, PS4/PS5, 8BitDo, Switch)
   ============================================================================== */

(function () {
  let currentFocusElem = null;
  let lastNavTime = 0;
  const NAV_DELAY = 180; // ms de délai anti-rebond pour la navigation

  // État des boutons pour détection du front montant (press unique)
  const prevButtonsState = {};

  const gamepadBadge = document.getElementById("gamepad-badge");
  const gamepadName = document.getElementById("gamepad-name");

  function getFocusableElements() {
    // Si le clavier virtuel est ouvert, restreindre la navigation à celui-ci
    const scope = window.OSK && window.OSK.isOpen()
      ? document.getElementById("osk-overlay")
      : document.querySelector(".wizard-step.active") || document;

    const elems = Array.from(scope.querySelectorAll("[data-focusable]")).filter((el) => {
      return el.offsetParent !== null && !el.disabled;
    });
    return elems;
  }

  function setFocus(elem) {
    if (currentFocusElem) {
      currentFocusElem.classList.remove("gamepad-focus");
    }
    currentFocusElem = elem;
    if (currentFocusElem) {
      currentFocusElem.classList.add("gamepad-focus");
      currentFocusElem.focus();
      currentFocusElem.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  }

  function navigate(direction) {
    const focusables = getFocusableElements();
    if (focusables.length === 0) return;

    if (!currentFocusElem || !focusables.includes(currentFocusElem)) {
      setFocus(focusables[0]);
      return;
    }

    const currentRect = currentFocusElem.getBoundingClientRect();
    const currentCenter = {
      x: currentRect.left + currentRect.width / 2,
      y: currentRect.top + currentRect.height / 2,
    };

    const candidates = [];

    focusables.forEach((elem) => {
      if (elem === currentFocusElem) return;
      const rect = elem.getBoundingClientRect();
      const center = {
        x: rect.left + rect.width / 2,
        y: rect.top + rect.height / 2,
      };

      if (direction === "up" && rect.bottom <= currentRect.top + 25) {
        const primaryDist = currentRect.top - rect.bottom;
        const secondaryDist = Math.abs(center.x - currentCenter.x);
        candidates.push({ elem, primaryDist, secondaryDist });
      } else if (direction === "down" && rect.top >= currentRect.bottom - 25) {
        const primaryDist = rect.top - currentRect.bottom;
        const secondaryDist = Math.abs(center.x - currentCenter.x);
        candidates.push({ elem, primaryDist, secondaryDist });
      } else if (direction === "left" && rect.right <= currentRect.left + 25) {
        const primaryDist = currentRect.left - rect.right;
        const secondaryDist = Math.abs(center.y - currentCenter.y);
        candidates.push({ elem, primaryDist, secondaryDist });
      } else if (direction === "right" && rect.left >= currentRect.right - 25) {
        const primaryDist = rect.left - currentRect.right;
        const secondaryDist = Math.abs(center.y - currentCenter.y);
        candidates.push({ elem, primaryDist, secondaryDist });
      }
    });

    if (candidates.length === 0) return;

    // Trouver le palier le plus proche sur l'axe principal
    const minPrimary = Math.min(...candidates.map((c) => c.primaryDist));
    // Tolérance de 60px pour regrouper les éléments d'une même ligne / colonne
    const bandCandidates = candidates.filter((c) => c.primaryDist <= minPrimary + 60);

    // Parmi ceux-ci, sélectionner le plus proche sur l'axe secondaire
    bandCandidates.sort((a, b) => a.secondaryDist - b.secondaryDist);

    if (bandCandidates.length > 0) {
      setFocus(bandCandidates[0].elem);
    }
  }

  function handleButtonPress(btnIndex) {
    const isOskOpen = window.OSK && window.OSK.isOpen();

    // Bouton A (Index 0) : Valider / Clic
    if (btnIndex === 0) {
      if (currentFocusElem) {
        currentFocusElem.click();
      }
    }
    // Bouton B (Index 1) : Retour / Fermer
    else if (btnIndex === 1) {
      if (isOskOpen) {
        window.OSK.close();
      } else {
        const backBtn = document.querySelector(".wizard-step.active button[id^='btn-back-']");
        if (backBtn) backBtn.click();
      }
    }
    // Bouton X (Index 2) : Clavier Virtuel / Effacer
    else if (btnIndex === 2) {
      if (isOskOpen) {
        window.OSK.backspace();
      } else if (currentFocusElem && (currentFocusElem.tagName === "INPUT" || currentFocusElem.id === "btn-open-osk")) {
        const input = currentFocusElem.tagName === "INPUT" ? currentFocusElem : document.getElementById("wifi-password-input");
        if (input) window.OSK.open(input);
      }
    }
    // Bouton Y (Index 3) : Espace
    else if (btnIndex === 3) {
      if (isOskOpen) {
        window.OSK.type(" ");
      }
    }
    // Bouton Start (Index 9) : Valider étape ou Lancer installation
    else if (btnIndex === 9) {
      if (isOskOpen) {
        window.OSK.submit();
      } else {
        const nextBtn = document.querySelector(".wizard-step.active button[id^='btn-goto-'], .wizard-step.active #btn-start-install");
        if (nextBtn && !nextBtn.disabled) nextBtn.click();
      }
    }
  }

  // Boucle de scrutation de la manette (60 FPS)
  function pollGamepad() {
    const gamepads = navigator.getGamepads ? navigator.getGamepads() : [];
    let activePad = null;

    for (let i = 0; i < gamepads.length; i++) {
      if (gamepads[i] && gamepads[i].connected) {
        activePad = gamepads[i];
        break;
      }
    }

    if (activePad) {
      // Mise à jour de l'indicateur
      gamepadBadge.querySelector(".status-dot").className = "status-dot connected";
      gamepadName.textContent = activePad.id.slice(0, 24);

      const now = Date.now();

      // Navigation D-Pad & Sticks analogiques
      const axisX = activePad.axes[0] || 0;
      const axisY = activePad.axes[1] || 0;
      const dpadUp = activePad.buttons[12] && activePad.buttons[12].pressed;
      const dpadDown = activePad.buttons[13] && activePad.buttons[13].pressed;
      const dpadLeft = activePad.buttons[14] && activePad.buttons[14].pressed;
      const dpadRight = activePad.buttons[15] && activePad.buttons[15].pressed;

      if (now - lastNavTime > NAV_DELAY) {
        if (dpadUp || axisY < -0.5) {
          navigate("up");
          lastNavTime = now;
        } else if (dpadDown || axisY > 0.5) {
          navigate("down");
          lastNavTime = now;
        } else if (dpadLeft || axisX < -0.5) {
          navigate("left");
          lastNavTime = now;
        } else if (dpadRight || axisX > 0.5) {
          navigate("right");
          lastNavTime = now;
        }
      }

      // Événements d'appui sur les boutons (front montant)
      activePad.buttons.forEach((btn, idx) => {
        const wasPressed = prevButtonsState[idx] || false;
        if (btn.pressed && !wasPressed) {
          handleButtonPress(idx);
          // Visualisation de la manette dans l'étape 1
          triggerVisualIndicator(idx);
        }
        prevButtonsState[idx] = btn.pressed;
      });
    } else {
      gamepadBadge.querySelector(".status-dot").className = "status-dot disconnected";
      gamepadName.textContent = "Aucune manette détectée";
    }

    requestAnimationFrame(pollGamepad);
  }

  function triggerVisualIndicator(btnIdx) {
    let id = null;
    if (btnIdx === 0) id = "ind-a";
    if (btnIdx === 1) id = "ind-b";
    if (btnIdx === 2) id = "ind-x";
    if (btnIdx === 3) id = "ind-y";
    if (btnIdx >= 12 && btnIdx <= 15) id = "ind-dpad";

    if (id) {
      const el = document.getElementById(id);
      if (el) {
        el.classList.add("pressed");
        setTimeout(() => el.classList.remove("pressed"), 200);
      }
    }
  }

  window.GamepadEngine = {
    setFocus: setFocus,
    getFocus: () => currentFocusElem,
    refreshFocus: () => {
      const elems = getFocusableElements();
      if (elems.length > 0 && (!currentFocusElem || !elems.includes(currentFocusElem))) {
        // Privilégier un élément déjà sélectionné, puis le bouton d'action principal
        const preferred = elems.find(el => el.classList.contains("selected"))
          || elems.find(el => el.classList.contains("primary") || el.classList.contains("danger"))
          || elems[0];
        setFocus(preferred);
      }
    }
  };

  // Navigation au clavier physique (idéal pour VM ou clavier d'appoint)
  window.addEventListener("keydown", (e) => {
    const isTyping = document.activeElement && document.activeElement.tagName === "INPUT" && !window.OSK?.isOpen();
    if (isTyping && e.key !== "ArrowUp" && e.key !== "ArrowDown" && e.key !== "Escape" && e.key !== "Enter") {
      return;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      navigate("up");
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      navigate("down");
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      navigate("left");
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      navigate("right");
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      handleButtonPress(0);
    } else if (e.key === "Escape") {
      e.preventDefault();
      handleButtonPress(1);
    } else if (e.key === "Tab") {
      e.preventDefault();
      const elems = getFocusableElements();
      if (elems.length === 0) return;
      let idx = elems.indexOf(currentFocusElem);
      if (e.shiftKey) {
        idx = idx <= 0 ? elems.length - 1 : idx - 1;
      } else {
        idx = idx < 0 || idx >= elems.length - 1 ? 0 : idx + 1;
      }
      setFocus(elems[idx]);
    }
  });

  window.addEventListener("gamepadconnected", (e) => {
    console.log("Manette connectée :", e.gamepad.id);
    window.GamepadEngine.refreshFocus();
  });

  window.addEventListener("DOMContentLoaded", () => {
    requestAnimationFrame(pollGamepad);
    setTimeout(window.GamepadEngine.refreshFocus, 300);
  });
})();
