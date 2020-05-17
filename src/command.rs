use crate::{Buffer, DescriptorHeap, Device, Error, Image, TimerQueries};
use std::ops::Range;
use winapi::um::d3d12::D3D12_QUERY_TYPE_TIMESTAMP;

pub use d3d12::CmdListType as CmdBufferTy;

pub struct CommandBuffer {
    allocator: d3d12::CommandAllocator,
    cmd_buffer: d3d12::GraphicsCommandList,
}

impl Device {
    pub fn create_command_buffer(&self, ty: CmdBufferTy) -> Result<CommandBuffer, Error> {
        let (allocator, _) = self.create_command_allocator(ty);
        let (cmd_buffer, _) =
            self.create_graphics_command_list(ty, allocator, d3d12::PipelineState::null(), 0);
        cmd_buffer.close();
        Ok(CommandBuffer {
            allocator,
            cmd_buffer,
        })
    }
}

impl CommandBuffer {
    pub fn begin(&self) {
        self.allocator.reset();
        self.cmd_buffer
            .reset(self.allocator, d3d12::PipelineState::null());
    }

    pub fn copy_buffer(&self, src: &Buffer, dst: &Buffer) {
        unsafe {
            self.cmd_buffer
                .CopyResource(dst.0.as_mut_ptr(), src.0.as_mut_ptr());
        }
    }

    pub fn copy_image(&self, src: &Image, dst: &Image) {
        unsafe {
            self.cmd_buffer
                .CopyResource(dst.0.as_mut_ptr(), src.0.as_mut_ptr());
        }
    }

    pub fn bind_descriptor_heap(&self, heap: &DescriptorHeap) {
        self.cmd_buffer
            .set_descriptor_heaps(&[heap.heap_view, heap.heap_sampler]);
    }

    pub fn timestamp(&self, heap: &TimerQueries, query: usize) {
        unsafe {
            self.cmd_buffer
                .EndQuery(heap.0.as_mut_ptr(), D3D12_QUERY_TYPE_TIMESTAMP, query as _);
        }
    }

    pub fn copy_timestamps(&self, heap: &TimerQueries, queries: Range<usize>, buffer: &Buffer, buffer_offset: u32) {
        unsafe {
            self.cmd_buffer.ResolveQueryData(
                heap.0.as_mut_ptr(),
                D3D12_QUERY_TYPE_TIMESTAMP,
                queries.start as _,
                (queries.end - queries.start) as _,
                buffer.resource().as_mut_ptr(),
                buffer_offset as _,
            );
        }
    }

    pub fn end(&self) {
        self.cmd_buffer.close();
    }

    pub(crate) fn command_list(&self) -> d3d12::CommandList {
        self.cmd_buffer.as_list()
    }
}

impl std::ops::Deref for CommandBuffer {
    type Target = d3d12::GraphicsCommandList;
    fn deref(&self) -> &Self::Target {
        &self.cmd_buffer
    }
}
