use crate::blocks::ESDRBlock;
use crate::blocks::ESDRBlockInput;
use crate::consts;
use crate::params::Param;

use futuredsp::firdes;
use futuresdr::blocks::FirBuilder;
use futuresdr::runtime::Block;

#[derive(Clone, Copy, Default)]
pub struct Resamp2Block {}
impl ESDRBlock for Resamp2Block {
    fn name(self) -> &'static str {
        "Resamp 2"
    }

    fn params(self) -> Vec<Param> {
        vec![
            Param::input_stream("in").build(),
            Param::scalar("cutoff").initial_value(2000.0).build(),
            Param::scalar("transition").initial_value(10000.0).build(),
            Param::output_stream("out").build(),
        ]
    }

    fn block(self, input: ESDRBlockInput) -> Block {
        let cutoff = input.scalar("cutoff") / (consts::AUDIO_RATE * consts::AUDIO_MULT) as f64;
        let transition =
            input.scalar("transition") / (consts::AUDIO_RATE * consts::AUDIO_MULT) as f64;
        let audio_filter_taps = firdes::kaiser::lowpass::<f32>(cutoff, transition, 0.1);
        FirBuilder::new_resampling_with_taps::<f32, f32, _>(
            1,
            consts::AUDIO_MULT as usize,
            audio_filter_taps,
        )
    }
}
