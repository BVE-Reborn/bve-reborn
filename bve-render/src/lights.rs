use crate::*;
use bve::runtime::RenderLightDescriptor;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LightHandle(pub(crate) DefaultKey);

impl Renderer {
    pub fn add_light(&mut self, light_descriptor: RenderLightDescriptor) -> LightHandle {
        let handle = self.lights.insert(light_descriptor);

        LightHandle(handle)
    }

    pub fn set_light_descriptor(
        &mut self,
        LightHandle(light_idx): &LightHandle,
        light_descriptor: RenderLightDescriptor,
    ) {
        self.lights[*light_idx] = light_descriptor;
    }

    pub fn remove_light(&mut self, LightHandle(light_idx): &LightHandle) {
        self.lights.remove(*light_idx);
    }
}
