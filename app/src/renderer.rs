use crate::spin::Spin;
use macroquad::prelude::*;

const PADDING: f32 = 0.12;
const ROUNDING: f32 = 0.32;

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        1.0,
    )
}

fn spin_to_color(spin: Spin) -> Color {
    let angle = spin.raw() as u32;

    const QUARTER: u32 = 16384;
    const HALF: u32 = 32768;
    const THREE_QUARTERS: u32 = 49152;
    const FULL: u32 = 65536;

    let c0 = Color::new(0.10, 0.08, 0.12, 1.0);
    let c90 = Color::new(0.95, 0.72, 0.50, 1.0);
    let c180 = Color::new(1.00, 0.55, 0.78, 1.0);
    let c270 = Color::new(0.72, 0.58, 0.95, 1.0);

    let (from, to, position) = if angle < QUARTER {
        (c0, c90, angle)
    } else if angle < HALF {
        (c90, c180, angle - QUARTER)
    } else if angle < THREE_QUARTERS {
        (c180, c270, angle - HALF)
    } else {
        (c270, c0, angle - THREE_QUARTERS)
    };

    let base = lerp_color(
        from,
        to,
        position as f32 / QUARTER as f32,
    );

    let brightness = if angle <= HALF {
        angle as f32 / HALF as f32
    } else {
        (FULL - angle) as f32 / HALF as f32
    };

    Color::new(
        base.r * brightness,
        base.g * brightness,
        base.b * brightness,
        1.0,
    )
}


pub fn update_image(image: &mut Image, spins: &[Spin]) {
    let pixels = image.get_image_data_mut();
    assert_eq!(pixels.len(), spins.len());

    for (pixel, &spin) in pixels.iter_mut().zip(spins.iter()) {
        *pixel = spin_to_color(spin).into();
    }
}

// Shader wierzchołków
const VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying highp vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}
"#;

const FRAGMENT_SHADER: &str = r#"#version 100
precision highp float;

varying vec2 uv;

uniform sampler2D Texture;
uniform vec2 GridSize;
uniform float CellPixels;
uniform float Padding;
uniform float Radius;

void main() {
    vec2 index = floor(uv * GridSize);
    vec2 sample_uv = (index + 0.5) / GridSize;
    vec3 color = texture2D(Texture, sample_uv).rgb;

    if (CellPixels < 4.0) {
        gl_FragColor = vec4(color, 1.0);
        return;
    }

    vec2 p = fract(uv * GridSize) - 0.5;

    float half_size = 0.5 - Padding * 0.5;
    float r = min(Radius, half_size);

    vec2 q = abs(p) - vec2(half_size - r);
    float distance = length(max(q, 0.0))
                   + min(max(q.x, q.y), 0.0) - r;

    float aa = 0.5 / CellPixels;
    float mask = 1.0 - smoothstep(-aa, aa, distance);

    vec3 background = vec3(0.06, 0.06, 0.08);

    gl_FragColor = vec4(mix(background, color, mask), 1.0);
}
"#;

pub fn create_material(width: usize, height: usize) -> Material {
    let material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("GridSize", UniformType::Float2),
                UniformDesc::new("CellPixels", UniformType::Float1),
                UniformDesc::new("Padding", UniformType::Float1),
                UniformDesc::new("Radius", UniformType::Float1),
            ],
            ..Default::default()
        },
    )
    .expect("Nie można skompilować shadera");

    material.set_uniform("GridSize", vec2(width as f32, height as f32));
    material.set_uniform("Padding", PADDING);
    material.set_uniform("Radius", ROUNDING);

    material
}

pub fn render(texture: &Texture2D, material: &Material, width: usize, height: usize, cell_pixels: f32) {
    // Macroquad uses logical coordinates; keep the selected display's physical cell size.
    let cell_size = cell_pixels / screen_dpi_scale();

    let render_width = width as f32 * cell_size;
    let render_height = height as f32 * cell_size;

    let x = (screen_width() - render_width) * 0.5;
    let y = (screen_height() - render_height) * 0.5;

    material.set_uniform("CellPixels", cell_pixels);

    gl_use_material(material);

    draw_texture_ex(
        texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(render_width, render_height)),
            ..Default::default()
        },
    );

    gl_use_default_material();
}
