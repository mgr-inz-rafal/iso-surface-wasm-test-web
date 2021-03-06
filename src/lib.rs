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

    const WIDTH: u32 = 80;
    const HEIGHT: u32 = 192;

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

            Physics::new(input.x, y, input.r, input.vx, vy, 0)
        }
    }
    let custom_bouncy_blob = CustomBlob {};

    struct CustomSunBlob {}
    impl Ticker for CustomSunBlob {
        fn tick(&self, input: Physics, _: &(u32, u32)) -> Physics {
            Physics::new(input.x, input.y, input.r, input.vx, input.vy, 0)
        }
    }
    let custom_sun_blob = CustomSunBlob {};

    struct CustomFlickerBlob {
        position_table: Vec<(f64, f64)>,
    }
    impl CustomFlickerBlob {
        fn get_pos(&self, index: usize) -> &(f64, f64) {
            self.position_table
                .get(index)
                .expect("No position defined?")
        }
    }
    impl Ticker for CustomFlickerBlob {
        fn tick(&self, input: Physics, _: &(u32, u32)) -> Physics {
            let pos = self.get_pos(input.context);
            let mut frame = input.context + 1;
            if frame == self.position_table.len() {
                frame = 0;
            }
            Physics::new(pos.0 / 2.0, pos.1, input.r, input.vx, input.vy, frame)
        }
    }
    let custom_flicker_blob = CustomFlickerBlob {
        position_table: vec![
            (35.0, 143.0),
            (40.0, 142.0),
            (45.0, 140.0),
            (50.0, 137.0),
            (55.0, 133.0),
            (60.0, 127.0),
            (64.0, 120.0),
            (68.0, 113.0),
            (72.0, 104.0),
            (74.0, 95.0),
            (77.0, 85.0),
            (78.0, 74.0),
            (79.0, 64.0),
            (79.0, 53.0),
            (79.0, 42.0),
            (78.0, 31.0),
            (77.0, 21.0),
            (74.0, 11.0),
            (72.0, 2.0),
            (68.0, -6.0),
            (64.0, -14.0),
            (60.0, -20.0),
            (55.0, -26.0),
            (50.0, -30.0),
            (45.0, -34.0),
            (40.0, -36.0),
            (35.0, -36.0),
            (29.0, -36.0),
            (24.0, -34.0),
            (19.0, -30.0),
            (14.0, -26.0),
            (9.0, -20.0),
            (5.0, -14.0),
            (1.0, -6.0),
            (-2.0, 2.0),
            (-4.0, 11.0),
            (-7.0, 21.0),
            (-8.0, 31.0),
            (-9.0, 42.0),
            (-9.0, 53.0),
            (-9.0, 64.0),
            (-8.0, 74.0),
            (-7.0, 85.0),
            (-4.0, 95.0),
            (-2.0, 104.0),
            (1.0, 113.0),
            (5.0, 120.0),
            (9.0, 127.0),
            (14.0, 133.0),
            (19.0, 137.0),
            (24.0, 140.0),
            (29.0, 142.0),
        ],
    };

    let surface = Surface::new(WIDTH, HEIGHT);
    let mut scene = Scene::new(&surface);
    //scene.add_bouncer(10, 10, 10, 10, 10);

    scene.add_custom(0, 0, 2, 0, 0, Box::new(custom_flicker_blob));
    scene.add_custom(0, 0, 10, 0, 0, Box::new(custom_sun_blob));

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
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut data_vec), WIDTH, HEIGHT)
                .unwrap();
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
