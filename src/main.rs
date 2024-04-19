use geometry::Mesh;
use glam::{Vec2, /*Vec3, */Vec4, Mat4};
//use glam::Vec3Swizzles;
use glam::Vec4Swizzles;
use minifb::{Key, Window, WindowOptions, MouseMode};
use std::path::Path;

use std::time::{/*Duration,*/ Instant};
//use std::thread::sleep;

pub mod geometry;
pub use geometry::Vertex;
pub use geometry::Triangle;
pub mod texture;
pub use texture::Texture;
pub mod camera;
pub use camera::Camera;
pub mod transform;
pub use transform::Transform;


pub mod utils;
use utils::{coords_to_index, map_to_range, /*cofactor,*/ load_gltf};


const WIDTH: usize = 500;
const HEIGHT: usize = 500;
const WIDTH_F: f32 = 500.0;
const HEIGHT_F: f32 = 500.0;
//const PI: f32 = 3.14159265359;

/*pub fn find_min(pos0: glam::Vec2, pos1: glam::Vec2, pos2: glam::Vec2) -> (usize, usize) {

    let y_min = pos0.y.min(pos1.y.min(pos2.y));
    let mut x_min = pos0.x.min(pos1.x.min(pos2.x));

    if y_min == pos0.y && y_min == pos1.y{
        x_min = pos0.x.min(pos1.x);
    } else if y_min == pos0.y && y_min == pos2.y{
        x_min = pos0.x.min(pos2.x);
    } else if y_min == pos1.y && y_min == pos2.y{
        x_min = pos1.x.min(pos2.x);
    }

    (x_min as usize, y_min as usize)
}

pub fn find_max(pos0: glam::Vec2, pos1: glam::Vec2, pos2: glam::Vec2) -> (usize, usize) {

    let y_max = pos0.y.max(pos1.y.max(pos2.y));
    let mut x_max = pos0.x.max(pos1.x.max(pos2.x));

    if y_max == pos0.y && y_max == pos1.y{
        x_max = pos0.x.max(pos1.x);
    } else if y_max == pos0.y && y_max == pos2.y {
        x_max = pos0.x.max(pos2.x);
    } else if y_max == pos1.y && y_max == pos2.y {
        x_max = pos1.x.max(pos2.x);
    }

    (x_max as usize, y_max as usize)
}*/

pub fn clear_buffer(buffer: &mut Vec<u32>) {
    let length = buffer.len();
    *buffer = vec![0; length];
}

pub fn clear_z_buffer(buffer: &mut Vec<f32>) {
    let length = buffer.len();
    *buffer = vec![1.0; length];
}

// Area of paralellogram
pub fn get_doubled_triangle_area(v0: glam::Vec2, v1: glam::Vec2, v2: glam::Vec2) -> f32 {
    let area = ((v1.x - v0.x) * (v2.y - v0.y)) - ((v1.y - v0.y) * (v2.x - v0.x));
    area
}

// main function which draws the color of pixels
pub fn draw_pixel(
    buffer: &mut Vec<u32>,
    z_buffer: &mut Vec<f32>,
    index: usize,
    x: f32, y: f32,
    sc0: Vec2,
    sc1: Vec2,
    sc2: Vec2,
    v0: Vertex,
    v1: Vertex,
    v2: Vertex,
    rec0: f32,
    rec1: f32,
    rec2: f32,
    reversed_global_area: f32,
    texture: &Texture)
{
    let p = glam::vec2(x, y);
    let w0 = get_doubled_triangle_area(p,   sc1, sc2) * reversed_global_area;
    let w1 = get_doubled_triangle_area(sc0, p,   sc2) * reversed_global_area;
    let w2 = get_doubled_triangle_area(sc0, sc1, p  ) * reversed_global_area;

    let z = w0 * v0.pos.z + w1 * v1.pos.z + w2 * v2.pos.z;

    let mut _a = 0;

    if z_buffer[index] < z { return; }

    z_buffer[index] = z;

    let correction = w0 * rec0 + w1 * rec1 + w2 * rec2;
    // 1/(1/z) = z
    let correction = 1.0 / correction;

    let mut tex_coords = w0 * v0.uv + w1 * v1.uv + w2 * v2.uv;
    tex_coords *= correction;
    let color = texture.argb_at_uv(tex_coords.x, tex_coords.y);

    buffer[index] = color;
}

pub fn line_from_points(pos0: glam::Vec2, pos1: glam::Vec2) -> (f32, f32, f32) {
    let a = pos1.y - pos0.y;
    let b = pos0.x - pos1.x;
    let c = a * pos1.x + b * pos1.y;

    // ax + by = c

    (a, b, c)
}

pub fn raster_triangle(
    mut v0: Vertex,
    mut v1: Vertex,
    mut v2: Vertex,
    mvp: &Mat4,
    texture: &Texture,
    buffer: &mut Vec<u32>,
    z_buffer: &mut Vec<f32>,
    viewport_size: Vec2,
) {

    let clip0 = *mvp * Vec4::from((v0.pos, 1.0));
    let clip1 = *mvp * Vec4::from((v1.pos, 1.0));
    let clip2 = *mvp * Vec4::from((v2.pos, 1.0));    

    let mut num_of_vertices_behind:i8 = 0;


    if clip0.z < 0.0 { num_of_vertices_behind += 1; }
    if clip1.z < 0.0 { num_of_vertices_behind += 1; }
    if clip2.z < 0.0 { num_of_vertices_behind += 1; }

    if num_of_vertices_behind == 3 {println!("the triangle is outside of clip space"); return;}

    if num_of_vertices_behind == 0
    {
        let rec0 = 1.0 / clip0.w;
        let rec1 = 1.0 / clip1.w;
        let rec2 = 1.0 / clip2.w;

        v0 = v0 * rec0;
        v1 = v1 * rec1;
        v2 = v2 * rec2;

        v0.pos = clip0.xyz() * rec0;
        v1.pos = clip1.xyz() * rec1;
        v2.pos = clip2.xyz() * rec2;

        draw_triangle(
            buffer,
            z_buffer,
            v0,
            v1,
            v2,
            rec0,
            rec1,
            rec2,
            texture,
            viewport_size,
            mvp);
        return;
    }
    else if num_of_vertices_behind == 1
    {
        let     vertex_to_slice: Vertex;
        let     clip_of_slice: Vec4;
        let mut vertex_to_stay0: Vertex;
        let     clip_of_stay0: Vec4;
        let mut vertex_to_stay1: Vertex;
        let     clip_of_stay1: Vec4;

        // Checking which vertex to slice
        if clip0.z < 0.0       { vertex_to_slice = v0; clip_of_slice = clip0; vertex_to_stay0 = v1; clip_of_stay0 = clip1; vertex_to_stay1 = v2; clip_of_stay1 = clip2; }
        else if clip1.z < 0.0  { vertex_to_slice = v1; clip_of_slice = clip1; vertex_to_stay0 = v0; clip_of_stay0 = clip0; vertex_to_stay1 = v2; clip_of_stay1 = clip2; }
        else                   { vertex_to_slice = v2; clip_of_slice = clip2; vertex_to_stay0 = v1; clip_of_stay0 = clip1; vertex_to_stay1 = v0; clip_of_stay1 = clip0; }

        // Calculating new vertices
        let coef0 = clip_of_stay0.z / (clip_of_stay0.z - clip_of_slice.z);
        let coef1 = clip_of_stay1.z / (clip_of_stay1.z - clip_of_slice.z);

        let mut new_vertex0 = vertex_to_stay0 + (vertex_to_slice - vertex_to_stay0) * coef0;
        let mut new_vertex1 = vertex_to_stay1 + (vertex_to_slice - vertex_to_stay1) * coef1;

        let new_clip0 = *mvp * Vec4::from((new_vertex0.pos, 1.0));
        let new_clip1 = *mvp * Vec4::from((new_vertex1.pos, 1.0));


        // Deviding by homogenyous coordinates
        let rec_stay0 = 1.0 / clip_of_stay0.w;
        vertex_to_stay0 = vertex_to_stay0 * rec_stay0;
        vertex_to_stay0.pos = clip_of_stay0.xyz() * rec_stay0;

        let rec_stay1 = 1.0 / clip_of_stay1.w;
        vertex_to_stay1 = vertex_to_stay1 * rec_stay1;
        vertex_to_stay1.pos = clip_of_stay1.xyz() * rec_stay1;

        let rec_new0 = 1.0 / new_clip0.w;
        new_vertex0 = new_vertex0 * rec_new0;
        new_vertex0.pos = new_clip0.xyz() * rec_new0;

        let rec_new1 = 1.0 / new_clip1.w;
        new_vertex1 = new_vertex1 * rec_new1;
        new_vertex1.pos = new_clip1.xyz() * rec_new1;


        draw_triangle(
            buffer,
            z_buffer,
            new_vertex0,
            vertex_to_stay0,
            new_vertex1,
            rec_new0,
            rec_stay0,
            rec_new1,
            texture,
            viewport_size,
            mvp);

        draw_triangle(
            buffer,
            z_buffer,
            vertex_to_stay1,
            vertex_to_stay0,
            new_vertex1,
            rec_stay1,
            rec_stay0,
            rec_new1,
            texture,
            viewport_size,
            mvp);
    }
    else
    {
        let mut vertex_to_stay: Vertex;
        let     clip_of_stay: Vec4;
        let     vertex_to_slice0: Vertex;
        let     clip_of_slice0: Vec4;
        let     vertex_to_slice1: Vertex;
        let     clip_of_slice1: Vec4;

        // Checking which vertex is inside our clip space
        if clip0.z > 0.0       { vertex_to_stay = v0; clip_of_stay = clip0; vertex_to_slice0 = v1; clip_of_slice0 = clip1; vertex_to_slice1 = v2; clip_of_slice1 = clip2; }
        else if clip1.z > 0.0  { vertex_to_stay = v1; clip_of_stay = clip1; vertex_to_slice0 = v0; clip_of_slice0 = clip0; vertex_to_slice1 = v2; clip_of_slice1 = clip2; }
        else                   { vertex_to_stay = v2; clip_of_stay = clip2; vertex_to_slice0 = v1; clip_of_slice0 = clip1; vertex_to_slice1 = v0; clip_of_slice1 = clip0; }

        // Calculating new vertices
        let coef0 = clip_of_stay.z / (clip_of_stay.z - clip_of_slice0.z);
        let coef1 = clip_of_stay.z / (clip_of_stay.z - clip_of_slice1.z);

        let mut new_vertex0 = vertex_to_stay + (vertex_to_slice0 - vertex_to_stay) * coef0;
        let mut new_vertex1 = vertex_to_stay + (vertex_to_slice1 - vertex_to_stay) * coef1;

        let new_clip0 = *mvp * Vec4::from((new_vertex0.pos, 1.0));
        let new_clip1 = *mvp * Vec4::from((new_vertex1.pos, 1.0));


        // Deviding by homogenyous coordinates
        let rec_stay0 = 1.0 / clip_of_stay.w;
        vertex_to_stay = vertex_to_stay * rec_stay0;
        vertex_to_stay.pos = clip_of_stay.xyz() * rec_stay0;

        let rec_new0 = 1.0 / new_clip0.w;
        new_vertex0 = new_vertex0 * rec_new0;
        new_vertex0.pos = new_clip0.xyz() * rec_new0;

        let rec_new1 = 1.0 / new_clip1.w;
        new_vertex1 = new_vertex1 * rec_new1;
        new_vertex1.pos = new_clip1.xyz() * rec_new1;


        draw_triangle(
            buffer,
            z_buffer,
            new_vertex0,
            vertex_to_stay,
            new_vertex1,
            rec_new0,
            rec_stay0,
            rec_new1,
            texture,
            viewport_size,
            mvp);
    }
}

pub fn raster_mesh(
    mesh: &Mesh,
    _model: &Mat4,
    mvp: &Mat4,
    texture: &Texture,
    buffer: &mut Vec<u32>,
    z_buffer: &mut Vec<f32>,
    viewport_size: Vec2,
) {
    for triangle in mesh.triangles() {
        let vertices = mesh.get_vertices_from_triangle(*triangle);
        raster_triangle(
            *vertices[0],
            *vertices[1],
            *vertices[2],
            mvp,
            texture,
            buffer,
            z_buffer,
            viewport_size,
        );
    }
}

pub fn draw_triangle(
    buffer: &mut Vec<u32>,
    z_buffer: &mut Vec<f32>,
    clipped_v0: Vertex,
    clipped_v1: Vertex,
    clipped_v2: Vertex,
    rec0: f32,
    rec1: f32,
    rec2: f32,
    texture: &Texture,
    viewport_size: glam::Vec2,
    _mvp: &glam::Mat4, )
{
    // let rec0 = 1.0 / clip0.w;
    // let rec1 = 1.0 / clip1.w;
    // let rec2 = 1.0 / clip2.w;

    // v0 = v0 * rec0;
    // v1 = v1 * rec1;
    // v2 = v2 * rec2;

    //if clip0.z < 0.0 && clip1.z < 0.0 && clip2.z < 0.0 { return; }

    // This would be the output of the vertex shader (clip space)
    // then we perform perspective division to transform in ndc
    // now x,y,z componend of ndc are between -1 and 1
    // perspective division on all attributes
    

    //println!("{}, {}, {}", clipped_v0.pos, clipped_v1.pos, clipped_v2.pos);

    // screeen coordinates remapped to window
    let sc0 = glam::vec2(
        map_to_range(clipped_v0.pos.x, -1.0, 1.0, 0.0, viewport_size.x),
        map_to_range(-clipped_v0.pos.y, -1.0, 1.0, 0.0, viewport_size.y),
    );
    let sc1 = glam::vec2(
        map_to_range(clipped_v1.pos.x, -1.0, 1.0, 0.0, viewport_size.x),
        map_to_range(-clipped_v1.pos.y, -1.0, 1.0, 0.0, viewport_size.y),
    );
    let sc2 = glam::vec2(
        map_to_range(clipped_v2.pos.x, -1.0, 1.0, 0.0, viewport_size.x),
        map_to_range(-clipped_v2.pos.y, -1.0, 1.0, 0.0, viewport_size.y),
    );
    
    let dx_0 = (sc0.x - sc1.x).abs();
    let dx_1 = (sc2.x - sc1.x).abs();
    let dx_2 = (sc0.x - sc2.x).abs();

    let longest_dx = dx_0.max(dx_1.max(dx_2));
    let baseline: (f32, f32, f32);
    let pivot_line0: (f32, f32, f32);
    let pivot_line1: (f32, f32, f32);
    let pivot_point: glam::Vec2;

    if longest_dx == dx_0 {
        baseline = line_from_points(sc0, sc1);
        pivot_point = sc2;
        pivot_line0 = line_from_points(sc2, sc1);
        pivot_line1 = line_from_points(sc2, sc0);
    }
    else if longest_dx == dx_1 {
        baseline = line_from_points(sc2, sc1);
        pivot_point = sc0;
        pivot_line0 = line_from_points(sc0, sc1);
        pivot_line1 = line_from_points(sc2, sc0);
    }
    else {
        baseline = line_from_points(sc0, sc2);
        pivot_point = sc1;
        pivot_line0 = line_from_points(sc2, sc1);
        pivot_line1 = line_from_points(sc1, sc0);
    }

    let a = baseline.0;
    let b = baseline.1;
    let c = baseline.2;

    if a * pivot_point.x + b * pivot_point.y == c {
        return;
    }

    let min_x = (sc0.x.min(sc1.x.min(sc2.x)) as usize).max(0);
    let max_x = (sc0.x.max(sc1.x.max(sc2.x)) as usize).min(WIDTH-1);


    let reversed_global_area = 1.0 / get_doubled_triangle_area(sc0, sc1, sc2);


    if  sc0.y < 0.0            || sc1.y < 0.0            || sc2.y < 0.0            ||
        sc0.y > HEIGHT_F - 1.0 || sc1.y > HEIGHT_F - 1.0 || sc2.y > HEIGHT_F - 1.0
    {
        if (a*pivot_point.x + b*pivot_point.y - c).signum() != b.signum() // pivot point below the line
        {
            for x_usize in min_x..max_x
            {
                let x_f32 = x_usize as f32 - 0.5;
                let mut y_f32 = find_maximal_y(pivot_line0, pivot_line1, x_f32).max(0.0);
                
                y_f32 -= 0.5;
                y_f32 = y_f32.round();

                let mut y_usize = y_f32 as usize;

                let temporal_value = a*x_f32 - c;
                while (b*y_f32 + temporal_value).signum() != b.signum() && (b*y_f32 + temporal_value != 0.0) && y_usize < HEIGHT{
                    let index = coords_to_index(x_usize, y_usize, WIDTH);
                    let mut _a = 0;
                    if index >= WIDTH * HEIGHT
                    {
                        _a = 1;
                    }
                    draw_pixel(buffer, z_buffer, index, x_f32, y_f32, sc0, sc1, sc2, clipped_v0, clipped_v1, clipped_v2, rec0, rec1, rec2, reversed_global_area, texture);
                    y_f32 += 1.0;
                    y_usize += 1;
                }
            }
        }
        else                                                              // pivot point above the line
        {
            for x_usize in min_x..max_x
            {
                let x_f32 = x_usize as f32 - 0.5;
                let mut y_f32 = find_minimal_y(pivot_line0, pivot_line1, x_f32).min(HEIGHT_F - 1.0);


                let mut y_usize = y_f32 as usize;

                y_f32 -= 0.5;
                y_f32 = y_f32.round();

                let temporal_value = a*x_f32 - c;

                while (b*y_f32 + temporal_value).signum() == b.signum() && (b*y_f32 + temporal_value != 0.0) && y_usize > 0{
                    let index = coords_to_index(x_usize, y_usize, WIDTH);
                    let mut _a = 0;
                    if index >= WIDTH * HEIGHT
                    {
                        _a = 1;
                    }
                    draw_pixel(buffer, z_buffer, index, x_f32, y_f32, sc0, sc1, sc2, clipped_v0, clipped_v1, clipped_v2, rec0, rec1, rec2, reversed_global_area, texture);
                    y_f32 -= 1.0;
                    y_usize -= 1;
                }
            }
        }
    }
    else
    {
        if (a*pivot_point.x + b*pivot_point.y - c).signum() != b.signum() // pivot point below the line
        {
            for x_usize in min_x..max_x
            {
                let x_f32 = x_usize as f32 - 0.5;
                let mut y_f32 = find_maximal_y(pivot_line0, pivot_line1, x_f32);

                
                y_f32 -= 0.500001;
                y_f32 = y_f32.round();

                let mut y_usize = y_f32 as usize;

                let temporal_value = a*x_f32 - c;
                while (b*y_f32 + temporal_value).signum() != b.signum() && (b*y_f32 + temporal_value != 0.0) && y_usize < HEIGHT{
                    let index = coords_to_index(x_usize, y_usize, WIDTH);
                    let mut _a = 0;
                    if index >= WIDTH * HEIGHT
                    {
                        _a = 1;
                    }
                    draw_pixel(buffer, z_buffer, index, x_f32, y_f32, sc0, sc1, sc2, clipped_v0, clipped_v1, clipped_v2, rec0, rec1, rec2, reversed_global_area, texture);
                    y_f32 += 1.0;
                    y_usize += 1;
                }
            }
        }
        else                                                              // pivot point above the line
        {
            for x_usize in min_x..max_x
            {
                let x_f32 = x_usize as f32 - 0.5;
                let mut y_f32 = find_minimal_y(pivot_line0, pivot_line1, x_f32);

                let mut y_usize = y_f32 as usize;

                y_f32 -= 0.5;
                y_f32 = y_f32.round();
                

                let temporal_value = a*x_f32 - c;

                while (b*y_f32 + temporal_value).signum() == b.signum() && (b*y_f32 + temporal_value != 0.0) && y_usize > 0{
                    let index = coords_to_index(x_usize, y_usize, WIDTH);
                    let mut _a = 0;
                    if index >= WIDTH * HEIGHT
                    {
                        _a = 1;
                    }
                    draw_pixel(buffer, z_buffer, index, x_f32, y_f32, sc0, sc1, sc2, clipped_v0, clipped_v1, clipped_v2, rec0, rec1, rec2, reversed_global_area, texture);
                    y_f32 -= 1.0;
                    y_usize -= 1;
                }
            }
        }
    }
}

pub fn find_minimal_y(line0: (f32, f32, f32), line1: (f32, f32, f32), x: f32) -> f32 {
    if line0.1 == 0.0
    {
        (line1.0 * x - line1.2) / -line1.1
    }
    else if line1.1 == 0.0
    {
        (line0.0 * x - line0.2) / -line0.1
    }
    else
    {
        let y0 = (line0.0 * x - line0.2) / -line0.1;
        let y1 = (line1.0 * x - line1.2) / -line1.1;

        y0.min(y1)
    }
}

pub fn find_maximal_y(line0: (f32, f32, f32), line1: (f32, f32, f32), x: f32) -> f32 {
    if line0.1 == 0.0
    {
        (line1.0 * x - line1.2) / -line1.1
    }
    else if line1.1 == 0.0
    {
        (line0.0 * x - line0.2) / -line0.1
    }
    else
    {
        let y0 = (line0.0 * x - line0.2) / -line0.1;
        let y1 = (line1.0 * x - line1.2) / -line1.1;

        y0.max(y1)
    }
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut z_buffer: Vec<f32> = vec![1.0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("Window failed to load.\nCaused error: {}", e);
    });

    // let v0 = Vertex {
    //     pos: glam::vec3(-1.0, 1.0, 0.0),
    //     normal: glam::vec3(0.0, 1.0, 0.0),
    //     c: glam::vec3(255.0, 235.0, 59.0),
    //     uv: glam::vec2(1.0, 0.0),
    // };
    // let v1 = Vertex {
    //     pos: glam::vec3(1.0, 1.0, 0.0),
    //     normal: glam::vec3(0.0, 1.0, 0.0),
    //     c: glam::vec3(0.0, 255.0, 0.0),
    //     uv: glam::vec2(0.0, 0.0)
    // };
    // let v2 = Vertex {
    //     pos: glam::vec3(-1.0, -1.0, 0.0),
    //     normal: glam::vec3(0.0, 1.0, 0.0),
    //     c: glam::vec3(0.0, 0.0, 255.0),
    //     uv: glam::vec2(1.0, 1.0)
    // };
    // let v3 = Vertex {
    //     pos: glam::vec3(1.0, -1.0, 0.0),
    //     normal: glam::vec3(0.0, 1.0, 0.0),
    //     c: glam::vec3(0.0, 0.0, 255.0),
    //     uv: glam::vec2(0.0, 1.0)
    // };

    let aspect_ratio = WIDTH_F / HEIGHT_F;

    let mut camera = Camera {
        aspect_ratio,
        transform: Transform::from_translation(glam::vec3(0.0, 0.0, 3.0)),
        frustum_far: 100.0,
        ..Default::default()
    };

    let mesh = load_gltf(Path::new("assets/helmet.gltf"));
    //let mesh = load_gltf(Path::new("assets/cube.gltf"));

    let transform_of_go = Transform::from_rotation(glam::Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0));

    //camera.transform.translation = glam::vec3(1.0, 1.0, 1.0);

    let texture = Texture::load(Path::new("assets/bojan.jpg"));
    //let texture = Texture::load(Path::new("assets/albedo.jpg"));

    let window_size = glam::vec2(WIDTH as f32, HEIGHT as f32);

    let mut mouse_pos = (WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
    
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut now = Instant::now();
    
    while window.is_open() && !window.is_key_down(Key::Escape) {

        let dt = now.elapsed().as_millis() as f32 / 1000.0;
        now = Instant::now();

        handle_camera(&mut camera, &window, &mut mouse_pos, dt);
        clear_buffer(&mut buffer);
        clear_z_buffer(&mut z_buffer);
        // raster_triangle(
        //     v0, v1, v2, &(camera.projection() * camera.view() * transform_of_go.local()), &texture, &mut buffer, window_size
        // );
        // raster_triangle(
        //     v1, v2, v3, &(camera.projection() * camera.view() * transform_of_go.local()), &texture, &mut buffer, window_size
        // );

        raster_mesh(
            &mesh,
            &transform_of_go.local(),
            &(camera.projection() * camera.view() * transform_of_go.local()),
            &texture,
            &mut buffer,
            &mut z_buffer,
            window_size,
        );

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

pub fn handle_camera(camera: &mut Camera, window: &Window, mouse_pos: &mut (f32, f32), dt: f32) {

    let mut axis = glam::vec2(0.0, 0.0);
    // we will make registering later

    if window.is_key_down(Key::A) {
        axis.x -= 1.0;
    }
    if window.is_key_down(Key::D) {
        axis.x += 1.0;
    }
    if window.is_key_down(Key::W) {
        axis.y += 1.0;
    }
    if window.is_key_down(Key::S) {
        axis.y -= 1.0;
    }


    camera.transform.translation += camera.transform.right() * camera.speed * axis.x * dt
                                + camera.transform.forward() * camera.speed * axis.y * dt;

    if window.is_key_down(Key::Space) { // Move up
        camera.transform.translation.y += camera.speed * dt;
    }

    if window.is_key_down(Key::LeftShift) { // Move down
        camera.transform.translation.y -= camera.speed * dt;
    }

    let new_mouse_pos = window.get_mouse_pos(MouseMode::Clamp).unwrap();

    let rot_x = camera.sens * (mouse_pos.0 - new_mouse_pos.0);
    let rot_y = camera.sens * (mouse_pos.1 - new_mouse_pos.1);

    *mouse_pos = new_mouse_pos;

    let cam_rotation = glam::Quat::to_euler(camera.transform.rotation, glam::EulerRot::XYZ);
    camera.transform.rotation = Transform::from_rotation(glam::Quat::from_euler(glam::EulerRot::XYZ, cam_rotation.0 + (rot_y / 360.0 * camera.sens), cam_rotation.1 + (rot_x / 360.0 * camera.sens), cam_rotation.2)).rotation;
}