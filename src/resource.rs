use crate::{Device, Error};

pub use winapi::um::d3d12::{
    D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL as RESOURCE_FLAG_DEPTH_STENCIL,
    D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET as RESOURCE_FLAG_RENDER_TARGET,
    D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS as RESOURCE_FLAG_WRITE,
    D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE as RESOURCE_FLAG_NON_READONLY,
    D3D12_RESOURCE_STATES as ResourceStates, D3D12_RESOURCE_STATE_COMMON as RESOURCE_STATE_GENERAL,
    D3D12_RESOURCE_STATE_COPY_DEST as RESOURCES_STATE_TRANSFER_DST,
    D3D12_RESOURCE_STATE_COPY_SOURCE as RESOURCES_STATE_TRANSFER_SRC,
    D3D12_RESOURCE_STATE_PRESENT as RESOURCE_STATE_PRESENT,
    D3D12_RESOURCE_STATE_UNORDERED_ACCESS as RESOURCE_STATE_UNORDERED_ACCESS,
};

pub use winapi::shared::dxgiformat::DXGI_FORMAT as Format;
pub use winapi::shared::dxgiformat::*;

use std::ptr;
use winapi::shared::dxgitype::DXGI_SAMPLE_DESC;
use winapi::um::d3d12::*;
use winapi::Interface;

pub struct Buffer(pub(crate) d3d12::Resource);

impl Buffer {
    pub fn resource(&self) -> &d3d12::Resource {
        &self.0
    }

    pub fn copy_from_host(&self, offset: isize, data: &[u8]) {
        unsafe {
            let mut ptr = ptr::null_mut();
            self.0.Map(0, &D3D12_RANGE { Begin: 0, End: 0 }, &mut ptr);
            ptr::copy_nonoverlapping(data.as_ptr(), ptr.offset(offset) as _, data.len());
            self.0.Unmap(0, ptr::null());
        }
    }

    pub fn copy_to_host(&self, offset: isize, data: &mut [u8]) {
        unsafe {
            let mut ptr = ptr::null_mut();
            self.0.Map(0, ptr::null(), &mut ptr);
            ptr::copy_nonoverlapping(ptr.offset(offset) as _, data.as_mut_ptr(), data.len());
            self.0.Unmap(0, ptr::null());
        }
    }
}

pub struct Image(pub(crate) d3d12::Resource);

impl Image {
    pub fn resource(&self) -> &d3d12::Resource {
        &self.0
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ImageType {
    D1,
    D2,
    D3,
}

#[derive(Debug, Copy, Clone)]
pub enum HeapType {
    Device,
    Upload,
    Readback,
}

impl HeapType {
    fn as_d3d12(&self) -> u32 {
        match self {
            HeapType::Device => D3D12_HEAP_TYPE_DEFAULT,
            HeapType::Upload => D3D12_HEAP_TYPE_UPLOAD,
            HeapType::Readback => D3D12_HEAP_TYPE_READBACK,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Extent {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

#[derive(Debug, Clone)]
pub struct ImageDesc {
    pub ty: ImageType,
    pub flags: u32,
    pub format: Format,
    pub extent: Extent,
    pub mip_levels: u32,
}

#[derive(Debug, Clone)]
pub struct BufferDesc {
    pub flags: u32,
    pub size: u32,
}

impl Device {
    pub fn create_image_committed(
        &self,
        desc: &ImageDesc,
        heap: HeapType,
        initial: ResourceStates,
    ) -> Result<Image, Error> {
        let d3d12_desc = D3D12_RESOURCE_DESC {
            Dimension: match desc.ty {
                ImageType::D1 => D3D12_RESOURCE_DIMENSION_TEXTURE1D,
                ImageType::D2 => D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                ImageType::D3 => D3D12_RESOURCE_DIMENSION_TEXTURE3D,
            },
            Alignment: 0,
            Width: desc.extent.width as _,
            Height: desc.extent.height as _,
            DepthOrArraySize: desc.extent.depth as _,
            MipLevels: desc.mip_levels as _,
            Format: desc.format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
            Flags: desc.flags,
        };

        let heap_properties = unsafe { self.GetCustomHeapProperties(0, heap.as_d3d12()) };

        let mut image = d3d12::Resource::null();
        let _hr = unsafe {
            self.CreateCommittedResource(
                &heap_properties,
                D3D12_HEAP_FLAG_ALLOW_ALL_BUFFERS_AND_TEXTURES, // Resource Heap Tier 2 required
                &d3d12_desc,
                initial,
                ptr::null(),
                &ID3D12Resource::uuidof(),
                image.mut_void(),
            )
        };

        Ok(Image(image))
    }

    pub fn create_buffer_committed(
        &self,
        desc: &BufferDesc,
        heap: HeapType,
        initial: ResourceStates,
    ) -> Result<Buffer, Error> {
        let d3d12_desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Alignment: 0,
            Width: desc.size as _,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: DXGI_FORMAT_UNKNOWN,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: desc.flags,
        };

        let heap_properties = unsafe { self.GetCustomHeapProperties(0, heap.as_d3d12()) };

        let mut buffer = d3d12::Resource::null();
        let _hr = unsafe {
            self.CreateCommittedResource(
                &heap_properties,
                D3D12_HEAP_FLAG_ALLOW_ALL_BUFFERS_AND_TEXTURES, // Resource Heap Tier 2 required
                &d3d12_desc,
                initial,
                ptr::null(),
                &ID3D12Resource::uuidof(),
                buffer.mut_void(),
            )
        };

        Ok(Buffer(buffer))
    }
}
