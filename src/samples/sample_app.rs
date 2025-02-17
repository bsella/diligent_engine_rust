use imgui::{Ui, WindowFlags};

use crate::{
    bindings::{self, NativeWindow},
    core::{
        device_context::ResourceStateTransitionMode,
        engine_factory::{AsEngineFactory, EngineCreateInfo},
        graphics_types::{AdapterMemoryInfo, AdapterType, GraphicsAdapterInfo, RenderDeviceType},
        swap_chain::SwapChain,
        vk::engine_factory_vk::{get_engine_factory_vk, EngineVkCreateInfo},
    },
    tools::{
        imgui::{
            events::imgui_handle_event,
            renderer::{ImguiRenderer, ImguiRendererCreateInfo},
        },
        native_app::{
            app::{App, GoldenImageMode},
            events::{EventHandler, EventResult},
        },
    },
};

use super::sample::SampleBase;

pub struct SampleApp<Sample: SampleBase> {
    _app_title: String,
    swap_chain: SwapChain,

    _golden_image_mode: GoldenImageMode,
    _golden_pixel_tolerance: u32,

    sample: Sample,

    vsync: bool,

    current_time: f64,

    _width: u16,
    _height: u16,

    imgui_renderer: ImguiRenderer,
}

impl<GenericSample: SampleBase> SampleApp<GenericSample> {
    fn _get_title(&self) -> &str {
        self._app_title.as_str()
    }

    fn window_resize(&mut self, width: u32, height: u32) {
        self.sample.pre_window_resize();

        self.swap_chain.resize(width, height, None);

        let swap_chain_desc = self.swap_chain.get_desc();

        self.sample
            .window_resize(swap_chain_desc.Width, swap_chain_desc.Height);
    }

    fn update(&mut self, current_time: f64, elapsed_time: f64) {
        self.current_time = current_time;

        // TODO : update app settings

        self.sample.update(current_time, elapsed_time);
    }

    fn update_ui(&mut self) -> &mut Ui {
        let ui = self.imgui_renderer.new_frame();

        let swap_chain_desc = self.swap_chain.get_desc();

        let adapters_wnd_width = swap_chain_desc.Width.min(330);

        if let Some(_window_tokend) = ui
            .window("Adapters")
            .size([adapters_wnd_width as f32, 0.0], imgui::Condition::Always)
            .position(
                [
                    (swap_chain_desc.Width as f32 - adapters_wnd_width as f32).max(10.0) - 10.0,
                    10.0,
                ],
                imgui::Condition::Always,
            )
            .flags(WindowFlags::NO_RESIZE)
            .collapsed(true, imgui::Condition::FirstUseEver)
            .begin()
        {
            ui.text_disabled(format!("Adapter: {} ({} MB)", "test", 5));

            ui.checkbox("VSync", &mut self.vsync);
        }
        ui
    }

    fn render(&self) {
        let context = self.sample.get_immediate_context();
        context.clear_stats();

        let rtv = self.swap_chain.get_current_back_buffer_rtv();
        let dsv = self.swap_chain.get_depth_buffer_dsv();

        context.set_render_targets(&[&rtv], Some(&dsv), ResourceStateTransitionMode::Transition);

        self.sample.render(&self.swap_chain);

        // Restore default render target in case the sample has changed it
        context.set_render_targets(&[&rtv], Some(&dsv), ResourceStateTransitionMode::Transition);
    }

    fn present(&mut self) {
        // TODO screen capture

        self.swap_chain.present(if self.vsync { 1 } else { 0 });

        // TODO screen capture
    }
}

impl<GenericSample: SampleBase> App for SampleApp<GenericSample> {
    fn new(
        device_type: RenderDeviceType,
        mut engine_create_info: EngineCreateInfo,
        window: Option<&NativeWindow>,
        initial_width: u16,
        initial_height: u16,
    ) -> Self {
        let swap_chain_desc = bindings::SwapChainDesc::default();

        //#[cfg(any(
        //    feature = "D3D11_SUPPORTED",
        //    feature = "D3D12_SUPPORTED",
        //    feature = "VULKAN_SUPPORTED",
        //    feature = "WEBGPU_SUPPORTED"
        //))]
        fn find_adapter(
            mut adapter_index: Option<usize>,
            adapter_type: AdapterType,
            adapters: &[GraphicsAdapterInfo],
        ) -> Option<usize> {
            let mut adapter_type = adapter_type.clone();

            if let Some(adap_id) = adapter_index {
                if adap_id < adapters.len() {
                    adapter_type = adapters.get(adap_id).unwrap().adapter_type.clone();
                } else {
                    //LOG_ERROR_MESSAGE("Adapter ID (", AdapterId, ") is invalid. Only ", Adapters.size(), " compatible ", (Adapters.size() == 1 ? "adapter" : "adapters"), " present in the system");
                    adapter_index = None;
                }
            }

            if adapter_index.is_none() && adapter_type != AdapterType::Unknown {
                adapter_index = adapters
                    .iter()
                    .position(|adapter| adapter.adapter_type == adapter_type)
                    .map_or(None, |id| Some(id));
            };

            if adapter_index.is_none() {
                adapter_type = AdapterType::Unknown;

                let mut curr_adapter_mem: Option<&AdapterMemoryInfo> = None;
                let mut curr_total_memory = 0u64;

                for (i, adapter) in adapters.iter().enumerate() {
                    if adapter.adapter_type > adapter_type {
                        // Prefer Discrete over Integrated over Software
                        adapter_type = adapter.adapter_type.clone();
                        adapter_index = Some(i);
                    } else if adapter.adapter_type == adapter_type {
                        // Select adapter with more memory
                        let new_adapter_mem = &adapter.memory;
                        let new_total_memory = new_adapter_mem.local_memory
                            + new_adapter_mem.host_visible_memory
                            + new_adapter_mem.unified_memory;

                        if let Some(adapter_mem) = curr_adapter_mem {
                            let total_memory = adapter_mem.local_memory
                                + adapter_mem.host_visible_memory
                                + adapter_mem.unified_memory;
                            if total_memory > curr_total_memory {
                                curr_adapter_mem = Some(&new_adapter_mem);
                                curr_total_memory = total_memory;
                                adapter_index = Some(i);
                            }
                        } else {
                            curr_adapter_mem = Some(&new_adapter_mem);
                            curr_total_memory = new_total_memory;
                            adapter_index = Some(i);
                        }
                    }
                }
            }

            if let Some(adapter_index) = adapter_index {
                let adaper_description = &adapters.get(adapter_index).unwrap().description;
                println!("Using adapter {adapter_index}, : '{adaper_description}'");
            }

            adapter_index
        }

        let (render_device, immediate_contexts, deferred_contexts, swap_chain) = match device_type {
            RenderDeviceType::D3D11 => panic!(),
            RenderDeviceType::D3D12 => panic!(),
            RenderDeviceType::GL => panic!(),
            RenderDeviceType::GLES => panic!(),
            RenderDeviceType::VULKAN => {
                let engine_factory = get_engine_factory_vk();

                if let Some(adapter_index) = find_adapter(
                    None,
                    AdapterType::Unknown,
                    engine_factory
                        .as_engine_factory()
                        .enumerate_adapters(&engine_create_info.graphics_api_version)
                        .as_slice(),
                ) {
                    engine_create_info.adapter_index.replace(adapter_index);
                }

                let engine_vk_create_info = EngineVkCreateInfo::new(engine_create_info);

                let (render_device, immediate_contexts, deferred_contexts) = engine_factory
                    .create_device_and_contexts(&engine_vk_create_info)
                    .unwrap();

                let swap_chain = engine_factory
                    .create_swap_chain(
                        &render_device,
                        immediate_contexts.first().unwrap(),
                        &swap_chain_desc,
                        window,
                    )
                    .unwrap();

                (
                    render_device,
                    immediate_contexts,
                    deferred_contexts,
                    swap_chain,
                )
            }
            RenderDeviceType::METAL => panic!(),
            RenderDeviceType::WEBGPU => panic!(),
        };

        let sample = GenericSample::new(
            render_device,
            immediate_contexts,
            deferred_contexts,
            &swap_chain,
        );

        let imgui_renderer = ImguiRenderer::new(ImguiRendererCreateInfo::new(
            sample.get_render_device(),
            swap_chain.get_desc().ColorBufferFormat,
            swap_chain.get_desc().DepthBufferFormat,
            initial_width,
            initial_height,
        ));

        SampleApp::<GenericSample> {
            _app_title: GenericSample::get_name().to_string(),
            swap_chain,

            _golden_image_mode: GoldenImageMode::None,
            _golden_pixel_tolerance: 0,

            sample,

            vsync: false,

            current_time: 0.0,

            _width: initial_width,
            _height: initial_height,

            imgui_renderer,
        }
    }

    fn run<EH>(mut self, mut event_handler: EH) -> Result<(), std::io::Error>
    where
        EH: EventHandler,
    {
        'main: loop {
            while let Some(event) = event_handler.poll_event() {
                let event = event_handler.handle_event(&event);
                match event {
                    EventResult::Quit => break 'main,
                    EventResult::Continue => {}
                    EventResult::Resize { width, height } => {
                        self.window_resize(width as u32, height as u32)
                    }
                    _ => {}
                }

                let event = imgui_handle_event(self.imgui_renderer.io_mut(), event);

                self.sample.handle_event(event);
            }

            // TODO implement timer
            self.update(0.0, 0.0);

            self.render();

            self.update_ui();
            self.imgui_renderer.render(
                self.sample.get_immediate_context(),
                self.sample.get_render_device(),
            );

            self.present();

            //TODO update title
        }

        Ok(())
    }
}
