#!/usr/bin/env python3
"""Split app.css into component-scoped CSS files.
Parses flat rule blocks, classifies each, preserves @media wrappers."""

import os

def read_file(path):
    with open(path, 'r') as f:
        return f.read()

def write_file(path, content):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w') as f:
        f.write(content)

# --- CSS block parsing (handles nesting) ---

def parse_css_blocks(text):
    """Parse CSS into blocks. Each block is (selector, body, is_atrule, full_text).
    For @media, returns inner blocks separately."""
    blocks = []
    i = 0
    n = len(text)

    while i < n:
        # Skip whitespace and comments
        while i < n and text[i] in ' \t\n\r':
            i += 1
        if i >= n:
            break

        # Skip comments
        if i + 1 < n and text[i] == '/' and text[i+1] == '*':
            j = text.find('*/', i+2)
            if j == -1:
                break
            i = j + 2
            continue

        # Find the start of a CSS rule
        start = i
        # Find opening brace
        brace = text.find('{', i)
        if brace == -1:
            break

        selector = text[start:brace].strip()

        # Find matching closing brace
        depth = 1
        j = brace + 1
        while j < n and depth > 0:
            if text[j] == '{':
                depth += 1
            elif text[j] == '}':
                depth -= 1
            j += 1

        if depth != 0:
            break

        body = text[brace+1:j-1]
        full = text[start:j]

        blocks.append({
            'selector': selector,
            'body': body,
            'full': full,
            'is_atrule': selector.startswith('@'),
        })

        i = j

    return blocks

def get_first_class_from_selector(sel):
    """Get the first CSS class from a single selector."""
    # Strip pseudo-classes/elements
    s = sel.split(':')[0].split('::')[0].strip()
    # Get last class in case of descendant selectors
    parts = s.split()
    for p in parts:
        if p.startswith('.'):
            return p
    return ''

def selector_matches_prefixes(sel_text, prefixes):
    """Check if ALL comma-separated selectors match any of the prefixes."""
    selectors = [s.strip() for s in sel_text.split(',')]
    for s in selectors:
        cls = get_first_class_from_selector(s)
        if not cls:
            return False
        matched = False
        for pfx in prefixes:
            if cls.startswith(pfx):
                matched = True
                break
        if not matched:
            return False
    return True

def block_matches_prefixes(block, prefixes):
    """Check if a block matches (handles @media by checking inner rules)."""
    if block['is_atrule']:
        sel = block['selector']
        if sel.startswith('@keyframes'):
            # Check keyframes name against prefixes
            parts = sel.split()
            name = parts[1] if len(parts) > 1 else ''
            for pfx in prefixes:
                if pfx.startswith('@keyframes'):
                    kf_name = pfx.replace('@keyframes ', '').replace('@keyframes', '')
                    if name.startswith(kf_name):
                        return True
            return False
        elif sel.startswith('@media'):
            # Check ALL inner blocks match
            inner_blocks = parse_css_blocks(block['body'])
            if not inner_blocks:
                return False
            for ib in inner_blocks:
                if not block_matches_prefixes(ib, prefixes):
                    return False
            return True
        else:
            return False
    else:
        return selector_matches_prefixes(block['selector'], prefixes)

def extract_from_media_block(block, prefixes):
    """Extract matching inner rules from a @media block.
    Returns (matched_inner_blocks, unmatched_inner_blocks)."""
    inner_blocks = parse_css_blocks(block['body'])
    matched = []
    unmatched = []

    for ib in inner_blocks:
        if block_matches_prefixes(ib, prefixes):
            matched.append(ib)
        else:
            unmatched.append(ib)

    return matched, unmatched

def blocks_to_css(blocks):
    """Convert list of block dicts to CSS text."""
    return '\n\n'.join(b['full'].strip() for b in blocks) + '\n'

def make_media_block(media_selector, inner_blocks):
    """Create a @media block wrapping inner blocks."""
    inner_css = '\n'.join(b['full'].strip() for b in inner_blocks)
    return f'{media_selector} {{\n{inner_css}\n}}'

# --- Group definitions ---

GROUP_PREFIXES = {
    'vault': ['.vault-'],
    'sidebar': [
        '.sidebar', '.brand', '.brand-mark-button', '.brand-mark',
        '.brand-name', '.brand-subtitle', '.brand-copy',
        '.nav-group', '.nav-group-label', '.nav-entry',
        '.primary-nav', '.nav-disclosure',
        '.communications-subnav', '.communications-rail-dropdown',
        '.subnav-', '.sidebar-tools', '.settings-link',
        '.sidebar-rail-dropdown-backdrop',
        '.sidebar-settings-', '.sidebar-group-create', '.sidebar-config-',
        '.sidebar-preview-',
    ],
    'topbar': [
        '.topbar', '.topbar-title', '.top-actions',
        '.user-menu', '.user-menu-',
        '.menu-button', '.icon-button', '.segmented',
        '.search-bar', '.view-header', '.view-title-with-icon',
        '.hero-mark', '.section-tabs', '.pill-tabs',
        '.filter-bar', '.filter-tabs',
        '.primary-button', '.ghost-button', '.link-row', '.link-button',
        '.kbd',
    ],
    'notifications': [
        '.notifications-', '.notification-',
        '.workspace', '.workspace-status-strip',
    ],
    'panels': [
        '.panel', '.panel-', '.widget-frame', '.widget-',
        '.info-card', '.metric-grid', '.metric-card',
        '.stacked-rail', '.empty-panel', '.muted-copy',
        '.detail-list', '.detail-row', '.health-row', '.round-icon',
        '.chip', '.status-chip', '.health-chip', '.deadline',
        '.bar-row', '.mini-check', '.collection-row',
        '.source-card', '.source-strip', '.table-head', '.task-row',
        '.doc-row', '.person-compact', '.profile-panel', '.quick-icons',
        '.chat-pane', '.chat-body', '.chat-actions', '.bubble',
        '.date-divider', '.composer', '.conversation-list',
        '.feed-panel', '.feed-row', '.person-list',
        '.schedule-panel', '.schedule-list', '.compact-project',
        '.project-card-row', '.full-band', '.score-ring', '.donut',
        '.chart-panel',
        '.persons-list-panel', '.profile-head', '.person-row',
        '.hero-row', '.layout-edit-controls', '.layout-zone',
        '.communication-empty-', '.related-row', '.inline-error',
        '.inline-metrics', '.inline-copy', '.summary-numbers',
        '.home-metrics', '.radial-graph', '.graph-center', '.graph-chip',
        '.source-footer', '.source-badge',
        '.status-list', '.timeline-mini', '.big-score', '.progress',
        '.task-stack', '.task-actions', '.task-row-actions',
        '.feed-list', '.left-panels', '.new-tile',
        '.mail-', '.state-badge', '.draft-', '.health-strip',
        '.search-hint', '.importance-dot',
        '.layout-widget-', '.widget-drawer',
    ],
    'pages': [
        '.home-page', '.dashboard-grid', '.communications-page',
        '.three-pane', '.context-rail', '.persons-page', '.projects-page',
        '.tasks-page', '.tasks-layout', '.calendar-page', '.calendar-layout',
        '.documents-page', '.documents-layout', '.notes-page', '.notes-layout',
        '.knowledge-page', '.agents-page', '.agents-layout',
        '.organizations-page', '.org-layout', '.timeline-page', '.timeline-layout',
        '.settings-page', '.settings-layout', '.agent-card',
        '.org-row', '.event-row', '.note-card', '.document-row',
        '.project-side', '.telegram-rail', '.whatsapp-rail',
        '.person-detail', '.identity-candidate',
        '.graph-canvas', '.graph-node',
        '.week-board', '.event-list', '.new-event-form', '.org-detail-grid',
        '.timeline-event-row', '.task-table', '.task-group',
        '.docs-table', '.category-grid', '.tag-cloud', '.notes-list',
        '.settings-', '.setting-', '.appearance-', '.brightness-', '.accent-swatch',
        '.account-', '.wizard-', '.setup-', '.provider-', '.qr-',
        '.org-list-panel', '.org-detail-',
        '.project-hero', '.project-logo', '.project-meta-strip',
        '.project-empty-state', '.project-dashboard-grid', '.project-switcher',
        '.graph-', '.knowledge-', '.agent-', '.ai-', '.evidence-',
        '.timeline-', '.spark-', '.large-timeline',
        '.person-hero', '.person-cards', '.person-detail',
        '.doc-mini', '.notes-main', '.agent-main',
        '.identity-', '.node-detail-',
        '.conversation-', '.attachment-',
        '.event-block', '.time-grid', '.now-line', '.week-header',
        '.event-actions', '.event-detail', '.event-meta', '.event-type-chip',
        '.agenda-list', '.brief-', '.participant-chip',
        '.telegram-chat-pane', '.telegram-grid', '.telegram-inline-form',
        '.whatsapp-chat-pane', '.whatsapp-grid',
        '.calendar-layout', '.knowledge-layout', '.agents-layout',
        '.persons-layout', '.docs-layout', '.notes-layout',
        '.communications-grid', '.compact-form', '.checkbox-row',
        '.form-actions', '.form-row', '.oauth-box',
        '.drawer-backdrop', '.modal-backdrop', '.rail-dot',
        '.graph-chip-',
        '.citation-', '.background-option-', '.background-preview',
    ],
}

OUTPUT_FILES = {
    'vault': 'frontend/src/lib/components/vault/vault.css',
    'sidebar': 'frontend/src/lib/components/shell/sidebar.css',
    'topbar': 'frontend/src/lib/components/shell/topbar.css',
    'notifications': 'frontend/src/lib/components/shell/notifications.css',
    'panels': 'frontend/src/lib/components/shared/panels.css',
    'pages': 'frontend/src/lib/pages/pages.css',
}

IMPORTS = {
    'vault': ('frontend/src/lib/components/vault/VaultOnboarding.svelte', "import './vault.css';"),
    'sidebar': ('frontend/src/lib/components/shell/Sidebar.svelte', "import './sidebar.css';"),
    'topbar': ('frontend/src/lib/components/shell/Topbar.svelte', "import './topbar.css';"),
    'notifications': ('frontend/src/lib/components/shell/NotificationsDrawer.svelte', "import './notifications.css';"),
    'panels': ('frontend/src/lib/components/shared/WidgetEditChrome.svelte', "import './panels.css';"),
    'pages': ('frontend/src/routes/+layout.svelte', "import '$lib/pages/pages.css';"),
}

def process_blocks(blocks):
    """Process blocks recursively. Returns (extracted_by_group, remaining_blocks)."""
    extracted = {name: [] for name in GROUP_PREFIXES}
    remaining = []

    for block in blocks:
        if block['is_atrule'] and block['selector'].startswith('@media'):
            # Process @media block - extract inner rules per group
            inner_blocks = parse_css_blocks(block['body'])
            inner_extracted, inner_remaining = process_blocks(inner_blocks)

            # Add extracted inner blocks to their groups, wrapped in @media
            for group_name, inner_list in inner_extracted.items():
                if inner_list:
                    media_wrapper = make_media_block(block['selector'], inner_list)
                    extracted[group_name].append({
                        'selector': block['selector'],
                        'body': '\n'.join(b['full'] for b in inner_list),
                        'full': media_wrapper,
                        'is_atrule': True,
                    })

            # Keep remaining inner blocks in @media wrapper
            if inner_remaining:
                media_wrapper = make_media_block(block['selector'], inner_remaining)
                remaining.append({
                    'selector': block['selector'],
                    'body': '\n'.join(b['full'] for b in inner_remaining),
                    'full': media_wrapper,
                    'is_atrule': True,
                })
        elif block['is_atrule'] and block['selector'].startswith('@keyframes'):
            # Check if keyframes match any group
            matched = False
            for group_name, prefixes in GROUP_PREFIXES.items():
                if block_matches_prefixes(block, prefixes):
                    extracted[group_name].append(block)
                    matched = True
                    break
            if not matched:
                remaining.append(block)
        else:
            # Regular rule - check all groups
            matched = False
            for group_name, prefixes in GROUP_PREFIXES.items():
                if block_matches_prefixes(block, prefixes):
                    extracted[group_name].append(block)
                    matched = True
                    break
            if not matched:
                remaining.append(block)

    return extracted, remaining

def main():
    css_path = 'frontend/src/lib/styles/app.css.bak'
    css_text = read_file(css_path)

    top_blocks = parse_css_blocks(css_text)
    print(f"Parsed {len(top_blocks)} top-level blocks")

    extracted, remaining = process_blocks(top_blocks)

    total_extracted = 0
    for group_name in GROUP_PREFIXES:
        blocks = extracted[group_name]
        total_extracted += len(blocks)
        if blocks:
            content = blocks_to_css(blocks)
            write_file(OUTPUT_FILES[group_name], content)
            print(f"  {group_name}: {len(blocks)} blocks -> {OUTPUT_FILES[group_name]}")
        else:
            print(f"  {group_name}: 0 blocks")

    remaining_css = blocks_to_css(remaining)
    write_file('frontend/src/lib/styles/app.css', remaining_css)
    remaining_lines = len(remaining_css.strip().split('\n')) if remaining_css.strip() else 0
    print(f"app.css: {len(remaining)} blocks remaining ({remaining_lines} lines)")
    print(f"Total extracted: {total_extracted}")

    # Add imports
    for group_name, (import_file, import_line) in IMPORTS.items():
        if os.path.exists(import_file):
            content = read_file(import_file)
            if import_line not in content:
                lines = content.split('\n')
                found = False
                for i, line in enumerate(lines):
                    if '<script' in line and 'lang="ts"' in line:
                        lines.insert(i + 1, f'\t{import_line}')
                        found = True
                        break
                if found:
                    write_file(import_file, '\n'.join(lines))
                    print(f"  Added import to {import_file}")
                else:
                    print(f"  WARNING: no <script lang=\"ts\"> in {import_file}")
            else:
                print(f"  Import already present in {import_file}")
        else:
            print(f"  WARNING: file not found: {import_file}")

if __name__ == '__main__':
    main()
