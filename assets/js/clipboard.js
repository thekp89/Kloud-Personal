let notepadModal;
let notepadTextarea;
let notepadStatus;

let isTyping = false;
let typingTimeout;
let pollInterval;

function initNotepad() {
    notepadModal = document.getElementById('notepad-modal');
    notepadTextarea = document.getElementById('notepad-content');
    notepadStatus = document.getElementById('notepad-status');

    if (!notepadModal || !notepadTextarea) return;

    notepadTextarea.addEventListener('input', () => {
        isTyping = true;
        notepadStatus.innerText = 'Typing...';

        clearTimeout(typingTimeout);
        typingTimeout = setTimeout(() => {
            isTyping = false;
            saveClipboard();
        }, 500);
    });

    // Detect open/close to manage polling
    // Since we use showModal(), we can also listen to 'close' event
    notepadModal.addEventListener('close', () => {
        stopPolling();
    });
}

function toggleNotepad() {
    if (notepadModal.hasAttribute('open')) {
        notepadModal.close();
    } else {
        notepadModal.showModal();
        startPolling();
    }
}

async function fetchClipboard() {
    if (isTyping) return; // Don't overwrite if user is typing

    try {
        const response = await fetch('/api/clipboard');
        if (response.ok) {
            const text = await response.text();

            // Only update if changed
            if (notepadTextarea.value !== text) {
                // Save cursor position if focused
                const isFocused = document.activeElement === notepadTextarea;
                const start = notepadTextarea.selectionStart;
                const end = notepadTextarea.selectionEnd;

                notepadTextarea.value = text;

                if (isFocused) {
                    notepadTextarea.setSelectionRange(start, end);
                }
            }
        }
    } catch (e) {
        console.error("Error fetching clipboard:", e);
    }
}

async function saveClipboard() {
    const text = notepadTextarea.value;
    notepadStatus.innerText = 'Saving...';
    try {
        await fetch('/api/clipboard', {
            method: 'POST',
            body: text
        });
        notepadStatus.innerText = 'Saved';
    } catch (e) {
        notepadStatus.innerText = 'Error saving';
        console.error("Error saving clipboard:", e);
    }
}

function startPolling() {
    fetchClipboard();
    pollInterval = setInterval(fetchClipboard, 2000);
}

function stopPolling() {
    clearInterval(pollInterval);
}

// Copy/Paste Utils
async function copyToClipboard() {
    try {
        await navigator.clipboard.writeText(notepadTextarea.value);
        // Visual feedback
        const btn = document.getElementById('btn-copy');
        const originalText = btn.innerText;
        btn.innerText = 'Copied!';
        setTimeout(() => btn.innerText = originalText, 1500);
    } catch (err) {
        console.error('Failed to copy!', err);
    }
}

async function pasteFromClipboard() {
    try {
        const text = await navigator.clipboard.readText();
        // Insert at cursor position or replace?
        // Usually paste appends or inserts.
        // For simplicity, let's just append or replace if empty?
        // Ideally insert at cursor.
        const start = notepadTextarea.selectionStart;
        const end = notepadTextarea.selectionEnd;
        const val = notepadTextarea.value;

        notepadTextarea.value = val.slice(0, start) + text + val.slice(end);

        // Move cursor
        notepadTextarea.setSelectionRange(start + text.length, start + text.length);

        // Trigger save
        notepadTextarea.dispatchEvent(new Event('input'));
    } catch (err) {
        console.error('Failed to paste!', err);
        alert('Paste failed. You might need to allow clipboard permissions.');
    }
}

// Global expose
window.toggleNotepad = toggleNotepad;
window.copyToClipboard = copyToClipboard;
window.pasteFromClipboard = pasteFromClipboard;

document.addEventListener('DOMContentLoaded', initNotepad);
