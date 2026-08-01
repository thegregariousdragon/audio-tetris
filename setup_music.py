import os
import wave
import math
import struct

def generate_tone(filename, freq, duration_sec, bpm, waveform='square'):
    sample_rate = 44100
    num_samples = int(sample_rate * duration_sec)
    
    os.makedirs('assets/music', exist_ok=True)
    filepath = os.path.join('assets/music', filename)
    
    with wave.open(filepath, 'w') as wav_file:
        wav_file.setnchannels(1)
        wav_file.setsampwidth(2)
        wav_file.setframerate(sample_rate)
        
        # Simple beat envelope based on BPM
        beat_samples = int(sample_rate * (60.0 / bpm))
        
        for i in range(num_samples):
            t = float(i) / sample_rate
            
            # Note frequency
            if waveform == 'square':
                val = 1.0 if math.sin(2.0 * math.pi * freq * t) > 0 else -1.0
            elif waveform == 'sawtooth':
                val = 2.0 * (t * freq - math.floor(t * freq + 0.5))
            else: # sine
                val = math.sin(2.0 * math.pi * freq * t)
                
            # Envelope (pulsing to the beat)
            beat_pos = i % beat_samples
            envelope = math.exp(-3.0 * beat_pos / beat_samples)
            
            # Volume control
            sample = int(val * envelope * 16000.0)
            
            # Ensure within 16-bit bounds
            sample = max(-32768, min(32767, sample))
            wav_file.writeframes(struct.pack('<h', sample))

print("Generating EDM track...")
generate_tone('edm.wav', 65.41, 4.0, 128, 'sawtooth') # C2

print("Generating Rock track...")
generate_tone('rock.wav', 110.0, 4.0, 140, 'square') # A2

print("Generating Pop track...")
generate_tone('pop.wav', 261.63, 4.0, 120, 'sine') # C4

print("Done! Placeholder tracks created in assets/music/")
