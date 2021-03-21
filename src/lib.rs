use iso_surface::{scene::Scene, surface::Surface, Contextable, Physics, Ticker};
use itertools::Itertools;
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::sync::Mutex;
use std::{cell::RefCell, f64, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::{Document, ImageData};

lazy_static! {
    static ref GLOBAL_FRAME: Mutex<usize> = Mutex::new(0);
    static ref GLOBAL_DUMP_STATE: Mutex<HashSet<usize>> = Mutex::new(HashSet::new());
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

fn bare_bones() {
    log("Hello from Rust!");
    log_u32(42);
    log_many("Logging", "many values!");
}

// Next let's define a macro that's like `println!`, only it works for
// `console.log`. Note that `println!` doesn't actually work on the wasm target
// because the standard library currently just eats all output. To get
// `println!`-like behavior in your app you'll likely want a macro like this.

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// This function is automatically invoked after the wasm module is instantiated.
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    const WIDTH: u32 = 80;
    const HEIGHT: u32 = 192;

    // struct CustomBlob {};
    // impl CustomBlob {
    //     const BOUNCE_THRUST: f64 = -20.0;
    //     const GRAVITY_DRAG: f64 = 0.27;
    // }
    // impl Ticker for CustomBlob {
    //     fn tick(&self, input: Physics, dimension: &(u32, u32)) -> Physics {
    //         let y = input.y + input.vy;
    //         let vy = if y > dimension.1 as f64 {
    //             Self::BOUNCE_THRUST
    //         } else {
    //             input.vy + Self::GRAVITY_DRAG
    //         };

    //         Physics::new(input.x, y, input.r, input.vx, vy, 0)
    //     }
    // }
    // let custom_bouncy_blob = CustomBlob {};

    // struct CustomSunBlob {}
    // impl Ticker for CustomSunBlob {
    //     fn tick(&self, input: Physics, _: &(u32, u32)) -> Physics {
    //         Physics::new(input.x, input.y, input.r, input.vx, input.vy, 0)
    //     }
    // }
    // let custom_sun_blob = CustomSunBlob {};

    struct CustomFlickerBlobContext {
        frame: usize,
        frameskip: usize,
    }

    impl CustomFlickerBlobContext {
        const FRAME_SKIP: usize = 1;

        fn new(initial_frame: usize) -> Self {
            Self {
                frame: initial_frame,
                frameskip: Self::FRAME_SKIP,
            }
        }
    };

    impl Contextable for CustomFlickerBlobContext {
        fn frame(&self) -> usize {
            self.frame
        }

        fn set_frame(&mut self, frame: usize) {
            self.frame = frame;
            *(GLOBAL_FRAME.lock().unwrap()) = frame;
        }

        fn should_advance(&mut self) -> bool {
            if self.frameskip == 0 {
                self.frameskip = Self::FRAME_SKIP;
                true
            } else {
                self.frameskip -= 1;
                false
            }
        }
    }
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
    impl<C: Contextable> Ticker<C> for CustomFlickerBlob {
        fn tick(&self, input: Physics<C>, _: &(u32, u32)) -> Physics<C> {
            if let Some(mut context) = input.context {
                if context.should_advance() {
                    let mut frame = context.frame();
                    let pos = self.get_pos(frame);
                    frame = frame + 1;
                    console_log!("Frame {}", frame);
                    if frame == self.position_table.len() {
                        frame = 0;
                    }
                    context.set_frame(frame);
                    Physics::with_context(pos.0 / 2.0, pos.1, input.r, input.vx, input.vy, context)
                } else {
                    Physics::with_context(input.x, input.y, input.r, input.vx, input.vy, context)
                }
            } else {
                panic!("No context!");
            }
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

    struct CustomFlickerBlobRight {
        position_table: Vec<(f64, f64)>,
    }
    impl CustomFlickerBlobRight {
        fn get_pos(&self, index: usize) -> &(f64, f64) {
            self.position_table
                .get(index)
                .expect("No position defined?")
        }
    }
    impl<C: Contextable> Ticker<C> for CustomFlickerBlobRight {
        fn tick(&self, input: Physics<C>, _: &(u32, u32)) -> Physics<C> {
            if let Some(mut context) = input.context {
                if context.should_advance() {
                    let frame = context.frame();
                    let pos = self.get_pos(frame);
                    if frame == 0 {
                        context.set_frame(self.position_table.len() - 1)
                    } else {
                        context.set_frame(frame - 1)
                    }

                    Physics::with_context(
                        (pos.0 / 2.0) + (90.0 / 2.0),
                        pos.1,
                        input.r,
                        input.vx,
                        input.vy,
                        context,
                    )
                } else {
                    Physics::with_context(input.x, input.y, input.r, input.vx, input.vy, context)
                }
            } else {
                panic!("No context!");
            }
        }
    }
    let custom_flicker_blob_right = CustomFlickerBlobRight {
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

    scene.add_custom(
        0,
        0,
        3,
        0,
        0,
        CustomFlickerBlobContext::new(0),
        Box::new(custom_flicker_blob),
    );
    scene.add_custom(
        0,
        0,
        3,
        0,
        0,
        CustomFlickerBlobContext::new(0),
        Box::new(custom_flicker_blob_right),
    );
    //    scene.add_custom(0, 0, 10, 0, 0, 0, Box::new(custom_sun_blob));

    scene.tick();

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
        scene.tick();

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

        dump_pixel_values_to_console(&data_vec, &document);

        let data =
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut data_vec), WIDTH, HEIGHT)
                .unwrap();
        let _ = context.put_image_data(&data, 0.0, 0.0);
        data_index = 0;

        // Call if want to stop the animation
        // let _ = f.borrow_mut().take();
        // return;

        request_animation_frame(f.borrow().as_ref().unwrap());
    };

    *g.borrow_mut() = Some(Closure::wrap(Box::new(frame_logic) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

fn dump_pixel_values_to_console(data_vec: &Vec<u8>, document: &Document) {
    let global_frame = *(GLOBAL_FRAME.lock().unwrap());
    let global_dump_state = &mut *(GLOBAL_DUMP_STATE.lock().unwrap());
    if global_dump_state.contains(&global_frame) {
//        return;
    }
    console_log!("Actually dumping frame: {}", global_frame);
    global_dump_state.insert(global_frame);

    let colors: Vec<_> = data_vec.chunks_exact(4).map(|chunk| chunk[1]).collect();
    let s = colors.iter().join("\n");

    let text_box = document
        .get_element_by_id(format!("frame_header_{}", global_frame).as_str())
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
    text_box.set_inner_text(format!("Frame: {}", global_frame).as_str());

    let text_box = document
        .get_element_by_id(format!("frame_drop_{}", global_frame).as_str())
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
    text_box.set_inner_text(&s.to_string());
}
