#!/bin/bash
# Patch GPUI v1.3.7 for macOS 26 (Darwin 25) compatibility.
# The objc 0.2.7 crate cannot dynamically subclass NSApplication on macOS 26,
# so we switch to using objc_setAssociatedObject + base NSApplication.
#
# Run this once after `cargo fetch` on macOS 26+.

set -euo pipefail

CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
ZED_CHECKOUT=$(ls -d "$CARGO_HOME"/git/checkouts/zed-*/f1567cf 2>/dev/null | head -1)

if [ -z "$ZED_CHECKOUT" ]; then
    echo "Error: GPUI checkout not found. Run 'cargo fetch' first."
    exit 1
fi

PLATFORM_RS="$ZED_CHECKOUT/crates/gpui_macos/src/platform.rs"

if [ ! -f "$PLATFORM_RS" ]; then
    echo "Error: $PLATFORM_RS not found"
    exit 1
fi

if grep -q 'objc_getAssociatedObject' "$PLATFORM_RS"; then
    echo "GPUI already patched."
    exit 0
fi

echo "Patching $PLATFORM_RS for macOS 26 compatibility..."

# Use ed to apply the patch
python3 << 'PYEOF'
import re

with open(PLATFORM_RS) as f:
    content = f.read()

# 1. APP_CLASS: use NSApplication directly instead of subclassing
old_app_class = '''APP_CLASS = {
            let mut decl = ClassDecl::new("GPUIApplication", class!(NSApplication)).unwrap();
            decl.add_ivar::<*mut c_void>(MAC_PLATFORM_IVAR);
            decl.register()
        }'''
new_app_class = 'APP_CLASS = class!(NSApplication)'
content = content.replace(old_app_class, new_app_class)

# 2. Add associated object FFI declarations + key
assoc_decls = '''unsafe extern "C" {
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

'''

old_ivar = 'const MAC_PLATFORM_IVAR: &str = "platform";'
content = content.replace(old_ivar, assoc_decls + old_ivar)

# 3. Replace set_ivar with objc_setAssociatedObject in run()
old_set_ivar = '''(*app).set_ivar(MAC_PLATFORM_IVAR, self_ptr);
            (*app_delegate).set_ivar(MAC_PLATFORM_IVAR, self_ptr);

            let pool = NSAutoreleasePool::new(nil);
            app.run();
            pool.drain();

            (*app).set_ivar(MAC_PLATFORM_IVAR, null_mut::<c_void>());
            (*NSWindow::delegate(app)).set_ivar(MAC_PLATFORM_IVAR, null_mut::<c_void>());'''

new_set_ivar = '''let key = (&PLATFORM_ASSOC_KEY) as *const u8 as *const _;
            objc_setAssociatedObject(app as *mut _, key, self_ptr as *mut _, OBJC_ASSOCIATION_RETAIN);
            objc_setAssociatedObject(app_delegate as *mut _, key, self_ptr as *mut _, OBJC_ASSOCIATION_RETAIN);

            let pool = NSAutoreleasePool::new(nil);
            app.run();
            pool.drain();

            objc_setAssociatedObject(app as *mut _, key, null_mut::<c_void>() as *mut _, OBJC_ASSOCIATION_RETAIN);
            objc_setAssociatedObject(NSWindow::delegate(app) as *mut _, key, null_mut::<c_void>() as *mut _, OBJC_ASSOCIATION_RETAIN);'''

content = content.replace(old_set_ivar, new_set_ivar)

# 4. Replace get_ivar with objc_getAssociatedObject in get_mac_platform
old_get_ivar = 'let platform_ptr: *mut c_void = *object.get_ivar(MAC_PLATFORM_IVAR);'
new_get_ivar = '''let platform_ptr: *mut c_void = objc_getAssociatedObject(
            object as *mut Object as *mut _,
            (&PLATFORM_ASSOC_KEY) as *const u8 as *const _,
        );'''
content = content.replace(old_get_ivar, new_get_ivar)

with open(PLATFORM_RS, 'w') as f:
    f.write(content)

print("GPUI patch applied successfully.")
PYEOF
