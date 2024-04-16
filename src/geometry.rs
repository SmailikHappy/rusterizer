use glam::{Vec2, Vec3, Mat4, Vec4Swizzles, UVec3};
use std::ops::{Add, Mul, Sub};



#[derive(Debug, Clone)]
pub struct Mesh {
    triangles: Vec<UVec3>,
    vertices: Vec<Vertex>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            triangles: Vec::new(),
            vertices: Vec::new(),
        }
    }

    pub fn triangles(&self) -> &Vec<UVec3> {
        &self.triangles
    }

    pub fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn get_vertices_from_triangle(&self, triangle: UVec3) -> [&Vertex; 3] {
        [
            &self.vertices[triangle.x as usize],
            &self.vertices[triangle.y as usize],
            &self.vertices[triangle.z as usize],
        ]
    }

    pub fn from_vertices(triangles: &[UVec3], vertices: &[Vertex]) -> Self {
        let mut mesh = Mesh::new();
        mesh.add_section_from_vertices(triangles, vertices);
        mesh
    }

    // we can also do it with slices
    pub fn add_section_from_vertices(&mut self, triangles: &[UVec3], vertices: &[Vertex]) {
        let offset = self.vertices.len() as u32;
        let triangles: Vec<UVec3> = triangles.iter().map(|tri| *tri + offset).collect();
        self.triangles.extend_from_slice(&triangles);
        self.vertices.extend_from_slice(vertices);
    }

    pub fn add_section_from_buffers(
        &mut self,
        triangles: &[UVec3],
        positions: &[Vec3],
        normals: &[Vec3],
        colors: &[Vec3],
        uvs: &[Vec2],
    ) {
        self.triangles.extend_from_slice(triangles);

        let has_uvs = !uvs.is_empty();
        let has_colors = !colors.is_empty();

        for i in 0..positions.len() {
            let vertex = Vertex::new(
                positions[i],
                normals[i],
                if has_colors { colors[i] } else { Vec3::ONE },
                if has_uvs { uvs[i] } else { Vec2::ZERO },
            );
            self.vertices.push(vertex)
        }
    }

    pub fn load_from_gltf(mesh: &gltf::Mesh, buffers: &[gltf::buffer::Data]) -> Mesh {
        let mut positions: Vec<Vec3> = Vec::new();
        let mut tex_coords: Vec<Vec2> = Vec::new();
        let mut normals: Vec<Vec3> = Vec::new();
        let mut indices = vec![];
        // TODO: handle errors
        let mut result = Mesh::new();
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            if let Some(indices_reader) = reader.read_indices() {
                indices_reader.into_u32().for_each(|i| indices.push(i));
            }
            if let Some(positions_reader) = reader.read_positions() {
                positions_reader.for_each(|p| positions.push(Vec3::new(p[0], p[1], p[2])));
            }
            if let Some(normals_reader) = reader.read_normals() {
                normals_reader.for_each(|n| normals.push(Vec3::new(n[0], n[1], n[2])));
            }
            if let Some(tex_coord_reader) = reader.read_tex_coords(0) {
                tex_coord_reader
                    .into_f32()
                    .for_each(|tc| tex_coords.push(Vec2::new(tc[0], tc[1])));
            }

            let colors: Vec<Vec3> = positions.iter().map(|_| Vec3::ONE).collect();
            println!("Num indices: {:?}", indices.len());
            println!("tex_coords: {:?}", tex_coords.len());
            println!("positions: {:?}", positions.len());

            let triangles: Vec<UVec3> = indices
                .chunks_exact(3)
                .map(|tri| UVec3::new(tri[0], tri[1], tri[2]))
                .collect();
            result.add_section_from_buffers(&triangles, &positions, &normals, &colors, &tex_coords)
        }
        result
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    pub v0: Vertex,
    pub v1: Vertex,
    pub v2: Vertex,
}

impl Triangle {
    pub fn new(v0: Vertex, v1: Vertex, v2: Vertex) -> Self {
        Self { v0, v1, v2 }
    }

    pub fn transform(&self, matrix: &Mat4) -> Self {
        let p0 = *matrix * self.v0.pos.extend(1.0);
        let p1 = *matrix * self.v1.pos.extend(1.0);
        let p2 = *matrix * self.v2.pos.extend(1.0);

        let mut result = *self;

        result.v0.pos = p0.xyz();
        result.v1.pos = p1.xyz();
        result.v2.pos = p2.xyz();

        result
    }

    // pub fn reorder(&self, order: VerticesOrder) -> Self {
    //     match order {
    //         VerticesOrder::ABC => *self,
    //         VerticesOrder::ACB => Self::new(self.v0, self.v2, self.v1),
    //         VerticesOrder::BAC => Self::new(self.v1, self.v0, self.v2),
    //         VerticesOrder::BCA => Self::new(self.v1, self.v2, self.v0),
    //         VerticesOrder::CAB => Self::new(self.v2, self.v0, self.v1),
    //         VerticesOrder::CBA => Self::new(self.v2, self.v1, self.v0),
    //     }
    // }
}

#[derive(Debug, Copy, Clone)]

pub struct Vertex {
    pub pos: Vec3,
    pub normal: Vec3,
    pub c: Vec3,
    pub uv: Vec2,
}

impl Vertex {
    pub fn new(pos: Vec3, normal: Vec3, c: Vec3, uv: Vec2) -> Self {
        Self {
            pos,
            normal,
            c,
            uv,
        }
    }
}

impl Add for Vertex {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let pos = self.pos + rhs.pos;
        let normal = self.normal + rhs.normal;
        let c = self.c + rhs.c;
        let uv = self.uv + rhs.uv;
        Self::new(pos, normal, c, uv)
    }
}

impl Sub for Vertex {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let pos = self.pos - rhs.pos;
        let normal = self.normal - rhs.normal;
        let c = self.c - rhs.c;
        let uv = self.uv - rhs.uv;
        Self::new(pos, normal, c, uv)
    }
}

impl Mul<f32> for Vertex {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        let pos = self.pos * rhs;
        let normal = self.normal * rhs;
        let c = self.c * rhs;
        let uv = self.uv * rhs;
        Self::new(pos, normal, c, uv)
    }
}
