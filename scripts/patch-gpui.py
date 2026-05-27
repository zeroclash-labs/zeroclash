#!/usr/bin/env python3
"""Patch GPUI v1.3.7 for macOS 26 (Darwin 25) compatibility.

The objc 0.2.7 crate cannot dynamically subclass NSApplication or NSResponder
on macOS 26. This applies 7 patches:
1. objc_setAssociatedObject / objc_getAssociatedObject FFI declarations
2. APP_CLASS = class!(NSApplication) — no subclass
3. APP_DELEGATE_CLASS = class!(NSResponder) — no subclass
4. NSNotificationCenter block-based finish_launching callback
5. app.setActivationPolicy_ directly in run()
6. objc_setAssociatedObject instead of set_ivar
7. objc_getAssociatedObject instead of get_ivar

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

# 1. Add associated object FFI declarations
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

const OBJC_ASSOCIATION_RETAIN: std::ffi::c_ulong = 0x301;
static PLATFORM_ASSOC_KEY: u8 = 0;

'''
content = content.replace(
    'const MAC_PLATFORM_IVAR: &str = "platform";', assoc + 'const MAC_PLATFORM_IVAR: &str = "platform";'
)

# 2. Replace APP_CLASS subclass with base NSApplication
content = content.replace(
    '''APP_CLASS = {
            let mut decl = ClassDecl::new("GPUIApplication", class!(NSApplication)).unwrap();
            decl.add_ivar::<*mut c_void>(MAC_PLATFORM_IVAR);
            decl.register()
        }''',
    'APP_CLASS = class!(NSApplication);',
)

# 3. Replace delegate class registration with base NSResponder
start_marker = '        APP_DELEGATE_CLASS = unsafe {'
idx = content.find(start_marker)
if idx >= 0:
    depth = 0
    in_block = False
    for i, ch in enumerate(content[idx:], idx):
        if ch == '{':
            depth += 1
            in_block = True
        elif ch == '}':
            depth -= 1
            if in_block and depth == 0:
                j = i + 1
                while j < len(content) and content[j] in ' \t\n\r':
                    j += 1
                end = j + 1 if (j < len(content) and content[j] == '}') else i + 1
                content = content.replace(
                    content[idx:end],
                    '        APP_DELEGATE_CLASS = class!(NSResponder);\n    }',
                )
                break

# 4. NSNotificationCenter block-based callback
content = content.replace(
    '''state.finish_launching = Some(on_finish_launching);
            drop(state);''',
    '''// macOS 26: use NSNotificationCenter block instead of delegate
            let cb = std::sync::Mutex::new(Some(on_finish_launching));
            let block = block::ConcreteBlock::new(move || {
                if let Some(cb) = cb.lock().unwrap().take() {
                    cb();
                }
            });
            let block = block.copy();
            unsafe {
                let nc: id = msg_send![class!(NSNotificationCenter), defaultCenter];
                let name: id = msg_send![class!(NSString), stringWithUTF8String: "NSApplicationDidFinishLaunchingNotification\\0".as_ptr() as *const i8];
                let _: id = msg_send![nc, addObserverForName:name object:nil queue:nil usingBlock:&*block];
            }
            drop(state);''',
)

# 5. Set activation policy
content = content.replace(
    '''let app: id = msg_send![APP_CLASS, sharedApplication];
            let app_delegate: id = msg_send![APP_DELEGATE_CLASS, new];''',
    '''let app: id = msg_send![APP_CLASS, sharedApplication];
            // macOS 26: must set activation policy here since delegate won't
            app.setActivationPolicy_(NSApplicationActivationPolicyRegular);
            let app_delegate: id = msg_send![APP_DELEGATE_CLASS, new];''',
)

# 6. Replace set_ivar with objc_setAssociatedObject
content = content.replace(
    '''(*app).set_ivar(MAC_PLATFORM_IVAR, self_ptr);
            (*app_delegate).set_ivar(MAC_PLATFORM_IVAR, self_ptr);

            let pool = NSAutoreleasePool::new(nil);
            app.run();
            pool.drain();

            (*app).set_ivar(MAC_PLATFORM_IVAR, null_mut::<c_void>());
            (*NSWindow::delegate(app)).set_ivar(MAC_PLATFORM_IVAR, null_mut::<c_void>());''',
    '''let key = (&PLATFORM_ASSOC_KEY) as *const u8 as *const _;
            objc_setAssociatedObject(app as *mut _, key, self_ptr as *mut _, OBJC_ASSOCIATION_RETAIN);
            objc_setAssociatedObject(app_delegate as *mut _, key, self_ptr as *mut _, OBJC_ASSOCIATION_RETAIN);

            let pool = NSAutoreleasePool::new(nil);
            app.run();
            pool.drain();

            objc_setAssociatedObject(app as *mut _, key, null_mut::<c_void>() as *mut _, OBJC_ASSOCIATION_RETAIN);
            objc_setAssociatedObject(NSWindow::delegate(app) as *mut _, key, null_mut::<c_void>() as *mut _, OBJC_ASSOCIATION_RETAIN);''',
)

# 7. Replace get_ivar with objc_getAssociatedObject
content = content.replace(
    'let platform_ptr: *mut c_void = *object.get_ivar(MAC_PLATFORM_IVAR);',
    '''let platform_ptr: *mut c_void = objc_getAssociatedObject(
            object as *mut Object as *mut _,
            (&PLATFORM_ASSOC_KEY) as *const u8 as *const _,
        );''',
)

platform_rs.write_text(content)
print("GPUI patch applied successfully (7 fixes).")
