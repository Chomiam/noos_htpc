/* ==============================================================================
   Noos TV Dashboard - Virtual Keyboard Engine (OSK)
   Clavier virtuel TV compact haute précision (Style Android TV / Gboard TV)
   Adapté pour manette Gamepad, télécommande TV et souris/tactile.
   ============================================================================== */

(function () {
  'use strict';

  let activeInput = null;
  let currentLayout = 'azerty'; // 'azerty' | 'qwerty'
  let isShiftActive = false;
  let isSymbolsMode = false;
  let isGamepadActive = false;
  let focusedRow = 0;
  let focusedCol = 0;

  // Éléments du DOM
  let container = null;
  let panel = null;
  let grid = null;
  let targetLabel = null;
  let btnLayoutToggle = null;
  let btnClose = null;

  // Définitions des dispositions
  const LAYOUTS = {
    azerty: {
      name: 'AZERTY',
      rows: [
        ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
        ['a', 'z', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
        ['q', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'm'],
        [
          { label: '⇧', action: 'shift', class: 'key-fn key-shift' },
          'w', 'x', 'c', 'v', 'b', 'n', '.', '@',
          { label: '⌫', action: 'backspace', class: 'key-fn key-backspace' }
        ],
        [
          { label: '?123', action: 'symbols', class: 'key-fn key-mode' },
          { label: 'http://', char: 'http://', class: 'key-chip' },
          { label: ':', char: ':' },
          { label: '/', char: '/' },
          { label: 'ESPACE', action: 'space', class: 'key-space' },
          { label: '-', char: '-' },
          { label: '_', char: '_' },
          { label: 'VALIDER ↵', action: 'enter', class: 'key-enter' }
        ]
      ]
    },
    qwerty: {
      name: 'QWERTY',
      rows: [
        ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
        ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
        ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'],
        [
          { label: '⇧', action: 'shift', class: 'key-fn key-shift' },
          'z', 'x', 'c', 'v', 'b', 'n', 'm', '.', '@',
          { label: '⌫', action: 'backspace', class: 'key-fn key-backspace' }
        ],
        [
          { label: '?123', action: 'symbols', class: 'key-fn key-mode' },
          { label: 'http://', char: 'http://', class: 'key-chip' },
          { label: ':', char: ':' },
          { label: '/', char: '/' },
          { label: 'ESPACE', action: 'space', class: 'key-space' },
          { label: '-', char: '-' },
          { label: '_', char: '_' },
          { label: 'VALIDER ↵', action: 'enter', class: 'key-enter' }
        ]
      ]
    },
    symbols: {
      name: 'SYMBOLES',
      rows: [
        ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
        ['!', '"', '#', '$', '%', '&', '\'', '(', ')', '+'],
        ['*', '=', '<', '>', '[', ']', '{', '}', '\\', '|'],
        [
          { label: 'ABC', action: 'abc', class: 'key-fn key-mode' },
          '?', '~', '^', ',', ';', ':', '/', '.',
          { label: '⌫', action: 'backspace', class: 'key-fn key-backspace' }
        ],
        [
          { label: 'ABC', action: 'abc', class: 'key-fn key-mode' },
          { label: 'http://', char: 'http://', class: 'key-chip' },
          { label: 'https://', char: 'https://', class: 'key-chip' },
          { label: '.com', char: '.com', class: 'key-chip' },
          { label: 'ESPACE', action: 'space', class: 'key-space' },
          { label: '-', char: '-' },
          { label: '_', char: '_' },
          { label: 'VALIDER ↵', action: 'enter', class: 'key-enter' }
        ]
      ]
    }
  };

  function playSound(type) {
    try {
      if (type === 'tick' && window.playTickSound) window.playTickSound();
      else if (type === 'nav' && window.playNavSound) window.playNavSound();
      else if (type === 'confirm' && window.playConfirmSound) window.playConfirmSound();
      else if (type === 'cancel' && window.playCancelSound) window.playCancelSound();
    } catch (_) {}
  }

  function init() {
    container = document.getElementById('noos-tv-keyboard');
    if (!container) return;

    panel = container.querySelector('.tv-keyboard-panel');
    grid = document.getElementById('tv-keyboard-grid');
    targetLabel = document.getElementById('tv-kb-target-label');
    btnLayoutToggle = document.getElementById('tv-kb-btn-layout');
    btnClose = document.getElementById('tv-kb-btn-close');

    // Empêcher la perte de focus de l'input HTML lors des clics sur le clavier
    if (panel) {
      panel.addEventListener('mousedown', (e) => e.preventDefault());
      panel.addEventListener('pointerdown', (e) => e.preventDefault());
    }

    if (btnLayoutToggle) {
      btnLayoutToggle.addEventListener('click', (e) => {
        e.preventDefault();
        toggleLayout();
      });
    }

    if (btnClose) {
      btnClose.addEventListener('click', (e) => {
        e.preventDefault();
        close();
      });
    }

    // Détection universelle de la sélection d'un champ de saisie
    document.addEventListener('focusin', handleFocusIn);
    document.addEventListener('focusout', handleFocusOut);

    renderKeyboard();
  }

  function renderKeyboard() {
    if (!grid) return;
    grid.innerHTML = '';

    const activeSet = isSymbolsMode ? LAYOUTS.symbols : LAYOUTS[currentLayout];
    if (btnLayoutToggle) {
      btnLayoutToggle.textContent = isSymbolsMode ? 'SYMBOLES' : activeSet.name;
    }

    activeSet.rows.forEach((row, rowIndex) => {
      const rowDiv = document.createElement('div');
      rowDiv.className = 'tv-kb-row';

      row.forEach((item, colIndex) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.tabIndex = -1; // Ne pas interférer avec le Tab DOM standard
        btn.className = 'tv-kb-key';
        btn.dataset.row = rowIndex;
        btn.dataset.col = colIndex;

        if (typeof item === 'string') {
          const char = isShiftActive && !isSymbolsMode ? item.toUpperCase() : item;
          btn.textContent = char;
          btn.dataset.char = char;
          btn.addEventListener('click', (e) => {
            e.preventDefault();
            insertText(char);
          });
        } else {
          btn.textContent = item.label;
          if (item.class) btn.className += ' ' + item.class;
          if (item.action === 'shift' && isShiftActive) {
            btn.classList.add('active');
          }

          btn.addEventListener('click', (e) => {
            e.preventDefault();
            handleActionKey(item);
          });
        }

        // Animation d'appui
        btn.addEventListener('mousedown', () => btn.classList.add('tv-kb-active'));
        btn.addEventListener('mouseup', () => btn.classList.remove('tv-kb-active'));
        btn.addEventListener('mouseleave', () => btn.classList.remove('tv-kb-active'));

        rowDiv.appendChild(btn);
      });

      grid.appendChild(rowDiv);
    });

    updateKeyFocusUI();
  }

  function handleActionKey(item) {
    if (item.char) {
      insertText(item.char);
      return;
    }

    switch (item.action) {
      case 'shift':
        isShiftActive = !isShiftActive;
        playSound('nav');
        renderKeyboard();
        break;
      case 'symbols':
        isSymbolsMode = true;
        playSound('nav');
        renderKeyboard();
        break;
      case 'abc':
        isSymbolsMode = false;
        playSound('nav');
        renderKeyboard();
        break;
      case 'space':
        insertText(' ');
        break;
      case 'backspace':
        backspace();
        break;
      case 'enter':
        submit();
        break;
    }
  }

  function insertText(text) {
    if (!activeInput) return;

    const start = activeInput.selectionStart !== null ? activeInput.selectionStart : activeInput.value.length;
    const end = activeInput.selectionEnd !== null ? activeInput.selectionEnd : activeInput.value.length;
    const val = activeInput.value || '';

    activeInput.value = val.slice(0, start) + text + val.slice(end);
    const newPos = start + text.length;
    activeInput.selectionStart = newPos;
    activeInput.selectionEnd = newPos;

    // Déclencher les événements pour que les formulaires / bindings réagissent immédiatement
    activeInput.dispatchEvent(new Event('input', { bubbles: true, cancelable: true }));
    activeInput.dispatchEvent(new Event('change', { bubbles: true, cancelable: true }));

    // Si on a tapé une majuscule après Shift, revenir aux minuscules
    if (isShiftActive) {
      isShiftActive = false;
      renderKeyboard();
    }

    playSound('tick');
  }

  function backspace() {
    if (!activeInput) return;

    const start = activeInput.selectionStart !== null ? activeInput.selectionStart : activeInput.value.length;
    const end = activeInput.selectionEnd !== null ? activeInput.selectionEnd : activeInput.value.length;
    const val = activeInput.value || '';

    if (start !== end) {
      activeInput.value = val.slice(0, start) + val.slice(end);
      activeInput.selectionStart = activeInput.selectionEnd = start;
    } else if (start > 0) {
      activeInput.value = val.slice(0, start - 1) + val.slice(start);
      activeInput.selectionStart = activeInput.selectionEnd = start - 1;
    }

    activeInput.dispatchEvent(new Event('input', { bubbles: true, cancelable: true }));
    activeInput.dispatchEvent(new Event('change', { bubbles: true, cancelable: true }));

    playSound('cancel');
  }

  function submit() {
    if (!activeInput) {
      close();
      return;
    }

    playSound('confirm');
    activeInput.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', keyCode: 13, which: 13, bubbles: true }));
    activeInput.dispatchEvent(new Event('change', { bubbles: true }));

    // Passer au champ suivant s'il y en a un dans le formulaire
    const form = activeInput.closest('form');
    if (form) {
      const inputs = Array.from(form.querySelectorAll('input:not([type="hidden"]):not([type="checkbox"]):not([type="radio"]), textarea, select'));
      const idx = inputs.indexOf(activeInput);
      if (idx !== -1 && idx < inputs.length - 1) {
        // Champ suivant
        inputs[idx + 1].focus();
        return;
      } else {
        // Dernier champ : validation du formulaire
        const submitBtn = form.querySelector('button[type="submit"], input[type="submit"]');
        if (submitBtn) {
          submitBtn.click();
        }
      }
    }

    close();
  }

  function toggleLayout() {
    if (isSymbolsMode) {
      isSymbolsMode = false;
    } else {
      currentLayout = currentLayout === 'azerty' ? 'qwerty' : 'azerty';
    }
    playSound('nav');
    renderKeyboard();
  }

  function open(inputElem) {
    if (!inputElem || !container) return;

    if (activeInput && activeInput !== inputElem) {
      activeInput.classList.remove('input-keyboard-active');
    }

    activeInput = inputElem;
    activeInput.classList.add('input-keyboard-active');

    // Déterminer le titre du champ
    let labelText = 'Saisie de texte';
    if (inputElem.id) {
      const label = document.querySelector(`label[for="${inputElem.id}"]`);
      if (label) labelText = label.textContent.replace('*', '').trim();
    }
    if (labelText === 'Saisie de texte' && inputElem.placeholder) {
      labelText = inputElem.placeholder;
    }
    if (inputElem.type === 'password') {
      labelText = '🔒 ' + labelText;
    }
    if (targetLabel) targetLabel.textContent = labelText;

    container.classList.add('visible');
    isGamepadActive = false;
    updateKeyFocusUI();

    // S'assurer que le champ n'est pas masqué par le clavier
    setTimeout(() => {
      inputElem.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }, 50);
  }

  function close() {
    if (!container) return;
    container.classList.remove('visible');
    isGamepadActive = false;
    if (activeInput) {
      activeInput.classList.remove('input-keyboard-active');
      activeInput = null;
    }
    updateKeyFocusUI();
    playSound('cancel');
  }

  function isInputEligible(elem) {
    if (!elem || (elem.tagName !== 'INPUT' && elem.tagName !== 'TEXTAREA')) return false;
    const type = (elem.getAttribute('type') || 'text').toLowerCase();
    const ineligible = ['checkbox', 'radio', 'submit', 'button', 'reset', 'range', 'color', 'file', 'hidden'];
    return !ineligible.includes(type);
  }

  function handleFocusIn(e) {
    if (isInputEligible(e.target)) {
      open(e.target);
    }
  }

  function handleFocusOut(e) {
    if (isInputEligible(e.target)) {
      setTimeout(() => {
        const next = document.activeElement;
        // Si le nouvel élément actif n'est pas un input textuel éligible et qu'on ne navigue pas dans le clavier
        if (!isInputEligible(next) && !isGamepadActive) {
          // Si le focus est parti complètement ailleurs, fermer le clavier
          if (!container.contains(next)) {
            close();
          }
        }
      }, 180);
    }
  }

  // ============================================================================
  // GESTION MANETTE & NAVIGATION D-PAD
  // ============================================================================

  function updateKeyFocusUI() {
    if (!grid) return;
    grid.querySelectorAll('.tv-kb-key').forEach((k) => k.classList.remove('tv-kb-focused'));

    if (!isGamepadActive) return;

    const row = grid.children[focusedRow];
    if (row) {
      const key = row.children[focusedCol];
      if (key) {
        key.classList.add('tv-kb-focused');
      }
    }
  }

  function focusKeys(row, col) {
    isGamepadActive = true;
    focusedRow = Math.max(0, Math.min(row, (grid?.children.length || 1) - 1));
    const activeRow = grid?.children[focusedRow];
    const maxCol = (activeRow?.children.length || 1) - 1;
    focusedCol = Math.max(0, Math.min(col, maxCol));
    updateKeyFocusUI();
    playSound('nav');
  }

  function defocus() {
    isGamepadActive = false;
    updateKeyFocusUI();
    if (activeInput) {
      activeInput.focus();
    }
  }

  function moveFocus(rowDelta, colDelta) {
    if (!grid || !isGamepadActive) {
      focusKeys(0, 0);
      return;
    }

    const rowCount = grid.children.length;
    let newRow = focusedRow + rowDelta;

    if (newRow < 0) {
      // Remonter au champ input
      defocus();
      return;
    }

    if (newRow >= rowCount) {
      newRow = rowCount - 1;
    }

    const targetRowEl = grid.children[newRow];
    const colCount = targetRowEl.children.length;
    let newCol = focusedCol + colDelta;

    // Si on a changé de ligne, adapter l'index de colonne
    if (rowDelta !== 0) {
      const oldRowEl = grid.children[focusedRow];
      const oldColCount = oldRowEl.children.length;
      newCol = Math.round((focusedCol / (oldColCount - 1)) * (colCount - 1));
    }

    if (newCol < 0) newCol = 0;
    if (newCol >= colCount) newCol = colCount - 1;

    focusedRow = newRow;
    focusedCol = newCol;
    updateKeyFocusUI();
    playSound('nav');
  }

  function activateKey() {
    if (!grid || !isGamepadActive) return;
    const row = grid.children[focusedRow];
    if (row) {
      const key = row.children[focusedCol];
      if (key) {
        key.click();
      }
    }
  }

  // API Publique TVKeyboard
  window.TVKeyboard = {
    init,
    open,
    close,
    isOpen: () => container && container.classList.contains('visible'),
    isFocused: () => isGamepadActive,
    focusKeys,
    defocus,
    moveFocus,
    canMoveUp: () => focusedRow > 0,
    activateKey,
    type: insertText,
    backspace,
    submit,
    toggleLayout,
    getTargetInput: () => activeInput,
    isInput: isInputEligible
  };

  document.addEventListener('DOMContentLoaded', init);
})();
