use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use rodio;

use std::path::PathBuf;
use std::io::BufReader;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::fs::File;


/*
struct StreamStruct{
    output_device : Refrodio::Device,

    
}

fn playFile(filepath : Path){
    let file_path_string = file_path.to_str().unwrap();
    let file = std::fs::File::open(&file_path).unwrap();
    let file2 = std::fs::File::open(&file_path).unwrap();   
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
    sounds_only_sink.append(rodio::Decoder::new(BufReader::new(file2)).unwrap());
    println!("Playing sound: {}", file_path_string);

}
*/

fn play_thread(rx : Receiver<&PathBuf>, loop_sink : rodio::Sink, sounds_only_sink : rodio::Sink){

    loop{

        let file_path : &PathBuf = rx.recv().unwrap();

        let file_path_string = file_path.to_str().unwrap();
        let file = std::fs::File::open(&file_path).unwrap();
        let file2 = std::fs::File::open(&file_path).unwrap();   
        loop_sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
        sounds_only_sink.append(rodio::Decoder::new(BufReader::new(file2)).unwrap());
        println!("Playing sound: {}", file_path_string);

    }

}


pub fn sound_thread(rx: Receiver<&PathBuf>, input_device_index: usize, output_device_index: usize, loop_device_index: usize){

    let host = cpal::default_host();

    let devices: Vec<_> = host
        .devices()
        .expect("No available sound devices")
        .collect();
    let input_device = devices
        .get(input_device_index)
        .expect("invalid input device specified");
    let loop_device = devices
        .get(loop_device_index)
        .expect("invalid loop device specified");
    let output_device = devices      
        .get(output_device_index)
        .expect("invalid output device specified");
    println!("  Using Devices: ");
    println!(
        "Input:  {}. \"{}\"",
        input_device_index,
        input_device.name().unwrap()
    );
    println!(
        "Output:  {}. \"{}\"",
        output_device_index,
        output_device.name().unwrap()
    );
    println!(
        "Loopback:  {}. \"{}\"",
        loop_device_index,
        loop_device.name().unwrap()
    );

    // Input configs
    if let Ok(conf) = input_device.default_input_format() {
        println!("    Default input stream format:\n      {:?}", conf);
    }

    
    let loop_sink = rodio::Sink::new(&loop_device);
    let sounds_only_sink = rodio::Sink::new(&output_device);


    std::thread::spawn(move || {
        play_thread(rx, loop_sink, sounds_only_sink);
    });
    

    let sink = rodio::Sink::new(&loop_device);

    let host = cpal::default_host();
    let event_loop = host.event_loop();

    let input_format = input_device.default_input_format().unwrap();

    // // Build streams.
    println!(
        "Attempting to build input stream with `{:?}`.",
        input_format
    );
    let input_stream_id = event_loop
        .build_input_stream(&input_device, &input_format)
        .unwrap();
    println!("Successfully built input stream.");
    

    event_loop
        .play_stream(input_stream_id.clone())
        .expect("Fail loopStream");

    event_loop.run(move |id, result| {
        let data = match result {
            Ok(data) => data,
            Err(err) => {
                eprintln!("an error occurred on stream {:?}: {}", id, err);
                return;
            }
        };

        match data {
            cpal::StreamData::Input {
                buffer: cpal::UnknownTypeInputBuffer::F32(buffer),
            } => {
                let mut new_buffer = Vec::new();
                for sample in buffer.iter() {
                    let sample = cpal::Sample::to_f32(sample);
                    new_buffer.push(sample);
                }
                let buffer = rodio::buffer::SamplesBuffer::new(
                    input_format.channels,
                    input_format.sample_rate.0,
                    new_buffer,
                );
                sink.append(buffer);
            }
            _ => panic!("we're expecting f32 data"),
        }
    });
    
    
}