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
const DETAIL_METADATA_SEPARATOR: &str = "\n--OVERHEAD-MENU-METADATA--\n";

#[cfg(target_os = "macos")]
#[link(name = "WebKit", kind = "framework")]
unsafe extern "C" {}

pub struct AircraftDetails {
    pub title: String,
    pub flight_url: String,
    pub jetphotos_url: String,
    pub registration: Option<String>,
    pub photo_url: Option<String>,
    pub photo_page: Option<String>,
    pub photo_photographer: Option<String>,
    pub lines: Vec<String>,
    pub hex: String,
    pub latitude: f64,
    pub longitude: f64,
    pub home_latitude: f64,
    pub home_longitude: f64,
}

impl AircraftDetails {
    fn html(&self) -> String {
        let lines = self
            .lines
            .iter()
            .map(|line| format!("<li>{}</li>", escape_html(line)))
            .collect::<String>();
        let hex: String = self
            .hex
            .chars()
            .filter(|character| character.is_ascii_hexdigit())
            .collect();
        let map = self.map_html();
        format!(
            r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<style>
*{{box-sizing:border-box}}body{{margin:0;padding:10px;background:#eaf0f6;color:#172536;font:12px -apple-system,BlinkMacSystemFont,sans-serif}}
.panel{{overflow:hidden;border-radius:14px;background:#fff;box-shadow:0 5px 22px rgba(20,43,68,.14)}}
header{{padding:14px 16px 12px;background:linear-gradient(125deg,#10263e,#1b4d70);color:#fff}}
.brand{{font-size:9px;font-weight:700;letter-spacing:1.4px;color:#afc7da}}.brand span{{float:right;color:#85e0c0;letter-spacing:.8px}}
h1{{font-size:20px;line-height:1.2;margin:7px 0 0;font-weight:700;letter-spacing:-.3px}}
.photo{{height:156px;background:linear-gradient(135deg,#d8e3eb,#f2f5f7);overflow:hidden;display:flex;align-items:center;justify-content:center;color:#637487;position:relative}}
.photo img{{width:100%;height:100%;object-fit:cover;display:none}}#photo-placeholder{{font-size:12px}}
.credit{{font-size:10px;color:#718096;padding:7px 13px 8px;min-height:28px}}a{{color:#23699b;text-decoration:none;font-weight:600}}
.actions{{display:flex;gap:7px;padding:0 12px 10px}}.action{{display:inline-block;padding:6px 9px;border-radius:7px;background:#edf5fa;color:#215c82;font-size:10px}}
.section{{padding:10px 12px 0}}.section-head{{display:flex;justify-content:space-between;align-items:center;margin:0 1px 7px;color:#718096;font-size:9px;font-weight:700;letter-spacing:1px}}
.live-dot{{color:#15966c;letter-spacing:.5px}}#map{{height:162px;border-radius:9px;overflow:hidden;background:#dbe5eb;position:relative}}#map img.tile{{position:absolute;width:256px;height:256px;max-width:none}}
.pin{{position:absolute;width:14px;height:14px;border:2px solid white;border-radius:50%;box-shadow:0 1px 4px #27384a;transform:translate(-50%,-50%)}}
.plane{{background:#ed5c55}}.home{{background:#3884cf}}.legend{{position:absolute;left:8px;top:8px;background:rgba(255,255,255,.94);padding:5px 7px;border-radius:6px;font-size:9px;z-index:1000}}
.map-credit{{font-size:9px;color:#8492a1;margin:5px 2px 8px}}
.details{{padding:3px 12px 12px}}ul{{list-style:none;padding:0;margin:0;display:grid;grid-template-columns:1fr 1fr;gap:6px}}li{{background:#f2f6f9;border:1px solid #e6edf2;border-radius:8px;padding:7px 8px;overflow-wrap:anywhere;color:#34495c;font-size:11px}}
</style></head><body>
<main class="panel"><header><div class="brand">OVERHEAD <span>● LIVE AIRCRAFT</span></div><h1>{title}</h1></header>
<div class="photo"><span id="photo-placeholder">Finding an aircraft photo…</span><img id="photo" alt="Aircraft photo"></div>
<div class="credit" id="photo-credit">Photo by JetPhotos contributors</div>
<div class="actions"><a class="action" href="{flight_url}" target="_blank">Open Flightradar24 ↗</a><a class="action" href="{jetphotos_url}" target="_blank">More JetPhotos ↗</a></div>
<section class="section"><div class="section-head"><span>POSITION</span><span class="live-dot">● LIVE</span></div>{map}<div class="map-credit">Map data © <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a></div></section>
<section class="details"><div class="section-head"><span>AIRCRAFT DETAILS</span></div><ul>{lines}</ul></section></main>
<script>
const photo=document.getElementById('photo'), placeholder=document.getElementById('photo-placeholder'), credit=document.getElementById('photo-credit');
function showPhoto(src, photographer, source, linkUrl) {{
 if(!src)return false; photo.src=src; photo.style.display='block'; placeholder.style.display='none';
 credit.textContent='Photo by '+(photographer||'Unknown')+' · ';
 const link=document.createElement('a'); link.href=linkUrl; link.textContent=source; link.target='_blank'; credit.appendChild(link); return true;
}}
const jetPhoto='{photo_url}', jetPage='{photo_page}', jetPhotographer='{photo_photographer}';
const jetFound=showPhoto(jetPhoto,jetPhotographer,'JetPhotos',jetPage||'{jetphotos_url}');
Promise.resolve(jetFound).then(found=>{{if(found)return;return fetch('pub/photos/hex/{hex}').then(r=>r.ok?r.json():null).then(d=>{{
 const p=d?.photos?.[0]; if(!p)return false; const src=p.thumbnail_large?.src||p.thumbnail?.src;
 return showPhoto(src,p.photographer,'Planespotters.net','https://www.planespotters.net');
}})}}).then(found=>{{if(!found)placeholder.textContent='No aircraft photo available'}}).catch(()=>{{placeholder.textContent='No aircraft photo available'}});
</script></body></html>"#,
            title = escape_html(&self.title),
            flight_url = escape_html(&self.flight_url),
            jetphotos_url = escape_html(&self.jetphotos_url),
            photo_url = escape_js(self.photo_url.as_deref().unwrap_or_default()),
            photo_page = escape_js(self.photo_page.as_deref().unwrap_or_default()),
            photo_photographer = escape_js(self.photo_photographer.as_deref().unwrap_or_default()),
            lines = lines,
            map = map,
            hex = hex,
        )
    }

    fn map_html(&self) -> String {
        const WIDTH: f64 = 392.0;
        const HEIGHT: f64 = 220.0;
        let distance = distance_miles(
            self.home_latitude,
            self.home_longitude,
            self.latitude,
            self.longitude,
        );
        let zoom = if distance < 3.0 {
            11
        } else if distance < 8.0 {
            10
        } else if distance < 20.0 {
            9
        } else if distance < 45.0 {
            8
        } else if distance < 100.0 {
            7
        } else {
            6
        };
        let world = |lat: f64, lon: f64| {
            let lat = lat.clamp(-85.051_128_78, 85.051_128_78).to_radians();
            let scale = 256.0 * 2_f64.powi(zoom);
            let x = (lon + 180.0) / 360.0 * scale;
            let y = (1.0 - (lat.tan() + 1.0 / lat.cos()).ln() / std::f64::consts::PI) / 2.0 * scale;
            (x, y)
        };
        let center = world(
            (self.latitude + self.home_latitude) / 2.0,
            (self.longitude + self.home_longitude) / 2.0,
        );
        let left = center.0 - WIDTH / 2.0;
        let top = center.1 - HEIGHT / 2.0;
        let tile_x0 = (left / 256.0).floor() as i64;
        let tile_x1 = ((left + WIDTH) / 256.0).floor() as i64;
        let tile_y0 = (top / 256.0).floor() as i64;
        let tile_y1 = ((top + HEIGHT) / 256.0).floor() as i64;
        let tiles = (tile_x0..=tile_x1)
            .flat_map(|x| (tile_y0..=tile_y1).map(move |y| (x, y)))
            .map(|(x, y)| {
                format!(
                    "<img class=\"tile\" alt=\"\" src=\"https://tile.openstreetmap.org/{zoom}/{x}/{y}.png\" style=\"left:{:.1}px;top:{:.1}px\">",
                    x as f64 * 256.0 - left,
                    y as f64 * 256.0 - top
                )
            })
            .collect::<String>();
        let plane = world(self.latitude, self.longitude);
        let home = world(self.home_latitude, self.home_longitude);
        format!(
            "<div id=\"map\">{tiles}<span class=\"legend\">🔴 Aircraft　🔵 Home</span><span class=\"pin plane\" title=\"Aircraft\" style=\"left:{:.1}px;top:{:.1}px\"></span><span class=\"pin home\" title=\"Home\" style=\"left:{:.1}px;top:{:.1}px\"></span></div>",
            plane.0 - left,
            plane.1 - top,
            home.0 - left,
            home.1 - top
        )
    }
}

fn distance_miles(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (lat1, lat2) = (lat1.to_radians(), lat2.to_radians());
    let dlat = lat2 - lat1;
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    2.0 * 3958.8 * a.sqrt().asin()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_js(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CocoaPoint {
    x: f64,
    y: f64,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CocoaSize {
    width: f64,
    height: f64,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CocoaRect {
    origin: CocoaPoint,
    size: CocoaSize,
}

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

    extern "C" fn menu_item_selected(_: &mut Object, _: Sel, sender: *mut Object) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            use objc::{class, msg_send};
            use objc2_foundation::{NSString, NSURL};
            use std::ffi::CStr;
            if sender.is_null() {
                return;
            }
            let represented: *mut Object = msg_send![sender, representedObject];
            if represented.is_null() {
                return;
            }
            let text_ptr: *const std::ffi::c_char = msg_send![represented, UTF8String];
            if text_ptr.is_null() {
                return;
            }
            let Ok(url_text) = CStr::from_ptr(text_ptr).to_str() else { return };
            let url_string = NSString::from_str(&url_text);
            let Some(url) = NSURL::URLWithString(&url_string) else { return };
            let workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
            let _: objc::runtime::BOOL = msg_send![workspace, openURL: (&*url as *const _ as *mut Object)];
        }));
    }

    extern "C" fn menu_needs_update(_: &mut Object, _: Sel, menu: *mut Object) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            populate_detail_menu(menu)
        }));
        if let Err(payload) = result {
            let message = payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                .unwrap_or("unknown aircraft menu error");
            eprintln!("aircraft detail submenu failed: {message}");
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
            decl.add_method(
                sel!(menuItemSelected:),
                menu_item_selected as extern "C" fn(&mut Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(menuNeedsUpdate:),
                menu_needs_update as extern "C" fn(&mut Object, Sel, *mut Object),
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
unsafe fn populate_detail_menu(menu: *mut objc::runtime::Object) {
    use std::ffi::CStr;

    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    use objc2_foundation::NSURL;

    if menu.is_null() {
        return;
    }
    unsafe {
        let count: usize = msg_send![menu, numberOfItems];
        if count > 0 {
            return;
        }
        let title: *mut Object = msg_send![menu, title];
        if title.is_null() {
            return;
        }
        let html_ptr: *const std::ffi::c_char = msg_send![title, UTF8String];
        if html_ptr.is_null() {
            return;
        }
        let payload = CStr::from_ptr(html_ptr).to_string_lossy().into_owned();
        let mut parts = payload.splitn(3, DETAIL_METADATA_SEPARATOR);
        let html = parts.next().unwrap_or_default().to_owned();
        let flight_url = parts.next().unwrap_or_default();
        let jetphotos_url = parts.next().unwrap_or_default();
        let (Some(configuration_class), Some(web_view_class)) = (
            Class::get("WKWebViewConfiguration"),
            Class::get("WKWebView"),
        ) else {
            add_plain_detail_rows(menu, "Aircraft details are unavailable on this Mac.");
            return;
        };
        let frame = CocoaRect {
            origin: CocoaPoint { x: 0.0, y: 0.0 },
            size: CocoaSize {
                width: 420.0,
                height: 620.0,
            },
        };
        let configuration: *mut Object = msg_send![configuration_class, new];
        if configuration.is_null() {
            add_plain_detail_rows(menu, "Aircraft details could not be loaded.");
            return;
        }
        let web_view: *mut Object = msg_send![web_view_class, alloc];
        if web_view.is_null() {
            add_plain_detail_rows(menu, "Aircraft details could not be loaded.");
            return;
        }
        let web_view: *mut Object =
            msg_send![web_view, initWithFrame: frame configuration: configuration];
        if web_view.is_null() {
            add_plain_detail_rows(menu, "Aircraft details could not be loaded.");
            return;
        }
        let user_agent = nsstring(concat!(
            "Overhead/",
            env!("CARGO_PKG_VERSION"),
            " (+https://github.com/y-u-s-u-f/overhead)"
        ));
        let user_agent_ptr = &*user_agent as *const _ as *mut Object;
        let _: () = msg_send![web_view, setCustomUserAgent: user_agent_ptr];
        let html = nsstring(&html);
        // The local JetPhotos API is the document origin so WebKit permits its HTTP request.
        let base_string = nsstring("http://127.0.0.1:8787/");
        let base = NSURL::URLWithString(&base_string)
            .expect("could not create aircraft detail base URL");
        let html_ptr = &*html as *const _ as *mut Object;
        let base_ptr = &*base as *const _ as *mut Object;
        let _: *mut Object = msg_send![web_view, loadHTMLString: html_ptr baseURL: base_ptr];

        let item_title = nsstring("Aircraft details");
        let item = new_menu_item(
            &item_title,
            sel!(menuItemSelected:),
            TARGET.load(Ordering::SeqCst) as *mut Object,
        );
        if item.is_null() {
            return;
        }
        let _: () = msg_send![item, setView: web_view];
        let _: () = msg_send![menu, addItem: item];

        let open_title = nsstring("Open on Flightradar24 ↗");
        let open_item = new_menu_item(
            &open_title,
            sel!(menuItemSelected:),
            TARGET.load(Ordering::SeqCst) as *mut Object,
        );
        let flight_url = nsstring(flight_url);
        let flight_url_ptr = &*flight_url as *const _ as *mut Object;
        add_external_menu_item(menu, open_item, flight_url_ptr);
        let jetphotos_title = nsstring("Search photos on JetPhotos ↗");
        let jetphotos_item = new_menu_item(
            &jetphotos_title,
            sel!(menuItemSelected:),
            TARGET.load(Ordering::SeqCst) as *mut Object,
        );
        let jetphotos_url = nsstring(jetphotos_url);
        let jetphotos_url_ptr = &*jetphotos_url as *const _ as *mut Object;
        add_external_menu_item(menu, jetphotos_item, jetphotos_url_ptr);
    }
}

#[cfg(target_os = "macos")]
unsafe fn add_plain_detail_rows(menu: *mut objc::runtime::Object, message: &str) {
    use objc::{msg_send, sel, sel_impl};

    let title = nsstring(message);
    let item = unsafe {
        new_menu_item(
            &title,
            sel!(menuItemSelected:),
            TARGET.load(Ordering::SeqCst) as *mut objc::runtime::Object,
        )
    };
    if !item.is_null() {
        unsafe {
            let _: () = msg_send![item, setEnabled: false];
            let _: () = msg_send![menu, addItem: item];
        }
    }
}

#[cfg(target_os = "macos")]
pub fn update(aircraft: &[AircraftDetails]) {
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl};

    let status_item = STATUS_ITEM.load(Ordering::SeqCst) as *mut Object;
    let menu = MENU.load(Ordering::SeqCst) as *mut Object;
    let target = TARGET.load(Ordering::SeqCst) as *mut Object;
    if status_item.is_null() || menu.is_null() || target.is_null() {
        return;
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let button: *mut Object = msg_send![status_item, button];
        let title = nsstring(&format!("✈ {}", aircraft.len()));
        let title_ptr = &*title as *const _ as *mut Object;
        let _: () = msg_send![button, setTitle: title_ptr];
        let tooltip = nsstring(&format!("{} aircraft in your zone", aircraft.len()));
        let tooltip_ptr = &*tooltip as *const _ as *mut Object;
        let _: () = msg_send![button, setToolTip: tooltip_ptr];
        let _: () = msg_send![menu, removeAllItems];

        let heading = nsstring(&format!("Nearby aircraft ({})", aircraft.len()));
        add_menu_item(menu, &heading, sel!(showOverhead:), target, None);
        let heading_item: *mut Object = msg_send![menu, itemAtIndex: 0usize];
        let _: () = msg_send![heading_item, setEnabled: false];

        if aircraft.is_empty() {
            let empty = nsstring("No aircraft nearby");
            add_menu_item(menu, &empty, sel!(showOverhead:), target, None);
            let empty_item: *mut Object = msg_send![menu, itemAtIndex: 1usize];
            let _: () = msg_send![empty_item, setEnabled: false];
        } else {
            for aircraft in aircraft.iter().take(crate::feed::MENU_AIRCRAFT_LIMIT) {
                let title = nsstring(&aircraft.title);
                let payload = format!(
                    "{}{}{}{}{}",
                    aircraft.html(),
                    DETAIL_METADATA_SEPARATOR,
                    aircraft.flight_url,
                    DETAIL_METADATA_SEPARATOR,
                    aircraft.jetphotos_url
                );
                let detail = nsstring(&payload);
                let detail_ptr = &*detail as *const _ as *mut Object;
                let detail_menu: *mut Object = msg_send![class!(NSMenu), alloc];
                let detail_menu: *mut Object = msg_send![detail_menu, initWithTitle: detail_ptr];
                let _: () = msg_send![detail_menu, setDelegate: target];
                add_menu_item(menu, &title, sel!(menuItemSelected:), target, None);
                let index: usize = msg_send![menu, numberOfItems];
                let parent_item: *mut Object = msg_send![menu, itemAtIndex: (index - 1)];
                let _: () = msg_send![parent_item, setSubmenu: detail_menu];
            }
        }

        let separator: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![menu, addItem: separator];
        let show_title = nsstring("Show Overhead");
        add_menu_item(menu, &show_title, sel!(showOverhead:), target, None);
        let separator: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![menu, addItem: separator];
        let quit_title = nsstring("Quit Overhead");
        add_menu_item(menu, &quit_title, sel!(quitOverhead:), target, None);
    }));
    if let Err(payload) = result {
        let message = payload
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
            .unwrap_or("unknown menu bar update error");
        eprintln!("menu bar update failed: {message}");
    }
}

#[cfg(target_os = "macos")]
unsafe fn add_external_menu_item(
    menu: *mut objc::runtime::Object,
    item: *mut objc::runtime::Object,
    url: *mut objc::runtime::Object,
) {
    use objc::{msg_send, sel, sel_impl};
    if item.is_null() || url.is_null() {
        return;
    }
    unsafe {
        let _: () = msg_send![item, setRepresentedObject: url];
        let _: () = msg_send![menu, addItem: item];
    }
}

#[cfg(target_os = "macos")]
unsafe fn add_menu_item(
    menu: *mut objc::runtime::Object,
    title: &objc2_foundation::NSString,
    action: objc::runtime::Sel,
    target: *mut objc::runtime::Object,
    represented_object: Option<*mut objc::runtime::Object>,
) {
    use objc::{class, msg_send, sel, sel_impl};
    unsafe {
        let item = new_menu_item(title, action, target);
        if item.is_null() {
            return;
        }
        if let Some(object) = represented_object {
            let _: () = msg_send![item, setRepresentedObject: object];
        }
        let _: () = msg_send![menu, addItem: item];
    }
}

#[cfg(target_os = "macos")]
unsafe fn new_menu_item(
    title: &objc2_foundation::NSString,
    action: objc::runtime::Sel,
    target: *mut objc::runtime::Object,
) -> *mut objc::runtime::Object {
    use objc::{class, msg_send, sel, sel_impl};
    unsafe {
        let key = nsstring("");
        let title_ptr = title as *const _ as *mut objc::runtime::Object;
        let key_ptr = &*key as *const _ as *mut objc::runtime::Object;
        let item: *mut objc::runtime::Object = msg_send![class!(NSMenuItem), alloc];
        if item.is_null() {
            return item;
        }
        let item: *mut objc::runtime::Object =
            msg_send![item, initWithTitle: title_ptr action: action keyEquivalent: key_ptr];
        if !item.is_null() && !target.is_null() {
            let _: () = msg_send![item, setTarget: target];
        }
        item
    }
}

#[cfg(target_os = "macos")]
fn nsstring(value: &str) -> objc2::rc::Retained<objc2_foundation::NSString> {
    objc2_foundation::NSString::from_str(value)
}

#[cfg(not(target_os = "macos"))]
pub fn init() {}

#[cfg(not(target_os = "macos"))]
pub fn update(_aircraft: &[AircraftDetails]) {}
