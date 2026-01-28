use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl AxRect {
    pub fn center_x(&self) -> f64 {
        self.x + (self.width / 2.0)
    }
}

#[cfg(test)]
pub trait AxProvider {
    fn bundle_ids(&self) -> Vec<String>;
}

#[cfg(test)]
#[derive(Default)]
pub struct MockAx {
    bundles: Vec<String>,
}

#[cfg(test)]
impl MockAx {
    pub fn with_bundle(bundle_id: &str) -> Self {
        Self {
            bundles: vec![bundle_id.to_string()],
        }
    }

    #[allow(dead_code)]
    pub fn add_bundle(&mut self, bundle_id: &str) {
        self.bundles.push(bundle_id.to_string());
    }
}

#[cfg(test)]
impl AxProvider for MockAx {
    fn bundle_ids(&self) -> Vec<String> {
        self.bundles.clone()
    }
}

#[cfg(test)]
pub fn find_wechat_app(provider: &dyn AxProvider) -> Option<String> {
    provider
        .bundle_ids()
        .into_iter()
        .find(|bundle| bundle == "com.tencent.xinWeChat" || bundle == "com.tencent.WeChat")
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
mod native {
    use super::*;
    use crate::ui_automation::macos::ax_path::{resolve_path, AxNodeInfo, AxPathStep};
    use core_foundation::array::{CFArray, CFArrayRef};
    use core_foundation::base::{CFRelease, CFRetain, CFTypeRef, TCFType};
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::number::CFNumber;
    use core_foundation::string::{CFString, CFStringRef};
    use core_graphics::geometry::{CGPoint, CGRect, CGSize};
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, KeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    use objc::{class, msg_send, sel, sel_impl};
    use objc::runtime::Object;
    use std::ffi::CString;
    use std::ptr;
    use std::ffi::c_void;

    type AXUIElementRef = *const std::ffi::c_void;
    type AXValueRef = *const std::ffi::c_void;
    type AXValueType = i32;
    type AXError = i32;

    const AX_SUCCESS: AXError = 0;
    const AX_VALUE_CGRECT: AXValueType = 3;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
        fn AXUIElementCopyAttributeValue(
            element: AXUIElementRef,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> AXError;
        fn AXUIElementSetAttributeValue(
            element: AXUIElementRef,
            attribute: CFStringRef,
            value: CFTypeRef,
        ) -> AXError;
        fn AXIsProcessTrustedWithOptions(options: CFTypeRef) -> bool;
        fn AXValueGetType(value: AXValueRef) -> AXValueType;
        fn AXValueGetValue(value: AXValueRef, the_type: AXValueType, value_ptr: *mut c_void) -> bool;
    }

    #[derive(Debug)]
    pub struct AxElement {
        element: AXUIElementRef,
    }

    // SAFETY: AXUIElementRef is a Core Foundation object; we retain/release
    // and only use it via system APIs.
    unsafe impl Send for AxElement {}
    unsafe impl Sync for AxElement {}

    impl Clone for AxElement {
        fn clone(&self) -> Self {
            unsafe {
                CFRetain(self.element as _);
            }
            Self {
                element: self.element,
            }
        }
    }

    impl Drop for AxElement {
        fn drop(&mut self) {
            unsafe {
                CFRelease(self.element as _);
            }
        }
    }

    impl AxElement {
        pub fn from_raw(element: AXUIElementRef) -> Option<Self> {
            if element.is_null() {
                return None;
            }
            unsafe {
                CFRetain(element as _);
            }
            Some(Self { element })
        }

        pub fn raw(&self) -> AXUIElementRef {
            self.element
        }
    }

    pub struct AxClient {
        #[allow(dead_code)]
        pid: i32,
        app: AxElement,
    }

    impl AxClient {
        pub fn new() -> Result<Self> {
            let bundle_ids = ["com.tencent.xinWeChat", "com.tencent.WeChat"];
            for bundle_id in bundle_ids {
                if let Some(pid) = running_app_pid(bundle_id) {
                    let app = unsafe { AXUIElementCreateApplication(pid) };
                    if let Some(app) = AxElement::from_raw(app) {
                        return Ok(Self { pid, app });
                    }
                }
            }
            Err(anyhow!("WeChat app not running"))
        }

        #[allow(dead_code)]
        pub fn app(&self) -> &AxElement {
            &self.app
        }

        #[allow(dead_code)]
        pub fn pid(&self) -> i32 {
            self.pid
        }

        pub fn windows(&self) -> Vec<AxElement> {
            copy_attribute_array(&self.app, &cfstr("AXWindows"))
                .unwrap_or_default()
        }

        pub fn front_window(&self) -> Option<AxElement> {
            self.windows().into_iter().next()
        }
    }

    pub fn check_accessibility() -> bool {
        let prompt_key = CFString::new("AXTrustedCheckOptionPrompt");
        let prompt_value = CFNumber::from(1i32);
        let dict = CFDictionary::from_CFType_pairs(&[(prompt_key.as_CFType(), prompt_value.as_CFType())]);
        unsafe { AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef() as _) }
    }

    pub fn focus_element(element: &AxElement) -> Result<()> {
        let value = CFNumber::from(1i32);
        set_attribute_value(element, &cfstr("AXFocused"), value.as_concrete_TypeRef() as _)
    }

    pub fn send_page_down() -> Result<()> {
        let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
            .map_err(|_| anyhow!("CGEventSource failed"))?;
        let key_down = CGEvent::new_keyboard_event(source.clone(), KeyCode::PAGE_DOWN, true)
            .map_err(|_| anyhow!("CGEvent keydown failed"))?;
        let key_up = CGEvent::new_keyboard_event(source, KeyCode::PAGE_DOWN, false)
            .map_err(|_| anyhow!("CGEvent keyup failed"))?;
        key_down.post(CGEventTapLocation::HID);
        key_up.post(CGEventTapLocation::HID);
        Ok(())
    }

    pub fn paste_text(text: &str) -> Result<()> {
        set_clipboard_text(text)?;
        let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
            .map_err(|_| anyhow!("CGEventSource failed"))?;
        let key_down = CGEvent::new_keyboard_event(source.clone(), KeyCode::COMMAND, true)
            .map_err(|_| anyhow!("CGEvent keydown failed"))?;
        let v_down = CGEvent::new_keyboard_event(source.clone(), 0x09, true)
            .map_err(|_| anyhow!("CGEvent V down failed"))?;
        let v_up = CGEvent::new_keyboard_event(source.clone(), 0x09, false)
            .map_err(|_| anyhow!("CGEvent V up failed"))?;
        let key_up = CGEvent::new_keyboard_event(source, KeyCode::COMMAND, false)
            .map_err(|_| anyhow!("CGEvent keyup failed"))?;
        v_down.set_flags(CGEventFlags::CGEventFlagCommand);
        v_up.set_flags(CGEventFlags::CGEventFlagCommand);
        key_down.post(CGEventTapLocation::HID);
        v_down.post(CGEventTapLocation::HID);
        v_up.post(CGEventTapLocation::HID);
        key_up.post(CGEventTapLocation::HID);
        Ok(())
    }

    pub fn set_attribute_value(element: &AxElement, attr: &CFString, value: CFTypeRef) -> Result<()> {
        let result = unsafe { AXUIElementSetAttributeValue(element.raw(), attr.as_concrete_TypeRef() as _, value) };
        if result == AX_SUCCESS {
            Ok(())
        } else {
            Err(anyhow!("AX set attribute failed"))
        }
    }

    pub fn copy_attribute_string(element: &AxElement, attr: &CFString) -> Option<String> {
        let value = copy_attribute_value(element, attr)?;
        let string = unsafe { CFString::wrap_under_get_rule(value as CFStringRef) };
        Some(string.to_string())
    }

    pub fn copy_attribute_array(element: &AxElement, attr: &CFString) -> Option<Vec<AxElement>> {
        let value = copy_attribute_value(element, attr)?;
        let array = unsafe { CFArray::<CFTypeRef>::wrap_under_get_rule(value as CFArrayRef) };
        let mut results = Vec::new();
        for item in array.iter() {
            let raw = *item as AXUIElementRef;
            if let Some(element) = AxElement::from_raw(raw) {
                results.push(element);
            }
        }
        Some(results)
    }

    pub fn collect_session_titles(list: &AxElement) -> Vec<String> {
        let mut titles = Vec::new();
        for row in children(list) {
            let texts = collect_static_texts(&row, 6);
            if let Some(title) = pick_session_title(&texts) {
                titles.push(title);
            }
        }
        titles
    }

    pub fn find_lists_with_titles(root: &AxElement, depth: usize) -> Vec<(AxElement, Vec<String>)> {
        let mut items = Vec::new();
        walk(root, depth, &mut |element| {
            if let Some(role) = role(element) {
                if role == "AXOutline" || role == "AXTable" || role == "AXList" {
                    let titles = collect_session_titles(element);
                    if !titles.is_empty() {
                        items.push((element.clone(), titles));
                    }
                }
            }
        });
        items
    }

    pub fn children(element: &AxElement) -> Vec<AxElement> {
        copy_attribute_array(element, &cfstr("AXChildren")).unwrap_or_default()
    }

    pub fn resolve_ax_path(element: &AxElement, steps: &[AxPathStep]) -> Option<AxElement> {
        resolve_path(
            element.clone(),
            steps,
            |item| AxNodeInfo {
                role: role(item),
                title: title(item),
            },
            children,
        )
    }

    pub fn resolve_any_path(element: &AxElement, paths: &[&[AxPathStep]]) -> Option<AxElement> {
        for path in paths {
            if let Some(found) = resolve_ax_path(element, path) {
                return Some(found);
            }
        }
        None
    }

    pub fn role(element: &AxElement) -> Option<String> {
        copy_attribute_string(element, &cfstr("AXRole"))
    }

    pub fn title(element: &AxElement) -> Option<String> {
        copy_attribute_string(element, &cfstr("AXTitle"))
    }

    pub fn value(element: &AxElement) -> Option<String> {
        copy_attribute_string(element, &cfstr("AXValue"))
    }

    #[allow(dead_code)]
    pub fn first_static_text(element: &AxElement, depth: usize) -> Option<String> {
        if depth == 0 {
            return None;
        }
        if role(element).as_deref() == Some("AXStaticText") {
            if let Some(value) = value(element) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        for child in children(element) {
            if let Some(found) = first_static_text(&child, depth - 1) {
                return Some(found);
            }
        }
        None
    }

    pub fn collect_static_texts(element: &AxElement, depth: usize) -> Vec<String> {
        let mut results = Vec::new();
        collect_static_texts_inner(element, depth, &mut results);
        results
    }

    pub fn find_input_element(root: &AxElement, depth: usize) -> Option<AxElement> {
        if depth == 0 {
            return None;
        }
        if let Some(role) = role(root) {
            if role == "AXTextArea" || role == "AXTextField" {
                return Some(root.clone());
            }
        }
        for child in children(root) {
            if let Some(found) = find_input_element(&child, depth - 1) {
                return Some(found);
            }
        }
        None
    }

    pub fn frame(element: &AxElement) -> Option<AxRect> {
        let value = copy_attribute_value(element, &cfstr("AXFrame"))?;
        let value_ref = value as AXValueRef;
        let value_type = unsafe { AXValueGetType(value_ref) };
        if value_type != AX_VALUE_CGRECT {
            return None;
        }
        let mut rect = CGRect::new(&CGPoint::new(0.0, 0.0), &CGSize::new(0.0, 0.0));
        let ok = unsafe {
            AXValueGetValue(
                value_ref,
                AX_VALUE_CGRECT,
                &mut rect as *mut _ as *mut c_void,
            )
        };
        if !ok {
            return None;
        }
        Some(AxRect {
            x: rect.origin.x,
            y: rect.origin.y,
            width: rect.size.width,
            height: rect.size.height,
        })
    }

    pub fn set_input_value(element: &AxElement, text: &str) -> Result<()> {
        let value = CFString::new(text);
        set_attribute_value(element, &cfstr("AXValue"), value.as_concrete_TypeRef() as _)
    }

    pub fn set_clipboard_text(text: &str) -> Result<()> {
        let c_string = CString::new(text).map_err(|_| anyhow!("Clipboard text invalid"))?;
        unsafe {
            let ns_string: *mut Object = msg_send![class!(NSString), alloc];
            let ns_string: *mut Object = msg_send![ns_string, initWithUTF8String: c_string.as_ptr()];
            let pasteboard: *mut Object = msg_send![class!(NSPasteboard), generalPasteboard];
            let _: i64 = msg_send![pasteboard, clearContents];
            let type_string = CString::new("public.utf8-plain-text").map_err(|_| anyhow!("Clipboard type invalid"))?;
            let ns_type: *mut Object = msg_send![class!(NSString), alloc];
            let ns_type: *mut Object = msg_send![ns_type, initWithUTF8String: type_string.as_ptr()];
            let _: bool = msg_send![pasteboard, setString: ns_string forType: ns_type];
        }
        Ok(())
    }

    fn copy_attribute_value(element: &AxElement, attr: &CFString) -> Option<CFTypeRef> {
        let mut value: CFTypeRef = ptr::null();
        let result = unsafe {
            AXUIElementCopyAttributeValue(element.raw(), attr.as_concrete_TypeRef() as _, &mut value)
        };
        if result == AX_SUCCESS && !value.is_null() {
            Some(value)
        } else {
            None
        }
    }

    fn cfstr(value: &str) -> CFString {
        CFString::new(value)
    }

    fn collect_static_texts_inner(element: &AxElement, depth: usize, results: &mut Vec<String>) {
        if depth == 0 {
            return;
        }
        if role(element).as_deref() == Some("AXStaticText") {
            if let Some(value) = value(element) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    results.push(trimmed.to_string());
                }
            }
        }
        for child in children(element) {
            collect_static_texts_inner(&child, depth - 1, results);
        }
    }

    fn pick_session_title(texts: &[String]) -> Option<String> {
        let mut fallback = None;
        for item in texts {
            let trimmed = item.trim();
            if trimmed.is_empty() {
                continue;
            }
            if fallback.is_none() {
                fallback = Some(trimmed);
            }
            if looks_like_time(trimmed) {
                continue;
            }
            return Some(trimmed.to_string());
        }
        fallback.map(|item| item.to_string())
    }

    fn looks_like_time(text: &str) -> bool {
        let trimmed = text.trim();
        if is_clock_time(trimmed) {
            return true;
        }
        if is_date(trimmed) {
            return true;
        }
        let upper = trimmed.to_ascii_uppercase();
        upper.ends_with(" AM") || upper.ends_with(" PM") || upper == "AM" || upper == "PM"
    }

    fn is_clock_time(text: &str) -> bool {
        if text.len() != 5 {
            return false;
        }
        let bytes = text.as_bytes();
        bytes[2] == b':'
            && bytes[0].is_ascii_digit()
            && bytes[1].is_ascii_digit()
            && bytes[3].is_ascii_digit()
            && bytes[4].is_ascii_digit()
    }

    fn is_date(text: &str) -> bool {
        if text.len() != 10 {
            return false;
        }
        let bytes = text.as_bytes();
        bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[..4].iter().all(|b| b.is_ascii_digit())
            && bytes[5..7].iter().all(|b| b.is_ascii_digit())
            && bytes[8..10].iter().all(|b| b.is_ascii_digit())
    }

    fn walk(element: &AxElement, depth: usize, visit: &mut impl FnMut(&AxElement)) {
        if depth == 0 {
            return;
        }
        visit(element);
        for child in children(element) {
            walk(&child, depth - 1, visit);
        }
    }

    fn running_app_pid(bundle_id: &str) -> Option<i32> {
        let c_bundle = CString::new(bundle_id).ok()?;
        unsafe {
            let ns_string: *mut Object = msg_send![class!(NSString), alloc];
            let ns_string: *mut Object = msg_send![ns_string, initWithUTF8String: c_bundle.as_ptr()];
            let apps: *mut Object = msg_send![class!(NSRunningApplication), runningApplicationsWithBundleIdentifier: ns_string];
            let count: usize = msg_send![apps, count];
            if count == 0 {
                return None;
            }
            let app: *mut Object = msg_send![apps, objectAtIndex: 0usize];
            let pid: i32 = msg_send![app, processIdentifier];
            Some(pid)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::pick_session_title;

        #[test]
        fn picks_non_time_text() {
            let texts = vec![
                "09:11".to_string(),
                "Alice".to_string(),
                "See you tonight?".to_string(),
            ];
            assert_eq!(pick_session_title(&texts), Some("Alice".to_string()));
        }

        #[test]
        fn falls_back_to_first_text() {
            let texts = vec!["09:11".to_string()];
            assert_eq!(pick_session_title(&texts), Some("09:11".to_string()));
        }
    }

}

#[cfg(target_os = "macos")]
pub use native::*;
