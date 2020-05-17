mod command;
mod debug;
mod descriptor;
mod device;
mod error;
mod pipeline;
mod query;
mod resource;
mod svg;
mod wsi;

pub use crate::command::*;
pub use crate::debug::*;
pub use crate::descriptor::*;
pub use crate::device::*;
pub use crate::error::*;
pub use crate::pipeline::*;
pub use crate::query::*;
pub use crate::resource::*;
pub use crate::svg::*;
pub use crate::wsi::*;

pub struct Semaphore {
    fence: d3d12::Fence,
    event: d3d12::Event,
}

impl Semaphore {
    pub fn wait(&self, timestamp: u64) {
        self.fence.set_event_on_completion(self.event, timestamp);
        self.event.wait(!0);
    }
}

pub struct Queue {
    queue: d3d12::CommandQueue,
}

impl Queue {
    pub fn signal(&self, semaphore: &Semaphore, value: u64) {
        self.queue.signal(semaphore.fence, value);
    }

    pub fn submit(&self, cmd_buffers: &[&CommandBuffer]) {
        let cmd_lists = cmd_buffers
            .iter()
            .map(|buffer| buffer.command_list())
            .collect::<Vec<_>>();
        self.queue.execute_command_lists(&cmd_lists);
    }

    pub fn timing_frequency(&self) -> u64 {
        let mut freq = 0u64;
        unsafe { self.queue.GetTimestampFrequency(&mut freq); }
        freq
    }
}

pub unsafe fn as_u8_slice<T>(data: &[T]) -> &[u8] {
    let len = std::mem::size_of::<T>() * data.len();
    std::slice::from_raw_parts(data.as_ptr() as *const u8, len)
}
