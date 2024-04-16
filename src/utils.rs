use std::path::Path;

use glam::{/*Vec2, Vec3, */Mat4};

use crate::geometry::Mesh;
//clockwise
// pub fn edge_function(v0: Vec2, v1: Vec2, p: Vec2) -> f32 {
//     (p.x - v0.x) * (v1.y - v0.y) - (p.y - v0.y) * (v1.x - v0.x)
// }

// pub fn barycentric_coordinates(
//     point: Vec2,
//     v0: Vec2,
//     v1: Vec2,
//     v2: Vec2,
//     area: f32,
// ) -> Option<Vec3> {
//     let m0 = edge_function(point, v1, v2);
//     let m1 = edge_function(point, v2, v0);
//     let m2 = edge_function(point, v0, v1);
//     // instead of 3 divisions we can do 1/area *
//     let a = 1.0 / area;
//     if m0 >= 0.0 && m1 >= 0.0 && m2 >= 0.0 {
//         Some(glam::vec3(m0 * a, m1 * a, m2 * a))
//     } else {
//         None
//     }
// }

pub fn map_to_range<T>(v: T, a1: T, a2: T, b1: T, b2: T) -> T
where
    T: std::ops::Sub<Output = T>
        + std::ops::Div<Output = T>
        + std::ops::Mul<Output = T>
        + std::ops::Add<Output = T>
        + Copy,
{
    b1 + (v - a1) * (b2 - b1) / (a2 - a1)
}

pub fn index_to_coords(p: usize, width: usize) -> (usize, usize) {
    (p % width, p / width)
}

pub fn coords_to_index(x: usize, y: usize, width: usize) -> usize {
    (x + y * width) as usize
}

pub fn to_argb8(a: u8, r: u8, g: u8, b: u8) -> u32 {
    let mut argb: u32 = a as u32; //a
    argb = (argb << 8) + r as u32; //r
    argb = (argb << 8) + g as u32; //g
    argb = (argb << 8) + b as u32; //b
    argb
}

pub fn to_argb8_v(a: u8, c: glam::Vec3) -> u32 {
    let mut argb: u32 = a as u32; //a
    argb = (argb << 8) + c.x as u32; //r
    argb = (argb << 8) + c.y as u32; //g
    argb = (argb << 8) + c.z as u32; //b
    argb
}

pub fn lerp<T>(start: T, end: T, alpha: f32) -> T
where
    T: std::ops::Sub<Output = T>
        + std::ops::Mul<f32, Output = T>
        + std::ops::Add<Output = T>
        + Copy,
{
    start + (end - start) * alpha
}

pub fn minor(
    src: &[f32; 16],
    r0: usize,
    r1: usize,
    r2: usize,
    c0: usize,
    c1: usize,
    c2: usize,
) -> f32 {
    src[4 * r0 + c0] * (src[4 * r1 + c1] * src[4 * r2 + c2] - src[4 * r2 + c1] * src[4 * r1 + c2])
        - src[4 * r0 + c1]
            * (src[4 * r1 + c0] * src[4 * r2 + c2] - src[4 * r2 + c0] * src[4 * r1 + c2])
        + src[4 * r0 + c2]
            * (src[4 * r1 + c0] * src[4 * r2 + c1] - src[4 * r2 + c0] * src[4 * r1 + c1])
}

pub fn cofactor(matrix: &Mat4) -> Mat4 {
    let src: [f32; 16] = matrix.to_cols_array();
    let mut dst: [f32; 16] = [0.0; 16];
    dst[0] = minor(&src, 1, 2, 3, 1, 2, 3);
    dst[1] = -minor(&src, 1, 2, 3, 0, 2, 3);
    dst[2] = minor(&src, 1, 2, 3, 0, 1, 3);
    dst[3] = -minor(&src, 1, 2, 3, 0, 1, 2);
    dst[4] = -minor(&src, 0, 2, 3, 1, 2, 3);
    dst[5] = minor(&src, 0, 2, 3, 0, 2, 3);
    dst[6] = -minor(&src, 0, 2, 3, 0, 1, 3);
    dst[7] = minor(&src, 0, 2, 3, 0, 1, 2);
    dst[8] = minor(&src, 0, 1, 3, 1, 2, 3);
    dst[9] = -minor(&src, 0, 1, 3, 0, 2, 3);
    dst[10] = minor(&src, 0, 1, 3, 0, 1, 3);
    dst[11] = -minor(&src, 0, 1, 3, 0, 1, 2);
    dst[12] = -minor(&src, 0, 1, 2, 1, 2, 3);
    dst[13] = minor(&src, 0, 1, 2, 0, 2, 3);
    dst[14] = -minor(&src, 0, 1, 2, 0, 1, 3);
    dst[15] = minor(&src, 0, 1, 2, 0, 1, 2);
    Mat4::from_cols_array(&dst)
}

pub fn load_gltf(path: &Path) -> Mesh {
    // handle loading textures, cameras, meshes here
    let (document, buffers, _images) = gltf::import(path).unwrap();

    for scene in document.scenes() {
        for node in scene.nodes() {
            println!(
                "Node #{} has {} children, camera: {:?}, mesh: {:?}, transform: {:?}",
                node.index(),
                node.children().count(),
                node.camera(),
                node.mesh().is_some(),
                node.transform(),
            );
            println!(
                "Node #{} has transform: trans {:?}, rot {:?}, scale {:?},",
                node.index(),
                node.transform().decomposed().0,
                node.transform().decomposed().1,
                node.transform().decomposed().2,
            );
            if let Some(mesh) = node.mesh() {
                return Mesh::load_from_gltf(&mesh, &buffers);
            }
        }
    }

    Mesh::new()
}