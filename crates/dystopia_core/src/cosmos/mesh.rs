use bevy::{
    asset::Asset,
    color::{ColorToComponents, LinearRgba},
    math::Vec3,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::Material2d,
};

#[derive(ShaderType, Debug, Clone, Copy)]
pub struct StarMaterialUniform {
    pub color: Vec3,
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Default, Clone, Copy)]
#[uniform(0, StarMaterialUniform)]
pub struct StarMaterial {
    pub color: LinearRgba,
}

impl From<&StarMaterial> for StarMaterialUniform {
    fn from(value: &StarMaterial) -> Self {
        Self {
            color: value.color.to_vec3(),
        }
    }
}

impl Material2d for StarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bodies/star.wgsl".into()
    }
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub struct RockyBodyMaterialUniform {
    pub color: Vec3,
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Default, Clone, Copy)]
#[uniform(0, RockyBodyMaterialUniform)]
pub struct RockyBodyMaterial {
    pub color: LinearRgba,
}

impl From<&RockyBodyMaterial> for RockyBodyMaterialUniform {
    fn from(value: &RockyBodyMaterial) -> Self {
        Self {
            color: value.color.to_vec3(),
        }
    }
}

impl Material2d for RockyBodyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bodies/rocky_body.wgsl".into()
    }
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub struct GiantBodyMaterialUniform {
    pub color: Vec3,
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Default, Clone, Copy)]
#[uniform(0, GiantBodyMaterialUniform)]
pub struct GiantBodyMaterial {
    pub color: LinearRgba,
}

impl From<&GiantBodyMaterial> for GiantBodyMaterialUniform {
    fn from(value: &GiantBodyMaterial) -> Self {
        Self {
            color: value.color.to_vec3(),
        }
    }
}

impl Material2d for GiantBodyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bodies/giant_body.wgsl".into()
    }
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub struct OrbitMaterialUniform {
    pub color: Vec3,
    pub width: f32,
    pub radius: f32,
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Default, Clone, Copy)]
#[uniform(0, OrbitMaterialUniform)]
pub struct OrbitMaterial {
    pub color: LinearRgba,
    pub width: f32,
    pub radius: f32,
}

impl From<&OrbitMaterial> for OrbitMaterialUniform {
    fn from(value: &OrbitMaterial) -> Self {
        Self {
            color: value.color.to_vec3(),
            width: value.width,
            radius: value.radius,
        }
    }
}

impl Material2d for OrbitMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bodies/orbit.wgsl".into()
    }
}
