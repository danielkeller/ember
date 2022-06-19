use crate::types::*;

use crate::error::Result;
use crate::instance::Instance;

use super::khr_surface::SurfaceKHR;
use super::load::MetalSurfaceFn;

pub struct EXTMetalSurface {
    fun: MetalSurfaceFn,
    instance: Arc<Instance>,
}

impl Instance {
    /// Creates an [EXTMetalSurface] extension object. Panics if the extension
    /// functions can't be loaded.
    pub fn ext_metal_surface(self: &Arc<Self>) -> EXTMetalSurface {
        EXTMetalSurface {
            fun: MetalSurfaceFn::new(&self),
            instance: self.clone(),
        }
    }
}

impl EXTMetalSurface {
    /// Creates a metal surface. The 'layer' member of
    /// [MetalSurfaceCreateInfoEXT] must refer to a valid Metal layer.
    pub unsafe fn create_metal_surface_ext(
        &self,
        info: &MetalSurfaceCreateInfoEXT,
    ) -> Result<SurfaceKHR> {
        let mut handle = None;
        (self.fun.create_metal_surface_ext)(
            self.instance.handle(),
            info,
            None,
            &mut handle,
        )?;
        Ok(SurfaceKHR::new(handle.unwrap(), self.instance.clone()))
    }
}
