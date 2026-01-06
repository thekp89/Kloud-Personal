
document.addEventListener('DOMContentLoaded', () => {
    const listContainer = document.getElementById('file-list');
    const breadcrumbsContainer = document.getElementById('breadcrumbs');
    const previewModal = document.getElementById('preview-modal');
    const previewContent = document.getElementById('preview-content');
    const closeModal = document.getElementById('close-modal');

    // Initial State Check (if injected by server)
    if (window.INITIAL_DATA) {
        renderDirectory(window.INITIAL_DATA);
    }

    // Navigation Handler
    async function navigate(path) {
        try {
            const url = `/list${path}?format=json`;
            const response = await fetch(url);
            if (!response.ok) throw new Error('Network response was not ok');
            const data = await response.json();

            // Update History
            history.pushState({ path }, '', `/list${path}`);

            renderDirectory(data);
        } catch (error) {
            console.error('Error navigating:', error);
            alert('Failed to load directory.');
        }
    }

    // Back Button Handler
    window.addEventListener('popstate', (event) => {
        if (event.state && event.state.path) {
            // Re-fetch to ensure freshness, or could cache.
            // Simplified: just reload for now or re-fetch.
            // Let's re-fetch.
            fetch(`/list${event.state.path}?format=json`)
                .then(r => r.json())
                .then(renderDirectory)
                .catch(e => window.location.reload());
        }
    });

    // Render Function
    function renderDirectory(data) {
        // Update Breadcrumbs
        breadcrumbsContainer.innerHTML = buildBreadcrumbs(data.current_path);

        // Update List
        listContainer.innerHTML = '';

        // Parent Directory Link (if not root)
        if (data.current_path !== '/' && data.current_path !== '') {
            const li = document.createElement('li');
            li.innerHTML = `<a href="#" onclick="event.preventDefault(); navigate('${getParentPath(data.current_path)}')" class="secondary">⬅ ..</a>`;
            listContainer.appendChild(li);
        }

        data.entries.forEach(entry => {
            const li = document.createElement('li');
            const icon = entry.is_dir ? '📁' : '📄';
            const name = entry.name;
            // Proper path joining
            const rawPath = data.current_path.endsWith('/') ? `${data.current_path}${name}` : `${data.current_path}/${name}`;

            if (entry.is_dir) {
                li.innerHTML = `
                    <div class="grid">
                        <div>
                            <a href="#" onclick="event.preventDefault(); navigate('${rawPath}')">
                                ${icon} ${name}
                            </a>
                        </div>
                        <div class="actions">
                             <a href="/download${rawPath}" role="button" class="outline contrast" style="font-size: 0.7em; padding: 2px 8px;">ZIP</a>
                        </div>
                    </div>
                `;
            } else {
                li.innerHTML = `
                     <div class="grid">
                        <div>
                            <a href="#" onclick="event.preventDefault(); openPreview('${name}', '${rawPath}', ${entry.size})">
                                ${icon} ${name}
                            </a>
                            <small class="muted">(${formatSize(entry.size)})</small>
                        </div>
                         <div class="actions">
                             <a href="/download${rawPath}" role="button" class="outline contrast" style="font-size: 0.7em; padding: 2px 8px;">⬇</a>
                        </div>
                    </div>
                `;
            }
            listContainer.appendChild(li);
        });
    }

    function buildBreadcrumbs(path) {
        if (!path || path === '/') return '<ul><li>/</li></ul>';
        const parts = path.split('/').filter(p => p);
        let html = '<ul><li><a href="#" onclick="event.preventDefault(); navigate(\'/\')">Home</a></li>';
        let current = '';
        parts.forEach((part, index) => {
            current += '/' + part;
            if (index === parts.length - 1) {
                html += `<li>${part}</li>`;
            } else {
                html += `<li><a href="#" onclick="event.preventDefault(); navigate('${current}')">${part}</a></li>`;
            }
        });
        html += '</ul>';
        return html;
    }

    function getParentPath(path) {
        if (!path || path === '/') return '/';
        const parts = path.split('/').filter(p => p);
        parts.pop();
        return '/' + parts.join('/');
    }

    // Preview Logic
    window.openPreview = function (name, path, size) {
        previewContent.innerHTML = '<p aria-busy="true">Loading...</p>';
        previewModal.showModal();

        const ext = name.split('.').pop().toLowerCase();
        const downloadUrl = `/download${path}`;

        if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'].includes(ext)) {
            previewContent.innerHTML = `<img src="${downloadUrl}" alt="${name}" style="max-height: 80vh; max-width: 100%;">`;
        } else if (['mp4', 'webm', 'ogg'].includes(ext)) {
            previewContent.innerHTML = `<video src="${downloadUrl}" controls style="max-height: 80vh; max-width: 100%;"></video>`;
        } else if (['mp3', 'wav'].includes(ext)) {
            previewContent.innerHTML = `<audio src="${downloadUrl}" controls style="width: 100%;"></audio>`;
        } else if (['pdf'].includes(ext)) {
            previewContent.innerHTML = `<iframe src="${downloadUrl}" style="width: 100%; height: 80vh; border: none;"></iframe>`;
        } else if (['txt', 'rs', 'js', 'css', 'html', 'json', 'toml', 'md'].includes(ext)) {
            fetch(downloadUrl)
                .then(r => r.text())
                .then(text => {
                    previewContent.innerHTML = `<pre><code>${escapeHtml(text)}</code></pre>`;
                })
                .catch(err => {
                    previewContent.innerHTML = `<p>Error loading content.</p>`;
                });
        } else {
            previewContent.innerHTML = `
                <article>
                    <header>Preview not available</header>
                    <p>File type .${ext} is not supported for preview.</p>
                    <a href="${downloadUrl}" role="button">Download File</a>
                </article>
            `;
        }
    };

    closeModal.addEventListener('click', () => {
        previewModal.close();
        previewContent.innerHTML = '';
    });

    // Close on click outside
    previewModal.addEventListener('click', (event) => {
        if (event.target === previewModal) {
            previewModal.close();
            previewContent.innerHTML = '';
        }
    });

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    function formatSize(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }
});
