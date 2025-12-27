// API Keys Management JavaScript

let currentAction = null;
let currentKeyId = null;

// DOM elements
const createKeyModal = document.getElementById('createKeyModal');
const confirmModal = document.getElementById('confirmModal');
const closeCreateModal = document.getElementById('closeCreateModal');
const closeConfirmModal = document.getElementById('closeConfirmModal');
const createKeyBtn = document.getElementById('createKeyBtn');
const createKeyForm = document.getElementById('createKeyForm');
const createdKeyDisplay = document.getElementById('createdKeyDisplay');
const createdKeyValue = document.getElementById('createdKeyValue');
const cancelConfirmBtn = document.getElementById('cancelConfirmBtn');
const confirmBtn = document.getElementById('confirmBtn');
const alertDiv = document.getElementById('alert');

// Open create key modal
createKeyBtn.onclick = function() {
    createKeyModal.style.display = 'block';
}

// Close modals
closeCreateModal.onclick = function() { createKeyModal.style.display = 'none'; }
closeConfirmModal.onclick = function() { confirmModal.style.display = 'none'; }
cancelConfirmBtn.onclick = function() { confirmModal.style.display = 'none'; }

// Close modals when clicking outside
window.onclick = function(event) {
    if (event.target === createKeyModal) createKeyModal.style.display = 'none';
    if (event.target === confirmModal) confirmModal.style.display = 'none';
}

// Create API key form submission
createKeyForm.onsubmit = async function(e) {
    e.preventDefault();
    
    const label = document.getElementById('keyLabel').value;
    const expiresDays = parseInt(document.getElementById('keyExpiration').value) || 0;
    
    // Get selected permissions
    const permissions = [];
    const checkboxes = document.querySelectorAll('input[name="permissions"]:checked');
    checkboxes.forEach(checkbox => permissions.push(checkbox.value));
    
    try {
        const response = await fetch('/api/admin/api-keys', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ label, enabled: true, permissions, expires_days: expiresDays })
        });
        
        const data = await response.json();
        
        if (data.success) {
            createdKeyValue.textContent = data.data.key;
            document.getElementById('createKeyForm').style.display = 'none';
            createdKeyDisplay.style.display = 'block';
            loadApiKeys();
            showAlert('API key created successfully!', 'success');
        } else {
            showAlert('Failed to create API key: ' + (data.message || 'Unknown error'), 'error');
        }
    } catch (error) {
        showAlert('Error creating API key: ' + error.message, 'error');
    }
}

function closeCreatedKeyDisplay() {
    createKeyModal.style.display = 'none';
    document.getElementById('createKeyForm').style.display = 'block';
    createdKeyDisplay.style.display = 'none';
    createKeyForm.reset();
}

function copyToClipboard() {
    navigator.clipboard.writeText(createdKeyValue.textContent).then(
        () => showAlert('API key copied to clipboard!', 'success'),
        (err) => showAlert('Failed to copy API key: ' + err, 'error')
    );
}

// Load API keys on page load
loadApiKeys();

async function loadApiKeys() {
    try {
        const response = await fetch('/api/admin/api-keys');
        const data = await response.json();
        
        if (data.success) {
            renderApiKeys(data.data);
        } else {
            showAlert('Failed to load API keys: ' + (data.message || 'Unknown error'), 'error');
        }
    } catch (error) {
        showAlert('Error loading API keys: ' + error.message, 'error');
    }
}

function renderApiKeys(keys) {
    const tbody = document.getElementById('apiKeysTableBody');
    
    if (keys.length === 0) {
        tbody.innerHTML = '<tr><td colspan="7" style="text-align: center; padding: 20px;"><p>No API keys found. Create your first API key!</p></td></tr>';
        return;
    }
    
    let html = '';
    keys.forEach(key => {
        const createdDate = new Date(key.created_at).toLocaleString();
        const expiresDate = key.expires_at ? new Date(key.expires_at).toLocaleString() : 'Never';
        const permissions = key.permissions.join(', ');
        const statusClass = key.enabled ? 'status-active' : 'status-disabled';
        const statusText = key.enabled ? 'Active' : 'Disabled';
        
        html += `<tr>
            <td>${escapeHtml(key.label)}</td>
            <td><span class="key-display">${escapeHtml(key.key_partial)}</span></td>
            <td>${createdDate}</td>
            <td>${expiresDate}</td>
            <td>${escapeHtml(permissions)}</td>
            <td><span class="status-badge ${statusClass}">${statusText}</span></td>
            <td class="actions">
                ${key.enabled 
                    ? `<button class="btn btn-danger" onclick="toggleKeyStatus('${key.id}', false)">Disable</button>`
                    : `<button class="btn btn-success" onclick="toggleKeyStatus('${key.id}', true)">Enable</button>`}
                <button class="btn btn-danger" onclick="confirmDelete('${key.id}')">Delete</button>
            </td>
        </tr>`;
    });
    
    tbody.innerHTML = html;
}

function toggleKeyStatus(keyId, enable) {
    currentAction = enable ? 'enable' : 'disable';
    currentKeyId = keyId;
    document.getElementById('confirmModalTitle').textContent = enable ? 'Enable API Key' : 'Disable API Key';
    document.getElementById('confirmModalMessage').textContent = enable 
        ? 'Are you sure you want to enable this API key?' 
        : 'Are you sure you want to disable this API key?';
    confirmModal.style.display = 'block';
}

function confirmDelete(keyId) {
    currentAction = 'delete';
    currentKeyId = keyId;
    document.getElementById('confirmModalTitle').textContent = 'Delete API Key';
    document.getElementById('confirmModalMessage').textContent = 'Are you sure you want to permanently delete this API key? This action cannot be undone.';
    confirmModal.style.display = 'block';
}

confirmBtn.onclick = async function() {
    if (!currentAction || !currentKeyId) return;

    try {
        let url = '';
        let method = 'POST';

        if (currentAction === 'enable') {
            url = `/api/admin/api-keys/${currentKeyId}/enable`;
        } else if (currentAction === 'disable') {
            url = `/api/admin/api-keys/${currentKeyId}/disable`;
        } else if (currentAction === 'delete') {
            url = `/api/admin/api-keys/${currentKeyId}`;
            method = 'DELETE';
        }

        const response = await fetch(url, {
            method: method,
            headers: { 'Content-Type': 'application/json' }
        });

        const data = await response.json();

        if (data.success) {
            showAlert(`API key ${currentAction}d successfully!`, 'success');
            loadApiKeys();
        } else {
            showAlert(`Failed to ${currentAction} API key: ` + (data.message || 'Unknown error'), 'error');
        }
    } catch (error) {
        showAlert(`Error ${currentAction}ing API key: ` + error.message, 'error');
    }

    confirmModal.style.display = 'none';
    currentAction = null;
    currentKeyId = null;
}

function showAlert(message, type) {
    alertDiv.textContent = message;
    alertDiv.className = 'alert alert-' + type;
    setTimeout(() => { alertDiv.style.display = 'none'; }, 5000);
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

