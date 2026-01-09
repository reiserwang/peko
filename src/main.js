// Peko Settings Window

const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

const EMOJIS = ['✨', '📓', '🌐', '💬', '🔍', '📧', '🎵', '📺', '🐙', '📝', '🎮', '🛒'];

let websites = [];

document.addEventListener('DOMContentLoaded', init);

async function init() {
  try {
    const settings = await invoke('get_settings');
    websites = settings.websites || [];
  } catch (error) {
    console.error('Failed to load settings:', error);
    websites = [];
  }

  render();
  setupEventListeners();
}

function setupEventListeners() {
  document.getElementById('add-btn').addEventListener('click', addWebsite);
  document.getElementById('save-btn').addEventListener('click', saveAndClose);

  // Close emoji pickers on outside click
  document.addEventListener('click', (e) => {
    if (!e.target.closest('.emoji-picker') && !e.target.closest('.website-emoji')) {
      document.querySelectorAll('.emoji-picker').forEach(p => p.remove());
    }
  });
}

function render() {
  const list = document.getElementById('websites-list');
  list.innerHTML = '';

  websites.forEach((website, index) => {
    const item = document.createElement('div');
    item.className = 'website-item';
    item.innerHTML = `
      <span class="website-number">${index + 1}</span>
      <button class="website-emoji" data-index="${index}" title="Change emoji">${website.emoji}</button>
      <div class="website-fields">
        <input type="text" class="input-name" placeholder="Name" value="${escapeHtml(website.name)}" data-index="${index}" data-field="name">
        <input type="url" class="input-url" placeholder="https://example.com" value="${escapeHtml(website.url)}" data-index="${index}" data-field="url">
      </div>
      <button class="delete-btn" data-index="${index}" title="Remove">🗑️</button>
    `;
    list.appendChild(item);
  });

  // Event listeners for inputs
  list.querySelectorAll('input').forEach(input => {
    input.addEventListener('input', (e) => {
      const index = parseInt(e.target.dataset.index);
      const field = e.target.dataset.field;
      websites[index][field] = e.target.value;
    });
  });

  // Delete buttons
  list.querySelectorAll('.delete-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      const index = parseInt(e.target.dataset.index);
      websites.splice(index, 1);
      render();
    });
  });

  // Emoji buttons
  list.querySelectorAll('.website-emoji').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      showEmojiPicker(btn, parseInt(btn.dataset.index));
    });
  });

  // Update add button state
  document.getElementById('add-btn').disabled = websites.length >= 5;
}

function addWebsite() {
  if (websites.length >= 5) return;

  const id = `site_${Date.now()}`;
  websites.push({
    id,
    name: '',
    url: '',
    emoji: EMOJIS[websites.length % EMOJIS.length]
  });

  render();

  // Focus the new name input
  setTimeout(() => {
    const inputs = document.querySelectorAll('.input-name');
    const lastInput = inputs[inputs.length - 1];
    if (lastInput) lastInput.focus();
  }, 50);
}

function showEmojiPicker(button, index) {
  // Remove existing pickers
  document.querySelectorAll('.emoji-picker').forEach(p => p.remove());

  const picker = document.createElement('div');
  picker.className = 'emoji-picker';

  EMOJIS.forEach(emoji => {
    const option = document.createElement('button');
    option.className = 'emoji-option';
    option.textContent = emoji;
    option.addEventListener('click', () => {
      websites[index].emoji = emoji;
      button.textContent = emoji;
      picker.remove();
    });
    picker.appendChild(option);
  });

  // Position picker
  const rect = button.getBoundingClientRect();
  picker.style.position = 'fixed';
  picker.style.top = `${rect.bottom + 4}px`;
  picker.style.left = `${rect.left}px`;

  document.body.appendChild(picker);
}

async function saveAndClose() {
  // Validate
  const valid = websites.filter(w => w.name.trim() && w.url.trim());

  if (valid.length === 0) {
    alert('Please add at least one website with a name and URL.');
    return;
  }

  // Ensure URLs have protocol
  valid.forEach(w => {
    if (!w.url.startsWith('http://') && !w.url.startsWith('https://')) {
      w.url = 'https://' + w.url;
    }
  });

  try {
    await invoke('save_websites', { websites: valid });
    const win = getCurrentWindow();
    await win.close();
  } catch (error) {
    console.error('Failed to save:', error);
    alert('Failed to save: ' + error);
  }
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}
