// Edge Hive Admin UI

const API_BASE = '/api';

const DEFAULT_HANDLER = `//! Handler for this endpoint
use edge_hive_sdk::prelude::*;

/// Handle incoming requests
pub fn handle(req: Request) -> Response {
    Response::ok(json!({
        "message": "Hello, World!",
        "path": req.path,
        "method": req.method
    }))
}
`;

let editor = null;

function app() {
    return {
        view: 'list',
        endpoints: [],
        current: {
            id: null,
            name: '',
            domain: '',
            path: '',
            method: 'GET',
            code: DEFAULT_HANDLER,
            compiled: false,
            enabled: false
        },
        loading: false,
        message: '',
        messageType: 'success',

        async init() {
            await this.loadEndpoints();
            this.$watch('view', (val) => {
                if (val === 'editor') {
                    this.$nextTick(() => this.initEditor());
                }
            });
        },

        async loadEndpoints() {
            this.loading = true;
            try {
                const res = await fetch(`${API_BASE}/endpoints`);
                const data = await res.json();
                if (data.success) {
                    this.endpoints = data.data;
                }
            } catch (e) {
                console.error('Failed to load endpoints:', e);
            }
            this.loading = false;
        },

        initEditor() {
            if (editor) {
                editor.setValue(this.current.code || DEFAULT_HANDLER);
                return;
            }

            require.config({ paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs' } });
            require(['vs/editor/editor.main'], () => {
                editor = monaco.editor.create(document.getElementById('editor'), {
                    value: this.current.code || DEFAULT_HANDLER,
                    language: 'rust',
                    theme: 'vs-dark',
                    minimap: { enabled: false },
                    automaticLayout: true,
                    fontSize: 14
                });
            });
        },

        newEndpoint() {
            this.current = {
                id: null,
                name: '',
                domain: '',
                path: '',
                method: 'GET',
                code: DEFAULT_HANDLER,
                compiled: false,
                enabled: false
            };
            this.message = '';
            this.view = 'editor';
        },

        async editEndpoint(ep) {
            this.current = { ...ep };
            // Load code
            if (ep.id) {
                try {
                    const res = await fetch(`${API_BASE}/endpoints/${ep.id}/code`);
                    const data = await res.json();
                    if (data.success) {
                        this.current.code = data.data || DEFAULT_HANDLER;
                    }
                } catch (e) {
                    console.error('Failed to load code:', e);
                }
            }
            this.message = '';
            this.view = 'editor';
        },

        async saveEndpoint() {
            const code = editor ? editor.getValue() : this.current.code;
            this.current.code = code;

            try {
                let res;
                if (this.current.id) {
                    // Update endpoint
                    res = await fetch(`${API_BASE}/endpoints/${this.current.id}`, {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(this.current)
                    });
                    // Update code separately
                    await fetch(`${API_BASE}/endpoints/${this.current.id}/code`, {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ code })
                    });
                } else {
                    // Create endpoint
                    res = await fetch(`${API_BASE}/endpoints`, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ ...this.current, code })
                    });
                }

                const data = await res.json();
                if (data.success) {
                    this.current = data.data;
                    this.showMessage('Saved successfully!', 'success');
                    await this.loadEndpoints();
                } else {
                    this.showMessage(data.error || 'Save failed', 'error');
                }
            } catch (e) {
                this.showMessage('Failed to save: ' + e.message, 'error');
            }
        },

        async compileEndpoint() {
            if (!this.current.id) return;
            
            this.showMessage('Compiling...', 'success');
            try {
                const res = await fetch(`${API_BASE}/endpoints/${this.current.id}/compile`, { method: 'POST' });
                const data = await res.json();
                if (data.success) {
                    this.current.compiled = true;
                    this.showMessage('Compiled successfully!', 'success');
                    await this.loadEndpoints();
                } else {
                    this.showMessage(data.error || 'Compilation failed', 'error');
                }
            } catch (e) {
                this.showMessage('Compilation failed: ' + e.message, 'error');
            }
        },

        async toggleEndpoint() {
            if (!this.current.id) return;
            const action = this.current.enabled ? 'stop' : 'start';
            
            try {
                const res = await fetch(`${API_BASE}/endpoints/${this.current.id}/${action}`, { method: 'POST' });
                const data = await res.json();
                if (data.success) {
                    this.current.enabled = !this.current.enabled;
                    this.showMessage(`Endpoint ${action}ed!`, 'success');
                    await this.loadEndpoints();
                } else {
                    this.showMessage(data.error || `Failed to ${action}`, 'error');
                }
            } catch (e) {
                this.showMessage(`Failed to ${action}: ` + e.message, 'error');
            }
        },

        async deleteEndpoint(id) {
            if (!confirm('Delete this endpoint?')) return;
            
            try {
                const res = await fetch(`${API_BASE}/endpoints/${id}`, { method: 'DELETE' });
                const data = await res.json();
                if (data.success) {
                    await this.loadEndpoints();
                }
            } catch (e) {
                console.error('Delete failed:', e);
            }
        },

        showMessage(msg, type) {
            this.message = msg;
            this.messageType = type;
        }
    };
}

