use ragnarok::Error;
use std::mem;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const GROUP_X: u32 = 1;
const GROUP_Y: u32 = 32;

const WIDTH: u32 = GROUP_X * 1024;
const HEIGHT: u32 = GROUP_Y * 32;

const TILES_X: u32 = WIDTH / GROUP_X;
const TILES_Y: u32 = HEIGHT / GROUP_Y;

const NUM_FRAMES: u32 = 2;
const NUM_QUERIES: u32 = 2;

#[repr(C)]
struct Locals {
    num_tiles: [u32; 2],
    viewport_offset: [f32; 2],
    viewport_extent: [f32; 2],
    num_objects: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let debug_handler = ragnarok::debug_logger_add();

    let device_flags = if false {
        ragnarok::DeviceCreateFlags::DEBUG
    } else {
        ragnarok::DeviceCreateFlags::empty()
    };
    let device = ragnarok::Device::new(device_flags)?;
    let queue = device.create_queue(ragnarok::CmdBufferTy::Direct)?;
    let descriptor_heap = device.create_descriptor_heap(&ragnarok::DescriptorHeapDesc {
        num_views: 1024,
        num_samplers: 64,
    })?;
    let descriptor_pool = descriptor_heap.create_pool(0..128, 0..16);

    let sample_layout = device.create_pipeline_layout(&[
        // render target
        ragnarok::LayoutDesc::Descriptors(vec![ragnarok::BindingDesc {
            ty: ragnarok::DescriptorTy::UAV,
            space: 0,
            bindings: 0..1,
        }]),
        // locals
        ragnarok::LayoutDesc::Constant {
            space: 1,
            binding: 0,
            num: (mem::size_of::<Locals>() / 4) as _,
        },
        // path data
        ragnarok::LayoutDesc::Descriptors(vec![ragnarok::BindingDesc {
            ty: ragnarok::DescriptorTy::SRV,
            space: 1,
            bindings: 0..3,
        }]),
    ])?;

    let sample_cs =
        ragnarok::Shader::new_from_path("sample_cs", "assets/sample.hlsl", "test", "cs_6_0")?;
    let sample_pipeline = device.create_compute_pipeline(&sample_cs, &sample_layout)?;

    let buffer_cpu = device.create_buffer_committed(
        &ragnarok::BufferDesc {
            size: mem::size_of::<Locals>() as _,
            flags: 0,
        },
        ragnarok::HeapType::Upload,
        ragnarok::RESOURCE_STATE_GENERAL,
    )?;
    let buffer_gpu = device.create_buffer_committed(
        &ragnarok::BufferDesc {
            size: mem::size_of::<Locals>() as _,
            flags: ragnarok::RESOURCE_FLAG_WRITE,
        },
        ragnarok::HeapType::Device,
        ragnarok::RESOURCE_STATE_GENERAL,
    )?;
    buffer_cpu.copy_from_host(0, unsafe {
        ragnarok::as_u8_slice(&[Locals {
            num_tiles: [TILES_X, TILES_Y],
            viewport_offset: [0.0, 0.0],
            viewport_extent: [WIDTH as _, HEIGHT as _],
            num_objects: 0,
        }])
    });
    device.create_buffer_storage_view(
        &buffer_gpu,
        descriptor_pool.view_cpu(0),
        &ragnarok::StorageBufferDesc {
            elements: 0..1,
            stride: 2,
        },
    );

    let image_gpu = device.create_image_committed(
        &ragnarok::ImageDesc {
            ty: ragnarok::ImageType::D2,
            flags: ragnarok::RESOURCE_FLAG_WRITE,
            format: ragnarok::DXGI_FORMAT_R8G8B8A8_UNORM,
            extent: ragnarok::Extent {
                width: WIDTH,
                height: HEIGHT,
                depth: 1,
            },
            mip_levels: 1,
        },
        ragnarok::HeapType::Device,
        ragnarok::RESOURCE_STATE_GENERAL,
    )?;
    device.create_image_storage_view(
        &image_gpu,
        descriptor_pool.view_cpu(1),
        &ragnarok::StorageImageDesc {
            ty: ragnarok::ImageViewType::D2,
            format: ragnarok::DXGI_FORMAT_R8G8B8A8_UNORM,
            mip_level: 0,
            array_layers: 0..1,
        },
    );

    let svg_path_bez = ragnarok::parse_svg("assets/Ghostscript_Tiger.svg")?;
    let svg_path = ragnarok::generate_gpu_data(&svg_path_bez);
    dbg!(&svg_path.objects.len());
    let svg_objects_cpu = device.create_buffer_committed(
        &ragnarok::BufferDesc {
            size: (svg_path.objects.len() * mem::size_of::<ragnarok::Object>()) as _,
            flags: 0,
        },
        ragnarok::HeapType::Upload,
        ragnarok::RESOURCE_STATE_GENERAL,
    )?;
    let svg_objects_gpu = device.create_buffer_committed(
        &ragnarok::BufferDesc {
            size: (svg_path.objects.len() * mem::size_of::<ragnarok::Object>()) as _,
            flags: 0,
        },
        ragnarok::HeapType::Device,
        ragnarok::RESOURCE_STATE_GENERAL,
    )?;
    svg_objects_cpu.copy_from_host(0, unsafe { ragnarok::as_u8_slice(&svg_path.objects) });
    let svg_primitives_cpu = device.create_buffer_committed(
        &ragnarok::BufferDesc {
            size: (svg_path.primitives.len() * 4) as _,
            flags: 0,
        },
        ragnarok::HeapType::Upload,
        ragnarok::RESOURCE_STATE_GENERAL,
    )?;
    let svg_primitives_gpu = device.create_buffer_committed(
        &ragnarok::BufferDesc {
            size: (svg_path.primitives.len() * 4) as _,
            flags: 0,
        },
        ragnarok::HeapType::Device,
        ragnarok::RESOURCE_STATE_GENERAL,
    )?;
    svg_primitives_cpu.copy_from_host(0, unsafe { ragnarok::as_u8_slice(&svg_path.primitives) });

    let svg_data_cpu = device.create_buffer_committed(
        &ragnarok::BufferDesc {
            size: (svg_path.data.len() * 4) as _,
            flags: 0,
        },
        ragnarok::HeapType::Upload,
        ragnarok::RESOURCE_STATE_GENERAL,
    )?;
    let svg_data_gpu = device.create_buffer_committed(
        &ragnarok::BufferDesc {
            size: (svg_path.data.len() * 4) as _,
            flags: 0,
        },
        ragnarok::HeapType::Device,
        ragnarok::RESOURCE_STATE_GENERAL,
    )?;
    svg_data_cpu.copy_from_host(0, unsafe { ragnarok::as_u8_slice(&svg_path.data) });

    device.create_buffer_uniform_view(
        &svg_objects_gpu,
        descriptor_pool.view_cpu(2),
        &ragnarok::UniformBufferDesc {
            elements: 0..svg_path.objects.len(),
            stride: mem::size_of::<ragnarok::Object>(),
        },
    );
    device.create_buffer_uniform_view(
        &svg_primitives_gpu,
        descriptor_pool.view_cpu(3),
        &ragnarok::UniformBufferDesc {
            elements: 0..svg_path.primitives.len(),
            stride: mem::size_of::<u32>(),
        },
    );
    device.create_buffer_uniform_view(
        &svg_data_gpu,
        descriptor_pool.view_cpu(4),
        &ragnarok::UniformBufferDesc {
            elements: 0..svg_path.data.len(),
            stride: mem::size_of::<u32>(),
        },
    );

    let upload_buffer = device.create_command_buffer(ragnarok::CmdBufferTy::Direct)?;
    let upload_fence = device.create_semaphore()?;
    upload_buffer.begin();
    upload_buffer.copy_buffer(&buffer_cpu, &buffer_gpu);
    upload_buffer.copy_buffer(&svg_objects_cpu, &svg_objects_gpu);
    upload_buffer.copy_buffer(&svg_primitives_cpu, &svg_primitives_gpu);
    upload_buffer.copy_buffer(&svg_data_cpu, &svg_data_gpu);
    upload_buffer.end();
    queue.signal(&upload_fence, 1);
    queue.submit(&[&upload_buffer]);
    upload_fence.wait(1);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)?;

    let swapchain = device.create_swapchain(&queue, &window, NUM_FRAMES)?;
    let present_sync = device.create_semaphore()?;

    let cmd_buffers = (0..NUM_FRAMES)
        .map(|_| device.create_command_buffer(ragnarok::CmdBufferTy::Direct))
        .collect::<Result<Box<_>, Error>>()?;

    let timer_queries = (0..NUM_FRAMES)
        .map(|_| device.create_timer_queries(2))
        .collect::<Result<Box<_>, Error>>()?;
    let timer_buffer = device.create_buffer_committed(
        &ragnarok::BufferDesc {
            size: NUM_FRAMES * NUM_QUERIES * 8,
            flags: 0,
        },
        ragnarok::HeapType::Readback,
        ragnarok::RESOURCE_STATE_GENERAL,
    )?;

    let timer_freq = queue.timing_frequency();
    let mut tick = 0u64;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                if tick >= NUM_FRAMES as u64 {
                    present_sync.wait(tick - NUM_FRAMES as u64);

                    let mut times = [0u64; NUM_QUERIES as usize];
                    let times_raw = unsafe {
                        std::slice::from_raw_parts_mut(times.as_mut_ptr() as *mut u8, times.len() * 8)
                    };
                    timer_buffer.copy_to_host(8 * NUM_QUERIES as isize * (tick % NUM_FRAMES as u64) as isize, times_raw);
                    let dt0 = times[1] - times[0];

                    window.set_title(&format!("ragnarok :: dt_0: {:.2}ms", dt0 as f64 / timer_freq as f64 * 1000.0));
                }

                let frame = swapchain.acquire();
                let frame_image = &swapchain.render_targets()[frame];
                let cmd_buf = &cmd_buffers[frame];
                let timer_query = &timer_queries[frame];

                cmd_buf.begin();
                cmd_buf.resource_barrier(&[
                    d3d12::ResourceBarrier::transition(
                        *frame_image.resource(),
                        0,
                        ragnarok::RESOURCE_STATE_PRESENT,
                        ragnarok::RESOURCES_STATE_TRANSFER_DST,
                        0,
                    ),
                    d3d12::ResourceBarrier::transition(
                        *image_gpu.resource(),
                        0,
                        ragnarok::RESOURCES_STATE_TRANSFER_SRC,
                        ragnarok::RESOURCE_STATE_UNORDERED_ACCESS,
                        0,
                    ),
                ]);
                cmd_buf.bind_descriptor_heap(&descriptor_heap);
                cmd_buf.set_compute_root_signature(sample_layout);
                cmd_buf.set_compute_root_descriptor_table(0, descriptor_pool.view_gpu(1));
                cmd_buf.set_compute_root_descriptor_table(2, descriptor_pool.view_gpu(2));

                let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
                let target_height = 200.0;
                let locals = Locals {
                    num_tiles: [TILES_X, TILES_Y],
                    viewport_offset: [0.0, 0.0],
                    viewport_extent: [aspect_ratio * target_height, target_height],
                    num_objects: svg_path.objects.len() as _,
                };
                unsafe {
                    cmd_buf.SetComputeRoot32BitConstants(
                        1,
                        (mem::size_of::<Locals>() / 4) as _,
                        &locals as *const _ as _,
                        0,
                    );
                }

                cmd_buf.set_pipeline_state(sample_pipeline);
                cmd_buf.timestamp(timer_query, 0);
                cmd_buf.dispatch([TILES_X, TILES_Y, 1]);
                cmd_buf.timestamp(timer_query, 1);
                cmd_buf.resource_barrier(&[d3d12::ResourceBarrier::transition(
                    *image_gpu.resource(),
                    0,
                    ragnarok::RESOURCE_STATE_UNORDERED_ACCESS,
                    ragnarok::RESOURCES_STATE_TRANSFER_SRC,
                    0,
                )]);
                cmd_buf.copy_image(&image_gpu, frame_image);
                cmd_buf.resource_barrier(&[d3d12::ResourceBarrier::transition(
                    *frame_image.resource(),
                    0,
                    ragnarok::RESOURCES_STATE_TRANSFER_DST,
                    ragnarok::RESOURCE_STATE_PRESENT,
                    0,
                )]);
                cmd_buf.copy_timestamps(timer_query, 0..2, &timer_buffer, 8 * NUM_QUERIES * frame as u32);
                cmd_buf.end();

                queue.submit(&[&cmd_buf]);
                queue.signal(&present_sync, tick);

                swapchain.present();

                tick += 1;
            }
            Event::LoopDestroyed => {
                ragnarok::debug_logger_remove(debug_handler);
            }
            _ => (),
        }
    })
}
