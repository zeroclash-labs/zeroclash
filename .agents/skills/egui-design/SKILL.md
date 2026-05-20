---
name: egui-design
description: Create distinctive, production-grade egui interfaces with high design quality. Use this skill when the user asks to build egui components, widgets, pages, or desktop GUI applications in Rust — especially when they mention UI, layout, styling, or visual design for egui. Generates creative, polished Rust code that avoids generic desktop GUI aesthetics.
---

This skill guides creation of distinctive egui interfaces that avoid generic desktop GUI aesthetics. Implement real working Rust code with attention to visual details and creative choices.

The user provides egui interface requirements: a component, page, widget, or application. They may describe the purpose, audience, or technical constraints.

## Design Thinking

Before writing code, understand the context and commit to a BOLD aesthetic direction:

- **Purpose**: What does this interface do? Who uses it and when? Is it a developer tool, a consumer app, a dashboard, a settings panel?
- **Tone**: Pick a clear direction: refined minimalism, data-dense dashboard, playful/colorful, dark and atmospheric, retro/terminal, soft/pastel, industrial/utilitarian, editorial/magazine. There are many flavors — commit to one and execute it thoroughly.
- **Constraints**: egui version, existing design system (if any), platform (Windows/macOS/Linux), performance needs (every frame redraws in immediate mode).
- **Differentiation**: What makes this interface unforgettable? What's the one visual detail someone will remember?

**CRITICAL**: egui is an immediate-mode GUI — every frame redraws everything. Design decisions live in code, not stylesheets. Aesthetic intent must be expressed through every widget call. Bold maximalism and refined minimalism both work — the key is intentionality, not intensity.

Then implement working Rust code that:
- Compiles and renders with the target egui version
- Is visually striking within egui's capabilities
- Cohesive with a clear aesthetic point-of-view
- Meticulously refined in every detail (spacing, color, typography, interaction)

## egui Aesthetics Guidelines

Focus on:

- **Design Tokens**: Centralize colors, spacing, corner radii, and font sizes into named constants or a config struct. Every visual value in widget code should reference a token, never a hardcoded literal. This enables dark/light mode switching and ensures consistent spacing rhythm. For colors, use semantic names (surface, border, accent, text_primary, text_muted, danger, success) rather than literal names (blue, gray).

- **Typography**: Use `egui::RichText` for all labels. Vary size, weight (`strong()`), italics, and color to establish clear visual hierarchy. Avoid plain `ui.label("text")` — always style text intentionally. Use `TextStyle::Monospace` for data, code, and logs; proportional for UI labels and headings.

- **Color & Theme**: Commit to a cohesive palette. Support dark mode natively by checking `ctx.style().visuals.dark_mode` or using a palette-switching function. Use dominant surface/fill colors with sharp accent highlights — timid, evenly-distributed palettes feel generic. Backgrounds should have atmospheric depth (subtle color, not plain `Color32::BLACK` or `Color32::WHITE`). Borders and separators should be low-contrast — they guide the eye without shouting.

- **Spatial Composition**: Use consistent spacing tokens (e.g., 4/8/12/16/24 px steps). Create visual rhythm through proportional gaps and margins. egui's layout system (horizontal, vertical, Grid) rewards clean alignment — asymmetry is harder to achieve and should be intentional, not accidental. Use `ui.add_space()` for breathing room between sections. Use `egui::Frame` margins for container padding. Generous whitespace elevates perceived quality.

- **Containers & Cards**: Use `egui::Frame` with `corner_radius()`, `inner_margin()`, `fill()`, and `stroke()` for content containers. The standard card pattern is `frame.show(ui, |ui| { ... })`. Nest frames for grouped sections. Vary corner radii by hierarchy (small for inline elements, medium for cards, large for modals). A subtle border stroke adds definition without heaviness.

- **Custom Painting**: Use `ui.painter()` for backgrounds, dividers, decorative accents, and data visualization. Key primitives: `rect_filled(rect, rounding, color)`, `line_segment([a, b], stroke)`, `circle_filled(center, radius, color)`. Use `ui.allocate_exact_size()` or `ui.allocate_space()` to reserve canvas area, then paint into the returned rect. A well-placed filled rect or gradient-like layered transparency elevates a widget from functional to polished.

- **Interaction States**: Style hover, click, selection, and disabled states. Change background fill or border on hover. Highlight the active/selected item with accent color. Dim disabled controls. Use `ui.interact(rect, id, Sense::click())` for non-standard hit targets. These micro-interactions make the interface feel responsive and alive — an interface without them feels dead.

- **Data Display**: For tables, use `egui::Grid` with `striped(true)` for readability. For scrollable content, wrap in `ScrollArea::vertical()`. For real-time data, use custom painting (bars, lines, sparklines) rather than tables of numbers. Data should tell a story at a glance.

NEVER produce generic egui aesthetics: default visuals without customization, monotonous gray panels, unstyled `ui.label()` calls, hardcoded pixel values scattered through widget code, no hover or selection feedback, identical rectangular cards with default spacing.

Interpret creatively within egui's constraints. Vary between light and dark themes, different color palettes, different spatial rhythms. Never converge on the same common choices across generations.

**IMPORTANT**: egui has no CSS, no HTML layout engine, no web fonts, and very limited animation support. Work within these constraints rather than fighting them. Elegance comes from mastering egui's primitives — Frame, RichText, painter, and layout. A perfectly styled label with intentional color, weight, and spacing is more impactful than a complex CSS animation would be on the web. The richness comes from thoughtful composition of simple elements.

## Widget Implementation Patterns

Follow these conventions for composable, maintainable egui code:

- **Function signature**: The first parameter is always `ui: &mut egui::Ui`. Additional parameters carry data (shared references `&T`) and callbacks (closures or `&mut dyn FnMut`). Return type is `()` — widgets mutate state through parameters, not return values.

- **Design token access**: At the top of every widget function, resolve the active palette: `let colors = palette(ui.ctx());` (if using a design system) or read `ui.ctx().style().visuals`. Use token references throughout the function body.

- **Composition**: Widgets compose by function call, not struct impl. A page function calls card functions, which call row functions. Keep functions small and focused — if a function exceeds ~60 lines, extract a sub-widget.

- **State management**: Separate rendering from side effects. Render functions produce UI and collect user intent (via clicked flags, callback closures, or command enums). Side effects (network calls, state mutation) happen after rendering, not during.
