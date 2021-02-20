use std::{cell::RefCell, f64, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::{CanvasRenderingContext2d, ImageData};

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}

// This function is automatically invoked after the wasm module is instantiated.
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let mut scene = iso_surface::Scene::new();
    scene.add_blob(40, 80, 10, 10, 10);
    scene.add_blob(147, 150, 25, 4, 4);
    scene.add_blob(500, 450, 50, -2, 2);
    scene.add_blob(300.0, 20.0, 60.0, -0.7, -0.7);
    scene.add_blob(1000, 666, 10, -10, 10);
    let surface = iso_surface::Surface::new(1024, 768);

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
    data_vec.resize(1024 * 768 * 4, 0);

    let mut i = 0;
    let mut data_index = 0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        for y in 0..768 {
            for x in 0..1024 {
                let color = iso_surface::Surface::pixel_color(&scene, x, y);
                data_vec[data_index] = color.0;
                data_index += 1;
                data_vec[data_index] = color.1;
                data_index += 1;
                data_vec[data_index] = color.2;
                data_index += 1;
                data_vec[data_index] = 255;
                data_index += 1;
                // data_vec.push(color.0);
                // data_vec.push(color.1);
                // data_vec.push(color.2);
                // data_vec.push(255);

                // data_vec.push(255);
                // data_vec.push(0);
                // data_vec.push(0);
                // data_vec.push(255);

                //    let color = format!("rgb({},{},{})", color.0, color.1, color.2);
                //    let color_str = color.as_str();
                //    context.set_fill_style(&color_str.into());
                //    context.fill_rect(x as f64, y as f64, 1.0, 1.0);
            }
        }
        //console_log!("Data vec size: {}", data_vec.len());
        let data =
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut data_vec), 1024, 768).unwrap();
        let _ = context.put_image_data(&data, 0.0, 0.0);
        data_index = 0;

        // if i > 300 {
        //     body().set_text_content(Some("All done!"));

        //     // Drop our handle to this closure so that it will get cleaned
        //     // up once we return.
        //     let _ = f.borrow_mut().take();
        //     return;
        // }

        // Set the body's text content to how many times this
        // requestAnimationFrame callback has fired.
        i += 1;
//        console_log!("Frame: {}", i);
        //let text = format!("requestAnimationFrame has been called {} times.", i);
        //      body().set_text_content(Some(&text));

        // Schedule ourself for another requestAnimationFrame callback.
        scene.tick();
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

/*
fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let mut scene = iso_surface::Scene::new();
    scene.add_blob(40, 80, 20);
    scene.add_blob(147, 150, 30);
    let surface = iso_surface::Surface::new(640, 480, &scene);

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

    for y in 0..480 {
        for x in 0..640 {
            let color = surface.pixel_color(x, y);
            let color = format!("rgb({},{},{})", color.0, color.1, color.2);
            let color_str = color.as_str();
            context.set_fill_style(&color_str.into());
            context.fill_rect(x as f64, y as f64, 1.0, 1.0);
        }
    }

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

*/
