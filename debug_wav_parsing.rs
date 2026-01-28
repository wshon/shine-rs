use hound;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "tests/audio/voice-recorder-testing-1-2-3-sound-file.wav";
    
    // Method 1: Using hound library (current Rust approach)
    println!("=== Using hound library ===");
    let mut reader = hound::WavReader::open(file_path)?;
    let spec = reader.spec();
    println!("Spec: {:?}", spec);
    
    let samples: Result<Vec<i16>, _> = reader.samples::<i16>().collect();
    let hound_samples = samples?;
    println!("Hound - Total samples: {}", hound_samples.len());
    println!("Hound - First 16 samples: {:?}", &hound_samples[..16.min(hound_samples.len())]);
    
    // Method 2: Manual parsing (similar to Shine's approach)
    println!("\n=== Manual parsing (Shine-like) ===");
    let mut file = File::open(file_path)?;
    let mut buffer = [0u8; 4];
    
    // Read RIFF header
    file.read_exact(&mut buffer)?;
    println!("RIFF: {:?}", std::str::from_utf8(&buffer));
    
    file.read_exact(&mut buffer)?;
    let file_size = u32::from_le_bytes(buffer);
    println!("File size: {}", file_size);
    
    file.read_exact(&mut buffer)?;
    println!("WAVE: {:?}", std::str::from_utf8(&buffer));
    
    // Find data chunk (skip other chunks like LIST)
    let mut data_offset = None;
    let mut data_size = 0;
    
    loop {
        if file.read_exact(&mut buffer).is_err() {
            break;
        }
        let chunk_id = buffer;
        
        file.read_exact(&mut buffer)?;
        let chunk_size = u32::from_le_bytes(buffer);
        
        println!("Found chunk: {:?}, size: {}", std::str::from_utf8(&chunk_id), chunk_size);
        
        if &chunk_id == b"data" {
            data_offset = Some(file.stream_position()?);
            data_size = chunk_size;
            break;
        } else {
            // Skip chunk (with padding if odd size)
            let skip_size = if chunk_size % 2 == 0 { chunk_size } else { chunk_size + 1 };
            file.seek(SeekFrom::Current(skip_size as i64))?;
        }
    }
    
    if let Some(offset) = data_offset {
        println!("Data chunk found at offset: {}, size: {}", offset, data_size);
        
        // Read first 32 bytes of PCM data
        file.seek(SeekFrom::Start(offset))?;
        let mut pcm_buffer = vec![0u8; 32.min(data_size as usize)];
        file.read_exact(&mut pcm_buffer)?;
        
        // Convert to i16 samples
        let manual_samples: Vec<i16> = pcm_buffer
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        
        println!("Manual - First 16 samples: {:?}", manual_samples);
        
        // Compare with hound
        let matches = hound_samples[..manual_samples.len().min(hound_samples.len())] == manual_samples[..manual_samples.len().min(hound_samples.len())];
        println!("First samples match: {}", matches);
        
        if !matches {
            println!("Difference found!");
            for (i, (h, m)) in hound_samples.iter().zip(manual_samples.iter()).enumerate() {
                if h != m {
                    println!("  Sample {}: hound={}, manual={}", i, h, m);
                    if i > 10 { break; }
                }
            }
        }
    }
    
    Ok(())
}