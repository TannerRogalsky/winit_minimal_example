#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(not(target_arch = "wasm32"))]
use glutin::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};

use glow::HasContext;

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

pub fn main() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("Minimalism")
        .with_inner_size(LogicalSize::new(720.0, 480.0));

    #[cfg(target_arch = "wasm32")]
    let (event_loop, window, gl) = {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowExtWebSys;

        let window = window_builder.build(&event_loop).unwrap();

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
        let gl = glow::Context::from_webgl1_context(webgl_context);
        (event_loop, window, gl)
    };

    #[cfg(not(target_arch = "wasm32"))]
    let (windowed_context, gl) = {
        let windowed_context = ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(window_builder, &event_loop)
            .unwrap();
        let windowed_context = unsafe { windowed_context.make_current().unwrap() };
        let gl = glow::Context::from_loader_function(|s| {
            windowed_context.get_proc_address(s) as *const _
        });

        unsafe {
            let vao = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
            gl.bind_vertex_array(Some(vao));
        }

        (windowed_context, gl)
    };

    #[repr(C)]
    struct Vertex {
        position: [f32; 2],
        color: [f32; 4],
    }

    unsafe {
        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

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

        let stride_distance = ((2 + 4) * std::mem::size_of::<f32>()) as i32;
        // Set up the vertex attributes
        let pos_attrib = gl.get_attrib_location(shader, "position") as u32;
        gl.enable_vertex_attrib_array(pos_attrib);
        gl.vertex_attrib_pointer_f32(pos_attrib, 2, glow::FLOAT, false, stride_distance, 0i32);
        let col_attrib = gl.get_attrib_location(shader, "color") as u32;
        gl.enable_vertex_attrib_array(col_attrib);
        gl.vertex_attrib_pointer_f32(
            col_attrib,
            4,
            glow::FLOAT,
            false,
            stride_distance,
            2 * std::mem::size_of::<f32>() as i32,
        );

        gl.clear_color(0.0, 0.0, 0.0, 1.0);
    }

    let mut prev = instant::Instant::now();
    let mut time = std::time::Duration::default();

    event_loop.run(move |event, _event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, window_id: _window_id } => match event {
                WindowEvent::RedrawRequested => {
                    time += prev.elapsed();
                    prev = instant::Instant::now();
                    let t = time.as_millis() as f32 / 1000.0;
                    let a = t.sin().powi(2);
                    let b = (t + std::f32::consts::PI * (1.0 / 3.0)).sin().powi(2);
                    let c = (t + std::f32::consts::PI * (2.0 / 3.0)).sin().powi(2);
                    let verts = [
                        Vertex {
                            position: [-0.5, -0.5],
                            color: [a, b, c, 1.0],
                        },
                        Vertex {
                            position: [0.0, 0.5],
                            color: [c, a, b, 1.0],
                        },
                        Vertex {
                            position: [0.5, -0.5],
                            color: [b, c, a, 1.0],
                        },
                    ];

                    unsafe {
                        gl.buffer_data_u8_slice(
                            glow::ARRAY_BUFFER,
                            std::slice::from_raw_parts(
                                verts.as_ptr() as *const u8,
                                verts.len() * std::mem::size_of::<Vertex>(),
                            ),
                            glow::STATIC_DRAW,
                        );
                        gl.clear(glow::COLOR_BUFFER_BIT);
                        gl.draw_arrays(glow::TRIANGLES, 0, 3);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    windowed_context.swap_buffers().unwrap();
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            },
            Event::EventsCleared => {
                #[cfg(target_arch = "wasm32")]
                window.request_redraw();
                #[cfg(not(target_arch = "wasm32"))]
                windowed_context.window().request_redraw();
                *control_flow = ControlFlow::Poll;
            }
            _ => (),
        }
    })
}
