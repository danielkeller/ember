// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::buffer::Buffer;
use crate::enums::*;
use crate::ffi::Array;
use crate::image::Image;
use crate::types::*;

use super::CommandRecording;

impl<'rec, 'pool> CommandRecording<'rec, 'pool> {
    /// The reference count of `dst` is incremented. Offset and size are rounded
    /// down to the nearest multiple of 4. Returns [`Error::OutOfBounds`] if they
    /// are out of bounds.
    #[doc = crate::man_link!(vkCmdFillBuffer)]
    pub fn fill_buffer(
        &mut self, dst: &'pool Buffer, offset: u64, size: Option<u64>,
        data: u32,
    ) {
        let offset = offset & !3;
        let size = match size {
            Some(size) => {
                if !dst.bounds_check(offset, size) {
                    panic!("Buffer offset out of bounds");
                }
                size & !3
            }
            None => {
                if !dst.bounds_check(offset, 0) {
                    panic!("Buffer offset out of bounds");
                }
                u64::MAX
            }
        };
        unsafe {
            (self.device.fun().cmd_fill_buffer)(
                self.buffer.handle_mut(),
                dst.handle(),
                offset,
                size,
                data,
            );
        }
    }

    /// The reference counts of `src` and `dst` are incremented.
    /// Returns [`Error::OutOfBounds`] if a region is out of bounds.
    #[doc = crate::man_link!(vkCmdCopyBuffer)]
    pub fn copy_buffer(
        &mut self, src: &'pool Buffer, dst: &'pool Buffer,
        regions: &[BufferCopy],
    ) {
        for r in regions {
            if !src.bounds_check(r.src_offset, r.size)
                || !dst.bounds_check(r.dst_offset, r.size)
            {
                panic!("Buffer region out of bounds")
            }
        }
        unsafe {
            (self.device.fun().cmd_copy_buffer)(
                self.buffer.handle_mut(),
                src.handle(),
                dst.handle(),
                regions.len() as u32,
                Array::from_slice(regions).expect("Buffer regions empty"),
            );
        }
    }

    /// The reference counts of `src` and `dst` are incremented.
    /// Returns [`Error::OutOfBounds`] if a region is out of bounds. Returns
    /// [`Error::InvalidArgument`] if `regions` is empty.
    #[doc = crate::man_link!(vkCmdCopyBufferToImage)]
    pub fn copy_buffer_to_image(
        &mut self, src: &'pool Buffer, dst: &'pool Image,
        dst_layout: ImageLayout, regions: &[BufferImageCopy],
    ) {
        for r in regions {
            let bytes = image_byte_size_3d(dst.format(), r.image_extent)
                .expect("Image region size overflows")
                .checked_mul(r.image_subresource.layer_count as u64)
                .expect("Image region size overflows");
            if !dst.bounds_check(
                r.image_subresource.mip_level,
                r.image_offset,
                r.image_extent,
            ) || !dst.array_bounds_check(
                r.image_subresource.base_array_layer,
                r.image_subresource.layer_count,
            ) || !src.bounds_check(r.buffer_offset, bytes)
            {
                panic!("Image region out of bounds")
            }
        }
        unsafe {
            (self.device.fun().cmd_copy_buffer_to_image)(
                self.buffer.handle_mut(),
                src.handle(),
                dst.handle(),
                dst_layout,
                regions.len() as u32,
                Array::from_slice(regions).expect("Image regions empty"),
            );
        }
    }

    /// The reference counts of `src` and `dst` are incremented.
    /// Returns [`Error::OutOfBounds`] if a region is out of bounds. Returns
    /// [`Error::InvalidArgument`] if `regions` is empty.
    #[doc = crate::man_link!(vkCmdBlitImage)]
    pub fn blit_image(
        &mut self, src: &'pool Image, src_layout: ImageLayout,
        dst: &'pool Image, dst_layout: ImageLayout, regions: &[ImageBlit],
        filter: Filter,
    ) {
        for r in regions {
            if !src.array_bounds_check(
                r.src_subresource.base_array_layer,
                r.src_subresource.layer_count,
            ) || !src.offset_bounds_check(
                r.src_subresource.mip_level,
                r.src_offsets[0],
            ) || !src.offset_bounds_check(
                r.src_subresource.mip_level,
                r.src_offsets[1],
            ) {
                panic!("Region out of bounds of source image")
            }
            if !dst.array_bounds_check(
                r.dst_subresource.base_array_layer,
                r.dst_subresource.layer_count,
            ) || !dst.offset_bounds_check(
                r.dst_subresource.mip_level,
                r.dst_offsets[0],
            ) || !dst.offset_bounds_check(
                r.dst_subresource.mip_level,
                r.dst_offsets[1],
            ) {
                panic!("Region out of bounds of destination image")
            }
        }
        unsafe {
            (self.device.fun().cmd_blit_image)(
                self.buffer.handle_mut(),
                src.handle(),
                src_layout,
                dst.handle(),
                dst_layout,
                regions.len() as u32,
                Array::from_slice(regions).expect("No blit regions"),
                filter,
            );
        }
    }

    /// The reference count of `image` is incremented. Returns
    /// [`Error::InvalidArgument`] if `ranges` is empty.
    #[doc = crate::man_link!(vkCmdClearColorImage)]
    pub fn clear_color_image(
        &mut self, image: &'pool Image, layout: ImageLayout,
        color: ClearColorValue, ranges: &[ImageSubresourceRange],
    ) {
        let array = Array::from_slice(ranges).expect("No clear ranges");
        unsafe {
            (self.device.fun().cmd_clear_color_image)(
                self.buffer.handle_mut(),
                image.handle(),
                layout,
                &color,
                ranges.len() as u32,
                array,
            )
        }
    }
}
