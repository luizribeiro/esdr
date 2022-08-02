use crate::blocks::ESDRBlock;
use crate::blocks::ESDRBlockInput;
use crate::consts;
use crate::params::Param;

use futuresdr::blocks::SoapySourceBuilder;
use futuresdr::runtime::Block;

#[derive(Clone, Copy, Default)]
pub struct SoapySDRBlock {}
impl ESDRBlock for SoapySDRBlock {
    fn name(self) -> &'static str {
        "Soapy SDR"
    }

    fn block(self, input: ESDRBlockInput) -> Block {
        SoapySourceBuilder::new()
            .filter("")
            .freq(input.scalar("freq") + consts::FREQ_OFFSET)
            .sample_rate(consts::RATE)
            .gain(input.scalar("gain"))
            .build()
    }

    fn params(self) -> Vec<Param> {
        vec![
            Param::output_stream("out").build(),
            Param::scalar("freq")
                .initial_value(90900000.0)
                .allow_updates(true)
                .build(),
            Param::scalar("gain").initial_value(30.0).build(),
        ]
    }
}
