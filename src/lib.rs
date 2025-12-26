use std::sync::{Arc, atomic::AtomicBool};

use crossbeam::atomic::AtomicCell;
use nih_plug::prelude::*;
use nih_plug_egui::{EguiState, create_egui_editor};

use crate::buffer_size::Note;

mod buffer_size;
mod dir;
mod editor;

pub struct RollingSamplerClone {
    params: Arc<RollingSamplerCloneParams>,
    channel_buffers: Vec<Vec<f32>>
}

#[derive(Params)]
pub struct RollingSamplerCloneParams {
    // We want the GUI state to persist between different instances
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[nested(group = "buffer-size")]
    buffer_size: Arc<buffer_size::BufferSize>,

    #[persist = "gui-theme"]
    theme_type: Arc<AtomicCell<editor::ThemeType>>,

    /// Clears the buffer when the transport starts playing
    #[persist = "clear-on-play"]
    clear_on_play: Arc<AtomicBool>,

    /// Removes silence at the beginning and end of an audio clip before it is saved
    /// This excludes the first and last samples in the clip if they are equal to zero
    /// Audio clips that consist of only silence won't be saved regardless
    #[persist = "trim-silence"]
    trim_silence: Arc<AtomicBool>,

    // / Directory where saved clips are stored
    // #[persist = "clip_path"]
    // clip_path: Arc<Mutex<PathBuf>>
}

impl Default for RollingSamplerCloneParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(600, 150),
            buffer_size: Arc::new(buffer_size::BufferSize {
                unit: Arc::new(AtomicCell::new(buffer_size::BufferSizeUnit::Seconds)),
                seconds: Arc::new(AtomicF32::new(10.0)),
                notes: Arc::new(AtomicCell::new(Note(1, 4))), }),
            theme_type: Arc::new(AtomicCell::new(editor::ThemeType::Dark)),
            clear_on_play: Arc::new(AtomicBool::new(false)),
            trim_silence: Arc::new(AtomicBool::new(true)),
            // clip_path: Arc::new(Mutex::new(dir::default_clip_dir()))
        }
    }
}

// I presume that this is how we set the state of the plugin when it is first loaded
impl Default for RollingSamplerClone {
    fn default() -> Self {
        Self {
            params: Arc::new(RollingSamplerCloneParams::default()),
            channel_buffers: Vec::new(),
        }
    }
}

impl Plugin for RollingSamplerClone {
    const NAME: &'static str = "Rolling Sampler Clone";
    const VENDOR: &'static str = "danmarkowal";
    const URL: &'static str = "https://github.com/danmarkowal/";
    const EMAIL: &'static str = "danmarkowal@gmail.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        // stereo
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),

            aux_input_ports: &[],
            aux_output_ports: &[],

            names: PortNames::const_default(),
        },
        // mono
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        }
    ];

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> std::sync::Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let editor_state = params.editor_state.clone();
        // cannot move editor_state twice so we need two clones of the state arc
        let closure_state = params.editor_state.clone();

        create_egui_editor(
            // this clone lives for as long as the editor exists
            editor_state, (),
            |ctx, _| {
                editor::build_ui(ctx);
            },
            // move captures by value
            move |ctx, setter, _state| {
                editor::update_ui(ctx, closure_state.as_ref(), params.as_ref(), setter);
            })
    }
}

impl Vst3Plugin for RollingSamplerClone {
    const VST3_CLASS_ID: [u8; 16] = *b"dmkrollingsample";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Analyzer];
}

nih_export_vst3!(RollingSamplerClone);
