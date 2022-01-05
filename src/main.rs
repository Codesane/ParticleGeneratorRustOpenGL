#[macro_use]
extern crate glium;

use std::io::Cursor;
use std::time::Duration;
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

    let image = image::load(Cursor::new(&include_bytes!("./star_09.png")),
                            image::ImageFormat::Png).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

    implement_vertex!(Vertex, position, tex_coords);

    let shape = make_quad(0.01);
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
            gl_Position = vec4(position, 1.0, 1.0) + vec4(world_position, 1.0);
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

    let num_particles = 2000000;

    let mut rng = rand::thread_rng();
    let mut per_instance = {
        #[derive(Copy, Clone)]
        struct Attr {
            world_position: (f32, f32, f32),
            color: (f32, f32, f32, f32),
        }

        implement_vertex!(Attr, world_position, color);

        let data = (0..num_particles).map(|_| {
            let rnd = 0.5 + rng.gen::<f32>() * 0.5;

            let part = 1.0/255.0;
            let color: (f32, f32, f32, f32) = match rng.gen_range(1..4) {
                1 => (part*252.0, part*215.0, part*3.0, 1.0),
                2 => (part*235.0, part*64.0, part*52.0, 1.0),
                3 => (part*235.0, part*198.0, part*52.0, 1.0),
                _ => panic!("Nope"),
            };

            Attr {
                world_position: (0.0, 0.0, 0.0),
                color,
            }
        }).collect::<Vec<_>>();

        glium::vertex::VertexBuffer::dynamic(&display, &data).unwrap()
    };

    let mut animation_swap_time = std::time::Instant::now();
    let mut sign: i32 = 1;



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
            let time = std::time::Instant::now();
            if time - animation_swap_time > Duration::from_secs(1) {
                sign = sign * -1;
                animation_swap_time = std::time::Instant::now();
            }

            let d_angle = std::f32::consts::PI * 2.0 / num_particles as f32;

            let mut rng = rand::thread_rng();
            let mut angle: f32 = 0.0;
            for instance in per_instance.map().iter_mut() {
                let dx = angle.cos() / 10.0;
                let dy = angle.sin() / 10.0;

                let amplitude = rng.gen::<f32>();
                instance.world_position.0 += dx * amplitude * sign as f32;
                instance.world_position.1 += dy * amplitude * sign as f32;

                // let rnd = 0.5 + rng.gen::<f32>() * 0.5;
                //
                // match rng.gen_range(1..4) {
                //     1 => instance.color = (rnd, 0.0, 0.0, 1.0),
                //     2 => instance.color = (0.0, rnd, 0.0, 1.0),
                //     3 => instance.color = (0.0, 0.0, rnd, 1.0),
                //     _ => println!("Unknown"),
                // }

                angle += d_angle;
            }
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        let blend = {
            use glium::Blend;
            use glium::BlendingFunction;
            use glium::LinearBlendingFactor;

            Blend {
                color: BlendingFunction::Addition {
                    source: LinearBlendingFactor::SourceAlpha,
                    destination: LinearBlendingFactor::OneMinusSourceAlpha,
                },
                alpha: BlendingFunction::Addition {
                    source: LinearBlendingFactor::SourceAlpha,
                    destination: LinearBlendingFactor::OneMinusSourceAlpha
                },
                constant_value: (0.0, 0.0, 0.0, 0.0)
            }
        };


        let params = glium::DrawParameters {
            // blend,
            .. Default::default()
        };

        let uniforms = uniform! {
            tex: &texture,
        };

        target.draw(
            (&vertex_buffer, per_instance.per_instance().unwrap()),
            &indices,
            &program,
            &uniforms,
            &params
        ).unwrap();
        target.finish().unwrap();
    });
}