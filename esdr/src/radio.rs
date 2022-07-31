use crate::ui::ESDRGraph;

use futuredsp::firdes;
use futuresdr::async_io;
use futuresdr::blocks::audio::AudioSink;
use futuresdr::blocks::Apply;
use futuresdr::blocks::FirBuilder;
use futuresdr::blocks::SoapySourceBuilder;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

const RATE: f64 = 1000000.0;
const AUDIO_RATE: u32 = 48000;
const AUDIO_MULT: u32 = 5;
const GAIN: f64 = 30.0;
const FREQUENCY: f64 = 90900000.0;

pub struct Radio {
    task: async_task::Task<Result<Flowgraph, anyhow::Error>>,
}

fn build_flowgraph(graph: &ESDRGraph) -> Flowgraph {
    let sample_rate = RATE as u32;
    let freq_offset = RATE / 4.0;
    println!("Frequency Offset {:?}", freq_offset);

    // Create the `Flowgraph` where the `Block`s will be added later on
    let mut fg = Flowgraph::new();

    // Create a new SoapySDR block with the given parameters
    let src = SoapySourceBuilder::new()
        .filter("")
        .freq(FREQUENCY + freq_offset)
        .sample_rate(RATE)
        .gain(GAIN)
        .build();

    // shift
    let mut last = Complex32::new(1.0, 0.0);
    let add = Complex32::from_polar(
        1.0,
        (2.0 * std::f64::consts::PI * freq_offset / RATE) as f32,
    );
    let shift = Apply::new(move |v: &Complex32| -> Complex32 {
        last *= add;
        last * v
    });

    // Downsample before demodulation
    let interp = (AUDIO_RATE * AUDIO_MULT) as usize;
    let decim = sample_rate as usize;
    println!("interp {}   decim {}", interp, decim);
    let resamp1 = FirBuilder::new_resampling::<Complex32>(interp, decim);

    // Demodulation block using the conjugate delay method
    // See https://en.wikipedia.org/wiki/Detector_(radio)#Quadrature_detector
    let mut last = Complex32::new(0.0, 0.0); // store sample x[n-1]
    let demod = Apply::new(move |v: &Complex32| -> f32 {
        let arg = (v * last.conj()).arg(); // Obtain phase of x[n] * conj(x[n-1])
        last = *v;
        arg
    });

    // Design filter for the audio and decimate by 5.
    // Ideally, this should be a FM de-emphasis filter, but the following works.
    let cutoff = 2_000.0 / (AUDIO_RATE * AUDIO_MULT) as f64;
    let transition = 10_000.0 / (AUDIO_RATE * AUDIO_MULT) as f64;
    println!("cutoff {}   transition {}", cutoff, transition);
    let audio_filter_taps = firdes::kaiser::lowpass::<f32>(cutoff, transition, 0.1);
    let resamp2 = FirBuilder::new_resampling_with_taps::<f32, f32, _>(
        1,
        AUDIO_MULT as usize,
        audio_filter_taps,
    );

    // Single-channel `AudioSink` with the downsampled rate (sample_rate / (8*5) = 48_000)
    let snk = AudioSink::new(AUDIO_RATE, 1);

    // Add all the blocks to the `Flowgraph`...
    let src = fg.add_block(src);
    let shift = fg.add_block(shift);
    let resamp1 = fg.add_block(resamp1);
    let demod = fg.add_block(demod);
    let resamp2 = fg.add_block(resamp2);
    let snk = fg.add_block(snk);

    // ... and connect the ports appropriately
    fg.connect_stream(src, "out", shift, "in").unwrap();
    fg.connect_stream(shift, "out", resamp1, "in").unwrap();
    fg.connect_stream(resamp1, "out", demod, "in").unwrap();
    fg.connect_stream(demod, "out", resamp2, "in").unwrap();
    fg.connect_stream(resamp2, "out", snk, "in").unwrap();

    return fg;
}

pub fn start(graph: &ESDRGraph) -> Radio {
    let fg = build_flowgraph(graph);
    let (res, mut _handle) = async_io::block_on(Runtime::new().start(fg));
    return Radio { task: res };
}
