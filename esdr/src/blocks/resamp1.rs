use crate::blocks::ESDRBlock;
use crate::blocks::ESDRBlockInput;
use crate::consts;
use crate::param::Param;

use futuresdr::blocks::FirBuilder;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;

#[derive(Clone, Copy, Default)]
pub struct Resamp1Block {}
impl ESDRBlock for Resamp1Block {
    fn name(self) -> &'static str {
        "Resamp 1"
    }

    fn block(self, _input: ESDRBlockInput) -> Block {
        let interp = (consts::AUDIO_RATE * consts::AUDIO_MULT) as usize;
        let decim = consts::RATE as usize;
        FirBuilder::new_resampling::<Complex32>(interp, decim)
    }

    fn params(self) -> Vec<Param> {
        vec![
            Param::input_stream("in").build(),
            Param::output_stream("out").build(),
        ]
    }
}
