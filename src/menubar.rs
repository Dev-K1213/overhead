//! A menu bar control keeps the hidden app's window reachable while it tracks flights.

#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(target_os = "macos")]
static STATUS_ITEM: AtomicUsize = AtomicUsize::new(0);
#[cfg(target_os = "macos")]
static MENU: AtomicUsize = AtomicUsize::new(0);
#[cfg(target_os = "macos")]
static TARGET: AtomicUsize = AtomicUsize::new(0);

#[cfg(target_os = "macos")]
pub fn init() {
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel};
    use objc::{class, msg_send, sel, sel_impl};
    use objc2_foundation::NSString;

    extern "C" fn show(_: &mut Object, _: Sel, _: *mut Object) {
        crate::dock::show();
        unsafe {
            let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let _: () = msg_send![app, activateIgnoringOtherApps: true];
        }
    }

    extern "C" fn quit(_: &mut Object, _: Sel, _: *mut Object) {
        unsafe {
            let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let _: () = msg_send![app, terminate: std::ptr::null_mut::<Object>()];
        }
    }

    unsafe {
        // GPUI sets the app to regular activation policy during startup. Switch back to
        // accessory mode so the window and status item work without a Dock icon.
        let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let _: objc::runtime::BOOL = msg_send![app, setActivationPolicy: 1isize];

        let target_class = if let Some(class) = Class::get("OverheadMenuBarTarget") {
            class
        } else {
            let mut decl = ClassDecl::new("OverheadMenuBarTarget", class!(NSObject))
                .expect("could not register menu bar target");
            decl.add_method(
                sel!(showOverhead:),
                show as extern "C" fn(&mut Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(quitOverhead:),
                quit as extern "C" fn(&mut Object, Sel, *mut Object),
            );
            decl.register()
        };

        let target: *mut Object = msg_send![target_class, new];
        TARGET.store(target as usize, Ordering::SeqCst);

        let status_bar: *mut Object = msg_send![class!(NSStatusBar), systemStatusBar];
        let status_item: *mut Object = msg_send![status_bar, statusItemWithLength: -1.0f64];
        // Keep the status item alive for the process; NSStatusBar does not own our reference.
        let _: *mut Object = msg_send![status_item, retain];
        let _: () = msg_send![status_item, setVisible: true];
        let button: *mut Object = msg_send![status_item, button];
        let title = NSString::from_str("✈");
        let title_ptr = &*title as *const _ as *mut Object;
        let _: () = msg_send![button, setTitle: title_ptr];
        let description = NSString::from_str("Overhead");
        let description_ptr = &*description as *const _ as *mut Object;
        let _: () = msg_send![button, setToolTip: description_ptr];

        let menu_title = nsstring("Overhead");
        let menu_title_ptr = &*menu_title as *const _ as *mut Object;
        let menu: *mut Object = msg_send![class!(NSMenu), alloc];
        let menu: *mut Object = msg_send![menu, initWithTitle: menu_title_ptr];
        let _: () = msg_send![status_item, setMenu: menu];
        STATUS_ITEM.store(status_item as usize, Ordering::SeqCst);
        MENU.store(menu as usize, Ordering::SeqCst);
    }
}

#[cfg(target_os = "macos")]
pub fn update(aircraft: &[String]) {
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl};

    let status_item = STATUS_ITEM.load(Ordering::SeqCst) as *mut Object;
    let menu = MENU.load(Ordering::SeqCst) as *mut Object;
    let target = TARGET.load(Ordering::SeqCst) as *mut Object;
    if status_item.is_null() || menu.is_null() || target.is_null() {
        return;
    }

    unsafe {
        let button: *mut Object = msg_send![status_item, button];
        let title = nsstring(&format!("✈ {}", aircraft.len()));
        let title_ptr = &*title as *const _ as *mut Object;
        let _: () = msg_send![button, setTitle: title_ptr];
        let tooltip = nsstring(&format!("{} aircraft in your zone", aircraft.len()));
        let tooltip_ptr = &*tooltip as *const _ as *mut Object;
        let _: () = msg_send![button, setToolTip: tooltip_ptr];
        let _: () = msg_send![menu, removeAllItems];

        let heading = nsstring(&format!("Nearby aircraft ({})", aircraft.len()));
        add_menu_item(menu, &heading, sel!(showOverhead:), target);
        let heading_item: *mut Object = msg_send![menu, itemAtIndex: 0usize];
        let _: () = msg_send![heading_item, setEnabled: false];

        if aircraft.is_empty() {
            let empty = nsstring("No aircraft nearby");
            add_menu_item(menu, &empty, sel!(showOverhead:), target);
            let empty_item: *mut Object = msg_send![menu, itemAtIndex: 1usize];
            let _: () = msg_send![empty_item, setEnabled: false];
        } else {
            for row in aircraft.iter().take(8) {
                let title = nsstring(row);
                add_menu_item(menu, &title, sel!(showOverhead:), target);
                let index: usize = msg_send![menu, numberOfItems];
                let item: *mut Object = msg_send![menu, itemAtIndex: (index - 1)];
                let _: () = msg_send![item, setEnabled: false];
            }
        }

        let separator: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![menu, addItem: separator];
        let show_title = nsstring("Show Overhead");
        add_menu_item(menu, &show_title, sel!(showOverhead:), target);
        let separator: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![menu, addItem: separator];
        let quit_title = nsstring("Quit Overhead");
        add_menu_item(menu, &quit_title, sel!(quitOverhead:), target);
    }
}

#[cfg(target_os = "macos")]
unsafe fn add_menu_item(
    menu: *mut objc::runtime::Object,
    title: &objc2_foundation::NSString,
    action: objc::runtime::Sel,
    target: *mut objc::runtime::Object,
) {
    use objc::{class, msg_send, sel, sel_impl};
    unsafe {
        let key = nsstring("");
        let title_ptr = title as *const _ as *mut objc::runtime::Object;
        let key_ptr = &*key as *const _ as *mut objc::runtime::Object;
        let item: *mut objc::runtime::Object = msg_send![class!(NSMenuItem), alloc];
        let item: *mut objc::runtime::Object =
            msg_send![item, initWithTitle: title_ptr action: action keyEquivalent: key_ptr];
        if !target.is_null() {
            let _: () = msg_send![item, setTarget: target];
        }
        let _: () = msg_send![menu, addItem: item];
    }
}

#[cfg(target_os = "macos")]
fn nsstring(value: &str) -> objc2::rc::Retained<objc2_foundation::NSString> {
    objc2_foundation::NSString::from_str(value)
}

#[cfg(not(target_os = "macos"))]
pub fn init() {}

#[cfg(not(target_os = "macos"))]
pub fn update(_aircraft: &[String]) {}
