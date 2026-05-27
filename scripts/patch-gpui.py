#!/usr/bin/env python3
"""Patch GPUI v1.3.7 for macOS 26 (Darwin 25) compatibility.

The objc 0.2.7 crate cannot dynamically subclass NSApplication or NSResponder
on macOS 26, so this patch:
1. Uses base NSApplication and NSResponder classes (no custom subclasses)
2. Uses objc_setAssociatedObject instead of custom ivars
3. Calls on_finish_launching() directly since delegate callbacks won't fire

Usage: python3 scripts/patch-gpui.py
"""

import os
import re
import sys
from pathlib import Path

carho_home = os.environ.get("CARGO_HOME", os.path.expanduser("~/.cargo"))
checkouts = Path(carho_home) / "git" / "checkouts"

zed_dirs = list(checkouts.glob("zed-*/f1567cf"))
if not zed_dirs:
    print("Error: GPUI checkout not found. Run 'cargo fetch' first.")
    sys.exit(1)

platform_rs = zed_dirs[0] / "crates" / "gpui_macos" / "src" / "platform.rs"
if not platform_rs.exists():
    print(f"Error: {platform_rs} not found")
    sys.exit(1)

content = platform_rs.read_text()

if "objc_getAssociatedObject" in content:
    print("GPUI already patched.")
    sys.exit(0)

print(f"Patching {platform_rs} for macOS 26 compatibility...")

# 1. Add associated object FFI declarations before MAC_PLATFORM_IVAR
assoc_decls = """unsafe extern "C" {
    #[link(name = "objc", kind = "dylib")]
    fn objc_setAssociatedObject(
        object: *mut std::ffi::c_void,
        key: *const std::ffi::c_void,
        value: *mut std::ffi::c_void,
        policy: std::ffi::c_ulong,
    );
    #[link(name = "objc", kind = "dylib")]
    fn objc_getAssociatedObject(
        object: *mut std::ffi::c_void,
        key: *const std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
}

const OBJC_ASSOCIATION_RETAIN: std::ffi::c_ulong = 0x301;
static PLATFORM_ASSOC_KEY: u8 = 0;

"""
old = 'const MAC_PLATFORM_IVAR: &str = "platform";'
content = content.replace(old, assoc_decls + old)

# 2. Replace build_classes() — use base classes, no subclassing
old = '''APP_CLASS = {
            let mut decl = ClassDecl::new("GPUIApplication", class!(NSApplication)).unwrap();
            decl.add_ivar::<*mut c_void>(MAC_PLATFORM_IVAR);
            decl.register()
        }'''
content = content.replace(old, "APP_CLASS = class!(NSApplication)")

# 3. Replace entire delegate class registration with base NSResponder
#    (the delegate block starts with "APP_DELEGATE_CLASS = {" and ends with "decl.register()" plus "}" and another "}")
old = re.search(
    r'APP_DELEGATE_CLASS = \{[^}]*ClassDecl::new\("GPUIApplicationDelegate"[^}]*decl\.register\(\)\s*\}\s*\}',
    content, re.DOTALL
)
if old:
    content = content.replace(old.group(0), "APP_DELEGATE_CLASS = class!(NSResponder);\n    }")
else:
    print("Warning: could not find delegate class block — trying alternative approach")
    # Fallback: find the whole delegate class block manually
    start = content.find('APP_DELEGATE_CLASS = {')
    if start >= 0:
        depth = 0
        end = start
        for i, ch in enumerate(content[start:], start):
            if ch == '{':
                depth += 1
            elif ch == '}':
                depth -= 1
                if depth == 0:
                    end = i + 1
                    # Check if this belongs to build_classes (need another })
                    j = end
                    while j < len(content) and content[j] in ' \t\n':
                        j += 1
                    if content[j] == '}':
                        end = j + 1
                    break
        old_block = content[start:end]
        content = content.replace(old_block, "APP_DELEGATE_CLASS = class!(NSResponder);\n    }")

# 4. Call on_finish_launching() directly (delegate won't fire it)
old = '''state.finish_launching = Some(on_finish_launching);
            drop(state);'''
new = '''// macOS 26: delegate callbacks won't fire, call directly
            on_finish_launching();
            drop(state);'''
content = content.replace(old, new)

# 5. Replace set_ivar with objc_setAssociatedObject in run()
old = '''(*app).set_ivar(MAC_PLATFORM_IVAR, self_ptr);
            (*app_delegate).set_ivar(MAC_PLATFORM_IVAR, self_ptr);

            let pool = NSAutoreleasePool::new(nil);
            app.run();
            pool.drain();

            (*app).set_ivar(MAC_PLATFORM_IVAR, null_mut::<c_void>());
            (*NSWindow::delegate(app)).set_ivar(MAC_PLATFORM_IVAR, null_mut::<c_void>());'''

new = '''let key = (&PLATFORM_ASSOC_KEY) as *const u8 as *const _;
            objc_setAssociatedObject(app as *mut _, key, self_ptr as *mut _, OBJC_ASSOCIATION_RETAIN);
            objc_setAssociatedObject(app_delegate as *mut _, key, self_ptr as *mut _, OBJC_ASSOCIATION_RETAIN);

            let pool = NSAutoreleasePool::new(nil);
            app.run();
            pool.drain();

            objc_setAssociatedObject(app as *mut _, key, null_mut::<c_void>() as *mut _, OBJC_ASSOCIATION_RETAIN);
            objc_setAssociatedObject(NSWindow::delegate(app) as *mut _, key, null_mut::<c_void>() as *mut _, OBJC_ASSOCIATION_RETAIN);'''

content = content.replace(old, new)

# 6. Replace get_ivar with objc_getAssociatedObject in get_mac_platform
old = "let platform_ptr: *mut c_void = *object.get_ivar(MAC_PLATFORM_IVAR);"
new = '''let platform_ptr: *mut c_void = objc_getAssociatedObject(
            object as *mut Object as *mut _,
            (&PLATFORM_ASSOC_KEY) as *const u8 as *const _,
        );'''
content = content.replace(old, new)

platform_rs.write_text(content)
print("GPUI patch applied successfully.")
