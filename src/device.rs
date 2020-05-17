use crate::{CmdBufferTy, Error, Pipeline, PipelineLayout, Queue, Semaphore, Shader};
use winapi::shared::winerror;

pub use d3d12::FactoryCreationFlags as DeviceCreateFlags;

const FEATURE_LEVEL: d3d12::FeatureLevel = d3d12::FeatureLevel::L12_1;

type Factory = d3d12::Factory4;
type Adapter = d3d12::Adapter1;
type D3DDevice = d3d12::Device;

pub struct Device {
    pub(crate) factory: Factory,
    device: D3DDevice,
}

impl Device {
    pub fn new(flags: DeviceCreateFlags) -> Result<Self, Error> {
        if flags.contains(DeviceCreateFlags::DEBUG) {
            let (debug, _) = d3d12::Debug::get_interface();
            debug.enable_layer();
        }

        let (factory, _) = d3d12::Factory4::create(flags);

        // Find suitable adapter and open device.
        let adapter = Self::select_adapter(&factory);
        let (device, _) = D3DDevice::create(adapter, FEATURE_LEVEL);

        Ok(Device { factory, device })
    }

    fn select_adapter(factory: &Factory) -> Adapter {
        let mut adapter_id = 0;
        loop {
            let (adapter, hr) = factory.enumerate_adapters(adapter_id);
            if hr == winerror::DXGI_ERROR_NOT_FOUND {
                break;
            }

            adapter_id += 1;

            // Check for D3D12 support
            {
                let (device, hr) = D3DDevice::create(adapter, d3d12::FeatureLevel::L12_0);
                if !winerror::SUCCEEDED(hr) {
                    continue;
                }
                unsafe {
                    device.destroy();
                }
            };

            return adapter;
        }

        panic!("Couldn't find suitable D3D12 adapter");
    }

    pub fn create_semaphore(&self) -> Result<Semaphore, Error> {
        let (fence, _) = self.create_fence(0);
        let event = d3d12::Event::create(false, false);

        Ok(Semaphore { fence, event })
    }

    pub fn create_queue(&self, ty: CmdBufferTy) -> Result<Queue, Error> {
        let (queue, _) = self.device.create_command_queue(
            ty,
            d3d12::Priority::Normal,
            d3d12::CommandQueueFlags::empty(),
            0,
        );

        Ok(Queue { queue })
    }

    pub fn create_compute_pipeline(
        &self,
        shader: &Shader,
        layout: &PipelineLayout,
    ) -> Result<Pipeline, Error> {
        let (pipeline, _) = self.device.create_compute_pipeline_state(
            *layout,
            shader.bytecode(),
            0,
            d3d12::CachedPSO::null(),
            d3d12::PipelineStateFlags::empty(),
        );
        Ok(pipeline)
    }
}

impl std::ops::Deref for Device {
    type Target = D3DDevice;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
