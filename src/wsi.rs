//! Window System Interface
//!
//! Currently only supporting winit.

use crate::{Device, Error, Image, Queue};
use winapi::shared::{dxgiformat, dxgitype};
use winit::{platform::windows::WindowExtWindows, window::Window};

pub struct Swapchain {
    swapchain: d3d12::SwapChain3,
    render_targets: Vec<Image>,
}

impl Device {
    pub fn create_swapchain(
        &self,
        present_queue: &Queue,
        window: &Window,
        buffer_size: u32,
    ) -> Result<Swapchain, Error> {
        let hwnd = window.hwnd();
        let size = window.inner_size();

        let desc = d3d12::SwapchainDesc {
            width: size.width as _,
            height: size.height as _,
            format: dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            stereo: false,
            sample: d3d12::SampleDesc {
                count: 1,
                quality: 0,
            },
            buffer_usage: dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            buffer_count: buffer_size,
            scaling: d3d12::Scaling::Identity,
            swap_effect: d3d12::SwapEffect::FlipDiscard,
            alpha_mode: d3d12::AlphaMode::Ignore,
            flags: 0,
        };
        let (swapchain, _) = self.factory.as_factory2().create_swapchain_for_hwnd(
            present_queue.queue,
            hwnd as _,
            &desc,
        );

        let render_targets = (0..buffer_size)
            .map(|i| {
                let (image, _) = swapchain.as_swapchain0().get_buffer(i);
                Image(image)
            })
            .collect();

        let (swapchain, _) = unsafe { swapchain.cast() };

        Ok(Swapchain {
            swapchain,
            render_targets,
        })
    }
}

impl Swapchain {
    pub fn render_targets(&self) -> &[Image] {
        &self.render_targets
    }

    pub fn acquire(&self) -> usize {
        self.swapchain.get_current_back_buffer_index() as _
    }

    pub fn present(&self) {
        unsafe {
            self.swapchain.as_swapchain0().Present(0, 0);
        }
    }
}
