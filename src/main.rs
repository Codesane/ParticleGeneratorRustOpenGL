#![windows_subsystem = "windows"]

#[macro_use]
extern crate glium;

use std::io::Cursor;
use std::thread;

use glium::glutin::dpi::LogicalSize;
use rand::Rng;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

fn make_quad(size: f32) -> [Vertex; 6] {
    return [
        Vertex { position: [-size,  size], tex_coords: [0.0, 1.0] },
        Vertex { position: [ size, -size], tex_coords: [1.0, 0.0] },
        Vertex { position: [-size, -size], tex_coords: [0.0, 0.0] },
        Vertex { position: [-size,  size], tex_coords: [0.0, 1.0] },
        Vertex { position: [ size, -size], tex_coords: [1.0, 0.0] },
        Vertex { position: [ size,  size], tex_coords: [1.0, 1.0] },
    ]
}

fn main() {
    #[allow(unused_imports)]
    use glium::{glutin, Surface};

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    display.gl_window().window().set_inner_size(LogicalSize {
        width: 900,
        height: 900,
    });

    let image = image::load(Cursor::new(&include_bytes!("../assets/particle.png")),
                            image::ImageFormat::Png).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

    implement_vertex!(Vertex, position, tex_coords);

    let shape = make_quad(0.02);
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        in vec2 tex_coords;
        in vec3 world_position;
        in vec4 color;

        out vec4 v_color;
        out vec2 v_tex_coords;

        void main() {
            v_color = color;
            v_tex_coords = tex_coords;
            float scale = max(1 - sqrt(pow(world_position.x, 2) + pow(world_position.y, 2)), 0.2);
            gl_Position = vec4((position * scale), 1.0, 1.0) + vec4(world_position, 0.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140
        in vec2 v_tex_coords;
        in vec4 v_color;
        out vec4 color;
        uniform sampler2D tex;
        void main() {
            color = texture2D(tex, v_tex_coords) * v_color;
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let num_particles = 40000;

    let mut rng = rand::thread_rng();
    let mut per_instance = {
        #[derive(Copy, Clone)]
        struct Attr {
            world_position: (f32, f32, f32),
            color: (f32, f32, f32, f32),
        }

        implement_vertex!(Attr, world_position, color);

        let data = (0..num_particles).map(|_| {
            let color: (f32, f32, f32, f32) = match rng.gen_range(1..=6) {
                1 => (252.0/255.0, 215.0/255.0, 3.0/255.0, 1.0),
                2 => (235.0/255.0, 64.0/255.0, 52.0/255.0, 1.0),
                3 => (235.0/255.0, 198.0/255.0, 52.0/255.0, 1.0),
                4 => (55.0/255.0, 235.0/255.0, 52.0/255.0, 1.0),
                5 => (235.0/255.0, 52.0/255.0, 195.0/255.0, 1.0),
                6 => (171.0/255.0, 52.0/255.0, 235.0/255.0, 1.0),
                _ => panic!("Invalid random color range"),
            };

            Attr {
                world_position: (0.0, 0.0, 0.0),
                color,
            }
        }).collect::<Vec<_>>();

        glium::vertex::VertexBuffer::dynamic(&display, &data).unwrap()
    };

    let mut rng = rand::thread_rng();
    let velocity = per_instance.map().iter().map(|_| 0.5 + rng.gen_range(1..3) as f32).collect::<Vec<_>>();

    let particles_blend_mode = {
        use glium::{Blend, BlendingFunction, LinearBlendingFactor};

        Blend {
            color: BlendingFunction::Addition {
                source: LinearBlendingFactor::One,
                destination: LinearBlendingFactor::One,
            },
            alpha: BlendingFunction::Addition {
                source: LinearBlendingFactor::One,
                destination: LinearBlendingFactor::One,
            },
            constant_value: (0.0, 0.0, 0.0, 0.0)
        }
    };

    let particles_draw_params = glium::DrawParameters {
        blend: particles_blend_mode,
        .. Default::default()
    };

    let uniforms = uniform! {
        tex: texture,
    };

    event_loop.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let current_time = std::time::Instant::now();
        let next_frame_time = current_time +
            std::time::Duration::from_nanos(16_666_667);

        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        {
            let d_angle = (std::f32::consts::PI * 2.0) / num_particles as f32;

            let mut angle: f32 = 0.0;
            for (index, instance) in per_instance.map().iter_mut().enumerate() {
                let dx = angle.cos() / 100.0;
                let dy = angle.sin() / 100.0;

                let v = velocity[index];

                instance.world_position.0 += dx * v;
                instance.world_position.1 += dy * v;

                let distance = instance.world_position.0.powf(2.0) + instance.world_position.1.powf(2.0);

                if distance > (2.0 + rng.gen_range(0..5) as f32) {
                    instance.world_position.0 = 0.0;
                    instance.world_position.1 = 0.0;
                }

                angle += d_angle;
            }
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        target.draw(
            (&vertex_buffer, per_instance.per_instance().unwrap()),
            &indices,
            &program,
            &uniforms,
            &particles_draw_params
        ).unwrap();
        target.finish().unwrap();
    });
}