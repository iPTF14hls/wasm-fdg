#![feature(clamp)]

mod force_directed_graph;
mod utils;

use crate::utils::{arena, document, get_arena_bounds, window};
use cfg_if::cfg_if;
use js_sys::Array;
use lazy_static::lazy_static;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};
use serde::Deserialize;

use specs::{World, WorldExt};
use wasm_bindgen::prelude::*;

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
    let mut world = WORLD.lock().unwrap();
    {
        let mut delta = world.write_resource::<DeltaTime>();
        *delta = DeltaTime(delta_ms());
    }
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
pub fn update_arena_size(w: f64, h: f64) {
    use force_directed_graph::ArenaSize;
    use specs::WorldExt;
    let world = WORLD.lock().unwrap();
    let mut size = world.write_resource::<ArenaSize>();
    *size = ArenaSize((w, h));
}

//input
//x: f64, 
//y: f64, 
//w: f64, 
//h: f64, 
//xv: f64, y
//v: f64, 
//text: &str
use crate::force_directed_graph::{Position, Velocity, Collider};


#[derive(Deserialize)]
pub struct EntityArgs {
    pub pos: Option<Position>,
    pub vel: Option<Velocity>,
    pub colds: Option<Collider>,
    pub html: String,
}

#[wasm_bindgen]
pub fn spawn_entity(args: &JsValue, classes: Array) -> Result<String, JsValue> {
    let class: Vec<String> = classes.iter().filter_map(|elem| elem.as_string()).collect();
    let id: u64 = ID.fetch_add(1, Ordering::Relaxed);
    let args: EntityArgs = match args.into_serde() {
        Ok(args) => args,
        Err(e) => return Err(JsValue::from(format!("Could not parse args. {}", e))),
    };

    //Dom shenanigans
    let arena = arena().unwrap();
    let elem = document().create_element("div")?;
    elem.set_inner_html(args.html.as_str());
    let class_name = class
        .into_iter()
        .fold(String::new(), |acc, s| format!("{} {}", acc, s));
    elem.set_class_name(&(class_name.trim()));
    let id = format!("elem{}",id);
    elem.set_id(&id);
    arena.append_child(&elem)?;

    //Now lets set up the real entity
    use crate::force_directed_graph::{DomElement, Repel}; 
    use specs::prelude::*;

    let mut world = WORLD.lock().unwrap();
    let ent = world.create_entity()
        .with(args.pos.unwrap_or_default())
        .with(args.vel.unwrap_or_default())
        .with(DomElement{id: id.clone()})
        .with(Repel{charge: 50.});

    let ent = match args.colds {
        Some(cold) => ent.with(cold),
        None => ent,
    };

    ent.build();

    Ok(id)
}

#[wasm_bindgen]
pub fn print_arena_statistics() -> String {
    match get_arena_bounds() {
        Some(bounds) => format!("Width {}\nHeight {}", bounds.width(), bounds.height()),
        None => String::from("Arena not initialized"),
    }
}
