use crate::buffer::Buffer;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::ffi::Array;
use crate::image::Image;
use crate::types::*;

use super::CommandRecording;

impl<'a> CommandRecording<'a> {
    /// The reference count of 'dst' is incremented. Offset and size are rounded
    /// down to the nearest multiple of 4. Returns [Error::OutOfBounds] if they
    /// are out of bounds.
    #[doc = crate::man_link!(vkCmdFillBuffer)]
    pub fn fill_buffer(
        &mut self,
        dst: &Arc<Buffer>,
        offset: u64,
        size: Option<u64>,
        data: u32,
    ) -> Result<()> {
        let offset = offset & !3;
        let size = match size {
            Some(size) => {
                if !dst.bounds_check(offset, size) {
                    return Err(Error::OutOfBounds);
                }
                size & !3
            }
            None => {
                if !dst.bounds_check(offset, 0) {
                    return Err(Error::OutOfBounds);
                }
                u64::MAX
            }
        };
        self.add_resource(dst.clone());
        unsafe {
            (self.pool.device.fun.cmd_fill_buffer)(
                self.buffer.handle.borrow_mut(),
                dst.handle(),
                offset,
                size,
                data,
            );
        }
        Ok(())
    }

    /// The reference counts of 'src' and 'dst' are incremented.
    /// Returns [Error::OutOfBounds] if a region is out of bounds.
    #[doc = crate::man_link!(vkCmdCopyBuffer)]
    pub fn copy_buffer(
        &mut self,
        src: &Arc<Buffer>,
        dst: &Arc<Buffer>,
        regions: &[BufferCopy],
    ) -> Result<()> {
        for r in regions {
            if !src.bounds_check(r.src_offset, r.size)
                || !dst.bounds_check(r.dst_offset, r.size)
            {
                return Err(Error::OutOfBounds);
            }
        }
        unsafe {
            (self.pool.device.fun.cmd_copy_buffer)(
                self.buffer.handle.borrow_mut(),
                src.handle(),
                dst.handle(),
                regions.len() as u32,
                Array::from_slice(regions).ok_or(Error::InvalidArgument)?,
            );
        }
        self.add_resource(src.clone());
        self.add_resource(dst.clone());
        Ok(())
    }

    /// The reference counts of 'src' and 'dst' are incremented.
    /// Returns [Error::OutOfBounds] if a region is out of bounds. Returns
    /// [Error::InvalidArgument] if 'regions' is empty.
    #[doc = crate::man_link!(vkCmdCopyBufferToImage)]
    pub fn copy_buffer_to_image(
        &mut self,
        src: &Arc<Buffer>,
        dst: &Arc<Image>,
        dst_layout: ImageLayout,
        regions: &[BufferImageCopy],
    ) -> Result<()> {
        for r in regions {
            let bytes = image_byte_size_3d(dst.format(), r.image_extent)
                .ok_or(Error::OutOfBounds)?
                .checked_mul(r.image_subresource.layer_count as u64)
                .ok_or(Error::OutOfBounds)?;
            if !dst.bounds_check(
                r.image_subresource.mip_level,
                r.image_offset,
                r.image_extent,
            ) || !dst.array_bounds_check(
                r.image_subresource.base_array_layer,
                r.image_subresource.layer_count,
            ) || !src.bounds_check(r.buffer_offset, bytes)
            {
                return Err(Error::OutOfBounds);
            }
        }
        unsafe {
            (self.pool.device.fun.cmd_copy_buffer_to_image)(
                self.buffer.handle.borrow_mut(),
                src.handle(),
                dst.handle(),
                dst_layout,
                regions.len() as u32,
                Array::from_slice(regions).ok_or(Error::InvalidArgument)?,
            );
        }
        self.add_resource(src.clone());
        self.add_resource(dst.clone());
        Ok(())
    }

    /// The reference counts of 'src' and 'dst' are incremented.
    /// Returns [Error::OutOfBounds] if a region is out of bounds. Returns
    /// [Error::InvalidArgument] if 'regions' is empty.
    #[doc = crate::man_link!(vkCmdBlitImage)]
    pub fn blit_image(
        &mut self,
        src: &Arc<Image>,
        src_layout: ImageLayout,
        dst: &Arc<Image>,
        dst_layout: ImageLayout,
        regions: &[ImageBlit],
        filter: Filter,
    ) -> Result<()> {
        for r in regions {
            if !src.array_bounds_check(
                r.src_subresource.base_array_layer,
                r.src_subresource.layer_count,
            ) || !dst.array_bounds_check(
                r.dst_subresource.base_array_layer,
                r.dst_subresource.layer_count,
            ) || !src.offset_bounds_check(
                r.src_subresource.mip_level,
                r.src_offsets[0],
            ) || !src.offset_bounds_check(
                r.src_subresource.mip_level,
                r.src_offsets[1],
            ) || !dst.offset_bounds_check(
                r.dst_subresource.mip_level,
                r.dst_offsets[0],
            ) || !dst.offset_bounds_check(
                r.dst_subresource.mip_level,
                r.dst_offsets[1],
            ) {
                return Err(Error::OutOfBounds);
            }
        }
        unsafe {
            (self.pool.device.fun.cmd_blit_image)(
                self.buffer.handle.borrow_mut(),
                src.handle(),
                src_layout,
                dst.handle(),
                dst_layout,
                regions.len() as u32,
                Array::from_slice(regions).ok_or(Error::InvalidArgument)?,
                filter,
            );
        }
        self.add_resource(src.clone());
        self.add_resource(dst.clone());
        Ok(())
    }

    /// The reference count of 'image' is incremented. Returns
    /// [Error::InvalidArgument] if 'ranges' is empty.
    #[doc = crate::man_link!(vkCmdClearColorImage)]
    pub fn clear_color_image(
        &mut self,
        image: &Arc<Image>,
        layout: ImageLayout,
        color: ClearColorValue,
        ranges: &[ImageSubresourceRange],
    ) -> Result<()> {
        let array = Array::from_slice(ranges).ok_or(Error::InvalidArgument)?;
        unsafe {
            (self.pool.device.fun.cmd_clear_color_image)(
                self.buffer.handle.borrow_mut(),
                image.handle(),
                layout,
                &color,
                ranges.len() as u32,
                array,
            )
        }

        self.add_resource(image.clone());

        Ok(())
    }
}
