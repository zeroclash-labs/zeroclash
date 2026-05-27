#!/usr/bin/env python3
"""Patch GPUI v1.3.7 for macOS 26 (Darwin 25) compatibility.

The objc 0.2.7 crate cannot dynamically subclass NSApplication on macOS 26,
so this patch uses the base NSApplication class and stores the platform
pointer via objc_setAssociatedObject (ASSIGN policy) instead of custom ivars.

The GPUIApplicationDelegate (NSResponder subclass) is kept as-is — it works.

Usage: python3 scripts/patch-gpui.py
"""

import os
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

# 1. Add associated object FFI declarations (ASSIGN policy — no retain/release)
assoc = '''unsafe extern "C" {
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

const OBJC_ASSOCIATION_ASSIGN: std::ffi::c_ulong = 0;
static PLATFORM_ASSOC_KEY: u8 = 0;

'''
content = content.replace(
    'const MAC_PLATFORM_IVAR: &str = "platform";',
    assoc + 'const MAC_PLATFORM_IVAR: &str = "platform";',
)

# 2. Use base NSApplication (NSResponder delegate subclass is fine on macOS 26)
content = content.replace(
    '''APP_CLASS = {
            let mut decl = ClassDecl::new("GPUIApplication", class!(NSApplication)).unwrap();
            decl.add_ivar::<*mut c_void>(MAC_PLATFORM_IVAR);
            decl.register()
        }''',
    'APP_CLASS = class!(NSApplication);',
)

# 3. Replace set_ivar with objc_setAssociatedObject (ASSIGN) in run()
content = content.replace(
    '''(*app).set_ivar(MAC_PLATFORM_IVAR, self_ptr);
            (*app_delegate).set_ivar(MAC_PLATFORM_IVAR, self_ptr);

            let pool = NSAutoreleasePool::new(nil);
            app.run();
            pool.drain();

            (*app).set_ivar(MAC_PLATFORM_IVAR, null_mut::<c_void>());
            (*NSWindow::delegate(app)).set_ivar(MAC_PLATFORM_IVAR, null_mut::<c_void>());''',
    '''let key = (&PLATFORM_ASSOC_KEY) as *const u8 as *const _;
            objc_setAssociatedObject(app as *mut _, key, self_ptr as *mut _, OBJC_ASSOCIATION_ASSIGN);
            objc_setAssociatedObject(app_delegate as *mut _, key, self_ptr as *mut _, OBJC_ASSOCIATION_ASSIGN);

            let pool = NSAutoreleasePool::new(nil);
            app.run();
            pool.drain();

            objc_setAssociatedObject(app as *mut _, key, null_mut::<c_void>() as *mut _, OBJC_ASSOCIATION_ASSIGN);
            objc_setAssociatedObject(NSWindow::delegate(app) as *mut _, key, null_mut::<c_void>() as *mut _, OBJC_ASSOCIATION_ASSIGN);''',
)

# 4. Replace get_ivar with objc_getAssociatedObject in get_mac_platform
content = content.replace(
    'let platform_ptr: *mut c_void = *object.get_ivar(MAC_PLATFORM_IVAR);',
    '''let platform_ptr: *mut c_void = objc_getAssociatedObject(
            object as *mut Object as *mut _,
            (&PLATFORM_ASSOC_KEY) as *const u8 as *const _,
        );''',
)

# 5. Set activation policy directly (belt-and-suspenders)
content = content.replace(
    '''let app: id = msg_send![APP_CLASS, sharedApplication];
            let app_delegate: id = msg_send![APP_DELEGATE_CLASS, new];''',
    '''let app: id = msg_send![APP_CLASS, sharedApplication];
            app.setActivationPolicy_(NSApplicationActivationPolicyRegular);
            let app_delegate: id = msg_send![APP_DELEGATE_CLASS, new];''',
)

platform_rs.write_text(content)
print("GPUI patch applied successfully.")
