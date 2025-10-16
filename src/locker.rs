use anyhow::Result;

#[cfg(target_os = "macos")]
pub fn show_lock_screen() -> Result<()> {
    use cocoa::appkit::{
        NSApp, NSApplication, NSApplicationActivationPolicy, NSBackingStoreBuffered, NSColor,
        NSScreen, NSTextField, NSWindow, NSWindowStyleMask,
    };
    use cocoa::base::{id, nil, NO, YES};
    use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel};
    use objc::{class, msg_send, sel, sel_impl};
    use std::sync::{Arc, Mutex};

    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        let app = NSApp();
        if app == nil {
            let app = NSApplication::sharedApplication(nil);
            app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular);
        }

        let app = NSApp();
        let screen = NSScreen::mainScreen(nil);
        let screen_frame: NSRect = msg_send![screen, frame];

        let window_class = create_window_class();
        let style_mask = NSWindowStyleMask::NSBorderlessWindowMask;

        let window: id = msg_send![window_class, alloc];
        let window: id = msg_send![
            window,
            initWithContentRect: screen_frame
            styleMask: style_mask
            backing: NSBackingStoreBuffered
            defer: NO
        ];

        let _: () = msg_send![window, setLevel: 1000i32];
        let black_color: id = msg_send![class!(NSColor), blackColor];
        let _: () = msg_send![window, setBackgroundColor: black_color];
        let _: () = msg_send![window, setCollectionBehavior: 1 << 10];
        let _: () = msg_send![window, setAcceptsMouseMovedEvents: YES];
        let _: () = msg_send![window, makeKeyAndOrderFront: nil];
        let _: () = msg_send![window, orderFrontRegardless];

        let content_view: id = msg_send![window, contentView];

        let label: id = NSTextField::alloc(nil);
        let label_frame = NSRect::new(
            NSPoint::new(screen_frame.size.width / 2.0 - 150.0, screen_frame.size.height / 2.0 + 50.0),
            NSSize::new(300.0, 40.0),
        );
        let _: id = msg_send![label, initWithFrame: label_frame];
        let text = NSString::alloc(nil).init_str("Screen Locked");
        let _: () = msg_send![label, setStringValue: text];
        let _: () = msg_send![label, setBezeled: NO];
        let _: () = msg_send![label, setDrawsBackground: NO];
        let _: () = msg_send![label, setEditable: NO];
        let _: () = msg_send![label, setSelectable: NO];
        let white_color: id = msg_send![class!(NSColor), whiteColor];
        let _: () = msg_send![label, setTextColor: white_color];
        let _: () = msg_send![label, setAlignment: 1i64];
        let font: id = msg_send![class!(NSFont), systemFontOfSize: 28.0];
        let _: () = msg_send![label, setFont: font];
        let _: () = msg_send![content_view, addSubview: label];

        let secure_field: id = msg_send![class!(NSSecureTextField), alloc];
        let field_frame = NSRect::new(
            NSPoint::new(screen_frame.size.width / 2.0 - 150.0, screen_frame.size.height / 2.0),
            NSSize::new(300.0, 30.0),
        );
        let _: id = msg_send![secure_field, initWithFrame: field_frame];
        let placeholder = NSString::alloc(nil).init_str("Enter PIN");
        let _: () = msg_send![secure_field, setPlaceholderString: placeholder];
        let _: () = msg_send![secure_field, setBezeled: YES];
        let _: () = msg_send![secure_field, setDrawsBackground: YES];
        let _: () = msg_send![content_view, addSubview: secure_field];
        let _: () = msg_send![window, makeFirstResponder: secure_field];

        let button: id = msg_send![class!(NSButton), alloc];
        let button_frame = NSRect::new(
            NSPoint::new(screen_frame.size.width / 2.0 - 60.0, screen_frame.size.height / 2.0 - 50.0),
            NSSize::new(120.0, 32.0),
        );
        let _: id = msg_send![button, initWithFrame: button_frame];
        let button_text = NSString::alloc(nil).init_str("Unlock");
        let _: () = msg_send![button, setTitle: button_text];
        let _: () = msg_send![button, setBezelStyle: 1i64];
        let _: () = msg_send![content_view, addSubview: button];

        let state = Arc::new(Mutex::new(UnlockState {
            window,
            secure_field,
            app,
        }));

        let target = create_target(state);
        let _: () = msg_send![button, setTarget: target];
        let _: () = msg_send![button, setAction: sel!(handleUnlock:)];
        let _: () = msg_send![button, setKeyEquivalent: NSString::alloc(nil).init_str("\r")];

        let _: () = msg_send![app, activateIgnoringOtherApps: YES];
        let _: () = msg_send![window, makeKeyAndOrderFront: nil];
        let _: () = msg_send![window, makeFirstResponder: secure_field];

        app.run();
    }

    Ok(())
}

#[cfg(target_os = "macos")]
struct UnlockState {
    window: cocoa::base::id,
    secure_field: cocoa::base::id,
    app: cocoa::base::id,
}

#[cfg(target_os = "macos")]
unsafe fn create_window_class() -> *const objc::runtime::Class {
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel, BOOL};
    use objc::{class, msg_send, sel, sel_impl};

    let superclass = class!(NSWindow);
    let mut decl = ClassDecl::new("LockWindow", superclass).unwrap();

    extern "C" fn can_become_key(_this: &Object, _cmd: Sel) -> BOOL {
        cocoa::base::YES
    }

    extern "C" fn can_become_main(_this: &Object, _cmd: Sel) -> BOOL {
        cocoa::base::YES
    }

    decl.add_method(
        sel!(canBecomeKeyWindow),
        can_become_key as extern "C" fn(&Object, Sel) -> BOOL,
    );

    decl.add_method(
        sel!(canBecomeMainWindow),
        can_become_main as extern "C" fn(&Object, Sel) -> BOOL,
    );

    decl.register()
}

#[cfg(target_os = "macos")]
unsafe fn create_target(state: std::sync::Arc<std::sync::Mutex<UnlockState>>) -> cocoa::base::id {
    use cocoa::foundation::NSString;
    use objc::declare::ClassDecl;
    use objc::runtime::{Object, Sel};
    use objc::{class, msg_send, sel, sel_impl};
    use std::sync::{Arc, Mutex};

    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("UnlockTarget", superclass).unwrap();

    extern "C" fn handle_unlock(this: &Object, _cmd: Sel, _sender: cocoa::base::id) {
        unsafe {
            use cocoa::foundation::NSString;
            use objc::{class, msg_send, sel, sel_impl};
            use std::sync::{Arc, Mutex};

            let state_ptr: *mut std::ffi::c_void = *this.get_ivar("state");
            let state = Arc::from_raw(state_ptr as *const Mutex<UnlockState>);

            let pin = {
                let locked_state = state.lock().unwrap();
                let string_value: cocoa::base::id = msg_send![locked_state.secure_field, stringValue];
                let utf8_ptr: *const i8 = msg_send![string_value, UTF8String];
                std::ffi::CStr::from_ptr(utf8_ptr).to_string_lossy().to_string()
            };

            if crate::auth::verify_pin_internal(&pin) {
                let locked_state = state.lock().unwrap();
                let _: () = msg_send![locked_state.app, stop: cocoa::base::nil];
                let _: () = msg_send![locked_state.window, close];
            } else {
                let locked_state = state.lock().unwrap();
                let empty = NSString::alloc(cocoa::base::nil).init_str("");
                let _: () = msg_send![locked_state.secure_field, setStringValue: empty];
            }

            std::mem::forget(state);
        }
    }

    decl.add_ivar::<*mut std::ffi::c_void>("state");
    decl.add_method(
        sel!(handleUnlock:),
        handle_unlock as extern "C" fn(&Object, Sel, cocoa::base::id),
    );

    let target_class = decl.register();
    let target: cocoa::base::id = msg_send![target_class, alloc];
    let target: cocoa::base::id = msg_send![target, init];

    let state_ptr = Arc::into_raw(state) as *mut std::ffi::c_void;
    (*target).set_ivar("state", state_ptr);

    target
}

#[cfg(target_os = "windows")]
pub fn show_lock_screen() -> Result<()> {
    anyhow::bail!("Windows implementation coming soon");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_compiles() {
        assert!(true);
    }
}
