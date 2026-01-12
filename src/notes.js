// Notes Panel

const { invoke } = window.__TAURI__.core;

let saveTimeout = null;

document.addEventListener('DOMContentLoaded', init);

async function init() {
    const editor = document.getElementById('notes-editor');
    const preview = document.getElementById('notes-preview');

    // Load saved notes
    try {
        const content = await invoke('get_notes');
        editor.value = content;
        renderPreview();
    } catch (error) {
        console.error('Failed to load notes:', error);
    }

    // Auto-save on input with debounce
    editor.addEventListener('input', () => {
        renderPreview();

        if (saveTimeout) clearTimeout(saveTimeout);
        saveTimeout = setTimeout(async () => {
            try {
                await invoke('save_notes', { content: editor.value });
            } catch (error) {
                console.error('Failed to save notes:', error);
            }
        }, 500);
    });

    // Tab switching
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const mode = btn.dataset.mode;

            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');

            if (mode === 'edit') {
                editor.classList.remove('hidden');
                preview.classList.add('hidden');
            } else {
                editor.classList.add('hidden');
                preview.classList.remove('hidden');
                renderPreview();
            }
        });
    });
}

function renderPreview() {
    const editor = document.getElementById('notes-editor');
    const preview = document.getElementById('notes-preview');

    if (typeof marked !== 'undefined') {
        preview.innerHTML = marked.parse(editor.value || '*No notes yet*');
    } else {
        preview.innerHTML = editor.value.replace(/\n/g, '<br>');
    }
}
