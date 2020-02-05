#![feature(clamp)]
#![feature(impl_trait_in_bindings)]

mod force_directed_graph;
mod utils;

use crate::utils::{arena, document, get_arena_bounds, window, middle};
use cfg_if::cfg_if;
use js_sys::Array;
use lazy_static::lazy_static;
use specs::World;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};
use wasm_bindgen::prelude::*;
use log::info;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

lazy_static! {
    static ref PTIME: Mutex<Option<f64>> = Mutex::new(None);
    static ref WORLD: Mutex<World> = Mutex::new(force_directed_graph::initialize_world());
    static ref ID: AtomicU64 = AtomicU64::new(0);
}

fn delta_ms() -> f64 {
    let mut time = PTIME.lock().unwrap();
    let ctime = window()
        .performance()
        .expect("performance should be available")
        .now();

    let delta = if let Some(t) = *time { ctime - t } else { 0. };
    *time = Some(ctime);

    delta
}

#[wasm_bindgen]
pub fn tick() {
    use force_directed_graph::DeltaTime;
    use specs::{Read, ReadStorage, System, World, WorldExt, WriteStorage};
    let mut world = WORLD.lock().unwrap();
    {
        let mut delta = world.write_resource::<DeltaTime>();
        *delta = DeltaTime(delta_ms());
    }
    info!("tick");
    crate::force_directed_graph::execute_systems(&world);
    world.maintain();
}

cfg_if! {
    if #[cfg(feature = "console_log")] {
        fn init_log() {
            use log::Level;
            console_log::init_with_level(Level::Trace).expect("error initializing log");
        }
    } else {
        fn init_log() {}
    }
}

#[wasm_bindgen]
pub fn init() {
    crate::utils::set_panic_hook();
    init_log();
}

#[wasm_bindgen]
pub fn update_mouse_position(x: f64, y: f64) {
    use force_directed_graph::MousePos;
    use specs::WorldExt;
    let world = WORLD.lock().unwrap();
    let mut position = world.write_resource::<MousePos>();
    *position = MousePos((x, y));
}

#[wasm_bindgen]
pub fn spawn_entity(text: &str, classes: Array) -> Result<(), JsValue> {
    let class: Vec<String> = classes.iter().filter_map(|elem| elem.as_string()).collect();
    let id: u64 = ID.fetch_add(1, Ordering::Relaxed);
    
    //Dom shenanigans
    let arena = arena().unwrap();
    let elem = document().create_element("div")?;
    elem.set_inner_html(text);
    let class_name = class
        .into_iter()
        .fold(String::new(), |acc, s| format!("{} {}", acc, s));
    elem.set_class_name(&(class_name.trim()));
    let id = format!("elem{}",id);
    elem.set_id(&id);
    arena.append_child(&elem)?;

    //Now lets set up the real entity
    use crate::force_directed_graph::{Position, Velocity, DomElement, MouseAttract}; 
    use specs::prelude::*;

    let mut world = WORLD.lock().unwrap();
    let (x, y) = middle(arena.get_bounding_client_rect());
    world.create_entity()
        .with(Position{x, y})
        .with(Velocity::default())
        .with(DomElement{id})
        .with(MouseAttract)
        .build();

    Ok(())
}

#[wasm_bindgen]
pub fn print_arena_statistics() -> String {
    match get_arena_bounds() {
        Some(bounds) => format!("Width {}\nHeight {}", bounds.width(), bounds.height()),
        None => String::from("Arena not initialized"),
    }
}
