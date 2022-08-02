use crate::blocks::ESDRBlock;
use crate::blocks::ESDRBlockInput;
use crate::params::Param;

use futuresdr::blocks::Apply;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;

#[derive(Clone, Copy, Default)]
pub struct FMDemodulatorBlock {}
impl ESDRBlock for FMDemodulatorBlock {
    fn name(self) -> &'static str {
        "FM Demodulator"
    }

    fn params(self) -> Vec<Param> {
        vec![
            Param::input_stream("in").build(),
            Param::output_stream("out").build(),
        ]
    }

    fn block(self, _input: ESDRBlockInput) -> Block {
        let mut last = Complex32::new(0.0, 0.0); // store sample x[n-1]
        Apply::new(move |v: &Complex32| -> f32 {
            let arg = (v * last.conj()).arg(); // Obtain phase of x[n] * conj(x[n-1])
            last = *v;
            arg
        })
    }
}
