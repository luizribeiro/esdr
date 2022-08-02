use crate::blocks::ESDRBlock;
use crate::blocks::ESDRBlockInput;
use crate::consts;
use crate::param::Param;

use futuresdr::blocks::Apply;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;

#[derive(Clone, Copy, Default)]
pub struct ShiftBlock {}
impl ESDRBlock for ShiftBlock {
    fn name(self) -> &'static str {
        "Shift"
    }

    fn block(self, _input: ESDRBlockInput) -> Block {
        let mut last = Complex32::new(1.0, 0.0);
        let add = Complex32::from_polar(
            1.0,
            (2.0 * std::f64::consts::PI * consts::FREQ_OFFSET / consts::RATE) as f32,
        );
        Apply::new(move |v: &Complex32| -> Complex32 {
            last *= add;
            last * v
        })
    }

    fn params(self) -> Vec<Param> {
        vec![
            Param::input_stream("in").build(),
            Param::output_stream("out").build(),
        ]
    }
}
