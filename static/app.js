// Global state
let currentBackoffice = null;
let currentSection = null;
let currentAction = null;
let backoffices = [];
let currentPage = 1;
let currentFilters = {};

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

    // Reset state
    currentPage = 1;
    currentFilters = {};

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
        const buttonClass = getActionButtonClass(action.type);
        const icon = getActionIcon(action.type);

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
    const listAction = currentSection.actions.find(a => a.type === 'list');
    if (listAction) {
        executeAction(listAction);
    }
}

// Execute an action
function executeAction(action) {
    currentAction = action;

    switch(action.type) {
        case 'list':
            loadListData(action);
            break;
        case 'form':
            showForm(action);
            break;
        case 'view':
            loadViewData(action);
            break;
        default:
            showError('Action type not supported: ' + action.type);
    }
}

// Load list data
function loadListData(action, page = 1) {
    currentPage = page;
    const url = `/api/backoffices/${currentBackoffice.id}/sections/${currentSection.id}/actions/${action.id}`;

    const params = { ...currentFilters };
    if (action.config && action.config.enable_pagination) {
        params.page = page;
        params.page_size = action.config.page_size || 20;
    }

    $('#data-area').html('<div class="text-center py-8"><div class="loading mx-auto"></div><p class="mt-4 text-gray-500">Loading...</p></div>');

    $.get(url, params, function(response) {
        renderTable(response.data, response.fields, response.config, response.pagination);
    }).fail(function(err) {
        showError('Failed to load data: ' + (err.responseJSON?.error || err.responseText));
    });
}

// Render data table
function renderTable(data, fields, config, pagination) {
    const $dataArea = $('#data-area');
    $dataArea.empty();

    // Render filters if configured
    if (config && config.filters && config.filters.length > 0) {
        renderFilters(config.filters);
    }

    if (!data || data.length === 0) {
        $dataArea.append('<p class="text-gray-500 text-center py-8">No data available</p>');
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
                    .text(formatFieldValue(value, field))
            );
        });

        // Action buttons for each row
        const $actionCell = $('<td>').addClass('px-6 py-4 whitespace-nowrap text-sm font-medium');

        // Edit button (look for form action with update mode)
        const updateAction = currentSection.actions.find(a =>
            a.type === 'form' && a.config && a.config.form_mode === 'update'
        );
        if (updateAction) {
            $actionCell.append(
                $('<button>')
                    .addClass('text-indigo-600 hover:text-indigo-900 mr-3')
                    .html('<i class="fas fa-edit"></i>')
                    .click(function() {
                        showForm(updateAction, row);
                    })
            );
        }

        // Delete button (look for form action with delete mode)
        const deleteAction = currentSection.actions.find(a =>
            a.type === 'form' && a.config && a.config.form_mode === 'delete'
        );
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

    const $tableContainer = $('<div>').addClass('table-container overflow-x-auto');
    $tableContainer.append($table);
    $dataArea.append($tableContainer);

    // Render pagination if enabled
    if (pagination) {
        renderPagination(pagination);
    }
}

// Render filters
function renderFilters(filters) {
    const $filterArea = $('<div>').addClass('mb-4 p-4 bg-gray-50 rounded-lg');
    const $filterTitle = $('<h3>').addClass('text-sm font-semibold text-gray-700 mb-3').text('Filters');
    $filterArea.append($filterTitle);

    const $filterForm = $('<div>').addClass('grid grid-cols-1 md:grid-cols-3 gap-4');

    filters.forEach(function(filter) {
        const $filterGroup = $('<div>');
        const $label = $('<label>')
            .addClass('block text-sm font-medium text-gray-700 mb-1')
            .text(filter.name);

        let $input;
        switch(filter.filter_type) {
            case 'text':
                $input = $('<input>')
                    .attr('type', 'text')
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md')
                    .attr('id', 'filter-' + filter.id);
                break;
            case 'number':
                $input = $('<input>')
                    .attr('type', 'number')
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md')
                    .attr('id', 'filter-' + filter.id);
                break;
            case 'date':
                $input = $('<input>')
                    .attr('type', 'date')
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md')
                    .attr('id', 'filter-' + filter.id);
                break;
            case 'boolean':
                $input = $('<select>')
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md')
                    .attr('id', 'filter-' + filter.id)
                    .append($('<option>').val('').text('All'))
                    .append($('<option>').val('true').text('Yes'))
                    .append($('<option>').val('false').text('No'));
                break;
            default:
                if (filter.filter_type.select && filter.filter_type.select.options) {
                    $input = $('<select>')
                        .addClass('w-full px-3 py-2 border border-gray-300 rounded-md')
                        .attr('id', 'filter-' + filter.id);
                    $input.append($('<option>').val('').text('All'));
                    filter.filter_type.select.options.forEach(function(opt) {
                        $input.append($('<option>').val(opt).text(opt));
                    });
                }
        }

        if ($input) {
            $filterGroup.append($label).append($input);
            $filterForm.append($filterGroup);
        }
    });

    const $filterButton = $('<button>')
        .addClass('px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 mt-4')
        .text('Apply Filters')
        .click(function() {
            applyFilters(filters);
        });

    $filterArea.append($filterForm).append($filterButton);
    $('#data-area').prepend($filterArea);
}

// Apply filters
function applyFilters(filters) {
    currentFilters = {};
    filters.forEach(function(filter) {
        const value = $('#filter-' + filter.id).val();
        if (value) {
            currentFilters[filter.field] = value;
        }
    });
    loadListData(currentAction, 1);
}

// Render pagination
function renderPagination(pagination) {
    const $paginationArea = $('<div>').addClass('mt-4 flex justify-between items-center');

    // Page info
    const $pageInfo = $('<div>').addClass('text-sm text-gray-700');
    const start = (pagination.page - 1) * pagination.page_size + 1;
    const end = Math.min(pagination.page * pagination.page_size, pagination.total_items);
    $pageInfo.text(`Showing ${start} to ${end} of ${pagination.total_items} results`);

    // Page buttons
    const $pageButtons = $('<div>').addClass('flex space-x-2');

    // Previous button
    if (pagination.page > 1) {
        $pageButtons.append(
            $('<button>')
                .addClass('px-3 py-1 border border-gray-300 rounded-md hover:bg-gray-50')
                .text('Previous')
                .click(function() {
                    loadListData(currentAction, pagination.page - 1);
                })
        );
    }

    // Page numbers
    for (let i = Math.max(1, pagination.page - 2); i <= Math.min(pagination.total_pages, pagination.page + 2); i++) {
        const $pageBtn = $('<button>')
            .addClass('px-3 py-1 border border-gray-300 rounded-md')
            .text(i)
            .click(function() {
                loadListData(currentAction, i);
            });

        if (i === pagination.page) {
            $pageBtn.addClass('bg-indigo-600 text-white border-indigo-600');
        } else {
            $pageBtn.addClass('hover:bg-gray-50');
        }

        $pageButtons.append($pageBtn);
    }

    // Next button
    if (pagination.page < pagination.total_pages) {
        $pageButtons.append(
            $('<button>')
                .addClass('px-3 py-1 border border-gray-300 rounded-md hover:bg-gray-50')
                .text('Next')
                .click(function() {
                    loadListData(currentAction, pagination.page + 1);
                })
        );
    }

    $paginationArea.append($pageInfo).append($pageButtons);
    $('#data-area').append($paginationArea);
}

// Format field value based on field type
function formatFieldValue(value, field) {
    if (field.field_type === 'boolean') {
        return value ? 'Yes' : 'No';
    }
    if (field.field_type === 'date' || field.field_type === 'datetime') {
        return value ? new Date(value).toLocaleString() : '';
    }
    return value;
}

// Show form (for create, update, delete)
function showForm(action, data = {}) {
    const config = action.config || {};

    let title = action.name;
    if (config.form_mode === 'create') {
        title = 'Create ' + currentSection.name;
    } else if (config.form_mode === 'update') {
        title = 'Update ' + currentSection.name;
    } else if (config.form_mode === 'delete') {
        title = 'Delete ' + currentSection.name;
    }

    $('#modal-title').text(title);
    renderForm(action.fields, data, config);
    $('#formModal').addClass('active');

    $('#dynamic-form').off('submit').on('submit', function(e) {
        e.preventDefault();
        submitForm(action, data);
    });
}

// Render dynamic form fields
function renderForm(fields, data, config = {}) {
    const $formFields = $('#form-fields');
    $formFields.empty();

    fields.forEach(function(field) {
        if (!field.editable && config.form_mode === 'create') return;

        const $fieldGroup = $('<div>').addClass('form-group');

        const $label = $('<label>')
            .addClass('block text-sm font-medium text-gray-700 mb-1')
            .attr('for', field.id)
            .text(field.name + (field.required ? ' *' : ''));

        $fieldGroup.append($label);

        const value = data[field.id] || field.default_value || '';
        let $input = createFieldInput(field, value);

        if (field.required) {
            $input.attr('required', true);
        }

        if (field.placeholder) {
            $input.attr('placeholder', field.placeholder);
        }

        $fieldGroup.append($input);

        if (field.help_text) {
            $fieldGroup.append(
                $('<p>').addClass('text-xs text-gray-500 mt-1').text(field.help_text)
            );
        }

        $formFields.append($fieldGroup);
    });

    // Update submit button text
    const submitText = config.submit_button_text || 'Submit';
    $('#submit-text').text(submitText);
}

// Create field input based on field type
function createFieldInput(field, value) {
    let $input;

    switch(field.field_type) {
        case 'textarea':
            const rows = field.config?.rows || 4;
            $input = $('<textarea>')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .attr('rows', rows)
                .val(value);

            if (field.config?.min_length) {
                $input.attr('minlength', field.config.min_length);
            }
            if (field.config?.max_length) {
                $input.attr('maxlength', field.config.max_length);
            }
            break;

        case 'select':
            $input = $('<select>')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500');

            if (field.config?.multiple) {
                $input.attr('multiple', true);
            }

            if (field.config?.options) {
                field.config.options.forEach(function(option) {
                    $input.append($('<option>')
                        .val(option.value)
                        .text(option.label)
                        .prop('selected', option.value === value)
                    );
                });
            }
            break;

        case 'boolean':
            const trueLabel = field.config?.true_label || 'Yes';
            const falseLabel = field.config?.false_label || 'No';

            $input = $('<input>')
                .attr('type', 'checkbox')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded')
                .prop('checked', value === true || value === 'true');
            break;

        case 'date':
        case 'datetime':
            $input = $('<input>')
                .attr('type', field.field_type === 'datetime' ? 'datetime-local' : 'date')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .val(value);

            if (field.config?.min_date) {
                $input.attr('min', field.config.min_date);
            }
            if (field.config?.max_date) {
                $input.attr('max', field.config.max_date);
            }
            break;

        case 'number':
            $input = $('<input>')
                .attr('type', 'number')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .val(value);

            if (field.config?.min !== undefined) {
                $input.attr('min', field.config.min);
            }
            if (field.config?.max !== undefined) {
                $input.attr('max', field.config.max);
            }
            if (field.config?.step !== undefined) {
                $input.attr('step', field.config.step);
            }
            break;

        case 'email':
            $input = $('<input>')
                .attr('type', 'email')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .val(value);

            if (field.config?.pattern) {
                $input.attr('pattern', field.config.pattern);
            }
            break;

        case 'password':
            $input = $('<input>')
                .attr('type', 'password')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .val(value);

            if (field.config?.min_length) {
                $input.attr('minlength', field.config.min_length);
            }
            if (field.config?.max_length) {
                $input.attr('maxlength', field.config.max_length);
            }
            break;

        case 'file':
            $input = $('<input>')
                .attr('type', 'file')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500');

            if (field.config?.multiple) {
                $input.attr('multiple', true);
            }
            if (field.config?.accepted_types) {
                $input.attr('accept', field.config.accepted_types.join(','));
            }
            break;

        case 'url':
            $input = $('<input>')
                .attr('type', 'url')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .val(value);

            if (field.config?.require_protocol) {
                $input.attr('pattern', 'https?://.+');
            }
            break;

        case 'phone':
            $input = $('<input>')
                .attr('type', 'tel')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .val(value);

            if (field.config?.validation_pattern) {
                $input.attr('pattern', field.config.validation_pattern);
            }
            break;

        case 'currency':
            const currencySymbol = field.config?.symbol || '$';
            const $currencyWrapper = $('<div>').addClass('relative');

            $input = $('<input>')
                .attr('type', 'number')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full pl-8 pr-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .val(value);

            if (field.config?.min !== undefined) {
                $input.attr('min', field.config.min);
            }
            if (field.config?.max !== undefined) {
                $input.attr('max', field.config.max);
            }

            const decimalPlaces = field.config?.decimal_places || 2;
            $input.attr('step', Math.pow(0.1, decimalPlaces));

            const $symbol = $('<span>')
                .addClass('absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-500')
                .text(currencySymbol);

            $currencyWrapper.append($symbol).append($input);
            $input = $currencyWrapper;
            break;

        case 'color':
            $input = $('<input>')
                .attr('type', 'color')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('h-10 w-full border border-gray-300 rounded-md cursor-pointer')
                .val(value || '#000000');
            break;

        case 'range':
            const $rangeWrapper = $('<div>');

            $input = $('<input>')
                .attr('type', 'range')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer')
                .val(value);

            if (field.config?.min !== undefined) {
                $input.attr('min', field.config.min);
            }
            if (field.config?.max !== undefined) {
                $input.attr('max', field.config.max);
            }
            if (field.config?.step !== undefined) {
                $input.attr('step', field.config.step);
            }

            if (field.config?.show_value) {
                const $valueDisplay = $('<span>')
                    .attr('id', field.id + '-value')
                    .addClass('block text-center mt-1 text-sm text-gray-600')
                    .text(value || field.config?.min || 0);

                $input.on('input', function() {
                    $valueDisplay.text($(this).val());
                });

                $rangeWrapper.append($input).append($valueDisplay);
                $input = $rangeWrapper;
            } else {
                $input = $('<div>').append($input);
            }
            break;

        case 'time':
            $input = $('<input>')
                .attr('type', 'time')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .val(value);

            if (field.config?.min_time) {
                $input.attr('min', field.config.min_time);
            }
            if (field.config?.max_time) {
                $input.attr('max', field.config.max_time);
            }
            if (field.config?.step_minutes) {
                $input.attr('step', field.config.step_minutes * 60);
            }
            break;

        case 'rating':
            const maxRating = field.config?.max_rating || 5;
            const icon = field.config?.icon || 'star';
            const iconClass = {
                star: 'fa-star',
                heart: 'fa-heart',
                circle: 'fa-circle',
                thumb: 'fa-thumbs-up'
            }[icon] || 'fa-star';

            const $ratingWrapper = $('<div>').addClass('flex items-center gap-1');
            $input = $('<input>')
                .attr('type', 'hidden')
                .attr('id', field.id)
                .attr('name', field.id)
                .val(value || 0);

            for (let i = 1; i <= maxRating; i++) {
                const $star = $('<i>')
                    .addClass(`fas ${iconClass} text-2xl cursor-pointer text-gray-300 hover:text-yellow-400`)
                    .attr('data-rating', i)
                    .click(function() {
                        const rating = $(this).data('rating');
                        $input.val(rating);
                        $ratingWrapper.find('i').each(function(idx) {
                            if (idx < rating) {
                                $(this).removeClass('text-gray-300').addClass('text-yellow-400');
                            } else {
                                $(this).removeClass('text-yellow-400').addClass('text-gray-300');
                            }
                        });
                    });

                if (i <= (value || 0)) {
                    $star.removeClass('text-gray-300').addClass('text-yellow-400');
                }

                $ratingWrapper.append($star);
            }

            $ratingWrapper.append($input);
            $input = $ratingWrapper;
            break;

        case 'tags':
            const $tagsWrapper = $('<div>').addClass('border border-gray-300 rounded-md p-2');
            const $tagsContainer = $('<div>').addClass('flex flex-wrap gap-2 mb-2').attr('id', field.id + '-tags');
            const $tagInput = $('<input>')
                .attr('type', 'text')
                .addClass('flex-1 px-2 py-1 border-0 focus:outline-none')
                .attr('placeholder', 'Type and press Enter');

            $input = $('<input>')
                .attr('type', 'hidden')
                .attr('id', field.id)
                .attr('name', field.id)
                .val(Array.isArray(value) ? value.join(',') : value || '');

            const tags = (Array.isArray(value) ? value : (value ? value.split(',') : []));
            tags.forEach(tag => {
                if (tag.trim()) {
                    addTag(tag.trim(), $tagsContainer, $input);
                }
            });

            $tagInput.on('keypress', function(e) {
                if (e.which === 13) {
                    e.preventDefault();
                    const tag = $(this).val().trim();
                    if (tag && (!field.config?.max_tags || $tagsContainer.children().length < field.config.max_tags)) {
                        addTag(tag, $tagsContainer, $input);
                        $(this).val('');
                    }
                }
            });

            $tagsWrapper.append($tagsContainer).append($tagInput).append($input);
            $input = $tagsWrapper;
            break;

        case 'image':
            $input = $('<input>')
                .attr('type', 'file')
                .attr('id', field.id)
                .attr('name', field.id)
                .attr('accept', 'image/*')
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500');

            if (field.config?.multiple) {
                $input.attr('multiple', true);
            }
            if (field.config?.accepted_formats) {
                $input.attr('accept', field.config.accepted_formats.map(f => 'image/' + f).join(','));
            }
            break;

        case 'json':
            $input = $('<textarea>')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 font-mono text-sm')
                .attr('rows', 8)
                .val(typeof value === 'object' ? JSON.stringify(value, null, 2) : value);

            if (field.config?.validate_on_change) {
                $input.on('change', function() {
                    try {
                        JSON.parse($(this).val());
                        $(this).removeClass('border-red-500').addClass('border-gray-300');
                    } catch (e) {
                        $(this).removeClass('border-gray-300').addClass('border-red-500');
                    }
                });
            }
            break;

        default: // text
            $input = $('<input>')
                .attr('type', 'text')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .val(value);

            if (field.config?.min_length) {
                $input.attr('minlength', field.config.min_length);
            }
            if (field.config?.max_length) {
                $input.attr('maxlength', field.config.max_length);
            }
            if (field.config?.pattern) {
                $input.attr('pattern', field.config.pattern);
            }
    }

    return $input;
}

// Submit form
function submitForm(action, existingData = {}) {
    const formData = $('#dynamic-form').serializeArray();
    const data = {};

    formData.forEach(function(field) {
        data[field.name] = field.value;
    });

    // Handle special field types
    action.fields.forEach(function(field) {
        switch (field.field_type) {
            case 'boolean':
                data[field.id] = $('#' + field.id).is(':checked');
                break;
            case 'tags':
                const tagsValue = $('#' + field.id).val();
                data[field.id] = tagsValue ? tagsValue.split(',') : [];
                break;
            case 'json':
                try {
                    const jsonValue = $('#' + field.id).val();
                    data[field.id] = jsonValue ? JSON.parse(jsonValue) : null;
                } catch (e) {
                    console.error('Invalid JSON for field ' + field.id, e);
                    data[field.id] = $('#' + field.id).val();
                }
                break;
            case 'rating':
                data[field.id] = parseInt($('#' + field.id).val()) || 0;
                break;
            case 'currency':
                const currencyValue = $('#' + field.id).val();
                data[field.id] = currencyValue ? parseFloat(currencyValue) : 0;
                break;
            case 'number':
                const numberValue = $('#' + field.id).val();
                data[field.id] = numberValue ? parseFloat(numberValue) : 0;
                break;
            case 'range':
                const rangeValue = $('#' + field.id).val();
                data[field.id] = rangeValue ? parseFloat(rangeValue) : 0;
                break;
        }
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

            if (action.config?.show_success_message !== false) {
                showSuccess('Operation completed successfully');
            }

            // Reload list if available
            const listAction = currentSection.actions.find(a => a.type === 'list');
            if (listAction) {
                loadListData(listAction, currentPage);
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
                const listAction = currentSection.actions.find(a => a.type === 'list');
                if (listAction) {
                    loadListData(listAction, currentPage);
                }
            },
            error: function(err) {
                showError('Delete failed: ' + (err.responseJSON?.error || err.responseText));
            }
        });
    }
}

// Load view data
function loadViewData(action) {
    const url = `/api/backoffices/${currentBackoffice.id}/sections/${currentSection.id}/actions/${action.id}`;

    $('#data-area').html('<div class="text-center py-8"><div class="loading mx-auto"></div><p class="mt-4 text-gray-500">Loading...</p></div>');

    $.get(url, function(response) {
        renderViewData(response.data, response.fields);
    }).fail(function(err) {
        showError('Failed to load data: ' + (err.responseJSON?.error || err.responseText));
    });
}

// Render view data
function renderViewData(data, fields) {
    const $dataArea = $('#data-area');
    $dataArea.empty();

    if (!data || data.length === 0) {
        $dataArea.html('<p class="text-gray-500 text-center py-8">No data available</p>');
        return;
    }

    const $details = $('<div>').addClass('space-y-4');

    data.forEach(function(item) {
        const $card = $('<div>').addClass('bg-gray-50 p-4 rounded-lg');

        fields.filter(f => f.visible).forEach(function(field) {
            const value = item[field.id] || '';
            $card.append(
                $('<div>').addClass('mb-2')
                    .append($('<strong>').addClass('text-gray-700').text(field.name + ': '))
                    .append($('<span>').addClass('text-gray-900').text(formatFieldValue(value, field)))
            );
        });

        $details.append($card);
    });

    $dataArea.append($details);
}

// Close modal
function closeModal() {
    $('#formModal').removeClass('active');
    $('#dynamic-form')[0].reset();
}

// Helper functions
function getActionButtonClass(actionType) {
    switch(actionType) {
        case 'form': return 'bg-green-600 hover:bg-green-700 text-white';
        case 'list': return 'bg-indigo-600 hover:bg-indigo-700 text-white';
        case 'view': return 'bg-blue-600 hover:bg-blue-700 text-white';
        default: return 'bg-gray-600 hover:bg-gray-700 text-white';
    }
}

function getActionIcon(actionType) {
    switch(actionType) {
        case 'form': return 'fa-edit';
        case 'list': return 'fa-list';
        case 'view': return 'fa-eye';
        default: return 'fa-cog';
    }
}

// Helper function for tags input
function addTag(tag, $container, $hiddenInput) {
    const $tag = $('<span>')
        .addClass('inline-flex items-center gap-1 px-2 py-1 bg-indigo-100 text-indigo-800 rounded text-sm')
        .text(tag);

    const $removeBtn = $('<button>')
        .attr('type', 'button')
        .addClass('text-indigo-600 hover:text-indigo-800')
        .html('&times;')
        .click(function() {
            $tag.remove();
            updateTagsValue($container, $hiddenInput);
        });

    $tag.append($removeBtn);
    $container.append($tag);
    updateTagsValue($container, $hiddenInput);
}

function updateTagsValue($container, $hiddenInput) {
    const tags = [];
    $container.find('span').each(function() {
        const text = $(this).text().replace('×', '').trim();
        if (text) tags.push(text);
    });
    $hiddenInput.val(tags.join(','));
}

function showError(message) {
    alert('Error: ' + message);
    console.error(message);
}

function showSuccess(message) {
    alert(message);
    console.log(message);
}
