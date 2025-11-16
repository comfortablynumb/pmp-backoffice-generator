// Global state
let currentBackoffice = null;
let currentSection = null;
let currentAction = null;
let backoffices = [];

// Initialize the application
$(document).ready(function() {
    loadBackoffices();
});

// Load all backoffices
function loadBackoffices() {
    $.get('/api/backoffices', function(data) {
        backoffices = data;
        renderBackofficeTabs();

        if (backoffices.length > 0) {
            selectBackoffice(backoffices[0].id);
        }
    }).fail(function(err) {
        showError('Failed to load backoffices: ' + err.responseText);
    });
}

// Render backoffice tabs in the top navigation
function renderBackofficeTabs() {
    const $tabs = $('#backoffice-tabs');
    $tabs.empty();

    backoffices.forEach(function(backoffice) {
        const $tab = $('<button>')
            .addClass('px-4 py-2 rounded-t-lg hover:bg-indigo-700 transition-colors')
            .attr('data-id', backoffice.id)
            .text(backoffice.name)
            .click(function() {
                selectBackoffice(backoffice.id);
            });

        $tabs.append($tab);
    });
}

// Select a backoffice
function selectBackoffice(backofficeId) {
    currentBackoffice = backoffices.find(b => b.id === backofficeId);

    if (!currentBackoffice) return;

    // Update active tab
    $('#backoffice-tabs button').removeClass('bg-indigo-800');
    $(`#backoffice-tabs button[data-id="${backofficeId}"]`).addClass('bg-indigo-800');

    // Render sections
    renderSections();

    // Select first section if available
    if (currentBackoffice.sections.length > 0) {
        selectSection(currentBackoffice.sections[0].id);
    }
}

// Render sections in the sidebar
function renderSections() {
    const $list = $('#sections-list');
    $list.empty();

    if (!currentBackoffice) return;

    currentBackoffice.sections.forEach(function(section) {
        const icon = section.icon || 'fa-folder';
        const $item = $('<li>')
            .addClass('cursor-pointer p-3 rounded-lg hover:bg-indigo-50 transition-colors')
            .attr('data-id', section.id)
            .html(`<i class="fas ${icon} mr-2"></i>${section.name}`)
            .click(function() {
                selectSection(section.id);
            });

        $list.append($item);
    });
}

// Select a section
function selectSection(sectionId) {
    if (!currentBackoffice) return;

    currentSection = currentBackoffice.sections.find(s => s.id === sectionId);

    if (!currentSection) return;

    // Update active section
    $('#sections-list li').removeClass('bg-indigo-100 font-semibold');
    $(`#sections-list li[data-id="${sectionId}"]`).addClass('bg-indigo-100 font-semibold');

    // Render section content
    renderSectionContent();
}

// Render section content with actions
function renderSectionContent() {
    const $content = $('#content-area');
    $content.empty();

    if (!currentSection) return;

    // Section header
    const $header = $('<div>').addClass('mb-6');
    $header.append($('<h1>').addClass('text-3xl font-bold text-gray-800').text(currentSection.name));

    // Action buttons
    const $actions = $('<div>').addClass('flex space-x-3 mt-4');

    currentSection.actions.forEach(function(action) {
        const buttonClass = getActionButtonClass(action.action_type);
        const icon = getActionIcon(action.action_type);

        const $btn = $('<button>')
            .addClass(`px-4 py-2 rounded-lg ${buttonClass} transition-colors`)
            .html(`<i class="fas ${icon} mr-2"></i>${action.name}`)
            .click(function() {
                executeAction(action);
            });

        $actions.append($btn);
    });

    $header.append($actions);
    $content.append($header);

    // Content area for displaying data
    const $dataArea = $('<div>').attr('id', 'data-area').addClass('bg-white rounded-lg shadow p-6');
    $content.append($dataArea);

    // Auto-load list action if available
    const listAction = currentSection.actions.find(a => a.action_type === 'list');
    if (listAction) {
        executeAction(listAction);
    }
}

// Execute an action
function executeAction(action) {
    currentAction = action;

    switch(action.action_type) {
        case 'list':
            loadListData(action);
            break;
        case 'create':
            showCreateForm(action);
            break;
        case 'update':
            showUpdateForm(action);
            break;
        case 'delete':
            confirmDelete(action);
            break;
        case 'view':
            loadViewData(action);
            break;
        default:
            showError('Action type not supported: ' + action.action_type);
    }
}

// Load list data
function loadListData(action) {
    const url = `/api/backoffices/${currentBackoffice.id}/sections/${currentSection.id}/actions/${action.id}`;

    $('#data-area').html('<div class="text-center py-8"><div class="loading mx-auto"></div><p class="mt-4 text-gray-500">Loading...</p></div>');

    $.get(url, function(response) {
        renderTable(response.data, response.fields);
    }).fail(function(err) {
        showError('Failed to load data: ' + (err.responseJSON?.error || err.responseText));
    });
}

// Render data table
function renderTable(data, fields) {
    const $dataArea = $('#data-area');
    $dataArea.empty();

    if (!data || data.length === 0) {
        $dataArea.html('<p class="text-gray-500 text-center py-8">No data available</p>');
        return;
    }

    const visibleFields = fields.filter(f => f.visible);

    const $table = $('<table>').addClass('min-w-full divide-y divide-gray-200');

    // Table header
    const $thead = $('<thead>').addClass('bg-gray-50');
    const $headerRow = $('<tr>');

    visibleFields.forEach(function(field) {
        $headerRow.append(
            $('<th>')
                .addClass('px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider')
                .text(field.name)
        );
    });

    $headerRow.append($('<th>').addClass('px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider').text('Actions'));
    $thead.append($headerRow);
    $table.append($thead);

    // Table body
    const $tbody = $('<tbody>').addClass('bg-white divide-y divide-gray-200');

    data.forEach(function(row) {
        const $tr = $('<tr>').addClass('hover:bg-gray-50');

        visibleFields.forEach(function(field) {
            const value = row[field.id] || '';
            $tr.append(
                $('<td>')
                    .addClass('px-6 py-4 whitespace-nowrap text-sm text-gray-900')
                    .text(value)
            );
        });

        // Action buttons for each row
        const $actionCell = $('<td>').addClass('px-6 py-4 whitespace-nowrap text-sm font-medium');

        // Edit button
        const updateAction = currentSection.actions.find(a => a.action_type === 'update');
        if (updateAction) {
            $actionCell.append(
                $('<button>')
                    .addClass('text-indigo-600 hover:text-indigo-900 mr-3')
                    .html('<i class="fas fa-edit"></i>')
                    .click(function() {
                        showUpdateForm(updateAction, row);
                    })
            );
        }

        // Delete button
        const deleteAction = currentSection.actions.find(a => a.action_type === 'delete');
        if (deleteAction) {
            $actionCell.append(
                $('<button>')
                    .addClass('text-red-600 hover:text-red-900')
                    .html('<i class="fas fa-trash"></i>')
                    .click(function() {
                        confirmDelete(deleteAction, row);
                    })
            );
        }

        $tr.append($actionCell);
        $tbody.append($tr);
    });

    $table.append($tbody);

    const $tableContainer = $('<div>').addClass('table-container');
    $tableContainer.append($table);
    $dataArea.append($tableContainer);
}

// Show create form
function showCreateForm(action) {
    $('#modal-title').text('Create ' + currentSection.name);
    renderForm(action.fields, {});
    $('#formModal').addClass('active');

    $('#dynamic-form').off('submit').on('submit', function(e) {
        e.preventDefault();
        submitForm(action);
    });
}

// Show update form
function showUpdateForm(action, data = {}) {
    $('#modal-title').text('Update ' + currentSection.name);
    renderForm(action.fields, data);
    $('#formModal').addClass('active');

    $('#dynamic-form').off('submit').on('submit', function(e) {
        e.preventDefault();
        submitForm(action, data);
    });
}

// Render dynamic form fields
function renderForm(fields, data) {
    const $formFields = $('#form-fields');
    $formFields.empty();

    fields.forEach(function(field) {
        if (!field.editable) return;

        const $fieldGroup = $('<div>').addClass('form-group');

        const $label = $('<label>')
            .addClass('block text-sm font-medium text-gray-700 mb-1')
            .attr('for', field.id)
            .text(field.name + (field.required ? ' *' : ''));

        $fieldGroup.append($label);

        const value = data[field.id] || field.default_value || '';
        let $input;

        switch(field.field_type) {
            case 'textarea':
                $input = $('<textarea>')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                    .attr('rows', 4)
                    .val(value);
                break;

            case 'select':
                $input = $('<select>')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500');

                if (field.validation && field.validation.options) {
                    field.validation.options.forEach(function(option) {
                        $input.append($('<option>').val(option).text(option));
                    });
                }
                $input.val(value);
                break;

            case 'boolean':
                $input = $('<input>')
                    .attr('type', 'checkbox')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .addClass('h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded')
                    .prop('checked', value === true || value === 'true');
                break;

            case 'date':
                $input = $('<input>')
                    .attr('type', 'date')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                    .val(value);
                break;

            case 'number':
                $input = $('<input>')
                    .attr('type', 'number')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                    .val(value);
                break;

            case 'email':
                $input = $('<input>')
                    .attr('type', 'email')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                    .val(value);
                break;

            case 'password':
                $input = $('<input>')
                    .attr('type', 'password')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                    .val(value);
                break;

            default: // text
                $input = $('<input>')
                    .attr('type', 'text')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                    .val(value);
        }

        if (field.required) {
            $input.attr('required', true);
        }

        $fieldGroup.append($input);
        $formFields.append($fieldGroup);
    });
}

// Submit form
function submitForm(action, existingData = {}) {
    const formData = $('#dynamic-form').serializeArray();
    const data = {};

    formData.forEach(function(field) {
        data[field.name] = field.value;
    });

    // Merge with existing data (for updates)
    const payload = Object.assign({}, existingData, data);

    $('#submit-text').hide();
    $('#submit-loading').removeClass('hidden');

    const url = `/api/backoffices/${currentBackoffice.id}/sections/${currentSection.id}/actions/${action.id}`;

    $.ajax({
        url: url,
        method: 'POST',
        contentType: 'application/json',
        data: JSON.stringify(payload),
        success: function(response) {
            closeModal();
            showSuccess('Operation completed successfully');

            // Reload list if available
            const listAction = currentSection.actions.find(a => a.action_type === 'list');
            if (listAction) {
                executeAction(listAction);
            }
        },
        error: function(err) {
            showError('Operation failed: ' + (err.responseJSON?.error || err.responseText));
        },
        complete: function() {
            $('#submit-text').show();
            $('#submit-loading').addClass('hidden');
        }
    });
}

// Confirm delete
function confirmDelete(action, data = {}) {
    if (confirm('Are you sure you want to delete this item?')) {
        const url = `/api/backoffices/${currentBackoffice.id}/sections/${currentSection.id}/actions/${action.id}`;

        $.ajax({
            url: url,
            method: 'POST',
            contentType: 'application/json',
            data: JSON.stringify(data),
            success: function() {
                showSuccess('Item deleted successfully');

                // Reload list
                const listAction = currentSection.actions.find(a => a.action_type === 'list');
                if (listAction) {
                    executeAction(listAction);
                }
            },
            error: function(err) {
                showError('Delete failed: ' + (err.responseJSON?.error || err.responseText));
            }
        });
    }
}

// Close modal
function closeModal() {
    $('#formModal').removeClass('active');
    $('#dynamic-form')[0].reset();
}

// Helper functions
function getActionButtonClass(actionType) {
    switch(actionType) {
        case 'create': return 'bg-green-600 hover:bg-green-700 text-white';
        case 'update': return 'bg-blue-600 hover:bg-blue-700 text-white';
        case 'delete': return 'bg-red-600 hover:bg-red-700 text-white';
        case 'list': return 'bg-indigo-600 hover:bg-indigo-700 text-white';
        default: return 'bg-gray-600 hover:bg-gray-700 text-white';
    }
}

function getActionIcon(actionType) {
    switch(actionType) {
        case 'create': return 'fa-plus';
        case 'update': return 'fa-edit';
        case 'delete': return 'fa-trash';
        case 'list': return 'fa-list';
        case 'view': return 'fa-eye';
        default: return 'fa-cog';
    }
}

function showError(message) {
    alert('Error: ' + message);
    console.error(message);
}

function showSuccess(message) {
    alert(message);
    console.log(message);
}
