---
name: gpui-design
description: Create distinctive, production-grade GPUI interfaces inspired by shadcn/ui design principles. Use this skill when the user asks to build GPUI components, widgets, pages, or desktop GUI applications in Rust — especially when they mention UI, layout, styling, or visual design for GPUI / Zed-style interfaces. Generates creative, polished Rust code that follows shadcn/ui aesthetics with GPUI's retained-mode GPU-accelerated architecture.
---

This skill guides creation of GPUI interfaces that follow shadcn/ui's design principles — beautiful, accessible, composable components. Implement real working Rust code with attention to visual details, accessibility, and the shadcn/ui "copy source" philosophy.

The user provides GPUI interface requirements: a component, page, widget, or application. They may describe the purpose, audience, or technical constraints.

## Design Thinking

Before writing code, understand the context and commit to a clear aesthetic direction:

- **Purpose**: What does this interface do? Who uses it? Is it a developer tool, settings panel, dashboard, or consumer application?
- **Tone**: shadcn/ui defaults to refined minimalism — clean surfaces, subtle shadows, high readability. Vary within this spectrum: data-dense dashboard, playful/colorful accents, dark atmospheric, soft/pastel, industrial/utilitarian. Commit to one and execute consistently.
- **Constraints**: GPUI version, existing theme system, platform (macOS primary, Linux/Windows via GPU), performance characteristics (retained mode means partial updates are efficient).
- **Differentiation**: What makes this interface memorable? shadcn/ui's strength is its restraint — every detail is intentional, nothing is gratuitous.

**CRITICAL**: GPUI is a retained-mode, GPU-accelerated framework. Unlike egui's immediate mode, elements persist in a tree and only changed subtrees re-render. Components are structs implementing `Render` or `RenderOnce`. Design decisions live in the component hierarchy and theme system. The shadcn/ui philosophy is "not a dependency — copy the source" — every component is yours to customize.

## shadcn/ui Design Principles Applied to GPUI

The shadcn/ui aesthetic is defined by specific, measurable choices:

### Color System (CSS Variables → Design Tokens)

shadcn/ui uses CSS variables for theming. In GPUI, define a `Theme` struct with semantic color tokens:

```rust
struct Theme {
    // Background hierarchy
    background: Hsla,        // --background
    foreground: Hsla,        // --foreground
    card: Hsla,              // --card
    card_foreground: Hsla,   // --card-foreground
    popover: Hsla,           // --popover
    popover_foreground: Hsla,// --popover-foreground
    // Brand
    primary: Hsla,           // --primary
    primary_foreground: Hsla,// --primary-foreground
    // Semantic
    secondary: Hsla,         // --secondary
    secondary_foreground: Hsla,
    muted: Hsla,             // --muted
    muted_foreground: Hsla,
    accent: Hsla,            // --accent
    accent_foreground: Hsla,
    destructive: Hsla,       // --destructive
    destructive_foreground: Hsla,
    // Borders & inputs
    border: Hsla,            // --border
    input: Hsla,             // --input
    ring: Hsla,              // --ring (focus ring)
    // Radius scale
    radius_sm: f32,          // calc(var(--radius) - 4px)
    radius: f32,             // --radius (default 0.5rem = 8px)
    radius_md: f32,          // calc(var(--radius) - 2px)
    radius_lg: f32,          // var(--radius)
    radius_xl: f32,          // calc(var(--radius) + 4px)
}
```

For dark mode, shadcn/ui inverts the background/foreground relationship while keeping the brand colors consistent. Provide both `light_theme()` and `dark_theme()` constructors.

### Spacing Scale

shadcn/ui uses a 4px base grid (Tailwind spacing scale). Define constants:

```rust
const SPACE_0: f32 = 0.0;
const SPACE_PX: f32 = 1.0;
const SPACE_0_5: f32 = 2.0;   // 0.125rem
const SPACE_1: f32 = 4.0;     // 0.25rem
const SPACE_2: f32 = 8.0;     // 0.5rem
const SPACE_3: f32 = 12.0;    // 0.75rem
const SPACE_4: f32 = 16.0;    // 1rem
const SPACE_5: f32 = 20.0;    // 1.25rem
const SPACE_6: f32 = 24.0;    // 1.5rem
const SPACE_8: f32 = 32.0;    // 2rem
const SPACE_10: f32 = 40.0;   // 2.5rem
const SPACE_12: f32 = 48.0;   // 3rem
```

Every margin, padding, and gap value must reference these tokens — never hardcode pixel values.

### Typography

shadcn/ui uses a clean sans-serif stack with clear hierarchy:

- **Headings**: `font_weight(FontWeight::SEMIBOLD)`, tracking tight
- **Body**: Regular weight, comfortable line height
- **Muted/Caption**: Smaller size, muted foreground color
- **Code**: Monospace font (e.g., `font_family("JetBrains Mono")` or system monospace)

Sizes follow a scale: xs (12px), sm (14px), base (16px), lg (18px), xl (20px), 2xl (24px), 3xl (30px), 4xl (36px).

### Border Radius

Every container, button, input, and card uses consistent radius:

- **sm** (4px): Inline elements, small badges
- **md** (6px): Buttons, inputs, compact cards
- **lg** (8px): Cards, dialogs, modals, dropdowns
- **xl** (12px): Large containers, sheets
- **full** (9999px): Pills, avatars

### Shadows

shadcn/ui uses layered shadows for depth:

- **sm**: Subtle elevation (cards on background)
- **md**: Medium elevation (dropdowns, popovers)
- **lg**: High elevation (modals, dialogs)
- **xl**: Highest elevation

In GPUI, use `.shadow()` with appropriate spread and blur values.

### Focus Rings

shadcn/ui uses a prominent ring for keyboard focus:

- 2px offset ring in `ring` color
- Applied via `focus_visible:` state
- Critical for keyboard accessibility

## GPUI Architecture Patterns

### Component Model

Every reusable UI piece is a component — a struct implementing `Render` or a function returning `impl IntoElement`:

```rust
// Function component (simple, stateless)
fn button(label: &str) -> impl IntoElement {
    div()
        .px(SPACE_4).py(SPACE_2)
        .bg(theme().primary)
        .text_color(theme().primary_foreground)
        .rounded(RADIUS_MD)
        .cursor(CursorStyle::PointingHand)
        .child(label)
}

// Struct component (stateful, reusable)
struct Card {
    header: Option<SharedString>,
    content: SharedString,
    footer: Option<SharedString>,
}

impl RenderOnce for Card {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .rounded(RADIUS_LG)
            .border_1()
            .border_color(theme().border)
            .bg(theme().card)
            .shadow(SHADOW_SM)
            .p(SPACE_6)
            .children(self.header.map(|h| {
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(TEXT_LG)
                    .mb(SPACE_2)
                    .child(h)
            }))
            .child(self.content)
            .children(self.footer.map(|f| {
                div()
                    .mt(SPACE_4)
                    .pt(SPACE_4)
                    .border_t_1()
                    .border_color(theme().border)
                    .child(f)
            }))
    }
}
```

### State Management

GPUI uses reactive state via `Model<T>`:

```rust
// Application state
struct AppState {
    count: u32,
    is_open: bool,
    items: Vec<String>,
}

// In component, subscribe to state changes
fn counter(model: Model<AppState>) -> impl IntoElement {
    let count = model.read(cx).count;
    h_flex()
        .gap(SPACE_2)
        .items_center()
        .child(
            div()
                .px(SPACE_3).py(SPACE_2)
                .rounded(RADIUS_MD)
                .bg(theme().secondary)
                .cursor(CursorStyle::PointingHand)
                .hover(|s| s.bg(theme().secondary.opacity(0.8)))
                .on_click(cx.listener(move |model, _, _window, cx| {
                    model.update(cx, |state, _cx| {
                        state.count -= 1;
                    });
                }))
                .child("-")
        )
        .child(Label::new(format!("{}", count)))
        .child(/* + button */)
}
```

### Layout System

GPUI uses a flexbox-like layout:

```rust
// Horizontal layout (row)
h_flex()
    .gap(SPACE_4)
    .items_center()
    .justify_between()
    .child(left_section)
    .child(right_section)

// Vertical layout (column)
v_flex()
    .gap(SPACE_3)
    .child(header)
    .child(body)
    .child(footer)

// Grid-like layouts
div().flex().flex_wrap().gap(SPACE_4)
    .children(items.map(|item| {
        div().w(PX(300.0)).flex_grow().child(render_item(item))
    }))
```

### Interactive States

shadcn/ui components have clear interactive states. Translate to GPUI:

| shadcn/ui State | GPUI Pattern |
|---|---|
| `hover:bg-accent` | `.hover(\|s\| s.bg(theme().accent))` |
| `focus-visible:ring-2` | `.focus_visible(\|s\| s.ring_2().ring(theme().ring))` |
| `active:scale-95` | `.active(\|s\| s.scale(0.95))` |
| `disabled:opacity-50` | `.when(disabled, \|s\| s.opacity(0.5))` |
| `data-[state=open]:bg-accent` | Conditional styling based on state enum |
| `aria-selected:bg-accent` | `.when(selected, \|s\| s.bg(theme().accent))` |

### Accessibility

shadcn/ui components are built on Radix primitives — accessibility is foundational. In GPUI:

- Every interactive element gets a `name` attribute for screen readers
- Focus order follows visual order
- Keyboard navigation: Enter/Space to activate, Escape to dismiss, Arrow keys for selections
- Use `focusable()` on interactive elements
- `aria_label()` for icon-only buttons
- High contrast mode support via theme toggle

## Component Catalog

When implementing common UI patterns, follow these shadcn/ui component conventions:

### Button

Variants: `default`, `destructive`, `outline`, `secondary`, `ghost`, `link`
Sizes: `default` (h-10 px-4 py-2), `sm` (h-9 px-3), `lg` (h-11 px-8), `icon` (h-10 w-10)

Key details:
- `inline-flex items-center justify-center gap-2`
- `rounded-md` by default
- `font-medium text-sm`
- Focus ring on keyboard focus
- Disabled state with reduced opacity
- Subtle scale-down on active/press

### Input

Key details:
- `flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm`
- `ring-offset-background` on focus
- `placeholder:text-muted-foreground`
- Disabled: `cursor-not-allowed opacity-50`

### Card

Key details:
- `rounded-lg border bg-card text-card-foreground shadow-sm`
- Header: `flex flex-col space-y-1.5 p-6`
- Content: `p-6 pt-0`
- Footer: `flex items-center p-6 pt-0`

### Dialog / Modal

Key details:
- Overlay: `fixed inset-0 bg-black/80`
- Content: `fixed left-[50%] top-[50%] translate-x-[-50%] translate-y-[-50%]`
- `rounded-lg border bg-background shadow-lg`
- Close button in top-right
- Escape to dismiss
- Focus trap within modal

### Dropdown Menu

Key details:
- Trigger toggles open/close
- `z-50 min-w-[8rem] overflow-hidden rounded-md border bg-popover p-1 shadow-md`
- Items: `rounded-sm px-2 py-1.5 text-sm` with hover highlight
- Separator: `-mx-1 my-1 h-px bg-muted`
- Arrow key navigation

### Tabs

Key details:
- List: `h-10 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground`
- Trigger: `rounded-sm px-3 py-1.5 text-sm font-medium`
- Active trigger: `bg-background text-foreground shadow-sm`
- Content panel with padding

### Badge

Variants: `default`, `secondary`, `destructive`, `outline`
Key details:
- `inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold`
- Focus ring for focusable badges

### Separator

Key details:
- Horizontal: `h-[1px] w-full bg-border`
- Vertical: `h-full w-[1px] bg-border`
- Never use pure black/white — always use `border` color token

## Implementation Guidelines

1. **Design tokens first**: Before any component, define the `Theme` struct and spacing constants. Every visual value references a token. This is the single most important rule.

2. **Composable primitives**: Build small, focused components that compose well. A `Button` shouldn't know about forms. A `Card` shouldn't know about data fetching.

3. **One component per module**: Organize by `components/button.rs`, `components/card.rs`, `components/dialog.rs`, etc. Each file exports a struct or function.

4. **Variants as enums**: Use Rust enums for component variants, not magic strings:

   ```rust
   enum ButtonVariant { Default, Destructive, Outline, Secondary, Ghost, Link }
   enum ButtonSize { Default, Sm, Lg, Icon }
   ```

5. **Conditional styling via `.when()`**: Chain `.when()` calls for state-dependent styles. This keeps rendering logic linear and readable.

6. **Keyboard navigation**: Every interactive component must handle keyboard events. Tab to focus, Enter/Space to activate, Escape to dismiss, Arrow keys to navigate within compound components.

7. **No dead code**: Every component must compile and render. This is not a design mockup — it's production code.

8. **Dark mode by default**: Always implement both light and dark themes. Check the current theme at render time and select the appropriate palette.

NEVER produce generic GPUI aesthetics: unstyled `div()` chains with default colors, hardcoded pixel values, no hover/active/focus states, missing keyboard navigation, identical rectangular containers, no design token system.

**IMPORTANT**: GPUI is retained-mode with GPU rendering — it does not use CSS, HTML, or web layout. Work within GPUI's element tree and styling builder pattern. The shadcn/ui spirit is in the design *intent* and *values*, not in copying CSS rules literally. A perfectly composed GPUI component with intentional color, spacing, typography, and interaction states embodies shadcn/ui's principles better than a literal CSS port ever could.
