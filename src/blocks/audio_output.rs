use crate::blocks::ESDRBlock;
use crate::blocks::ESDRBlockInput;
use crate::consts;
use crate::params::Param;

use futuresdr::blocks::audio::AudioSink;
use futuresdr::runtime::Block;

#[derive(Clone, Copy, Default)]
pub struct AudioOutputBlock {}
impl ESDRBlock for AudioOutputBlock {
    fn name(self) -> &'static str {
        "Audio Output"
    }

    fn params(self) -> Vec<Param> {
        vec![Param::input_stream("in").build()]
    }

    fn block(self, _input: ESDRBlockInput) -> Block {
        AudioSink::new(consts::AUDIO_RATE, 1)
    }
}
