use iso_surface::{scene::Scene, surface::Surface, Physics, Ticker};
use std::{cell::RefCell, f64, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::ImageData;

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

// This function is automatically invoked after the wasm module is instantiated.
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    struct CustomBlob {};
    impl CustomBlob {
        const BOUNCE_THRUST: f64 = -20.0;
        const GRAVITY_DRAG: f64 = 0.27;
    }
    impl Ticker for CustomBlob {
        fn tick(&self, input: Physics, dimension: &(u32, u32)) -> Physics {
            let y = input.y + input.vy;
            let vy = if y > dimension.1 as f64 {
                Self::BOUNCE_THRUST
            } else {
                input.vy + Self::GRAVITY_DRAG
            };

            Physics::new(input.x, y, input.r, input.vx, vy)
        }
    }
    let custom_bouncy_blob = CustomBlob {};

    struct CustomSunBlob {}
    impl CustomSunBlob {}
    impl Ticker for CustomSunBlob {
        fn tick(&self, input: Physics, _: &(u32, u32)) -> Physics {
            Physics::new(input.x, input.y, input.r, input.vx, input.vy)
        }
    }
    let custom_sun_blob = CustomSunBlob {};

    let surface = Surface::new(1024, 768);
    let mut scene = Scene::new(&surface);
    scene.add_bouncer(40, 80, 10, 10, 10);
    scene.add_bouncer(147, 150, 25, 4, 4);
    scene.add_custom(1024 / 3 * 2, 0, 30, 0, 1, Box::new(custom_bouncy_blob));
    scene.add_custom(35, 35, 101, 0, 1, Box::new(custom_sun_blob));

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    canvas.set_width(surface.width());
    canvas.set_height(surface.height());

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut data_vec = Vec::<u8>::new();
    data_vec.resize((surface.width() * surface.height() * 4) as usize, 0);

    let mut data_index = 0;
    let frame_logic = move || {
        for y in 0..surface.height() as i32 {
            for x in 0..surface.width() as i32 {
                let color = Surface::pixel_color(&scene, x, y);
                data_vec[data_index] = color.0;
                data_index += 1;
                data_vec[data_index] = color.1;
                data_index += 1;
                data_vec[data_index] = color.2;
                data_index += 1;
                data_vec[data_index] = 255;
                data_index += 1;
            }
        }
        let data =
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut data_vec), 1024, 768).unwrap();
        let _ = context.put_image_data(&data, 0.0, 0.0);
        data_index = 0;

        // Call if want to stop the animation
        // let _ = f.borrow_mut().take();
        // return;

        scene.tick();
        request_animation_frame(f.borrow().as_ref().unwrap());
    };

    *g.borrow_mut() = Some(Closure::wrap(Box::new(frame_logic) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}
