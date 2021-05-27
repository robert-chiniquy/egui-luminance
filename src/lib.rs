// TODO test how this affects fps + wasm size
// #![feature(allocator_api)]

#[macro_use]
mod utils;
pub mod scene;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
pub struct App {
    scene: scene::Scene,
}

#[wasm_bindgen]
impl App {
    pub fn new() -> Result<App, JsValue> {
        Ok(App {
            scene: scene::Scene::new(),
        })
    }

    pub fn tick(&mut self, t: f32) -> Result<(), JsValue> {
        self.scene.render(t);
        Ok(())
    }
}
