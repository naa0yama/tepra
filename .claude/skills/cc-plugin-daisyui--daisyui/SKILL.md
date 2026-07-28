---
name: cc-plugin-daisyui--daisyui
description: >-
  DaisyUI CSS component library plugin for Claude Code. Version 5.6.5 changes
  and usage notes. Use when working with the daisyui Claude Code plugin or
  building UI with DaisyUI components.
---

# daisyui 5.6.5

## Project Usage

- **Marketplace**: daisyui
- **Plugin name**: daisyui
- **Version**: 5.6.5 (tier 1, source `extraKnownMarketplaces.daisyui.source.ref`)
- **Source repo**: saadeghi/daisyui ref `v5.6.5`
- **Enabled**: yes
- **Installed**: yes (installed_plugins.json, installed at 5.6.0)

## Changes Since Knowledge Cutoff

### Breaking Changes

**v5.0.0 (March 2025)** — largest breaking change release:

- Removed `artboard` and `phone-*` classes; use Tailwind `w-*` / `h-*` instead
- Avatar state classes renamed: `online` → `avatar-online`, `offline` → `avatar-offline`
- Removed `bottom-nav`; replaced by `dock` component
- Removed `form-control` and `label-text`; use semantic `fieldset` / `legend` elements
- Input, select, textarea now have borders by default; use `-ghost` variant to remove border
- Menu item states: `disabled` → `menu-disabled`, `active` → `menu-active`
- Card: `card-bordered` → `card-border`
- Tabs: `tabs-lifted` → `tabs-lift`
- Table: removed `hover` class; use `hover:bg-base-300` instead
- Removed `btn-group` and `input-group`; use `join` component

**v5.6.0 (June 25, 2026)**:

- Button styles completely rewritten — checked, disabled, soft, ghost, link, and focus states changed
- Join styles simplified; nested join leakage prevented
- Menu elements now work as `.menu` containers (not wrapper-specific)

**v5.6.5**:

- `tab` class now prevents conflict with Tailwind's new optional `tab` utility

### New APIs / Components

**v5.0.0**:

- New button variants: `btn-dash`, `btn-soft`, `btn-xl`
- New badge variants: `badge-dash`, `badge-soft`, `badge-xl`
- Modal positioning: `modal-start`, `modal-end`
- Stack directional variants: `stack-bottom`, `stack-top`, `stack-start`, `stack-end`
- `step-icon` class for custom step icons
- Component size variant `xl` added across components
- Popover + CSS anchor positioning support for dropdowns
- Card variants: `card-sm`, `card-md`, `card-lg`, `card-xl`, `card-border`, `card-dash`

**v5.1.0 (September 1, 2025)**:

- New components: Hover Gallery, FAB / Speed Dial
- Native HTML `<select>` element styling (Chromium-based browsers)
- `prefers-reduced-motion` support across animations

**v5.2.0 (October 10, 2025)**:

- Drawer state variants: `is-drawer-open`, `is-drawer-close`
- Countdown 0–999 support with dynamic width and independent digit animation

**v5.3.0 (October 13, 2025)**:

- New CSS layers system for improved specificity — `btn-disabled` and card modifier classes now work with Tailwind variants

**v5.5.0 (November 11, 2025)**:

- New components: `hover-3d` (3D card effects), `text-rotate` (word rotation)
- `skeleton-text` animated gradient text variant
- `dropdown-close` modifier for forced dropdown closure
- Smooth transitions for details inside menu

**v5.6.0 (June 25, 2026)**:

- New components: Aura, OTP, Megamenu
- `range-vertical` for vertical range sliders
- HTML popover attribute support for modal
- Tooltip alignment utilities: `tooltip-start`, `tooltip-center`, `tooltip-end`
- Calendar styling integration for Vanilla Calendar Pro
- Responsive rating size modifiers
- Card focus and checked styling for selected/selectable states (`card focus`, `card checked`)

### Deprecations

None documented since May 2025 cutoff (v5.0 removed deprecated v4 classes).

## Gotchas

- v5.0.0 removed many v4 class names with no backward compat shim — audit all templates before upgrading from v4.
- Installed version (5.6.0) is behind skill ref (5.6.5); 5.6.1–5.6.5 are patch fixes:
  - 5.6.1: `btn-active` exposed as utility class
  - 5.6.2: outline/separators restored for joined buttons
  - 5.6.3: OTP alignment fix
  - 5.6.4: tooltip RTL positioning fix
  - 5.6.5: `tab` class conflict with Tailwind optional `tab` prevented
- CSS layers system (v5.3.0) improves Tailwind variant interop — if Tailwind variants were not working with DaisyUI modifiers, upgrading to ≥5.3.0 resolves it.
- Button style rewrite in v5.6.0 may require visual regression review in projects upgrading from 5.5.x.
- `prefers-reduced-motion`: v5.6.7 changed behavior from "no animation" to "slow animation" for loading indicators.
