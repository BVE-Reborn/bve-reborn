use crate::*;

pub struct LightHandle(pub(crate) u64);

pub struct PointLight {
    pub location: Vec3,
    pub radius: f32,
    pub strength: f32,
}

pub struct ConeLight {
    pub location: Vec3,
    pub radius: f32,
    pub direction: Vec3,
    pub angle: f32,
    pub strength: f32,
}

pub enum LightDescriptor {
    Point(PointLight),
    Cone(ConeLight),
}

impl Renderer {
    pub fn add_light(&mut self, light_descriptor: LightDescriptor) -> LightHandle {
        let handle = self.light_handle_count;
        self.light_handle_count += 1;
        self.lights.insert(handle, light_descriptor);

        LightHandle(handle)
    }

    pub fn remove_light(&mut self, LightHandle(light_idx): LightHandle) {
        self.lights.remove(&light_idx);
    }
}
