// to get the packets and metadata
use opus_headers::{parse_from_path, get_opus_packets_from_path};

// to do the actual decoding
use audiopus::coder::Decoder;
use audiopus::{SampleRate, Channels};
// to write the decoded samples into a wav
use hound::{WavSpec, WavWriter};

fn main() {

	let args: Vec<String> = std::env::args().skip(1).collect();

	if 2 != args.len() {
		panic!("usage: decode <input file> <output file>\nexpected 2 parameters, got {:#?}", args);
	}
	assert!(args.len() == 2);
	
	let path = std::path::Path::new(&args[0]);
	let headers = parse_from_path(path).unwrap();

	let channel_count = headers.id.channel_count as i32;
	let sample_rate = headers.id.input_sample_rate as i32;

	let packets = get_opus_packets_from_path(path).unwrap();
	
	let wav_spec = WavSpec {
		channels: channel_count as u16,
		sample_rate: sample_rate as u32,
		bits_per_sample: 16,
		sample_format: hound::SampleFormat::Int
	};

	let decoder_sample_rate = match sample_rate {
		8000 => {SampleRate::Hz8000}
		12000 => {SampleRate::Hz12000}
		16000 => {SampleRate::Hz16000}
		24000 => {SampleRate::Hz24000}
		48000 => {SampleRate::Hz48000}
		x => {panic!("unsupported SampleRate: {}", x);}
	};

	let decoder_channels = match channel_count {
		1 => {Channels::Mono}
		2 => {Channels::Stereo}
		x => {panic!("unsupported channel count: {}", x);}
	};
	
	let mut decoder = Decoder::new(
		decoder_sample_rate,
		decoder_channels
	).unwrap();

	// use a single 64k Buffer for now
	let mut output = vec![0; 64 * 1024];
	packets.iter().fold(WavWriter::create(&args[1], wav_spec).unwrap(), |mut writer, packet| {
		let len = decoder.decode(
			Some(&packet.data[..]),
			&mut output,
			false
		).unwrap();
		
		for sample in output[..len].iter() {
			writer.write_sample(*sample).unwrap();
		}
		writer
	}).finalize().unwrap();

	
}
