static ARENA: &str = "arena";

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a `document` on window")
}

pub fn arena() -> Option<web_sys::Element> {
    document().get_element_by_id(ARENA)
}

pub fn get_arena_bounds() -> Option<web_sys::DomRect> {
    Some(arena()?.get_bounding_client_rect())
}

pub fn middle(rect: web_sys::DomRect) -> (f64, f64) {
    (rect.width()/2., rect.height()/2.)
}