use crate::runtime::{ChunkAddress, Location};
use glam::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct ConeLight {
    pub direction: Vec3,
    pub angle: f32,
}

#[derive(Debug, Copy, Clone)]
pub enum LightType {
    Point,
    Cone(ConeLight),
}

#[derive(Debug, Copy, Clone)]
pub struct LightDescriptor {
    pub location: Location,
    pub radius: f32,
    pub strength: f32,
    pub ty: LightType,
}

impl LightDescriptor {
    #[must_use]
    pub fn into_render_light_descriptor(self, base_chunk: ChunkAddress) -> RenderLightDescriptor {
        RenderLightDescriptor {
            location: Vec3::from(*AsRef::<[f32; 3]>::as_ref(
                &self.location.to_relative_position(base_chunk),
            )),
            radius: self.radius,
            strength: self.strength,
            ty: self.ty,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RenderLightDescriptor {
    pub location: Vec3,
    pub radius: f32,
    pub strength: f32,
    pub ty: LightType,
}
