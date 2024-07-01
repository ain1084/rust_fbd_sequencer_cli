use clap::Parser;
use clap::ValueEnum;
use direct_ring_buffer;
use fbd_sequencer::DataAccessor;
use fbd_sequencer::{OutputMode, PsgTrait, Sequencer};
use std::{
    fs::File,
    io::{stdout, Read, Write},
    time::Duration,
};
use tinyaudio::{run_output_device, OutputDeviceParameters};
use byteorder::{ByteOrder, LittleEndian};

mod wrapper_psg;
mod wrapper_psg_lite;

const DEFAULT_SAMPLE_RATE: u32 = 44100;
const DEFAULT_CLOCK_RATE_MHZ: f32 = 2.0;

struct Player<'a> {
    context: fbd_sequencer::PlayContext<'a>,
    producer: direct_ring_buffer::Producer<f32>,
    sample_count: usize,
}

impl<'a> Player<'a> {
    fn new(
        sequencer: &'a fbd_sequencer::Sequencer,
        psg: &'a mut dyn PsgTrait,
        producer: direct_ring_buffer::Producer<f32>,
    ) -> Self {
        let mut instance = Self {
            context: sequencer.play(psg),
            producer,
            sample_count: 0,
        };
        instance.fill_buffer();
        instance
    }

    fn fill_buffer(&mut self) -> bool {
        if !self.context.is_playing() {
            return false;
        }
        self.sample_count += self
            .producer
            .write_slices(|buf, _offset| self.context.next_samples_f32(buf), None);
        true
    }

    fn sample_count(&self) -> usize {
        self.sample_count
    }
}

fn generate_wave_file(
    sequencer: Sequencer,
    psg: &mut dyn PsgTrait,
    sample_rate: u32,
    output: &str,
) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut player = sequencer.play(psg);
    player.set_max_loop_count(Some(1));
    let mut writer = hound::WavWriter::create(&output, spec).expect("file can't create");

    let buffer_size = sample_rate as usize;
    let mut buffer: Vec<i16> = Vec::with_capacity(buffer_size);
    unsafe { buffer.set_len(buffer_size) };
    loop {
        let size = player.next_samples_i16(&mut buffer);
        if size == 0 {
            break;
        }
        buffer[..size]
            .iter()
            .for_each(|v| writer.write_sample(*v).unwrap());
        print!("{} samples\r", writer.len());
        stdout().flush().unwrap();
    }
    writer.finalize().unwrap();
    println!("");
}

fn play_audio_device(sequencer: Sequencer, psg: &mut dyn PsgTrait, sample_rate: u32) {
    let sample_buffer_count = sample_rate / 20;
    let (producer, mut consumer) =
        direct_ring_buffer::create_ring_buffer::<f32>(sample_buffer_count as usize);
    let mut player = Player::new(&sequencer, psg, producer);

    let _device = run_output_device(
        OutputDeviceParameters {
            channels_count: 1,
            sample_rate: sample_rate as usize,
            channel_sample_count: sample_buffer_count as usize,
        },
        move |buf| {
            let buf_len = buf.len();
            let written = consumer.read_slices(
                |input, offset| {
                    buf[offset..offset + input.len()].copy_from_slice(input);
                    input.len()
                },
                Some(buf_len),
            );
            buf[written..].fill(f32::default());
        },
    )
    .unwrap();

    while player.fill_buffer() {
        std::thread::sleep(Duration::from_millis(25));
        print!("{} samples\r", player.sample_count());
        stdout().flush().unwrap();
    }
    // wait for drain
    std::thread::sleep(std::time::Duration::from_millis(500));
}

struct FileDataAccessor(Vec<u8>);

impl FileDataAccessor {
    pub fn new(mut file: File) -> Self {
        let mut vec: Vec<u8> = Vec::new();
        file.read_to_end(&mut vec).unwrap();
        Self(vec)
    }
}

impl DataAccessor for FileDataAccessor {
    fn read_byte(&self, index: u16) -> u8 {
        self.0[index as usize]
    }
    fn read_short(&self, index: u16) -> u16 {
        LittleEndian::read_u16(&self.0[index as usize..])
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
#[allow(non_camel_case_types)]
enum PsgCrate {
    psg,
    psg_lite
}

#[derive(Parser)]
#[clap(name = "fbdplay", version = env!("CARGO_PKG_VERSION"), about = "FBD Music player", arg_required_else_help = true)]
struct Cli {
    /// Sets the input .fbd file
    input: String,

    /// Sets the crate for waveform generation
    #[arg(short, long, value_enum, default_value_t = PsgCrate::psg)]
    psg_crate: PsgCrate,

    /// Sets the clock rate(Mz) (ex. 2.0, 1.7897725...)
    #[arg(short, long, default_value_t = DEFAULT_CLOCK_RATE_MHZ)]
    clock_rate: f32,

    /// Sets the sample rate(Hz)
    #[arg(short, long, default_value_t = DEFAULT_SAMPLE_RATE)]
    sample_rate: u32,

    /// Sets the generate .wav file
    output: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let file_data = FileDataAccessor::new(File::open(&cli.input).unwrap());
    let sequencer = Sequencer::new(&file_data);

    const MHZ_TO_HZ: f32 = 1_000_000.0;
    let clock_rate_hz = (cli.clock_rate * MHZ_TO_HZ) as u32;
    let mut psg: Box<dyn PsgTrait> = match cli.psg_crate {
        PsgCrate::psg => Box::new(wrapper_psg::PsgWrapper::new(clock_rate_hz, cli.sample_rate)),
        PsgCrate::psg_lite => Box::new(wrapper_psg_lite::PsgWrapper::new(clock_rate_hz, cli.sample_rate)),
    };

    println!("Title: {}", String::from_utf8(sequencer.title_iter().collect::<Vec<u8>>()).unwrap());
    println!("Clock rate: {:.7}Mz", cli.clock_rate);
    println!("Sample rate: {}Hz", cli.sample_rate);
    println!("Using {:?} crate", cli.psg_crate);
    if let Some(output) = cli.output {
        // Generate .wav file
        generate_wave_file(sequencer, psg.as_mut(), cli.sample_rate, &output);
    } else {
        // Play through audio device
        play_audio_device(sequencer, psg.as_mut(), cli.sample_rate);
    }
}
