#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[cfg(not(target_arch = "wasm32"))]
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
    ContextBuilder,
};

use glow::HasContext;
use instant::{Instant, Duration};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_main() {
    main();
}

#[cfg(target_arch = "wasm32")]
const DEFAULT_VERTEX_SHADER: &str = r#"attribute vec4 position;
attribute vec4 color;

varying vec4 Color;

void main() {
    gl_Position = position;
    Color = color;
}"#;

#[cfg(target_arch = "wasm32")]
const DEFAULT_FRAGMENT_SHADER: &str = r#"varying highp vec4 Color;

void main() {
    gl_FragColor = Color;
}"#;

#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_VERTEX_SHADER: &str = r#"#version 150
in vec4 position;
in vec4 color;

out vec4 Color;

void main() {
    Color = color;
    gl_Position = position;
}"#;

#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_FRAGMENT_SHADER: &str = r#"#version 150
in vec4 Color;

out vec4 outColor;

void main() {
    outColor = Color;
}"#;

#[cfg(target_arch = "wasm32")]
fn print(s: &str) {
    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(s));
}

#[cfg(not(target_arch = "wasm32"))]
fn print(s: &str) {
    println!("{}", s);
}

pub fn main() {
    #[cfg(target_arch = "wasm32")]
    let (event_loop, window, gl) = {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowExtWebSys;

        let event_loop = EventLoop::new();
        let window = Window::new(&event_loop).unwrap();

        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_with_node_1(&window.canvas())
            .unwrap();
        let webgl_context = window
            .canvas()
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::WebGlRenderingContext>()
            .unwrap();
        web_sys::console::log_1(&webgl_context);
        let gl = glow::Context::from_webgl1_context(webgl_context);
        (event_loop, window, gl)
    };

    #[cfg(not(target_arch = "wasm32"))]
    let (event_loop, windowed_context, gl) = {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().with_title("A fantastic window!");

        let windowed_context = ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(window, &event_loop)
            .unwrap();
        let windowed_context = unsafe { windowed_context.make_current().unwrap() };
        let gl = glow::Context::from_loader_function(|s| {
            windowed_context.get_proc_address(s) as *const _
        });
        (event_loop, windowed_context, gl)
    };

    unsafe {
        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
//        let ibo = gl.create_buffer().unwrap();

//        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ibo));
//        gl.blend_func_separate(
//            glow::SRC_ALPHA,
//            glow::ONE_MINUS_SRC_ALPHA,
//            glow::ONE,
//            glow::ONE_MINUS_SRC_ALPHA,
//        );
//        gl.enable(glow::BLEND);
        let vertex = gl.create_shader(glow::VERTEX_SHADER).unwrap();
        gl.shader_source(vertex, DEFAULT_VERTEX_SHADER);
        gl.compile_shader(vertex);
        if !gl.get_shader_compile_status(vertex) {
            panic!(gl.get_shader_info_log(vertex));
        }
        let fragment = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
        gl.shader_source(fragment, DEFAULT_FRAGMENT_SHADER);
        gl.compile_shader(fragment);
        if !gl.get_shader_compile_status(fragment) {
            panic!(gl.get_shader_info_log(fragment));
        }
        let shader = gl.create_program().unwrap();
        gl.attach_shader(shader, vertex);
        gl.attach_shader(shader, fragment);
        gl.link_program(shader);
        gl.use_program(Some(shader));

        gl.viewport(0, 0, 1, 1);

        let stride_distance = ((2 + 4) * std::mem::size_of::<f32>()) as i32;
        // Set up the vertex attributes
        let pos_attrib = gl.get_attrib_location(shader, "position") as u32;
        gl.enable_vertex_attrib_array(pos_attrib);
        gl.vertex_attrib_pointer_f32(pos_attrib, 2, glow::FLOAT, false, stride_distance, 0i32);
        let col_attrib = gl.get_attrib_location(shader, "color") as u32;
        gl.enable_vertex_attrib_array(col_attrib);
        gl.vertex_attrib_pointer_f32(col_attrib, 4, glow::FLOAT, false, stride_distance, 2 * std::mem::size_of::<f32>() as i32);

        #[repr(C)]
        struct Vertex {
            position: [f32; 2],
            color: [f32; 4],
        }

        let verts = [
            Vertex { position: [0.5, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
            Vertex { position: [0.0, 1.0], color: [0.0, 1.0, 0.0, 1.0] },
            Vertex { position: [1.0, 1.0], color: [0.0, 0.0, 1.0, 1.0] },
        ];
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            std::slice::from_raw_parts(
                verts.as_ptr() as *const u8,
                verts.len() * std::mem::size_of::<Vertex>()
            )
            ,
            glow::STATIC_DRAW
        );

        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        gl.draw_arrays(glow::TRIANGLES, 0, 3);
    }

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, window_id } => match event {
                WindowEvent::RedrawRequested => {
                    unsafe {
                        gl.clear_color(0.0, 0.0, 0.0, 1.0);
                        gl.clear(glow::COLOR_BUFFER_BIT);
                        gl.draw_arrays(glow::TRIANGLES, 0, 3);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    windowed_context.swap_buffers().unwrap();

                    print(format!("{:?}", event).as_str());
                },
                WindowEvent::CloseRequested => {
                    print(format!("Window {:?} has received the signal to close", window_id).as_str());

                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            },
            Event::EventsCleared => {
                #[cfg(target_arch = "wasm32")]
                window.request_redraw();
                #[cfg(not(target_arch = "wasm32"))]
                windowed_context.window().request_redraw();
                *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::new(1, 0))
            },
            _ => (),
        }
    })
}
