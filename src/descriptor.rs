use crate::{Buffer, Device, Error, Format, Image, DXGI_FORMAT_UNKNOWN};
use std::{mem, ops::Range, ptr};
use winapi::um::d3d12::*;

pub use d3d12::{CpuDescriptor, GpuDescriptor};

pub struct DescriptorHeap {
    pub(crate) heap_view: d3d12::DescriptorHeap,
    pub(crate) heap_sampler: d3d12::DescriptorHeap,

    increment_view: u32,
    increment_sampler: u32,
}

impl DescriptorHeap {
    pub fn create_pool(&self, views: Range<usize>, samplers: Range<usize>) -> DescriptorPool {
        let cpu_view = self.heap_view.start_cpu_descriptor();
        let gpu_view = self.heap_view.start_gpu_descriptor();

        let cpu_sampler = self.heap_sampler.start_cpu_descriptor();
        let gpu_sampler = self.heap_sampler.start_gpu_descriptor();

        DescriptorPool {
            view_start_cpu: CpuDescriptor {
                ptr: cpu_view.ptr + views.start * self.increment_view as usize,
            },
            view_start_gpu: GpuDescriptor {
                ptr: gpu_view.ptr + views.start as u64 * self.increment_view as u64,
            },
            num_views: views.end - views.start,
            increment_view: self.increment_view,

            sampler_start_cpu: CpuDescriptor {
                ptr: cpu_sampler.ptr + samplers.start * self.increment_sampler as usize,
            },
            sampler_start_gpu: GpuDescriptor {
                ptr: gpu_sampler.ptr + samplers.start as u64 * self.increment_sampler as u64,
            },
            num_samplers: samplers.end - samplers.start,
            increment_sampler: self.increment_sampler,
        }
    }
}

pub struct DescriptorHeapDesc {
    pub num_views: usize,
    pub num_samplers: usize,
}

pub struct DescriptorPool {
    view_start_cpu: CpuDescriptor,
    view_start_gpu: GpuDescriptor,
    increment_view: u32,
    num_views: usize,

    sampler_start_cpu: CpuDescriptor,
    sampler_start_gpu: GpuDescriptor,
    increment_sampler: u32,
    num_samplers: usize,
}

impl DescriptorPool {
    pub fn view_cpu(&self, offset: usize) -> CpuDescriptor {
        assert!(offset < self.num_views);
        CpuDescriptor {
            ptr: self.view_start_cpu.ptr + offset * self.increment_view as usize,
        }
    }

    pub fn sampler_cpu(&self, offset: usize) -> CpuDescriptor {
        assert!(offset < self.num_samplers);
        CpuDescriptor {
            ptr: self.sampler_start_cpu.ptr + offset * self.increment_sampler as usize,
        }
    }

    pub fn view_gpu(&self, offset: usize) -> GpuDescriptor {
        assert!(offset < self.num_views);
        GpuDescriptor {
            ptr: self.view_start_gpu.ptr + offset as u64 * self.increment_view as u64,
        }
    }

    pub fn sampler_gpu(&self, offset: usize) -> GpuDescriptor {
        assert!(offset < self.num_samplers);
        GpuDescriptor {
            ptr: self.sampler_start_gpu.ptr + offset as u64 * self.increment_sampler as u64,
        }
    }
}

pub struct StorageBufferDesc {
    pub elements: Range<usize>,
    pub stride: usize,
}

pub struct UniformBufferDesc {
    pub elements: Range<usize>,
    pub stride: usize,
}

pub enum ImageViewType {
    D2,
}

pub struct StorageImageDesc {
    pub ty: ImageViewType,
    pub format: Format,
    pub mip_level: usize,
    pub array_layers: Range<usize>,
}

impl Device {
    pub fn create_descriptor_heap(
        &self,
        desc: &DescriptorHeapDesc,
    ) -> Result<DescriptorHeap, Error> {
        let (heap_view, _) = d3d12::Device::create_descriptor_heap(
            self,
            desc.num_views as _,
            d3d12::DescriptorHeapType::CbvSrvUav,
            d3d12::DescriptorHeapFlags::SHADER_VISIBLE,
            0,
        );
        let (heap_sampler, _) = d3d12::Device::create_descriptor_heap(
            self,
            desc.num_samplers as _,
            d3d12::DescriptorHeapType::Sampler,
            d3d12::DescriptorHeapFlags::SHADER_VISIBLE,
            0,
        );
        let increment_view =
            self.get_descriptor_increment_size(d3d12::DescriptorHeapType::CbvSrvUav);
        let increment_sampler =
            self.get_descriptor_increment_size(d3d12::DescriptorHeapType::Sampler);

        Ok(DescriptorHeap {
            heap_view,
            heap_sampler,

            increment_view,
            increment_sampler,
        })
    }

    pub fn create_buffer_uniform_view(
        &self,
        buffer: &Buffer,
        descriptor: CpuDescriptor,
        desc: &UniformBufferDesc,
    ) {
        unsafe {
            let mut d3d12_desc = D3D12_SHADER_RESOURCE_VIEW_DESC {
                Format: DXGI_FORMAT_UNKNOWN,
                ViewDimension: D3D12_SRV_DIMENSION_BUFFER,
                Shader4ComponentMapping: 0x1688,
                ..mem::zeroed()
            };

            *d3d12_desc.u.Buffer_mut() = D3D12_BUFFER_SRV {
                FirstElement: desc.elements.start as _,
                NumElements: (desc.elements.end - desc.elements.start) as _,
                StructureByteStride: desc.stride as _,
                Flags: D3D12_BUFFER_SRV_FLAG_NONE,
            };

            self.CreateShaderResourceView(buffer.0.as_mut_ptr(), &d3d12_desc, descriptor);
        }
    }

    pub fn create_buffer_storage_view(
        &self,
        buffer: &Buffer,
        descriptor: CpuDescriptor,
        desc: &StorageBufferDesc,
    ) {
        unsafe {
            let mut d3d12_desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
                Format: DXGI_FORMAT_UNKNOWN,
                ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
                ..mem::zeroed()
            };

            *d3d12_desc.u.Buffer_mut() = D3D12_BUFFER_UAV {
                FirstElement: desc.elements.start as _,
                NumElements: (desc.elements.end - desc.elements.start) as _,
                StructureByteStride: desc.stride as _,
                CounterOffsetInBytes: 0,
                Flags: D3D12_BUFFER_UAV_FLAG_NONE,
            };

            self.CreateUnorderedAccessView(
                buffer.0.as_mut_ptr(),
                ptr::null_mut(),
                &d3d12_desc,
                descriptor,
            );
        }
    }

    pub fn create_image_storage_view(
        &self,
        image: &Image,
        descriptor: CpuDescriptor,
        desc: &StorageImageDesc,
    ) {
        unsafe {
            let mut d3d12_desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
                Format: desc.format,
                ..mem::zeroed()
            };

            match desc.ty {
                ImageViewType::D2 => {
                    if desc.array_layers.start == 0 && desc.array_layers.end == 1 {
                        d3d12_desc.ViewDimension = D3D12_UAV_DIMENSION_TEXTURE2D;
                        *d3d12_desc.u.Texture2D_mut() = D3D12_TEX2D_UAV {
                            MipSlice: desc.mip_level as _,
                            PlaneSlice: 0,
                        };
                    } else {
                        d3d12_desc.ViewDimension = D3D12_UAV_DIMENSION_TEXTURE2DARRAY;
                        *d3d12_desc.u.Texture2DArray_mut() = D3D12_TEX2D_ARRAY_UAV {
                            MipSlice: desc.mip_level as _,
                            FirstArraySlice: desc.array_layers.start as _,
                            ArraySize: (desc.array_layers.end - desc.array_layers.start) as _,
                            PlaneSlice: 0,
                        };
                    }
                }
            }

            self.CreateUnorderedAccessView(
                image.0.as_mut_ptr(),
                ptr::null_mut(),
                &d3d12_desc,
                descriptor,
            );
        }
    }
}
