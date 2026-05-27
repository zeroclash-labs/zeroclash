---
name: new-gpui-view
description: Scaffold a new GPUI view/page in zeroclash-ui following the existing pattern. Use when adding a new page to the GUI.
---

Create a new GPUI view file and wire it into the application. Follow the pattern from the existing 6 views (dashboard, proxies, profiles, connections, logs, settings).

## Steps

1. Create the view file at `crates/zeroclash-ui/src/views/<name>.rs`:
```rust
use gpui::{Context, MouseButton, SharedString, Window, div, prelude::*, px};

use crate::components::card::page_heading;
use crate::design::{Colors, SPACE_XL};
use crate::state::AppState;
use crate::theme::Theme;

pub fn <name>_page(
    state: &AppState,
    _window: &mut Window,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let c = theme.colors;

    div()
        .size_full()
        .p(px(SPACE_XL))
        .child(page_heading(c, "<Title>"))
}
```

2. Register the module in `crates/zeroclash-ui/src/views/mod.rs`:
```rust
pub mod <name>;
```

3. Add a variant to `Page` enum in `crates/zeroclash-ui/src/state.rs`:
```rust
<Name>,
```

4. Add a match arm in `render_content()` in the same file:
```rust
Page::<Name> => views::<name>::<name>_page(state, window, cx).into_any_element(),
```

5. Add a sidebar nav entry in `render_sidebar()`:
```rust
("<Title>", Page::<Name>),
```

6. Add the import in `state.rs`:
```rust
use crate::views::<name>;
```

7. Run verify to ensure everything compiles and passes clippy.
