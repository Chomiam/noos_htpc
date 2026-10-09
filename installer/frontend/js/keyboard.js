/* ==============================================================================
   Noos HTPC - Virtual Keyboard Engine (OSK)
   Gestion de la saisie au clavier virtuel pour TV & Gamepad
   ============================================================================== */

(function () {
  let targetInput = null;
  let isPassword = false;
  let isMasked = true;
  let currentText = "";

  const overlay = document.getElementById("osk-overlay");
  const preview = document.getElementById("osk-preview");
  const maskBtn = document.getElementById("osk-mask-btn");
  const grid = document.getElementById("osk-grid");

  // Disposition des touches TV (AZERTY / Chiffres / Symboles usuels)
  const rows = [
    ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
    ["A", "Z", "E", "R", "T", "Y", "U", "I", "O", "P"],
    ["Q", "S", "D", "F", "G", "H", "J", "K", "L", "M"],
    ["W", "X", "C", "V", "B", "N", ".", "-", "_", "@", "!", "$"]
  ];

  function initKeyboard() {
    grid.innerHTML = "";
    rows.forEach((row) => {
      const rowDiv = document.createElement("div");
      rowDiv.className = "keyboard-row";
      row.forEach((char) => {
        const btn = document.createElement("button");
        btn.className = "osk-key";
        btn.textContent = char;
        btn.dataset.char = char;
        btn.setAttribute("data-focusable", "");
        btn.addEventListener("click", () => typeCharacter(char));
        rowDiv.appendChild(btn);
      });
      grid.appendChild(rowDiv);
    });

    // Événements boutons spéciaux
    document.querySelectorAll(".osk-key.action").forEach((btn) => {
      btn.addEventListener("click", () => {
        const action = btn.dataset.action;
        if (action === "space") typeCharacter(" ");
        if (action === "backspace") backspace();
        if (action === "enter") submit();
        if (action === "close") close();
      });
    });

    maskBtn.addEventListener("click", () => {
      isMasked = !isMasked;
      maskBtn.textContent = isMasked ? "👁️ Afficher" : "🔒 Masquer";
      updatePreview();
    });
  }

  function updatePreview() {
    if (isPassword && isMasked) {
      preview.textContent = "•".repeat(currentText.length);
    } else {
      preview.textContent = currentText || "(vide)";
    }
  }

  function typeCharacter(char) {
    currentText += char;
    if (targetInput) targetInput.value = currentText;
    updatePreview();
  }

  function backspace() {
    if (currentText.length > 0) {
      currentText = currentText.slice(0, -1);
      if (targetInput) targetInput.value = currentText;
      updatePreview();
    }
  }

  function open(inputElem) {
    targetInput = inputElem;
    currentText = inputElem.value || "";
    isPassword = inputElem.type === "password";
    isMasked = true;
    maskBtn.style.display = isPassword ? "inline-block" : "none";
    updatePreview();
    overlay.classList.add("visible");

    // Donner le focus manette à la première touche
    setTimeout(() => {
      const firstKey = grid.querySelector(".osk-key");
      if (firstKey && window.GamepadEngine) {
        window.GamepadEngine.setFocus(firstKey);
      }
    }, 100);
  }

  function close() {
    overlay.classList.remove("visible");
    if (targetInput && window.GamepadEngine) {
      window.GamepadEngine.setFocus(targetInput);
    }
    targetInput = null;
  }

  function submit() {
    if (targetInput) {
      targetInput.dispatchEvent(new Event("change", { bubbles: true }));
    }
    close();
  }

  window.OSK = {
    init: initKeyboard,
    open: open,
    close: close,
    type: typeCharacter,
    backspace: backspace,
    submit: submit,
    isOpen: () => overlay.classList.contains("visible")
  };

  document.addEventListener("DOMContentLoaded", initKeyboard);
})();
