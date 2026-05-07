# Design System — Example

> Status: Draft
> Owner: TBD

## 1. Tokens

### Colors
| Token | Light | Dark | Use |
|---|---|---|---|
| `--bg` | `#ffffff` | `#0a0a0a` | Page background |
| `--fg` | `#0a0a0a` | `#fafafa` | Primary text |
| `--muted` | `#737373` | `#a3a3a3` | Secondary text |
| `--accent` | `#3b82f6` | `#60a5fa` | Primary buttons, links |
| `--success` | `#22c55e` | `#4ade80` | Done / success state |
| `--warning` | `#eab308` | `#facc15` | Pending / in-review state |
| `--danger` | `#ef4444` | `#f87171` | Errors, destructive actions |

### Spacing
- 4 / 8 / 12 / 16 / 24 / 32 / 48 / 64 px.

### Typography
- System font stack (`-apple-system, BlinkMacSystemFont, "Segoe UI", …`).
- Base size 14 px; line-height 1.5.

## 2. Components

### Button
- Variants: `primary` | `secondary` | `ghost` | `destructive`.
- States: default / hover / active / disabled / loading.
- Size: `sm` (28 px) | `md` (36 px) | `lg` (44 px).

### Card
- 1 px border, 6 px radius, subtle shadow.
- Padding 16 px, gap 12 px.

### Form fields
- Inline error in `--danger` below the field.
- Required marker `*` after the label.
- Disabled state at 50% opacity, `not-allowed` cursor.

## 3. Layout

- Max content width: 1024 px.
- Sidebar 240 px, sticky on desktop, drawer on mobile.

## 4. Accessibility

- Color contrast ≥ 4.5:1 for body text.
- Focus rings visible on every interactive element.
- All form fields labeled; errors announced via `aria-live="polite"`.
