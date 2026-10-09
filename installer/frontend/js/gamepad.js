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
    let bestElem = null;
    let minDistance = Infinity;

    focusables.forEach((elem) => {
      if (elem === currentFocusElem) return;
      const rect = elem.getBoundingClientRect();

      let isCandidate = false;
      let dist = Infinity;

      if (direction === "up" && rect.bottom <= currentRect.top + 20) {
        dist = Math.hypot(rect.left - currentRect.left, currentRect.top - rect.bottom);
        isCandidate = true;
      } else if (direction === "down" && rect.top >= currentRect.bottom - 20) {
        dist = Math.hypot(rect.left - currentRect.left, rect.top - currentRect.bottom);
        isCandidate = true;
      } else if (direction === "left" && rect.right <= currentRect.left + 20) {
        dist = Math.hypot(currentRect.left - rect.right, rect.top - currentRect.top);
        isCandidate = true;
      } else if (direction === "right" && rect.left >= currentRect.right - 20) {
        dist = Math.hypot(rect.left - currentRect.right, rect.top - currentRect.top);
        isCandidate = true;
      }

      if (isCandidate && dist < minDistance) {
        minDistance = dist;
        bestElem = elem;
      }
    });

    if (bestElem) {
      setFocus(bestElem);
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
        setFocus(elems[0]);
      }
    }
  };

  window.addEventListener("gamepadconnected", (e) => {
    console.log("Manette connectée :", e.gamepad.id);
    window.GamepadEngine.refreshFocus();
  });

  window.addEventListener("DOMContentLoaded", () => {
    requestAnimationFrame(pollGamepad);
    setTimeout(window.GamepadEngine.refreshFocus, 300);
  });
})();
