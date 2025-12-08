// Rust Edge Gateway Admin UI

const API_BASE = '/api';

const DEFAULT_HANDLER = `//! Handler for this endpoint
use rust_edge_gateway_sdk::prelude::*;

/// Handle incoming requests
fn handle(req: Request) -> Response {
    Response::ok(json!({
        "message": "Hello, World!",
        "path": req.path,
        "method": req.method
    }))
}

handler_loop!(handle);
`;

let editor = null;

function app() {
    return {
        view: 'endpoints',
        endpoints: [],
        services: [],
        currentEndpoint: {
            id: null,
            name: '',
            domain: '',
            path: '',
            method: 'GET',
            code: DEFAULT_HANDLER,
            compiled: false,
            enabled: false
        },
        currentService: {
            id: null,
            name: '',
            service_type: 'postgres',
            config: {},
            configJson: '{}',
            enabled: true
        },
        // Import state
        importFile: null,
        importOptions: {
            domain: '',
            domain_id: '',
            create_collection: false,
            compile: true,
            start: false
        },
        importResult: null,
        importing: false,
        dragover: false,

        loading: false,
        message: '',
        messageType: 'success',

        async init() {
            await this.loadEndpoints();
            await this.loadServices();
            this.$watch('view', (val) => {
                if (val === 'endpoint-editor') {
                    this.$nextTick(() => this.initEditor());
                }
            });
        },

        switchView(newView) {
            this.message = '';
            this.view = newView;
        },

        async loadEndpoints() {
            this.loading = true;
            try {
                const res = await fetch(`${API_BASE}/endpoints`);
                const data = await res.json();
                if (data.ok || data.success) {
                    this.endpoints = data.data || [];
                }
            } catch (e) {
                console.error('Failed to load endpoints:', e);
            }
            this.loading = false;
        },

        async loadServices() {
            try {
                const res = await fetch(`${API_BASE}/services`);
                const data = await res.json();
                if (data.ok || data.success) {
                    this.services = data.data || [];
                }
            } catch (e) {
                console.error('Failed to load services:', e);
            }
        },

        initEditor() {
            if (editor) {
                editor.setValue(this.currentEndpoint.code || DEFAULT_HANDLER);
                return;
            }

            require.config({ paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs' } });
            require(['vs/editor/editor.main'], () => {
                editor = monaco.editor.create(document.getElementById('editor'), {
                    value: this.currentEndpoint.code || DEFAULT_HANDLER,
                    language: 'rust',
                    theme: 'vs-dark',
                    minimap: { enabled: false },
                    automaticLayout: true,
                    fontSize: 14
                });
            });
        },

        // ===== Endpoint Methods =====
        newEndpoint() {
            this.currentEndpoint = {
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
            this.view = 'endpoint-editor';
        },

        async editEndpoint(ep) {
            this.currentEndpoint = { ...ep };
            if (ep.id) {
                try {
                    const res = await fetch(`${API_BASE}/endpoints/${ep.id}/code`);
                    const data = await res.json();
                    if (data.ok || data.success) {
                        this.currentEndpoint.code = data.data || DEFAULT_HANDLER;
                    }
                } catch (e) {
                    console.error('Failed to load code:', e);
                }
            }
            this.message = '';
            this.view = 'endpoint-editor';
        },

        async saveEndpoint() {
            const code = editor ? editor.getValue() : this.currentEndpoint.code;
            this.currentEndpoint.code = code;

            try {
                let res;
                if (this.currentEndpoint.id) {
                    res = await fetch(`${API_BASE}/endpoints/${this.currentEndpoint.id}`, {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(this.currentEndpoint)
                    });
                    await fetch(`${API_BASE}/endpoints/${this.currentEndpoint.id}/code`, {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ code })
                    });
                } else {
                    res = await fetch(`${API_BASE}/endpoints`, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ ...this.currentEndpoint, code })
                    });
                }

                const data = await res.json();
                if (data.ok || data.success) {
                    this.currentEndpoint = data.data;
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
            if (!this.currentEndpoint.id) return;

            this.showMessage('Compiling...', 'success');
            try {
                const res = await fetch(`${API_BASE}/endpoints/${this.currentEndpoint.id}/compile`, { method: 'POST' });
                const data = await res.json();
                if (data.ok || data.success) {
                    this.currentEndpoint.compiled = true;
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
            if (!this.currentEndpoint.id) return;
            const action = this.currentEndpoint.enabled ? 'stop' : 'start';

            try {
                const res = await fetch(`${API_BASE}/endpoints/${this.currentEndpoint.id}/${action}`, { method: 'POST' });
                const data = await res.json();
                if (data.ok || data.success) {
                    this.currentEndpoint.enabled = !this.currentEndpoint.enabled;
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
                if (data.ok || data.success) {
                    await this.loadEndpoints();
                }
            } catch (e) {
                console.error('Delete failed:', e);
            }
        },

        // ===== Service Methods =====
        newService() {
            this.currentService = {
                id: null,
                name: '',
                service_type: 'postgres',
                config: {},
                configJson: '{\n  "host": "localhost",\n  "port": 5432,\n  "database": "mydb"\n}',
                enabled: true
            };
            this.message = '';
            this.view = 'service-editor';
        },

        async editService(svc) {
            this.currentService = {
                ...svc,
                configJson: JSON.stringify(svc.config || {}, null, 2)
            };
            this.message = '';
            this.view = 'service-editor';
        },

        async saveService() {
            try {
                this.currentService.config = JSON.parse(this.currentService.configJson);
            } catch (e) {
                this.showMessage('Invalid JSON in configuration', 'error');
                return;
            }

            try {
                let res;
                const payload = {
                    name: this.currentService.name,
                    service_type: this.currentService.service_type,
                    config: this.currentService.config,
                    enabled: this.currentService.enabled
                };

                if (this.currentService.id) {
                    res = await fetch(`${API_BASE}/services/${this.currentService.id}`, {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(payload)
                    });
                } else {
                    res = await fetch(`${API_BASE}/services`, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(payload)
                    });
                }

                const data = await res.json();
                if (data.ok || data.success) {
                    this.currentService = { ...data.data, configJson: JSON.stringify(data.data.config || {}, null, 2) };
                    this.showMessage('Service saved!', 'success');
                    await this.loadServices();
                } else {
                    this.showMessage(data.error || 'Save failed', 'error');
                }
            } catch (e) {
                this.showMessage('Failed to save: ' + e.message, 'error');
            }
        },

        async testService(id) {
            if (!id) return;

            this.showMessage('Testing connection...', 'success');
            try {
                const res = await fetch(`${API_BASE}/services/${id}/test`, { method: 'POST' });
                const data = await res.json();
                if (data.ok || data.success) {
                    this.showMessage('Connection successful!', 'success');
                } else {
                    this.showMessage(data.error || 'Connection failed', 'error');
                }
            } catch (e) {
                this.showMessage('Test failed: ' + e.message, 'error');
            }
        },

        async deleteService(id) {
            if (!confirm('Delete this service?')) return;

            try {
                const res = await fetch(`${API_BASE}/services/${id}`, { method: 'DELETE' });
                const data = await res.json();
                if (data.ok || data.success) {
                    await this.loadServices();
                    this.showMessage('Service deleted', 'success');
                }
            } catch (e) {
                console.error('Delete failed:', e);
            }
        },

        // ===== Import Methods =====
        handleDrop(e) {
            this.dragover = false;
            const files = e.dataTransfer?.files;
            if (files && files.length > 0) {
                this.importFile = files[0];
            }
        },

        handleFileSelect(e) {
            const files = e.target?.files;
            if (files && files.length > 0) {
                this.importFile = files[0];
            }
        },

        async importBundle() {
            if (!this.importFile) {
                this.showMessage('Please select a file', 'error');
                return;
            }
            if (!this.importOptions.domain) {
                this.showMessage('Please enter a domain', 'error');
                return;
            }

            this.importing = true;
            this.importResult = null;
            this.message = '';

            try {
                const formData = new FormData();
                formData.append('bundle', this.importFile);

                const params = new URLSearchParams();
                params.set('domain', this.importOptions.domain);
                if (this.importOptions.domain_id) params.set('domain_id', this.importOptions.domain_id);
                if (this.importOptions.create_collection) params.set('create_collection', 'true');
                if (this.importOptions.compile) params.set('compile', 'true');
                if (this.importOptions.start) params.set('start', 'true');

                const res = await fetch(`${API_BASE}/import/bundle?${params.toString()}`, {
                    method: 'POST',
                    body: formData
                });

                const data = await res.json();
                if (data.ok || data.success) {
                    this.importResult = data.data;
                    this.showMessage('Import successful!', 'success');
                    await this.loadEndpoints();
                } else {
                    this.showMessage(data.error || 'Import failed', 'error');
                }
            } catch (e) {
                this.showMessage('Import failed: ' + e.message, 'error');
            }

            this.importing = false;
        },

        showMessage(msg, type) {
            this.message = msg;
            this.messageType = type;
        }
    };
}

