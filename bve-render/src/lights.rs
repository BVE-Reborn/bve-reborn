use crate::*;

pub struct LightHandle(pub(crate) u64);

pub struct ConeLight {
    pub direction: Vec3,
    pub angle: f32,
}

pub enum LightType {
    Point,
    Cone(ConeLight),
}

pub struct LightDescriptor {
    pub location: Vec3,
    pub radius: f32,
    pub strength: f32,
    pub ty: LightType,
}

impl Renderer {
    pub fn add_light(&mut self, light_descriptor: LightDescriptor) -> LightHandle {
        let handle = self.light_handle_count;
        self.light_handle_count += 1;
        self.lights.insert(handle, light_descriptor);

        LightHandle(handle)
    }

    pub fn set_light_descriptor(&mut self, LightHandle(light_idx): &LightHandle, light_descriptor: LightDescriptor) {
        self.lights[light_idx] = light_descriptor;
    }

    pub fn remove_light(&mut self, LightHandle(light_idx): LightHandle) {
        self.lights.remove(&light_idx);
    }
}
