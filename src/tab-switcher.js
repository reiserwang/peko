// Tab Switcher Overlay
// Shows when ⌘ is held, allows switching tabs with number keys

(function () {
    'use strict';

    // Only run in Tauri context
    if (!window.__TAURI__) return;

    const { invoke } = window.__TAURI__.core;

    let overlay = null;
    let hintElement = null;
    let cmdHoldTimer = null;
    let isOverlayVisible = false;
    let currentTabs = [];
    let activeTabId = null;
    const HOLD_DELAY = 300; // ms before showing overlay

    // Create overlay elements
    function createOverlay() {
        if (overlay) return;

        // Create main overlay container
        overlay = document.createElement('div');
        overlay.id = 'peko-tab-switcher';
        document.body.appendChild(overlay);

        // Create hint element
        hintElement = document.createElement('div');
        hintElement.id = 'peko-tab-hint';
        hintElement.innerHTML = 'Press <kbd>1</kbd>-<kbd>5</kbd> to switch tabs';
        document.body.appendChild(hintElement);
    }

    // Render tabs in overlay
    function renderTabs(tabs, activeId) {
        if (!overlay) createOverlay();

        overlay.innerHTML = tabs.map((tab, index) => `
      <div class="peko-tab-card ${tab.id === activeId ? 'active' : ''}" data-tab-id="${tab.id}">
        <span class="peko-tab-number">${index + 1}</span>
        <span class="peko-tab-emoji">${tab.emoji}</span>
        <span class="peko-tab-name">${escapeHtml(tab.name)}</span>
      </div>
    `).join('');

        // Add click handlers
        overlay.querySelectorAll('.peko-tab-card').forEach(card => {
            card.addEventListener('click', () => {
                const tabId = card.dataset.tabId;
                switchToTab(tabId);
            });
        });
    }

    // Show the overlay
    async function showOverlay() {
        try {
            const settings = await invoke('get_settings');
            currentTabs = settings.websites || [];
            activeTabId = settings.active_tab;

            if (currentTabs.length === 0) return;

            createOverlay();
            renderTabs(currentTabs, activeTabId);
            overlay.classList.add('visible');
            hintElement.classList.add('visible');
            isOverlayVisible = true;
        } catch (error) {
            console.error('Failed to show tab switcher:', error);
        }
    }

    // Hide the overlay
    function hideOverlay() {
        if (overlay) {
            overlay.classList.remove('visible');
        }
        if (hintElement) {
            hintElement.classList.remove('visible');
        }
        isOverlayVisible = false;
        clearTimeout(cmdHoldTimer);
        cmdHoldTimer = null;
    }

    // Switch to a specific tab
    async function switchToTab(tabId) {
        try {
            await invoke('switch_tab', { tabId });
            hideOverlay();
        } catch (error) {
            console.error('Failed to switch tab:', error);
        }
    }

    // Handle keydown
    function handleKeyDown(e) {
        // Check for Meta (Cmd) key press
        if (e.key === 'Meta' && !cmdHoldTimer && !isOverlayVisible) {
            cmdHoldTimer = setTimeout(() => {
                showOverlay();
            }, HOLD_DELAY);
        }

        // Handle number keys while overlay is visible
        if (isOverlayVisible && e.metaKey) {
            const num = parseInt(e.key);
            if (num >= 1 && num <= currentTabs.length) {
                e.preventDefault();
                e.stopPropagation();
                const targetTab = currentTabs[num - 1];
                if (targetTab) {
                    switchToTab(targetTab.id);
                }
            }
        }
    }

    // Handle keyup
    function handleKeyUp(e) {
        if (e.key === 'Meta') {
            hideOverlay();
        }
    }

    // Escape HTML
    function escapeHtml(str) {
        const div = document.createElement('div');
        div.textContent = str || '';
        return div.innerHTML;
    }

    // Initialize
    function init() {
        // Create overlay elements
        createOverlay();

        // Add event listeners
        document.addEventListener('keydown', handleKeyDown, true);
        document.addEventListener('keyup', handleKeyUp, true);

        // Hide on blur (window loses focus)
        window.addEventListener('blur', hideOverlay);

        console.log('[Peko] Tab switcher initialized');
    }

    // Wait for DOM
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
