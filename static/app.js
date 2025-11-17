// Global state
let currentBackoffice = null;
let currentSection = null;
let currentAction = null;
let backoffices = [];
let currentPage = 1;
let currentFilters = {};

// Dark mode functionality
function initDarkMode() {
    // Check for saved theme preference or default to light mode
    const savedTheme = localStorage.getItem('theme') || 'light';

    if (savedTheme === 'dark') {
        $('body').addClass('dark-mode');
        $('#theme-toggle i').removeClass('fa-moon').addClass('fa-sun');
    }

    // Theme toggle handler
    $('#theme-toggle').on('click', function() {
        $('body').toggleClass('dark-mode');

        if ($('body').hasClass('dark-mode')) {
            localStorage.setItem('theme', 'dark');
            $('#theme-toggle i').removeClass('fa-moon').addClass('fa-sun');
            showInfo('Dark mode enabled');
        } else {
            localStorage.setItem('theme', 'light');
            $('#theme-toggle i').removeClass('fa-sun').addClass('fa-moon');
            showInfo('Light mode enabled');
        }
    });
}

// Keyboard shortcuts
function initKeyboardShortcuts() {
    $(document).on('keydown', function(e) {
        // Check for modifier key (Ctrl on Windows/Linux, Cmd on Mac)
        const isMod = e.ctrlKey || e.metaKey;

        // Esc: Close modal
        if (e.key === 'Escape') {
            if ($('#formModal').hasClass('active')) {
                e.preventDefault();
                closeModal();
            }
        }

        // Ctrl/Cmd + K: Focus search
        if (isMod && e.key === 'k') {
            e.preventDefault();
            const $search = $('#table-search');
            if ($search.length) {
                $search.focus();
                showInfo('Search focused (Ctrl+K)');
            }
        }

        // Ctrl/Cmd + E: Export CSV
        if (isMod && e.key === 'e') {
            e.preventDefault();
            const exportBtn = document.querySelector('[data-action="export"]');
            if (exportBtn) {
                exportBtn.click();
            }
        }

        // Ctrl/Cmd + D: Toggle dark mode
        if (isMod && e.key === 'd') {
            e.preventDefault();
            $('#theme-toggle').click();
        }

        // Ctrl/Cmd + N: Open create form
        if (isMod && e.key === 'n') {
            e.preventDefault();
            if (currentSection) {
                const createAction = currentSection.actions.find(a =>
                    a.type === 'form' && a.config && a.config.form_mode === 'create'
                );
                if (createAction) {
                    showForm(createAction);
                    showInfo('Create form opened (Ctrl+N)');
                }
            }
        }

        // Ctrl/Cmd + /: Show keyboard shortcuts help
        if (isMod && e.key === '/') {
            e.preventDefault();
            showKeyboardShortcutsHelp();
        }
    });
}

// Show keyboard shortcuts help
function showKeyboardShortcutsHelp() {
    const shortcuts = [
        { keys: 'Ctrl+K / ⌘K', desc: 'Focus search input' },
        { keys: 'Ctrl+E / ⌘E', desc: 'Export table to CSV' },
        { keys: 'Ctrl+D / ⌘D', desc: 'Toggle dark mode' },
        { keys: 'Ctrl+N / ⌘N', desc: 'Open create form' },
        { keys: 'Esc', desc: 'Close modal' },
        { keys: 'Ctrl+/ / ⌘/', desc: 'Show this help' }
    ];

    let helpHtml = '<div class="p-4"><h3 class="text-lg font-bold mb-4">Keyboard Shortcuts</h3><table class="w-full">';

    shortcuts.forEach(shortcut => {
        helpHtml += `
            <tr class="border-b border-gray-200">
                <td class="py-2 pr-4 font-mono text-sm text-indigo-600">${shortcut.keys}</td>
                <td class="py-2 text-sm">${shortcut.desc}</td>
            </tr>
        `;
    });

    helpHtml += '</table></div>';

    $('#modal-title').text('Keyboard Shortcuts');
    $('#form-fields').html(helpHtml);
    $('#submit-text').parent().hide();
    $('#formModal').addClass('active');

    // Override form submit to just close
    $('#dynamic-form').off('submit').on('submit', function(e) {
        e.preventDefault();
        closeModal();
    });
}

// Override closeModal to reset form submit handler
const originalCloseModal = closeModal;
closeModal = function() {
    $('#submit-text').parent().show();
    originalCloseModal();
};

// Initialize the application
$(document).ready(function() {
    initDarkMode();
    initKeyboardShortcuts();
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

    // Toolbar with search and actions
    const $toolbar = $('<div>').addClass('mb-4 flex flex-col gap-3');

    // Search and export row
    const $searchRow = $('<div>').addClass('flex items-center gap-3');
    const $searchInput = $('<input>')
        .attr('type', 'text')
        .attr('id', 'table-search')
        .attr('placeholder', 'Search...')
        .addClass('flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500');

    const $searchIcon = $('<div>').addClass('flex items-center gap-2 text-gray-500');
    $searchIcon.html('<i class="fas fa-search"></i>');

    // Filter toggle button
    const $filterToggle = $('<button>')
        .addClass('px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors flex items-center gap-2')
        .html('<i class="fas fa-filter"></i> Filters')
        .click(function() {
            $('#filter-panel').toggleClass('hidden');
        });

    // Export dropdown button
    const $exportContainer = $('<div>').addClass('relative');
    const $exportBtn = $('<button>')
        .addClass('px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors flex items-center gap-2')
        .html('<i class="fas fa-download"></i> Export <i class="fas fa-chevron-down ml-1"></i>')
        .click(function(e) {
            e.stopPropagation();
            $('#export-dropdown').toggleClass('hidden');
        });

    const $exportDropdown = $('<div>')
        .attr('id', 'export-dropdown')
        .addClass('hidden absolute right-0 mt-2 w-48 bg-white rounded-lg shadow-xl border border-gray-200 z-50');

    const exportOptions = [
        { label: 'Export as CSV', icon: 'fa-file-csv', format: 'csv' },
        { label: 'Export as Excel', icon: 'fa-file-excel', format: 'xlsx' },
        { label: 'Export as JSON', icon: 'fa-file-code', format: 'json' },
        { label: 'Export as PDF', icon: 'fa-file-pdf', format: 'pdf' }
    ];

    exportOptions.forEach(option => {
        const $optionBtn = $('<button>')
            .addClass('w-full text-left px-4 py-2 hover:bg-gray-100 flex items-center gap-2 text-sm')
            .html(`<i class="fas ${option.icon}"></i> ${option.label}`)
            .click(function() {
                $('#export-dropdown').addClass('hidden');
                exportTable(data, fields, option.format);
            });
        $exportDropdown.append($optionBtn);
    });

    $exportContainer.append($exportBtn).append($exportDropdown);

    // Close dropdown when clicking outside
    $(document).on('click', function() {
        $('#export-dropdown').addClass('hidden');
    });

    // Import button
    const $importBtn = $('<button>')
        .addClass('px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center gap-2')
        .html('<i class="fas fa-upload"></i> Import')
        .click(function() {
            showImportDialog();
        });

    $searchRow.append($searchIcon).append($searchInput).append($filterToggle).append($exportContainer).append($importBtn);

    // Bulk actions row (initially hidden)
    const $bulkRow = $('<div>')
        .attr('id', 'bulk-actions-bar')
        .addClass('hidden items-center gap-3 bg-indigo-50 p-3 rounded-lg border border-indigo-200');

    const $selectedCount = $('<span>')
        .attr('id', 'selected-count')
        .addClass('text-sm font-medium text-indigo-900');

    const $bulkDelete = $('<button>')
        .addClass('px-3 py-1 bg-red-600 text-white rounded hover:bg-red-700 text-sm')
        .html('<i class="fas fa-trash"></i> Delete Selected')
        .click(function() {
            bulkDeleteRows();
        });

    const $bulkExport = $('<button>')
        .addClass('px-3 py-1 bg-green-600 text-white rounded hover:bg-green-700 text-sm')
        .html('<i class="fas fa-download"></i> Export Selected')
        .click(function() {
            bulkExportRows();
        });

    const $deselectAll = $('<button>')
        .addClass('px-3 py-1 bg-gray-600 text-white rounded hover:bg-gray-700 text-sm')
        .html('Deselect All')
        .click(function() {
            deselectAllRows();
        });

    $bulkRow.append($selectedCount).append($bulkDelete).append($bulkExport).append($deselectAll);

    $toolbar.append($searchRow).append($bulkRow);
    $dataArea.append($toolbar);

    // Advanced filter panel
    const $filterPanel = $('<div>')
        .attr('id', 'filter-panel')
        .addClass('hidden mb-4 p-4 bg-gray-50 rounded-lg border border-gray-200');

    const $filterTitle = $('<div>').addClass('flex items-center justify-between mb-3');
    $filterTitle.append($('<h3>').addClass('text-sm font-semibold text-gray-700').text('Advanced Filters'));

    const $filterPresets = $('<div>').addClass('flex gap-2');
    const $savePreset = $('<button>')
        .addClass('text-xs px-2 py-1 bg-indigo-600 text-white rounded hover:bg-indigo-700')
        .html('<i class="fas fa-save"></i> Save Preset')
        .click(function() { saveFilterPreset(); });

    const $loadPreset = $('<select>')
        .addClass('text-xs px-2 py-1 border border-gray-300 rounded')
        .html('<option value="">Load Preset...</option>')
        .change(function() { loadFilterPreset($(this).val()); });

    $filterPresets.append($savePreset).append($loadPreset);
    $filterTitle.append($filterPresets);

    const $filterGrid = $('<div>').attr('id', 'filter-grid').addClass('grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3');
    $filterPanel.append($filterTitle).append($filterGrid);
    $dataArea.append($filterPanel);

    // Render filters if configured
    if (config && config.filters && config.filters.length > 0) {
        renderFilters(config.filters);
    }

    if (!data || data.length === 0) {
        $dataArea.append('<p class="text-gray-500 text-center py-8">No data available</p>');
        return;
    }

    const visibleFields = fields.filter(f => f.visible);

    const $table = $('<table>').addClass('min-w-full divide-y divide-gray-200').attr('id', 'data-table');

    // Table header with checkboxes and sorting
    const $thead = $('<thead>').addClass('bg-gray-50');
    const $headerRow = $('<tr>');

    // Checkbox column header
    const $checkboxTh = $('<th>').addClass('px-6 py-3 text-left');
    const $selectAll = $('<input>')
        .attr('type', 'checkbox')
        .attr('id', 'select-all-checkbox')
        .addClass('rounded border-gray-300 text-indigo-600 focus:ring-indigo-500')
        .change(function() {
            toggleSelectAll($(this).is(':checked'));
        });
    $checkboxTh.append($selectAll);
    $headerRow.append($checkboxTh);

    visibleFields.forEach(function(field) {
        const $th = $('<th>')
            .addClass('px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:bg-gray-100')
            .attr('data-field-id', field.id);

        const $thContent = $('<div>').addClass('flex items-center justify-between gap-2');
        $thContent.append($('<span>').text(field.name));

        const $sortIcon = $('<i>')
            .addClass('fas fa-sort text-gray-400')
            .attr('data-sort-direction', 'none');
        $thContent.append($sortIcon);

        $th.append($thContent);
        $th.click(function() {
            sortByColumn(field.id, $sortIcon);
        });

        $headerRow.append($th);
    });

    $headerRow.append($('<th>').addClass('px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider').text('Actions'));
    $thead.append($headerRow);
    $table.append($thead);

    // Table body
    const $tbody = $('<tbody>').addClass('bg-white divide-y divide-gray-200');

    data.forEach(function(row, index) {
        const $tr = $('<tr>')
            .addClass('hover:bg-gray-50')
            .attr('data-row-index', index)
            .attr('data-row-id', row.id || index);

        // Checkbox column
        const $checkboxTd = $('<td>').addClass('px-6 py-4');
        const $checkbox = $('<input>')
            .attr('type', 'checkbox')
            .addClass('row-checkbox rounded border-gray-300 text-indigo-600 focus:ring-indigo-500')
            .data('row-data', row)
            .data('row-index', index)
            .change(function() {
                updateBulkActionsBar();
            });
        $checkboxTd.append($checkbox);
        $tr.append($checkboxTd);

        visibleFields.forEach(function(field) {
            const value = row[field.id] || '';
            const $cell = $('<td>')
                .addClass('px-6 py-4 whitespace-nowrap text-sm text-gray-900')
                .text(formatFieldValue(value, field))
                .attr('data-field-id', field.id)
                .attr('data-row-id', row.id || index);

            // Add inline editing for editable fields
            if (field.editable) {
                $cell.addClass('editable-cell cursor-pointer hover:bg-blue-50')
                    .attr('title', 'Double-click to edit')
                    .on('dblclick', function() {
                        makeInlineEditable($(this), field, row);
                    });
            }

            $tr.append($cell);
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

    // Add search functionality
    $searchInput.on('input', function() {
        const searchTerm = $(this).val().toLowerCase().trim();
        let visibleCount = 0;

        if (!searchTerm) {
            // Show all rows if search is empty
            $tbody.find('tr').show();
            updateSearchResultsCount(data.length);
            return;
        }

        // Filter table rows
        $tbody.find('tr').each(function() {
            const $row = $(this);
            const rowText = $row.text().toLowerCase();

            if (rowText.includes(searchTerm)) {
                $row.show();
                visibleCount++;
            } else {
                $row.hide();
            }
        });

        updateSearchResultsCount(visibleCount);
    });

    // Add search results count indicator
    const $searchCount = $('<div>')
        .attr('id', 'search-results-count')
        .addClass('text-sm text-gray-600 mt-2 hidden');
    $searchContainer.append($searchCount);

    function updateSearchResultsCount(count) {
        const $count = $('#search-results-count');
        if ($searchInput.val().trim()) {
            $count.text(`Showing ${count} of ${data.length} results`).removeClass('hidden');
        } else {
            $count.addClass('hidden');
        }
    }

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

        case 'slug':
            $input = $('<input>')
                .attr('type', 'text')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 font-mono')
                .val(value);

            if (field.config?.max_length) {
                $input.attr('maxlength', field.config.max_length);
            }
            break;

        case 'weekday':
            const weekdayFormat = field.config?.format || 'long';
            const weekdays = weekdayFormat === 'short'
                ? ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']
                : weekdayFormat === 'number'
                ? ['1', '2', '3', '4', '5', '6', '7']
                : ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday'];

            if (field.config?.multiple) {
                $input = $('<select>')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .attr('multiple', true)
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500');

                weekdays.forEach((day, idx) => {
                    $input.append($('<option>')
                        .val(idx + 1)
                        .text(day)
                    );
                });
            } else {
                $input = $('<select>')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500');

                weekdays.forEach((day, idx) => {
                    $input.append($('<option>')
                        .val(idx + 1)
                        .text(day)
                        .prop('selected', (idx + 1) == value)
                    );
                });
            }
            break;

        case 'month':
            const monthFormat = field.config?.format || 'long';
            const months = monthFormat === 'short'
                ? ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
                : monthFormat === 'number'
                ? ['1', '2', '3', '4', '5', '6', '7', '8', '9', '10', '11', '12']
                : ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];

            if (field.config?.multiple) {
                $input = $('<select>')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .attr('multiple', true)
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500');

                months.forEach((month, idx) => {
                    $input.append($('<option>')
                        .val(idx + 1)
                        .text(month)
                    );
                });
            } else {
                $input = $('<select>')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500');

                months.forEach((month, idx) => {
                    $input.append($('<option>')
                        .val(idx + 1)
                        .text(month)
                        .prop('selected', (idx + 1) == value)
                    );
                });
            }
            break;

        case 'geolocation':
            const $geoWrapper = $('<div>');
            const latValue = value?.lat || value?.latitude || '';
            const lngValue = value?.lng || value?.longitude || '';

            const $latInput = $('<input>')
                .attr('type', 'number')
                .attr('step', '0.000001')
                .attr('id', field.id + '-lat')
                .attr('placeholder', 'Latitude')
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 mb-2')
                .val(latValue);

            const $lngInput = $('<input>')
                .attr('type', 'number')
                .attr('step', '0.000001')
                .attr('id', field.id + '-lng')
                .attr('placeholder', 'Longitude')
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .val(lngValue);

            $input = $('<input>')
                .attr('type', 'hidden')
                .attr('id', field.id)
                .attr('name', field.id)
                .val(value ? JSON.stringify(value) : '');

            function updateGeoValue() {
                const lat = parseFloat($latInput.val());
                const lng = parseFloat($lngInput.val());
                if (!isNaN(lat) && !isNaN(lng)) {
                    $input.val(JSON.stringify({ lat, lng }));
                }
            }

            $latInput.on('change', updateGeoValue);
            $lngInput.on('change', updateGeoValue);

            $geoWrapper.append($latInput).append($lngInput).append($input);
            $input = $geoWrapper;
            break;

        case 'duration':
            const durationFormat = field.config?.format || 'hoursminutes';

            if (durationFormat === 'hoursminutes') {
                const $durationWrapper = $('<div>').addClass('flex gap-2');
                const hours = Math.floor((value || 0) / 60);
                const minutes = (value || 0) % 60;

                const $hoursInput = $('<input>')
                    .attr('type', 'number')
                    .attr('min', '0')
                    .attr('id', field.id + '-hours')
                    .attr('placeholder', 'Hours')
                    .addClass('w-1/2 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                    .val(hours);

                const $minutesInput = $('<input>')
                    .attr('type', 'number')
                    .attr('min', '0')
                    .attr('max', '59')
                    .attr('id', field.id + '-minutes')
                    .attr('placeholder', 'Minutes')
                    .addClass('w-1/2 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                    .val(minutes);

                $input = $('<input>')
                    .attr('type', 'hidden')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .val(value || 0);

                function updateDuration() {
                    const h = parseInt($hoursInput.val()) || 0;
                    const m = parseInt($minutesInput.val()) || 0;
                    $input.val(h * 60 + m);
                }

                $hoursInput.on('change', updateDuration);
                $minutesInput.on('change', updateDuration);

                $durationWrapper.append($hoursInput).append($minutesInput).append($input);
                $input = $durationWrapper;
            } else {
                $input = $('<input>')
                    .attr('type', 'number')
                    .attr('id', field.id)
                    .attr('name', field.id)
                    .attr('min', '0')
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                    .val(value);
            }
            break;

        case 'percentage':
            const $percentWrapper = $('<div>').addClass('relative');

            $input = $('<input>')
                .attr('type', 'number')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full pr-8 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
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

            const $percentSymbol = $('<span>')
                .addClass('absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-500')
                .text('%');

            $percentWrapper.append($input).append($percentSymbol);
            $input = $percentWrapper;
            break;

        case 'code':
            $input = $('<textarea>')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 font-mono text-sm bg-gray-50')
                .attr('rows', 10)
                .val(value);

            if (field.config?.min_lines) {
                $input.attr('rows', Math.max(10, field.config.min_lines));
            }
            break;

        case 'markdown':
            const $markdownWrapper = $('<div>');

            $input = $('<textarea>')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 font-mono')
                .attr('rows', 8)
                .val(value);

            if (field.config?.min_length) {
                $input.attr('minlength', field.config.min_length);
            }
            if (field.config?.max_length) {
                $input.attr('maxlength', field.config.max_length);
            }

            if (field.config?.enable_preview) {
                const $previewBtn = $('<button>')
                    .attr('type', 'button')
                    .addClass('mt-2 px-3 py-1 bg-gray-200 text-gray-700 rounded-md hover:bg-gray-300 text-sm')
                    .text('Toggle Preview')
                    .click(function() {
                        const $preview = $('#' + field.id + '-preview');
                        if ($preview.is(':visible')) {
                            $preview.hide();
                            $input.show();
                        } else {
                            $preview.html(marked ? marked.parse($input.val()) : $input.val()).show();
                            $input.hide();
                        }
                    });

                const $preview = $('<div>')
                    .attr('id', field.id + '-preview')
                    .addClass('w-full px-3 py-2 border border-gray-300 rounded-md bg-white prose max-w-none')
                    .hide();

                $markdownWrapper.append($input).append($previewBtn).append($preview);
                $input = $markdownWrapper;
            } else {
                $input = $('<div>').append($input);
            }
            break;

        case 'richtext':
            $input = $('<textarea>')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .attr('rows', 8)
                .val(value);

            if (field.config?.min_length) {
                $input.attr('minlength', field.config.min_length);
            }
            if (field.config?.max_length) {
                $input.attr('maxlength', field.config.max_length);
            }
            break;

        case 'ipaddress':
            $input = $('<input>')
                .attr('type', 'text')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 font-mono')
                .val(value);

            const ipVersion = field.config?.version || 'v4';
            if (ipVersion === 'v4') {
                $input.attr('placeholder', '192.168.1.1');
                $input.attr('pattern', '^((25[0-5]|(2[0-4]|1\\d|[1-9]|)\\d)\\.?\\b){4}$');
            } else if (ipVersion === 'v6') {
                $input.attr('placeholder', '2001:0db8:85a3:0000:0000:8a2e:0370:7334');
            }
            break;

        case 'multicheckbox':
            const $checkboxWrapper = $('<div>').addClass('space-y-2');
            const checkboxLayout = field.config?.layout || 'vertical';
            const layoutClass = checkboxLayout === 'horizontal' ? 'flex flex-wrap gap-4' : checkboxLayout === 'grid' ? 'grid grid-cols-2 gap-2' : 'space-y-2';

            const $checkboxContainer = $('<div>').addClass(layoutClass);
            $input = $('<input>')
                .attr('type', 'hidden')
                .attr('id', field.id)
                .attr('name', field.id)
                .val(Array.isArray(value) ? value.join(',') : value || '');

            if (field.config?.options) {
                field.config.options.forEach(option => {
                    const isChecked = Array.isArray(value) ? value.includes(option.value) : false;
                    const $checkboxLabel = $('<label>').addClass('flex items-center space-x-2 cursor-pointer');
                    const $checkbox = $('<input>')
                        .attr('type', 'checkbox')
                        .attr('value', option.value)
                        .addClass('h-4 w-4 text-indigo-600 border-gray-300 rounded')
                        .prop('checked', isChecked)
                        .prop('disabled', option.disabled || false)
                        .on('change', function() {
                            updateMultiCheckboxValue($checkboxContainer, $input);
                        });

                    $checkboxLabel.append($checkbox).append($('<span>').addClass('text-sm text-gray-700').text(option.label));
                    $checkboxContainer.append($checkboxLabel);
                });
            }

            $checkboxWrapper.append($checkboxContainer).append($input);
            $input = $checkboxWrapper;
            break;

        case 'radio':
            const $radioWrapper = $('<div>').addClass('space-y-2');
            const radioLayout = field.config?.layout || 'vertical';
            const radioLayoutClass = radioLayout === 'horizontal' ? 'flex flex-wrap gap-4' : radioLayout === 'cards' ? 'grid grid-cols-2 gap-2' : 'space-y-2';

            const $radioContainer = $('<div>').addClass(radioLayoutClass);

            if (field.config?.options) {
                field.config.options.forEach(option => {
                    const $radioLabel = $('<label>').addClass('flex items-start space-x-2 cursor-pointer p-2 border border-gray-200 rounded hover:bg-gray-50');
                    const $radio = $('<input>')
                        .attr('type', 'radio')
                        .attr('name', field.id)
                        .attr('id', field.id)
                        .attr('value', option.value)
                        .addClass('h-4 w-4 text-indigo-600 border-gray-300 mt-0.5')
                        .prop('checked', option.value === value);

                    const $labelText = $('<div>');
                    $labelText.append($('<span>').addClass('block text-sm font-medium text-gray-700').text(option.label));
                    if (option.description) {
                        $labelText.append($('<span>').addClass('block text-xs text-gray-500').text(option.description));
                    }

                    $radioLabel.append($radio).append($labelText);
                    $radioContainer.append($radioLabel);
                });
            }

            $radioWrapper.append($radioContainer);
            $input = $radioWrapper;
            break;

        case 'autocomplete':
            const $autocompleteWrapper = $('<div>').addClass('relative');
            $input = $('<input>')
                .attr('type', 'text')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .val(value)
                .attr('autocomplete', 'off');

            const $suggestionsList = $('<ul>')
                .attr('id', field.id + '-suggestions')
                .addClass('absolute z-10 w-full bg-white border border-gray-300 rounded-md shadow-lg mt-1 max-h-60 overflow-y-auto hidden');

            $input.on('input', function() {
                const inputVal = $(this).val().toLowerCase();
                const minChars = field.config?.min_chars || 1;

                if (inputVal.length < minChars) {
                    $suggestionsList.addClass('hidden');
                    return;
                }

                const maxResults = field.config?.max_results || 10;
                const caseSensitive = field.config?.case_sensitive || false;
                const options = field.config?.options || [];

                const filtered = options.filter(opt => {
                    const optVal = caseSensitive ? opt : opt.toLowerCase();
                    const searchVal = caseSensitive ? inputVal : inputVal.toLowerCase();
                    return optVal.includes(searchVal);
                }).slice(0, maxResults);

                $suggestionsList.empty();
                if (filtered.length > 0) {
                    filtered.forEach(opt => {
                        const $li = $('<li>')
                            .addClass('px-3 py-2 cursor-pointer hover:bg-indigo-50')
                            .text(opt)
                            .click(function() {
                                $input.val(opt);
                                $suggestionsList.addClass('hidden');
                            });
                        $suggestionsList.append($li);
                    });
                    $suggestionsList.removeClass('hidden');
                } else {
                    $suggestionsList.addClass('hidden');
                }
            });

            $autocompleteWrapper.append($input).append($suggestionsList);
            $input = $autocompleteWrapper;
            break;

        case 'signature':
            const $signatureWrapper = $('<div>');
            const width = field.config?.width || 400;
            const height = field.config?.height || 200;

            const $canvas = $('<canvas>')
                .attr('id', field.id + '-canvas')
                .attr('width', width)
                .attr('height', height)
                .addClass('border border-gray-300 rounded cursor-crosshair');

            $input = $('<input>')
                .attr('type', 'hidden')
                .attr('id', field.id)
                .attr('name', field.id)
                .val(value || '');

            const $clearBtn = $('<button>')
                .attr('type', 'button')
                .addClass('mt-2 px-3 py-1 bg-gray-200 text-gray-700 rounded-md hover:bg-gray-300 text-sm')
                .text('Clear Signature')
                .click(function() {
                    const canvas = document.getElementById(field.id + '-canvas');
                    const ctx = canvas.getContext('2d');
                    ctx.clearRect(0, 0, canvas.width, canvas.height);
                    $input.val('');
                });

            $signatureWrapper.append($canvas).append($clearBtn).append($input);
            $input = $signatureWrapper;
            break;

        case 'video':
            $input = $('<input>')
                .attr('type', 'file')
                .attr('id', field.id)
                .attr('name', field.id)
                .attr('accept', 'video/*')
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500');

            if (field.config?.multiple) {
                $input.attr('multiple', true);
            }
            if (field.config?.accepted_formats) {
                $input.attr('accept', field.config.accepted_formats.map(f => 'video/' + f).join(','));
            }
            break;

        case 'audio':
            $input = $('<input>')
                .attr('type', 'file')
                .attr('id', field.id)
                .attr('name', field.id)
                .attr('accept', 'audio/*')
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500');

            if (field.config?.multiple) {
                $input.attr('multiple', true);
            }
            if (field.config?.accepted_formats) {
                $input.attr('accept', field.config.accepted_formats.map(f => 'audio/' + f).join(','));
            }
            break;

        case 'barcode':
            $input = $('<input>')
                .attr('type', 'text')
                .attr('id', field.id)
                .attr('name', field.id)
                .addClass('w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 font-mono')
                .val(value);

            if (field.config?.validation_pattern) {
                $input.attr('pattern', field.config.validation_pattern);
            }
            break;

        case 'datetimerange':
            const $dateRangeWrapper = $('<div>').addClass('flex gap-2');
            const includeTime = field.config?.include_time || false;
            const inputType = includeTime ? 'datetime-local' : 'date';

            const $startInput = $('<input>')
                .attr('type', inputType)
                .attr('id', field.id + '-start')
                .attr('placeholder', 'Start')
                .addClass('w-1/2 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .val(value?.start || '');

            const $endInput = $('<input>')
                .attr('type', inputType)
                .attr('id', field.id + '-end')
                .attr('placeholder', 'End')
                .addClass('w-1/2 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500')
                .val(value?.end || '');

            $input = $('<input>')
                .attr('type', 'hidden')
                .attr('id', field.id)
                .attr('name', field.id)
                .val(value ? JSON.stringify(value) : '');

            function updateDateRangeValue() {
                const start = $startInput.val();
                const end = $endInput.val();
                if (start && end) {
                    $input.val(JSON.stringify({ start, end }));
                }
            }

            $startInput.on('change', updateDateRangeValue);
            $endInput.on('change', updateDateRangeValue);

            $dateRangeWrapper.append($startInput).append($endInput).append($input);
            $input = $dateRangeWrapper;
            break;

        case 'slider':
            const $sliderWrapper = $('<div>');
            const sliderMin = field.config?.min || 0;
            const sliderMax = field.config?.max || 100;
            const sliderStep = field.config?.step || 1;
            const handles = field.config?.handles || 2;

            // Simple implementation - would need a library like noUiSlider for proper multi-handle support
            $input = $('<input>')
                .attr('type', 'range')
                .attr('id', field.id)
                .attr('name', field.id)
                .attr('min', sliderMin)
                .attr('max', sliderMax)
                .attr('step', sliderStep)
                .addClass('w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer')
                .val(value || sliderMin);

            if (field.config?.show_values) {
                const $valueDisplay = $('<span>')
                    .attr('id', field.id + '-value')
                    .addClass('block text-center mt-1 text-sm text-gray-600')
                    .text(value || sliderMin);

                $input.on('input', function() {
                    $valueDisplay.text($(this).val());
                });

                $sliderWrapper.append($input).append($valueDisplay);
                $input = $sliderWrapper;
            }
            break;

        case 'colorpalette':
            const $paletteWrapper = $('<div>').addClass('space-y-2');
            const maxColors = field.config?.max_colors || 5;
            const $colorsContainer = $('<div>').addClass('flex flex-wrap gap-2').attr('id', field.id + '-colors');

            $input = $('<input>')
                .attr('type', 'hidden')
                .attr('id', field.id)
                .attr('name', field.id)
                .val(Array.isArray(value) ? value.join(',') : value || '');

            const colors = Array.isArray(value) ? value : (value ? value.split(',') : field.config?.default_colors || []);
            colors.forEach(color => {
                if (color.trim()) {
                    addColorToPalette(color.trim(), $colorsContainer, $input, maxColors);
                }
            });

            const $addColorBtn = $('<button>')
                .attr('type', 'button')
                .addClass('px-3 py-1 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 text-sm')
                .text('Add Color')
                .click(function() {
                    if ($colorsContainer.children().length < maxColors) {
                        const newColor = '#' + Math.floor(Math.random()*16777215).toString(16);
                        addColorToPalette(newColor, $colorsContainer, $input, maxColors);
                    }
                });

            $paletteWrapper.append($colorsContainer).append($addColorBtn).append($input);
            $input = $paletteWrapper;
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
            case 'percentage':
                const percentageValue = $('#' + field.id).val();
                data[field.id] = percentageValue ? parseFloat(percentageValue) : 0;
                break;
            case 'geolocation':
                try {
                    const geoValue = $('#' + field.id).val();
                    data[field.id] = geoValue ? JSON.parse(geoValue) : null;
                } catch (e) {
                    console.error('Invalid geolocation for field ' + field.id, e);
                    data[field.id] = null;
                }
                break;
            case 'duration':
                const durationValue = $('#' + field.id).val();
                data[field.id] = durationValue ? parseInt(durationValue) : 0;
                break;
            case 'weekday':
            case 'month':
                const selectValue = $('#' + field.id).val();
                if (field.config?.multiple) {
                    data[field.id] = selectValue ? selectValue.map(v => parseInt(v)) : [];
                } else {
                    data[field.id] = selectValue ? parseInt(selectValue) : null;
                }
                break;
            case 'multicheckbox':
                const checkboxValue = $('#' + field.id).val();
                data[field.id] = checkboxValue ? checkboxValue.split(',') : [];
                break;
            case 'datetimerange':
                try {
                    const rangeValue = $('#' + field.id).val();
                    data[field.id] = rangeValue ? JSON.parse(rangeValue) : null;
                } catch (e) {
                    console.error('Invalid date range for field ' + field.id, e);
                    data[field.id] = null;
                }
                break;
            case 'signature':
                const signatureValue = $('#' + field.id).val();
                data[field.id] = signatureValue || null;
                break;
            case 'colorpalette':
                const paletteValue = $('#' + field.id).val();
                data[field.id] = paletteValue ? paletteValue.split(',') : [];
                break;
            case 'slider':
                const sliderValue = $('#' + field.id).val();
                data[field.id] = sliderValue ? parseFloat(sliderValue) : 0;
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

function updateMultiCheckboxValue($container, $hiddenInput) {
    const selectedValues = [];
    $container.find('input[type="checkbox"]:checked').each(function() {
        selectedValues.push($(this).val());
    });
    $hiddenInput.val(selectedValues.join(','));
}

function addColorToPalette(color, $container, $hiddenInput, maxColors) {
    if ($container.children().length >= maxColors) return;

    const $colorItem = $('<div>').addClass('relative group');
    const $colorBox = $('<div>')
        .addClass('w-12 h-12 rounded border-2 border-gray-300 cursor-pointer')
        .css('background-color', color)
        .click(function() {
            const newColor = prompt('Enter new color (hex)', color);
            if (newColor && /^#[0-9A-Fa-f]{6}$/.test(newColor)) {
                $(this).css('background-color', newColor);
                updateColorPaletteValue($container, $hiddenInput);
            }
        });

    const $removeBtn = $('<button>')
        .attr('type', 'button')
        .addClass('absolute -top-1 -right-1 w-5 h-5 bg-red-500 text-white rounded-full text-xs opacity-0 group-hover:opacity-100 transition-opacity')
        .html('&times;')
        .click(function(e) {
            e.stopPropagation();
            $colorItem.remove();
            updateColorPaletteValue($container, $hiddenInput);
        });

    $colorItem.append($colorBox).append($removeBtn);
    $container.append($colorItem);
    updateColorPaletteValue($container, $hiddenInput);
}

function updateColorPaletteValue($container, $hiddenInput) {
    const colors = [];
    $container.find('> div > div').each(function() {
        const color = $(this).css('background-color');
        // Convert RGB to hex
        const rgb = color.match(/\d+/g);
        if (rgb) {
            const hex = '#' + rgb.map(x => {
                const hex = parseInt(x).toString(16);
                return hex.length === 1 ? '0' + hex : hex;
            }).join('');
            colors.push(hex);
        }
    });
    $hiddenInput.val(colors.join(','));
}

// Toast notification system
function showToast(message, type = 'info', duration = 5000) {
    const $container = $('#toast-container');

    // Create toast element
    const toastId = 'toast-' + Date.now();
    const icons = {
        success: 'fa-check-circle',
        error: 'fa-exclamation-circle',
        warning: 'fa-exclamation-triangle',
        info: 'fa-info-circle'
    };

    const $toast = $('<div>')
        .attr('id', toastId)
        .addClass(`toast toast-${type}`)
        .html(`
            <i class="fas ${icons[type]} toast-icon"></i>
            <div class="toast-message">${message}</div>
            <i class="fas fa-times toast-close"></i>
        `);

    // Add to container
    $container.append($toast);

    // Trigger animation
    setTimeout(() => {
        $toast.addClass('show');
    }, 10);

    // Auto-hide after duration
    const hideTimeout = setTimeout(() => {
        hideToast(toastId);
    }, duration);

    // Close button handler
    $toast.find('.toast-close').on('click', function() {
        clearTimeout(hideTimeout);
        hideToast(toastId);
    });
}

function hideToast(toastId) {
    const $toast = $('#' + toastId);
    $toast.removeClass('show').addClass('hide');

    // Remove from DOM after animation
    setTimeout(() => {
        $toast.remove();
    }, 300);
}

function showError(message) {
    showToast(message, 'error', 6000);
    console.error(message);
}

function showSuccess(message) {
    showToast(message, 'success', 4000);
    console.log(message);
}

function showWarning(message) {
    showToast(message, 'warning', 5000);
    console.warn(message);
}

function showInfo(message) {
    showToast(message, 'info', 4000);
    console.info(message);
}

// ===== ADVANCED EXPORT OPTIONS =====

// Main export function with format support
function exportTable(data, fields, format = 'csv') {
    if (!data || data.length === 0) {
        showWarning('No data to export');
        return;
    }

    // Get current filter state (only visible rows)
    const visibleData = getVisibleTableData(data);

    switch (format) {
        case 'csv':
            exportTableToCSV(visibleData, fields);
            break;
        case 'xlsx':
            exportTableToExcel(visibleData, fields);
            break;
        case 'json':
            exportTableToJSON(visibleData, fields);
            break;
        case 'pdf':
            exportTableToPDF(visibleData, fields);
            break;
        default:
            showError('Unsupported export format');
    }
}

// Get visible table data (respects current search/filter)
function getVisibleTableData(allData) {
    const visibleRows = [];
    $('#data-table tbody tr:visible').each(function() {
        const rowIndex = $(this).data('row-index');
        if (rowIndex !== undefined && allData[rowIndex]) {
            visibleRows.push(allData[rowIndex]);
        }
    });
    return visibleRows.length > 0 ? visibleRows : allData;
}

// Export table data to CSV
function exportTableToCSV(data, fields) {
    const visibleFields = fields.filter(f => f.visible);

    // Create CSV header
    const headers = visibleFields.map(f => f.name);
    const csvRows = [];
    csvRows.push(headers.join(','));

    // Add data rows
    data.forEach(row => {
        const values = visibleFields.map(field => {
            let value = row[field.id] || '';

            // Format value based on field type
            if (field.field_type === 'boolean') {
                value = value ? 'Yes' : 'No';
            } else if (field.field_type === 'date' || field.field_type === 'datetime') {
                value = value ? new Date(value).toLocaleString() : '';
            } else if (typeof value === 'object') {
                value = JSON.stringify(value);
            }

            // Escape quotes and wrap in quotes if contains comma, quote, or newline
            value = String(value).replace(/"/g, '""');
            if (value.includes(',') || value.includes('"') || value.includes('\n')) {
                value = `"${value}"`;
            }

            return value;
        });

        csvRows.push(values.join(','));
    });

    // Create CSV content
    const csvContent = csvRows.join('\n');

    // Create download link
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.setAttribute('href', url);

    // Generate filename with timestamp
    const timestamp = new Date().toISOString().slice(0, 10);
    const sectionName = currentSection ? currentSection.name.toLowerCase().replace(/\s+/g, '-') : 'data';
    link.setAttribute('download', `${sectionName}-export-${timestamp}.csv`);

    // Trigger download
    link.style.visibility = 'hidden';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);

    showSuccess(`Exported ${data.length} rows to CSV`);
}

// Export table data to JSON
function exportTableToJSON(data, fields) {
    const visibleFields = fields.filter(f => f.visible);

    // Convert data to JSON with only visible fields
    const jsonData = data.map(row => {
        const obj = {};
        visibleFields.forEach(field => {
            obj[field.name] = row[field.id] || '';
        });
        return obj;
    });

    // Create JSON content
    const jsonContent = JSON.stringify(jsonData, null, 2);

    // Create download link
    const blob = new Blob([jsonContent], { type: 'application/json;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.setAttribute('href', url);

    // Generate filename with timestamp
    const timestamp = new Date().toISOString().slice(0, 10);
    const sectionName = currentSection ? currentSection.name.toLowerCase().replace(/\s+/g, '-') : 'data';
    link.setAttribute('download', `${sectionName}-export-${timestamp}.json`);

    // Trigger download
    link.style.visibility = 'hidden';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);

    showSuccess(`Exported ${data.length} rows to JSON`);
}

// Export table data to Excel with formatting
function exportTableToExcel(data, fields) {
    const visibleFields = fields.filter(f => f.visible);

    // Create worksheet data
    const wsData = [];

    // Add headers
    const headers = visibleFields.map(f => f.name);
    wsData.push(headers);

    // Add data rows
    data.forEach(row => {
        const rowData = visibleFields.map(field => {
            let value = row[field.id];

            // Format value based on field type
            if (value === null || value === undefined) {
                return '';
            } else if (field.field_type === 'boolean' || field.field_type === 'toggle') {
                return value ? 'Yes' : 'No';
            } else if (field.field_type === 'date' || field.field_type === 'datetime') {
                return value ? new Date(value).toLocaleString() : '';
            } else if (typeof value === 'object') {
                return JSON.stringify(value);
            }

            return value;
        });
        wsData.push(rowData);
    });

    // Create workbook and worksheet
    const wb = XLSX.utils.book_new();
    const ws = XLSX.utils.aoa_to_sheet(wsData);

    // Set column widths
    const colWidths = visibleFields.map(field => {
        const maxLength = Math.max(
            field.name.length,
            ...data.slice(0, 100).map(row => {
                const val = String(row[field.id] || '');
                return val.length;
            })
        );
        return { wch: Math.min(Math.max(maxLength + 2, 10), 50) };
    });
    ws['!cols'] = colWidths;

    // Style header row
    const range = XLSX.utils.decode_range(ws['!ref']);
    for (let col = range.s.c; col <= range.e.c; col++) {
        const cellAddress = XLSX.utils.encode_cell({ r: 0, c: col });
        if (!ws[cellAddress]) continue;

        ws[cellAddress].s = {
            font: { bold: true, color: { rgb: "FFFFFF" } },
            fill: { fgColor: { rgb: "4F46E5" } },
            alignment: { horizontal: "center", vertical: "center" }
        };
    }

    // Add worksheet to workbook
    const sectionName = currentSection ? currentSection.name : 'Data';
    XLSX.utils.book_append_sheet(wb, ws, sectionName.substring(0, 31)); // Excel sheet names max 31 chars

    // Generate filename with timestamp
    const timestamp = new Date().toISOString().slice(0, 10);
    const filename = `${sectionName.toLowerCase().replace(/\s+/g, '-')}-export-${timestamp}.xlsx`;

    // Write file
    XLSX.writeFile(wb, filename);

    showSuccess(`Exported ${data.length} rows to Excel`);
}

// Export table data to PDF with formatting
function exportTableToPDF(data, fields) {
    const visibleFields = fields.filter(f => f.visible);

    // Initialize jsPDF
    const { jsPDF } = window.jspdf;
    const doc = new jsPDF({
        orientation: visibleFields.length > 6 ? 'landscape' : 'portrait',
        unit: 'mm',
        format: 'a4'
    });

    // Add title
    const sectionName = currentSection ? currentSection.name : 'Data Export';
    doc.setFontSize(18);
    doc.setTextColor(79, 70, 229); // Indigo color
    doc.text(sectionName, 14, 20);

    // Add metadata
    doc.setFontSize(10);
    doc.setTextColor(100);
    const timestamp = new Date().toLocaleString();
    doc.text(`Generated: ${timestamp}`, 14, 28);
    doc.text(`Total Records: ${data.length}`, 14, 33);

    // Prepare table data
    const tableHeaders = [visibleFields.map(f => f.name)];
    const tableData = data.map(row => {
        return visibleFields.map(field => {
            let value = row[field.id];

            // Format value based on field type
            if (value === null || value === undefined) {
                return '';
            } else if (field.field_type === 'boolean' || field.field_type === 'toggle') {
                return value ? 'Yes' : 'No';
            } else if (field.field_type === 'date' || field.field_type === 'datetime') {
                return value ? new Date(value).toLocaleString() : '';
            } else if (typeof value === 'object') {
                return JSON.stringify(value);
            }

            return String(value);
        });
    });

    // Generate table
    doc.autoTable({
        head: tableHeaders,
        body: tableData,
        startY: 40,
        theme: 'striped',
        headStyles: {
            fillColor: [79, 70, 229], // Indigo
            textColor: [255, 255, 255],
            fontStyle: 'bold',
            halign: 'center'
        },
        alternateRowStyles: {
            fillColor: [249, 250, 251] // Light gray
        },
        margin: { top: 40, left: 14, right: 14 },
        styles: {
            fontSize: 8,
            cellPadding: 3,
            overflow: 'linebreak',
            cellWidth: 'wrap'
        },
        columnStyles: visibleFields.reduce((acc, field, index) => {
            // Adjust column widths based on field type
            if (field.field_type === 'boolean' || field.field_type === 'toggle') {
                acc[index] = { cellWidth: 15 };
            } else if (field.field_type === 'date' || field.field_type === 'datetime') {
                acc[index] = { cellWidth: 30 };
            }
            return acc;
        }, {}),
        didDrawPage: function(data) {
            // Add page numbers
            const pageCount = doc.internal.getNumberOfPages();
            doc.setFontSize(8);
            doc.setTextColor(150);
            for (let i = 1; i <= pageCount; i++) {
                doc.setPage(i);
                doc.text(
                    `Page ${i} of ${pageCount}`,
                    doc.internal.pageSize.width / 2,
                    doc.internal.pageSize.height - 10,
                    { align: 'center' }
                );
            }
        }
    });

    // Generate filename with timestamp
    const fileTimestamp = new Date().toISOString().slice(0, 10);
    const filename = `${sectionName.toLowerCase().replace(/\s+/g, '-')}-export-${fileTimestamp}.pdf`;

    // Save PDF
    doc.save(filename);

    showSuccess(`Exported ${data.length} rows to PDF`);
}

// ===== DATA RELATIONSHIPS =====

// Fetch and display related data for a record
async function loadRelatedData(recordId, relationships) {
    if (!relationships || relationships.length === 0) {
        return null;
    }

    const relatedData = {};

    for (const rel of relationships) {
        try {
            // Fetch related records
            const response = await fetch(`/api/backoffices/${currentBackoffice.id}/sections/${rel.to_section}/actions/list_${rel.to_section}`);
            if (response.ok) {
                const data = await response.json();
                // Filter related records based on the relationship
                relatedData[rel.id] = {
                    config: rel,
                    records: data.filter(record => record[rel.to_field] === recordId)
                };
            }
        } catch (error) {
            console.error(`Failed to load related data for ${rel.name}:`, error);
        }
    }

    return relatedData;
}

// Display relationships panel in view/edit mode
function renderRelationshipsPanel(recordId, relationships, $container) {
    if (!relationships || relationships.length === 0) {
        return;
    }

    const $panel = $('<div>')
        .addClass('mt-6 border-t pt-6');

    const $header = $('<div>')
        .addClass('flex items-center gap-2 mb-4')
        .html('<i class="fas fa-link text-indigo-600"></i><h3 class="text-lg font-semibold text-gray-800">Related Data</h3>');

    $panel.append($header);

    // Create tabs for each relationship
    const $tabContainer = $('<div>').addClass('border-b border-gray-200 mb-4');
    const $tabButtons = $('<div>').addClass('flex space-x-2');
    const $tabContent = $('<div>');

    relationships.forEach((rel, index) => {
        const isActive = index === 0;

        // Tab button
        const $tabBtn = $('<button>')
            .addClass('px-4 py-2 border-b-2 transition-colors')
            .addClass(isActive ? 'border-indigo-600 text-indigo-600 font-semibold' : 'border-transparent text-gray-600 hover:text-gray-800')
            .attr('data-tab', rel.id)
            .text(`${rel.name} (${rel.relationship_type})`)
            .click(function() {
                // Switch tabs
                $tabButtons.find('button').removeClass('border-indigo-600 text-indigo-600 font-semibold').addClass('border-transparent text-gray-600');
                $(this).addClass('border-indigo-600 text-indigo-600 font-semibold').removeClass('border-transparent text-gray-600');

                $tabContent.find('.relationship-tab-content').addClass('hidden');
                $tabContent.find(`[data-tab-content="${rel.id}"]`).removeClass('hidden');
            });

        $tabButtons.append($tabBtn);

        // Tab content
        const $content = $('<div>')
            .addClass('relationship-tab-content')
            .addClass(isActive ? '' : 'hidden')
            .attr('data-tab-content', rel.id);

        // Loading placeholder
        $content.html('<div class="text-center py-8"><i class="fas fa-spinner fa-spin text-2xl text-gray-400"></i><p class="mt-2 text-gray-600">Loading related data...</p></div>');

        $tabContent.append($content);

        // Load related data asynchronously
        loadRelationshipData(recordId, rel, $content);
    });

    $tabContainer.append($tabButtons);
    $panel.append($tabContainer);
    $panel.append($tabContent);

    $container.append($panel);
}

// Load data for a specific relationship
async function loadRelationshipData(recordId, relationship, $container) {
    try {
        // Build query based on relationship
        const response = await fetch(`/api/backoffices/${currentBackoffice.id}/sections/${relationship.to_section}/actions/list_${relationship.to_section}`);

        if (!response.ok) {
            throw new Error('Failed to fetch related data');
        }

        const allData = await response.json();

        // Filter based on relationship
        const relatedRecords = allData.filter(record => {
            if (relationship.relationship_type === 'onetomany' || relationship.relationship_type === 'manytoone') {
                return String(record[relationship.to_field]) === String(recordId);
            }
            return false;
        });

        // Render related records table
        $container.empty();

        if (relatedRecords.length === 0) {
            $container.html(`
                <div class="text-center py-8 text-gray-500">
                    <i class="fas fa-inbox text-4xl mb-2"></i>
                    <p>No related records found</p>
                </div>
            `);
            return;
        }

        // Create mini table for related records
        const $table = $('<table>').addClass('min-w-full divide-y divide-gray-200');
        const $thead = $('<thead>').addClass('bg-gray-50');
        const $tbody = $('<tbody>').addClass('divide-y divide-gray-200');

        // Determine which fields to display
        const displayFields = relationship.display_fields || Object.keys(relatedRecords[0]).slice(0, 5);

        // Header
        const $headerRow = $('<tr>');
        displayFields.forEach(fieldId => {
            $headerRow.append(
                $('<th>').addClass('px-4 py-2 text-left text-xs font-medium text-gray-700 uppercase tracking-wider').text(fieldId)
            );
        });
        $headerRow.append($('<th>').addClass('px-4 py-2 text-right text-xs font-medium text-gray-700 uppercase tracking-wider').text('Actions'));
        $thead.append($headerRow);

        // Rows
        relatedRecords.forEach(record => {
            const $row = $('<tr>').addClass('hover:bg-gray-50');

            displayFields.forEach(fieldId => {
                let value = record[fieldId];
                if (value === null || value === undefined) value = '-';
                if (typeof value === 'object') value = JSON.stringify(value);

                $row.append(
                    $('<td>').addClass('px-4 py-2 text-sm text-gray-900').text(String(value).substring(0, 50))
                );
            });

            // Actions
            const $actions = $('<td>').addClass('px-4 py-2 text-right text-sm');
            const $viewBtn = $('<button>')
                .addClass('text-indigo-600 hover:text-indigo-900 ml-2')
                .html('<i class="fas fa-eye"></i>')
                .attr('title', 'View')
                .click(() => {
                    // Navigate to related record
                    showInfo(`View related record: ${record.id || 'N/A'}`);
                });

            $actions.append($viewBtn);
            $row.append($actions);

            $tbody.append($row);
        });

        $table.append($thead);
        $table.append($tbody);

        const $tableContainer = $('<div>').addClass('overflow-x-auto rounded-lg border border-gray-200');
        $tableContainer.append($table);

        $container.append($tableContainer);

        // Show count
        $container.prepend(
            $('<div>')
                .addClass('mb-3 text-sm text-gray-600')
                .html(`<i class="fas fa-list mr-2"></i>Found ${relatedRecords.length} related record(s)`)
        );

    } catch (error) {
        console.error('Error loading relationship data:', error);
        $container.html(`
            <div class="text-center py-8 text-red-500">
                <i class="fas fa-exclamation-triangle text-4xl mb-2"></i>
                <p>Failed to load related data</p>
                <p class="text-sm mt-1">${error.message}</p>
            </div>
        `);
    }
}

// Display relationship graph/visualization
function renderRelationshipGraph(relationships) {
    if (!relationships || relationships.length === 0) {
        return;
    }

    // Create a simple visual representation of relationships
    const $graph = $('<div>')
        .addClass('p-4 bg-gray-50 rounded-lg border border-gray-200 mt-4');

    const $title = $('<h4>')
        .addClass('font-semibold text-gray-800 mb-3 flex items-center gap-2')
        .html('<i class="fas fa-project-diagram"></i> Relationship Map');

    $graph.append($title);

    const $list = $('<div>').addClass('space-y-2');

    relationships.forEach(rel => {
        const $item = $('<div>')
            .addClass('flex items-center gap-2 text-sm p-2 bg-white rounded border border-gray-200');

        let icon = 'fa-arrow-right';
        if (rel.relationship_type === 'onetomany') {
            icon = 'fa-arrow-right';
        } else if (rel.relationship_type === 'manytoone') {
            icon = 'fa-arrow-left';
        } else if (rel.relationship_type === 'onetoone') {
            icon = 'fa-exchange-alt';
        }

        $item.html(`
            <span class="font-semibold text-indigo-600">${rel.from_section}</span>
            <i class="fas ${icon} text-gray-400"></i>
            <span class="font-semibold text-green-600">${rel.to_section}</span>
            <span class="text-gray-500 text-xs ml-auto">${rel.relationship_type}</span>
            ${rel.cascade_delete ? '<span class="text-xs text-red-500 ml-2" title="Cascade delete enabled"><i class="fas fa-trash-alt"></i></span>' : ''}
        `);

        $list.append($item);
    });

    $graph.append($list);

    return $graph;
}

// Handle cascade delete
async function handleCascadeDelete(recordId, relationships) {
    if (!relationships || relationships.length === 0) {
        return true;
    }

    // Find relationships with cascade delete enabled
    const cascadeRels = relationships.filter(rel => rel.cascade_delete);

    if (cascadeRels.length === 0) {
        return true;
    }

    // Warn user about cascade delete
    const relNames = cascadeRels.map(r => r.name).join(', ');
    const confirmed = confirm(
        `This will also delete related records in: ${relNames}\n\nAre you sure you want to continue?`
    );

    if (!confirmed) {
        return false;
    }

    // Delete related records
    for (const rel of cascadeRels) {
        try {
            // Fetch related records
            const response = await fetch(`/api/backoffices/${currentBackoffice.id}/sections/${rel.to_section}/actions/list_${rel.to_section}`);
            if (response.ok) {
                const allData = await response.json();
                const relatedRecords = allData.filter(record => String(record[rel.to_field]) === String(recordId));

                // Delete each related record
                for (const record of relatedRecords) {
                    await fetch(`/api/backoffices/${currentBackoffice.id}/sections/${rel.to_section}/actions/delete_${rel.to_section}`, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ id: record.id })
                    });
                }
            }
        } catch (error) {
            console.error(`Failed to cascade delete for ${rel.name}:`, error);
            showError(`Failed to delete related records in ${rel.name}`);
            return false;
        }
    }

    return true;
}

// ===== AUDIT TRAIL & HISTORY =====

// Display audit metadata in view/edit mode
function renderAuditMetadata(record, auditConfig, $container) {
    if (!auditConfig || !auditConfig.track_changes) {
        return;
    }

    const $panel = $('<div>')
        .addClass('mt-6 p-4 bg-gray-50 rounded-lg border border-gray-200');

    const $header = $('<div>')
        .addClass('flex items-center gap-2 mb-3')
        .html('<i class="fas fa-history text-gray-600"></i><h4 class="font-semibold text-gray-800">Audit Information</h4>');

    $panel.append($header);

    const $grid = $('<div>').addClass('grid grid-cols-2 gap-4 text-sm');

    // Created information
    if (auditConfig.track_created) {
        const createdAtField = auditConfig.created_at_field || 'created_at';
        const createdByField = auditConfig.created_by_field || 'created_by';

        if (record[createdAtField]) {
            $grid.append(`
                <div>
                    <span class="text-gray-600">Created:</span>
                    <span class="ml-2 font-medium">${new Date(record[createdAtField]).toLocaleString()}</span>
                </div>
            `);
        }

        if (record[createdByField]) {
            $grid.append(`
                <div>
                    <span class="text-gray-600">Created By:</span>
                    <span class="ml-2 font-medium">${record[createdByField]}</span>
                </div>
            `);
        }
    }

    // Updated information
    if (auditConfig.track_updated) {
        const updatedAtField = auditConfig.updated_at_field || 'updated_at';
        const updatedByField = auditConfig.updated_by_field || 'updated_by';

        if (record[updatedAtField]) {
            $grid.append(`
                <div>
                    <span class="text-gray-600">Last Updated:</span>
                    <span class="ml-2 font-medium">${new Date(record[updatedAtField]).toLocaleString()}</span>
                </div>
            `);
        }

        if (record[updatedByField]) {
            $grid.append(`
                <div>
                    <span class="text-gray-600">Updated By:</span>
                    <span class="ml-2 font-medium">${record[updatedByField]}</span>
                </div>
            `);
        }
    }

    $panel.append($grid);

    // Add view history button
    const $historyBtn = $('<button>')
        .addClass('mt-3 px-3 py-1 text-sm bg-indigo-600 text-white rounded hover:bg-indigo-700 transition-colors')
        .html('<i class="fas fa-clock-rotate-left mr-2"></i>View Change History')
        .click(() => showChangeHistory(record.id));

    $panel.append($historyBtn);

    $container.append($panel);
}

// Show change history modal
async function showChangeHistory(recordId) {
    // Create modal
    const $modal = $('<div>')
        .addClass('fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50')
        .attr('id', 'history-modal');

    const $content = $('<div>')
        .addClass('bg-white rounded-lg shadow-xl max-w-4xl w-full m-4 max-h-[90vh] overflow-y-auto');

    const $header = $('<div>')
        .addClass('p-4 border-b flex justify-between items-center');

    $header.append(`
        <h3 class="text-xl font-semibold flex items-center gap-2">
            <i class="fas fa-history text-indigo-600"></i>
            Change History
        </h3>
    `);

    const $closeBtn = $('<button>')
        .addClass('text-gray-500 hover:text-gray-700')
        .html('<i class="fas fa-times text-xl"></i>')
        .click(() => $modal.remove());

    $header.append($closeBtn);

    const $body = $('<div>')
        .addClass('p-4')
        .html('<div class="text-center py-8"><i class="fas fa-spinner fa-spin text-2xl text-gray-400"></i><p class="mt-2 text-gray-600">Loading history...</p></div>');

    $content.append($header, $body);
    $modal.append($content);
    $('body').append($modal);

    // Load change history
    try {
        const history = await fetchChangeHistory(recordId);
        renderChangeHistory(history, $body);
    } catch (error) {
        $body.html(`
            <div class="text-center py-8 text-red-500">
                <i class="fas fa-exclamation-triangle text-4xl mb-2"></i>
                <p>Failed to load change history</p>
                <p class="text-sm mt-1">${error.message}</p>
            </div>
        `);
    }

    // Close on background click
    $modal.click((e) => {
        if (e.target === $modal[0]) {
            $modal.remove();
        }
    });
}

// Fetch change history for a record
async function fetchChangeHistory(recordId) {
    // Simulate fetching history - in production this would call an API
    // This is a placeholder that would normally fetch from backend
    return [
        {
            id: '1',
            timestamp: new Date(Date.now() - 86400000).toISOString(),
            user: 'john@example.com',
            action: 'update',
            changes: {
                name: { old: 'Old Name', new: 'New Name' },
                status: { old: 'inactive', new: 'active' }
            }
        },
        {
            id: '2',
            timestamp: new Date(Date.now() - 172800000).toISOString(),
            user: 'admin@example.com',
            action: 'update',
            changes: {
                email: { old: 'old@example.com', new: 'new@example.com' }
            }
        },
        {
            id: '3',
            timestamp: new Date(Date.now() - 259200000).toISOString(),
            user: 'admin@example.com',
            action: 'create',
            changes: {}
        }
    ];
}

// Render change history with diff view
function renderChangeHistory(history, $container) {
    $container.empty();

    if (!history || history.length === 0) {
        $container.html(`
            <div class="text-center py-8 text-gray-500">
                <i class="fas fa-inbox text-4xl mb-2"></i>
                <p>No change history found</p>
            </div>
        `);
        return;
    }

    const $timeline = $('<div>').addClass('space-y-4');

    history.forEach((entry, index) => {
        const $entry = $('<div>')
            .addClass('flex gap-4 p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors');

        // Timeline indicator
        const $indicator = $('<div>')
            .addClass('flex flex-col items-center')
            .html(`
                <div class="w-10 h-10 rounded-full flex items-center justify-center ${
                    entry.action === 'create' ? 'bg-green-100 text-green-600' :
                    entry.action === 'update' ? 'bg-blue-100 text-blue-600' :
                    'bg-red-100 text-red-600'
                }">
                    <i class="fas ${
                        entry.action === 'create' ? 'fa-plus' :
                        entry.action === 'update' ? 'fa-edit' :
                        'fa-trash'
                    }"></i>
                </div>
                ${index < history.length - 1 ? '<div class="flex-1 w-0.5 bg-gray-200 mt-2"></div>' : ''}
            `);

        // Entry details
        const $details = $('<div>').addClass('flex-1');

        const $meta = $('<div>')
            .addClass('flex items-center gap-3 mb-2')
            .html(`
                <span class="font-semibold text-gray-900">${entry.user}</span>
                <span class="text-sm text-gray-500">${new Date(entry.timestamp).toLocaleString()}</span>
                <span class="px-2 py-1 text-xs rounded-full ${
                    entry.action === 'create' ? 'bg-green-100 text-green-800' :
                    entry.action === 'update' ? 'bg-blue-100 text-blue-800' :
                    'bg-red-100 text-red-800'
                }">${entry.action.toUpperCase()}</span>
            `);

        $details.append($meta);

        // Show changes with diff view
        if (entry.changes && Object.keys(entry.changes).length > 0) {
            const $changes = $('<div>').addClass('mt-2 space-y-1');

            Object.entries(entry.changes).forEach(([field, change]) => {
                const $change = $('<div>')
                    .addClass('text-sm p-2 bg-gray-50 rounded')
                    .html(`
                        <div class="font-medium text-gray-700 mb-1">${field}</div>
                        <div class="flex items-center gap-2">
                            <span class="px-2 py-1 bg-red-100 text-red-800 rounded line-through">${change.old || 'empty'}</span>
                            <i class="fas fa-arrow-right text-gray-400"></i>
                            <span class="px-2 py-1 bg-green-100 text-green-800 rounded">${change.new || 'empty'}</span>
                        </div>
                    `);

                $changes.append($change);
            });

            $details.append($changes);

            // Rollback button
            if (entry.action !== 'create') {
                const $rollbackBtn = $('<button>')
                    .addClass('mt-2 px-3 py-1 text-xs bg-amber-600 text-white rounded hover:bg-amber-700 transition-colors')
                    .html('<i class="fas fa-undo mr-1"></i>Rollback to this version')
                    .click(() => confirmRollback(entry));

                $details.append($rollbackBtn);
            }
        } else if (entry.action === 'create') {
            $details.append('<div class="text-sm text-gray-600">Record created</div>');
        }

        $entry.append($indicator, $details);
        $timeline.append($entry);
    });

    $container.append($timeline);
}

// Confirm and perform rollback
async function confirmRollback(historyEntry) {
    const confirmed = confirm(
        `Are you sure you want to rollback to this version?\n\nThis will undo all changes made after ${new Date(historyEntry.timestamp).toLocaleString()}`
    );

    if (!confirmed) {
        return;
    }

    try {
        // In production, this would call an API to perform the rollback
        showInfo('Rollback functionality would restore this version');

        // Close modal and refresh
        $('#history-modal').remove();

        // showSuccess('Record rolled back successfully');
    } catch (error) {
        showError(`Failed to rollback: ${error.message}`);
    }
}

// Display activity log
function showActivityLog() {
    // Create modal
    const $modal = $('<div>')
        .addClass('fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50')
        .attr('id', 'activity-modal');

    const $content = $('<div>')
        .addClass('bg-white rounded-lg shadow-xl max-w-6xl w-full m-4 max-h-[90vh] overflow-y-auto');

    const $header = $('<div>')
        .addClass('p-4 border-b flex justify-between items-center');

    $header.append(`
        <h3 class="text-xl font-semibold flex items-center gap-2">
            <i class="fas fa-list-ul text-indigo-600"></i>
            Activity Log
        </h3>
    `);

    const $closeBtn = $('<button>')
        .addClass('text-gray-500 hover:text-gray-700')
        .html('<i class="fas fa-times text-xl"></i>')
        .click(() => $modal.remove());

    $header.append($closeBtn);

    // Filters
    const $filters = $('<div>')
        .addClass('p-4 bg-gray-50 border-b flex gap-3')
        .html(`
            <select class="px-3 py-2 border border-gray-300 rounded-lg text-sm">
                <option value="all">All Actions</option>
                <option value="create">Create</option>
                <option value="update">Update</option>
                <option value="delete">Delete</option>
            </select>
            <select class="px-3 py-2 border border-gray-300 rounded-lg text-sm">
                <option value="all">All Users</option>
                <option value="admin">Admin</option>
                <option value="john">John</option>
            </select>
            <input type="date" class="px-3 py-2 border border-gray-300 rounded-lg text-sm" placeholder="From date">
            <input type="date" class="px-3 py-2 border border-gray-300 rounded-lg text-sm" placeholder="To date">
            <button class="px-4 py-2 bg-indigo-600 text-white rounded-lg text-sm hover:bg-indigo-700">
                <i class="fas fa-filter mr-2"></i>Filter
            </button>
        `);

    const $body = $('<div>')
        .addClass('p-4')
        .html('<div class="text-center py-8"><i class="fas fa-spinner fa-spin text-2xl text-gray-400"></i><p class="mt-2 text-gray-600">Loading activity log...</p></div>');

    $content.append($header, $filters, $body);
    $modal.append($content);
    $('body').append($modal);

    // Load activity log
    setTimeout(() => {
        const activities = generateSampleActivityLog();
        renderActivityLog(activities, $body);
    }, 500);

    // Close on background click
    $modal.click((e) => {
        if (e.target === $modal[0]) {
            $modal.remove();
        }
    });
}

// Generate sample activity log data
function generateSampleActivityLog() {
    return [
        {
            id: '1',
            timestamp: new Date(Date.now() - 3600000).toISOString(),
            user: 'admin@example.com',
            action: 'update',
            section: 'users',
            record_id: '123',
            description: 'Updated user profile'
        },
        {
            id: '2',
            timestamp: new Date(Date.now() - 7200000).toISOString(),
            user: 'john@example.com',
            action: 'create',
            section: 'products',
            record_id: '456',
            description: 'Created new product'
        },
        {
            id: '3',
            timestamp: new Date(Date.now() - 10800000).toISOString(),
            user: 'admin@example.com',
            action: 'delete',
            section: 'users',
            record_id: '789',
            description: 'Deleted inactive user'
        },
        {
            id: '4',
            timestamp: new Date(Date.now() - 14400000).toISOString(),
            user: 'john@example.com',
            action: 'update',
            section: 'settings',
            record_id: '1',
            description: 'Updated system settings'
        },
        {
            id: '5',
            timestamp: new Date(Date.now() - 18000000).toISOString(),
            user: 'admin@example.com',
            action: 'create',
            section: 'users',
            record_id: '999',
            description: 'Created new admin user'
        }
    ];
}

// Render activity log table
function renderActivityLog(activities, $container) {
    $container.empty();

    if (!activities || activities.length === 0) {
        $container.html(`
            <div class="text-center py-8 text-gray-500">
                <i class="fas fa-inbox text-4xl mb-2"></i>
                <p>No activities found</p>
            </div>
        `);
        return;
    }

    const $table = $('<table>').addClass('min-w-full divide-y divide-gray-200');
    const $thead = $('<thead>').addClass('bg-gray-50');
    const $tbody = $('<tbody>').addClass('divide-y divide-gray-200');

    // Header
    $thead.html(`
        <tr>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-700 uppercase tracking-wider">Timestamp</th>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-700 uppercase tracking-wider">User</th>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-700 uppercase tracking-wider">Action</th>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-700 uppercase tracking-wider">Section</th>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-700 uppercase tracking-wider">Record ID</th>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-700 uppercase tracking-wider">Description</th>
        </tr>
    `);

    // Rows
    activities.forEach(activity => {
        const $row = $('<tr>').addClass('hover:bg-gray-50');

        $row.html(`
            <td class="px-4 py-3 text-sm text-gray-900">${new Date(activity.timestamp).toLocaleString()}</td>
            <td class="px-4 py-3 text-sm text-gray-900">${activity.user}</td>
            <td class="px-4 py-3 text-sm">
                <span class="px-2 py-1 text-xs rounded-full ${
                    activity.action === 'create' ? 'bg-green-100 text-green-800' :
                    activity.action === 'update' ? 'bg-blue-100 text-blue-800' :
                    'bg-red-100 text-red-800'
                }">${activity.action.toUpperCase()}</span>
            </td>
            <td class="px-4 py-3 text-sm text-gray-900">${activity.section}</td>
            <td class="px-4 py-3 text-sm text-gray-900">${activity.record_id}</td>
            <td class="px-4 py-3 text-sm text-gray-600">${activity.description}</td>
        `);

        $tbody.append($row);
    });

    $table.append($thead, $tbody);

    const $tableContainer = $('<div>').addClass('overflow-x-auto rounded-lg border border-gray-200');
    $tableContainer.append($table);

    $container.append($tableContainer);

    // Export button
    const $exportBtn = $('<button>')
        .addClass('mt-4 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors')
        .html('<i class="fas fa-download mr-2"></i>Export Activity Log')
        .click(() => {
            exportActivityLogToCSV(activities);
        });

    $container.append($exportBtn);
}

// Export activity log to CSV
function exportActivityLogToCSV(activities) {
    const headers = ['Timestamp', 'User', 'Action', 'Section', 'Record ID', 'Description'];
    const csvRows = [];
    csvRows.push(headers.join(','));

    activities.forEach(activity => {
        const row = [
            new Date(activity.timestamp).toISOString(),
            activity.user,
            activity.action,
            activity.section,
            activity.record_id,
            activity.description
        ];
        csvRows.push(row.map(v => `"${String(v).replace(/"/g, '""')}"`).join(','));
    });

    const csvContent = csvRows.join('\n');
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.setAttribute('href', url);
    link.setAttribute('download', `activity-log-${new Date().toISOString().slice(0, 10)}.csv`);
    link.style.visibility = 'hidden';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);

    showSuccess('Activity log exported to CSV');
}

// Inline editing functionality
function makeInlineEditable($cell, field, row) {
    // Don't edit if already editing
    if ($cell.find('input, select, textarea').length > 0) {
        return;
    }

    const originalValue = row[field.id] || '';
    const formattedValue = formatFieldValue(originalValue, field);

    // Create input based on field type
    let $input;

    if (field.field_type === 'boolean' || field.field_type === 'toggle') {
        $input = $('<select>')
            .addClass('w-full px-2 py-1 border border-indigo-500 rounded focus:outline-none focus:ring-2 focus:ring-indigo-500')
            .append($('<option>').val('false').text('No').prop('selected', !originalValue))
            .append($('<option>').val('true').text('Yes').prop('selected', originalValue));
    } else if (field.field_type === 'textarea') {
        $input = $('<textarea>')
            .addClass('w-full px-2 py-1 border border-indigo-500 rounded focus:outline-none focus:ring-2 focus:ring-indigo-500')
            .val(originalValue)
            .attr('rows', 2);
    } else {
        let inputType = 'text';
        if (field.field_type === 'number' || field.field_type === 'currency') {
            inputType = 'number';
        } else if (field.field_type === 'email') {
            inputType = 'email';
        } else if (field.field_type === 'date') {
            inputType = 'date';
        }

        $input = $('<input>')
            .attr('type', inputType)
            .addClass('w-full px-2 py-1 border border-indigo-500 rounded focus:outline-none focus:ring-2 focus:ring-indigo-500')
            .val(originalValue);
    }

    // Save value function
    function saveValue() {
        let newValue = $input.val();

        // Convert boolean string to actual boolean
        if (field.field_type === 'boolean' || field.field_type === 'toggle') {
            newValue = newValue === 'true';
        } else if (field.field_type === 'number' || field.field_type === 'currency') {
            newValue = parseFloat(newValue) || 0;
        }

        // Only save if value changed
        if (newValue !== originalValue) {
            // Update the row data
            row[field.id] = newValue;

            // Find update action
            const updateAction = currentSection.actions.find(a =>
                a.type === 'form' && a.config && a.config.form_mode === 'update'
            );

            if (updateAction) {
                // Save to backend
                const url = `/api/backoffices/${currentBackoffice.id}/sections/${currentSection.id}/actions/${updateAction.id}`;

                $.ajax({
                    url: url,
                    method: 'POST',
                    contentType: 'application/json',
                    data: JSON.stringify(row),
                    success: function() {
                        $cell.text(formatFieldValue(newValue, field));
                        $cell.addClass('bg-green-100');
                        setTimeout(() => $cell.removeClass('bg-green-100'), 1000);
                        showSuccess('Field updated successfully');
                    },
                    error: function(err) {
                        $cell.text(formattedValue);
                        showError('Failed to update: ' + (err.responseJSON?.error || err.responseText));
                    }
                });
            } else {
                // No update action, just update UI
                $cell.text(formatFieldValue(newValue, field));
                showWarning('Saved locally only (no update action configured)');
            }
        } else {
            // Restore original text
            $cell.text(formattedValue);
        }
    }

    // Cancel editing function
    function cancelEdit() {
        $cell.text(formattedValue);
    }

    // Replace cell content with input
    $cell.empty().append($input);
    $input.focus().select();

    // Save on Enter, cancel on Escape
    $input.on('keydown', function(e) {
        if (e.key === 'Enter' && field.field_type !== 'textarea') {
            e.preventDefault();
            saveValue();
        } else if (e.key === 'Escape') {
            e.preventDefault();
            cancelEdit();
        }
    });

    // Save on blur (click outside)
    $input.on('blur', function() {
        // Small delay to allow Enter key to process first
        setTimeout(saveValue, 100);
    });
}

// ===== BULK OPERATIONS =====

// Toggle select all checkboxes
function toggleSelectAll(checked) {
    $('.row-checkbox').prop('checked', checked);
    updateBulkActionsBar();
}

// Update bulk actions bar visibility and count
function updateBulkActionsBar() {
    const selectedCount = $('.row-checkbox:checked').length;
    const $bulkBar = $('#bulk-actions-bar');
    const $selectAllCheckbox = $('#select-all-checkbox');

    if (selectedCount > 0) {
        $bulkBar.removeClass('hidden').addClass('flex');
        $('#selected-count').text(`${selectedCount} row(s) selected`);
    } else {
        $bulkBar.removeClass('flex').addClass('hidden');
    }

    // Update select-all checkbox state
    const totalCheckboxes = $('.row-checkbox').length;
    $selectAllCheckbox.prop('checked', selectedCount === totalCheckboxes && totalCheckboxes > 0);
}

// Deselect all rows
function deselectAllRows() {
    $('.row-checkbox').prop('checked', false);
    $('#select-all-checkbox').prop('checked', false);
    updateBulkActionsBar();
}

// Get selected rows data
function getSelectedRows() {
    const selectedRows = [];
    $('.row-checkbox:checked').each(function() {
        selectedRows.push($(this).data('row-data'));
    });
    return selectedRows;
}

// Bulk delete rows
function bulkDeleteRows() {
    const selectedRows = getSelectedRows();
    if (selectedRows.length === 0) {
        showWarning('No rows selected');
        return;
    }

    if (!confirm(`Are you sure you want to delete ${selectedRows.length} row(s)?`)) {
        return;
    }

    const deleteAction = currentSection.actions.find(a => a.type === 'delete' || a.action_type === 'delete');

    if (!deleteAction) {
        showError('No delete action configured for this section');
        return;
    }

    let completed = 0;
    let failed = 0;

    showProgress('Deleting rows...', 0, selectedRows.length);

    selectedRows.forEach((row, index) => {
        const url = `/api/backoffices/${currentBackoffice.id}/sections/${currentSection.id}/actions/${deleteAction.id}`;

        $.ajax({
            url: url,
            method: 'POST',
            contentType: 'application/json',
            data: JSON.stringify({ id: row.id }),
            success: function() {
                completed++;
                updateProgress(completed + failed, selectedRows.length);

                if (completed + failed === selectedRows.length) {
                    finishBulkOperation(completed, failed);
                }
            },
            error: function() {
                failed++;
                updateProgress(completed + failed, selectedRows.length);

                if (completed + failed === selectedRows.length) {
                    finishBulkOperation(completed, failed);
                }
            }
        });
    });
}

// Bulk export selected rows
function bulkExportRows() {
    const selectedRows = getSelectedRows();
    if (selectedRows.length === 0) {
        showWarning('No rows selected');
        return;
    }

    const visibleFields = currentAction.action_type.fields.filter(f => f.visible);
    exportTableToCSV(selectedRows, visibleFields);
    showSuccess(`Exported ${selectedRows.length} selected row(s)`);
}

// ===== COLUMN SORTING =====

let currentSortField = null;
let currentSortDirection = 'none';

function sortByColumn(fieldId, $sortIcon) {
    const $tbody = $('#data-table tbody');
    const rows = $tbody.find('tr').toArray();

    // Update sort direction
    const currentDirection = $sortIcon.attr('data-sort-direction');
    let newDirection;

    if (currentDirection === 'none' || currentDirection === 'desc') {
        newDirection = 'asc';
        $sortIcon.removeClass('fa-sort fa-sort-down').addClass('fa-sort-up');
    } else {
        newDirection = 'desc';
        $sortIcon.removeClass('fa-sort fa-sort-up').addClass('fa-sort-down');
    }

    // Reset other column icons
    $('th[data-field-id] i').not($sortIcon).removeClass('fa-sort-up fa-sort-down').addClass('fa-sort').attr('data-sort-direction', 'none');

    $sortIcon.attr('data-sort-direction', newDirection);
    currentSortField = fieldId;
    currentSortDirection = newDirection;

    // Sort rows
    rows.sort(function(a, b) {
        const aValue = $(a).find(`td[data-field-id="${fieldId}"]`).text().trim();
        const bValue = $(b).find(`td[data-field-id="${fieldId}"]`).text().trim();

        // Try to parse as numbers
        const aNum = parseFloat(aValue);
        const bNum = parseFloat(bValue);

        let comparison;
        if (!isNaN(aNum) && !isNaN(bNum)) {
            comparison = aNum - bNum;
        } else {
            comparison = aValue.localeCompare(bValue);
        }

        return newDirection === 'asc' ? comparison : -comparison;
    });

    // Reorder DOM
    $tbody.empty().append(rows);
    showInfo(`Sorted by ${fieldId} (${newDirection})`);
}

// ===== FILTER PRESETS =====

function saveFilterPreset() {
    const presetName = prompt('Enter a name for this filter preset:');
    if (!presetName) return;

    const currentFilters = {};
    $('#filter-grid input, #filter-grid select').each(function() {
        const $input = $(this);
        const fieldId = $input.attr('data-field-id');
        if (fieldId) {
            currentFilters[fieldId] = {
                value: $input.val(),
                operator: $input.attr('data-operator') || 'equals'
            };
        }
    });

    const presets = JSON.parse(localStorage.getItem('filterPresets') || '{}');
    presets[presetName] = currentFilters;
    localStorage.setItem('filterPresets', JSON.stringify(presets));

    // Update dropdown
    const $select = $('#filter-panel select');
    $select.append($('<option>').val(presetName).text(presetName));

    showSuccess(`Filter preset "${presetName}" saved`);
}

function loadFilterPreset(presetName) {
    if (!presetName) return;

    const presets = JSON.parse(localStorage.getItem('filterPresets') || '{}');
    const preset = presets[presetName];

    if (!preset) {
        showError('Preset not found');
        return;
    }

    // Apply preset filters
    Object.keys(preset).forEach(fieldId => {
        const filter = preset[fieldId];
        const $input = $(`#filter-grid input[data-field-id="${fieldId}"], #filter-grid select[data-field-id="${fieldId}"]`);
        if ($input.length) {
            $input.val(filter.value);
            $input.attr('data-operator', filter.operator);
        }
    });

    showSuccess(`Loaded filter preset "${presetName}"`);
}

// Load saved presets on startup
function initializeFilterPresets() {
    const presets = JSON.parse(localStorage.getItem('filterPresets') || '{}');
    const $select = $('#filter-panel select');

    Object.keys(presets).forEach(presetName => {
        $select.append($('<option>').val(presetName).text(presetName));
    });
}

// ===== CSV IMPORT =====

function showImportDialog() {
    const dialogHtml = `
        <div class="p-4">
            <h3 class="text-lg font-bold mb-4">Import CSV</h3>
            <p class="text-sm text-gray-600 mb-4">Select a CSV file to import. The first row should contain column headers.</p>

            <input type="file" id="csv-file-input" accept=".csv" class="mb-4 block w-full text-sm text-gray-500
                file:mr-4 file:py-2 file:px-4
                file:rounded file:border-0
                file:text-sm file:font-semibold
                file:bg-indigo-50 file:text-indigo-700
                hover:file:bg-indigo-100" />

            <div class="flex justify-end gap-2 mt-6">
                <button onclick="closeModal()" class="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50">Cancel</button>
                <button onclick="processCSVImport()" class="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700">Import</button>
            </div>
        </div>
    `;

    $('#modal-title').text('Import CSV');
    $('#form-fields').html(dialogHtml);
    $('#submit-text').parent().hide();
    $('#formModal').addClass('active');
}

function processCSVImport() {
    const fileInput = document.getElementById('csv-file-input');
    if (!fileInput.files.length) {
        showError('Please select a file');
        return;
    }

    const file = fileInput.files[0];
    const reader = new FileReader();

    reader.onload = function(e) {
        const csvContent = e.target.result;
        const rows = csvContent.split('\n').map(row => row.split(','));

        if (rows.length < 2) {
            showError('CSV file must have at least a header row and one data row');
            return;
        }

        const headers = rows[0].map(h => h.trim());
        const dataRows = rows.slice(1).filter(row => row.length > 1);

        closeModal();
        showProgress('Importing rows...', 0, dataRows.length);

        const createAction = currentSection.actions.find(a =>
            a.type === 'form' && a.config && a.config.form_mode === 'create'
        );

        if (!createAction) {
            showError('No create action configured for this section');
            return;
        }

        let completed = 0;
        let failed = 0;

        dataRows.forEach((row, index) => {
            const rowData = {};
            headers.forEach((header, i) => {
                if (row[i]) {
                    rowData[header] = row[i].trim();
                }
            });

            const url = `/api/backoffices/${currentBackoffice.id}/sections/${currentSection.id}/actions/${createAction.id}`;

            $.ajax({
                url: url,
                method: 'POST',
                contentType: 'application/json',
                data: JSON.stringify(rowData),
                success: function() {
                    completed++;
                    updateProgress(completed + failed, dataRows.length);

                    if (completed + failed === dataRows.length) {
                        finishBulkOperation(completed, failed, true);
                    }
                },
                error: function() {
                    failed++;
                    updateProgress(completed + failed, dataRows.length);

                    if (completed + failed === dataRows.length) {
                        finishBulkOperation(completed, failed, true);
                    }
                }
            });
        });
    };

    reader.readAsText(file);
}

// ===== PROGRESS INDICATORS =====

function showProgress(message, current, total) {
    const percentage = total > 0 ? Math.round((current / total) * 100) : 0;

    const progressHtml = `
        <div id="progress-indicator" class="fixed bottom-4 right-4 bg-white p-4 rounded-lg shadow-xl border border-gray-200 min-w-80 z-50">
            <div class="flex items-center gap-3 mb-2">
                <div class="loading"></div>
                <span class="text-sm font-medium">${message}</span>
            </div>
            <div class="w-full bg-gray-200 rounded-full h-2.5">
                <div class="bg-indigo-600 h-2.5 rounded-full transition-all" style="width: ${percentage}%"></div>
            </div>
            <div class="text-xs text-gray-500 mt-1">${current} / ${total} (${percentage}%)</div>
        </div>
    `;

    $('#progress-indicator').remove();
    $('body').append(progressHtml);
}

function updateProgress(current, total) {
    const percentage = total > 0 ? Math.round((current / total) * 100) : 0;
    $('#progress-indicator .bg-indigo-600').css('width', percentage + '%');
    $('#progress-indicator .text-xs').text(`${current} / ${total} (${percentage}%)`);
}

function hideProgress() {
    $('#progress-indicator').fadeOut(300, function() {
        $(this).remove();
    });
}

function finishBulkOperation(completed, failed, isImport = false) {
    setTimeout(() => {
        hideProgress();

        if (failed === 0) {
            showSuccess(`Successfully ${isImport ? 'imported' : 'processed'} ${completed} row(s)`);
        } else {
            showWarning(`Completed: ${completed}, Failed: ${failed}`);
        }

        // Reload data
        if (currentAction) {
            executeAction(currentAction);
        }

        // Deselect all
        deselectAllRows();
    }, 500);
}
