#!/usr/bin/env python3
"""
完整的MP3编码器参考数据生成器

这个脚本自动化地：
1. 使用Shine编码器的JSON调试模式处理指定的音频文件
2. 解析JSON调试输出提取MDCT系数、量化参数和比特流数据
3. 生成包含真实Shine参考值的JSON测试数据文件
4. 计算MP3文件哈希值用于验证

使用方法:
    python scripts/generate_reference_data.py
"""

import os
import sys
import json
import re
import hashlib
import subprocess
import wave
from datetime import datetime
from pathlib import Path

# 测试配置列表
TEST_CONFIGS = [
    {
        "name": "sample-3s_128k_3f_real",
        "audio_file": "tests/audio/sample-3s.wav",
        "bitrate": 128,
        "frames": 3,
        "description": "3秒音频样本，128kbps，3帧 - 真实Shine数据"
    },
    {
        "name": "voice_recorder_128k_3f_real", 
        "audio_file": "tests/audio/voice-recorder-testing-1-2-3-sound-file.wav",
        "bitrate": 128,
        "frames": 3,
        "description": "语音录音，128kbps，3帧 - 真实Shine数据"
    },
    {
        "name": "free_test_data_128k_3f_real",
        "audio_file": "tests/audio/Free_Test_Data_500KB_WAV.wav", 
        "bitrate": 128,
        "frames": 3,
        "description": "免费测试数据，128kbps，3帧 - 真实Shine数据"
    },
    {
        "name": "sample-3s_192k_3f_real",
        "audio_file": "tests/audio/sample-3s.wav",
        "bitrate": 192,
        "frames": 3,
        "description": "3秒音频样本，192kbps，3帧 - 真实Shine数据"
    }
]

def calculate_sha256(file_path):
    """计算文件的SHA256哈希值"""
    sha256_hash = hashlib.sha256()
    try:
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                sha256_hash.update(chunk)
        return sha256_hash.hexdigest().upper()
    except Exception as e:
        print(f"计算哈希值时出错 {file_path}: {e}")
        return ""

def read_wav_metadata(wav_path):
    """读取WAV文件元数据"""
    try:
        with wave.open(wav_path, 'rb') as wav_file:
            channels = wav_file.getnchannels()
            sample_rate = wav_file.getframerate()
            frames = wav_file.getnframes()
            sample_width = wav_file.getsampwidth()
            
            return {
                "channels": channels,
                "sample_rate": sample_rate,
                "frames": frames,
                "sample_width": sample_width
            }
    except Exception as e:
        print(f"读取WAV文件时出错 {wav_path}: {e}")
        return None

def run_shine_with_json_debug(audio_file, output_file, bitrate, max_frames):
    """使用JSON调试模式运行Shine编码器"""
    shine_exe = "ref/shine/shineenc.exe"
    
    if not os.path.exists(shine_exe):
        print(f"错误：找不到Shine编码器 {shine_exe}")
        return None, None
    
    if not os.path.exists(audio_file):
        print(f"错误：找不到音频文件 {audio_file}")
        return None, None
    
    # 转换为绝对路径
    audio_file_abs = os.path.abspath(audio_file)
    
    # 设置环境变量
    env = os.environ.copy()
    env["SHINE_JSON_DEBUG"] = "1"  # 启用JSON调试模式
    env["SHINE_MAX_FRAMES"] = str(max_frames)  # 限制帧数
    
    # 运行Shine编码器
    cmd = [shine_exe, "-b", str(bitrate), audio_file_abs, output_file]
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, 
                              cwd="ref/shine", env=env, encoding='utf-8', errors='replace')
        
        if result.returncode == 0:
            print(f"✓ Shine编码成功: {output_file}")
            # 合并stdout和stderr获取调试输出
            debug_output = result.stdout + result.stderr
            return debug_output, f"ref/shine/{output_file}"
        else:
            print(f"✗ Shine编码失败:")
            print(f"  命令: {' '.join(cmd)}")
            print(f"  返回码: {result.returncode}")
            print(f"  Stdout: {result.stdout}")
            print(f"  Stderr: {result.stderr}")
            return None, None
    except Exception as e:
        print(f"运行Shine编码器时出错: {e}")
        return None, None

def parse_json_debug_output(debug_output):
    """解析Shine的JSON调试输出"""
    frames = {}
    
    for line in debug_output.split('\n'):
        line = line.strip()
        if not line.startswith('{"type":'):
            continue
            
        try:
            data = json.loads(line)
            frame_num = data.get('frame')
            if not frame_num:
                continue
                
            # 初始化帧数据结构
            if frame_num not in frames:
                frames[frame_num] = {
                    'frame_number': frame_num,
                    'mdct_coefficients': {
                        'coefficients': [0, 0, 0],  # k=17, k=16, k=15
                        'l3_sb_sample': [0]
                    },
                    'quantization': {
                        'xrmax': 0,
                        'max_bits': 0,
                        'part2_3_length': 0,
                        'quantizer_step_size': 0,
                        'global_gain': 0
                    },
                    'bitstream': {
                        'padding': 0,
                        'bits_per_frame': 0,
                        'written': 0,
                        'slot_lag': 0.0
                    }
                }
            
            current_frame = frames[frame_num]
            
            # 解析不同类型的JSON数据
            if data['type'] == 'mdct_coeff':
                k = data.get('k')
                value = data.get('value')
                if k == 17:
                    current_frame['mdct_coefficients']['coefficients'][0] = value
                elif k == 16:
                    current_frame['mdct_coefficients']['coefficients'][1] = value
                elif k == 15:
                    current_frame['mdct_coefficients']['coefficients'][2] = value
            
            elif data['type'] == 'l3_sb_sample':
                samples = data.get('samples', [])
                if samples:
                    current_frame['mdct_coefficients']['l3_sb_sample'] = [samples[0]]
            
            elif data['type'] == 'quantization_xrmax':
                # 只取第一个通道第一个颗粒的数据作为代表
                if data.get('ch') == 0 and data.get('gr') == 0:
                    current_frame['quantization']['xrmax'] = data.get('xrmax', 0)
            
            elif data['type'] == 'quantization_max_bits':
                if data.get('ch') == 0 and data.get('gr') == 0:
                    current_frame['quantization']['max_bits'] = data.get('max_bits', 0)
            
            elif data['type'] == 'quantization_part2_3_length':
                if data.get('ch') == 0 and data.get('gr') == 0:
                    current_frame['quantization']['part2_3_length'] = data.get('part2_3_length', 0)
            
            elif data['type'] == 'quantization_global_gain':
                if data.get('ch') == 0 and data.get('gr') == 0:
                    current_frame['quantization']['quantizer_step_size'] = data.get('quantizer_step_size', 0)
                    current_frame['quantization']['global_gain'] = data.get('global_gain', 0)
            
            elif data['type'] == 'bitstream_params':
                current_frame['bitstream']['padding'] = data.get('padding', 0)
                current_frame['bitstream']['bits_per_frame'] = data.get('bits_per_frame', 0)
                current_frame['bitstream']['slot_lag'] = data.get('slot_lag', 0.0)
            
            elif data['type'] == 'frame_complete':
                current_frame['bitstream']['written'] = data.get('written', 0)
                
        except json.JSONDecodeError:
            continue
    
    # 转换为排序列表
    frame_list = []
    for frame_num in sorted(frames.keys()):
        frame_list.append(frames[frame_num])
    
    return frame_list

def parse_text_debug_output(debug_output):
    """解析文本调试输出（备用方案）"""
    frames = {}
    current_frame = None
    
    for line in debug_output.split('\n'):
        line = line.strip()
        
        # 解析帧特定的调试输出
        if "[SHINE DEBUG Frame" in line:
            # 提取帧号
            frame_match = re.search(r'Frame (\d+)', line)
            if frame_match:
                frame_num = int(frame_match.group(1))
                
                # 初始化新帧
                if frame_num not in frames:
                    frames[frame_num] = {
                        'frame_number': frame_num,
                        'mdct_coefficients': {
                            'coefficients': [0, 0, 0],
                            'l3_sb_sample': [0]
                        },
                        'quantization': {
                            'xrmax': 0,
                            'max_bits': 0,
                            'part2_3_length': 0,
                            'quantizer_step_size': 0,
                            'global_gain': 0
                        },
                        'bitstream': {
                            'padding': 0,
                            'bits_per_frame': 0,
                            'written': 0,
                            'slot_lag': 0.0
                        }
                    }
                
                current_frame = frames[frame_num]
            
            # 解析具体的调试值（现有的文本解析逻辑）
            if "MDCT coeff band 0 k 17:" in line:
                coeff_match = re.search(r'k 17: (-?\d+)', line)
                if coeff_match:
                    current_frame['mdct_coefficients']['coefficients'][0] = int(coeff_match.group(1))
            
            elif "MDCT coeff band 0 k 16:" in line:
                coeff_match = re.search(r'k 16: (-?\d+)', line)
                if coeff_match:
                    current_frame['mdct_coefficients']['coefficients'][1] = int(coeff_match.group(1))
            
            elif "MDCT coeff band 0 k 15:" in line:
                coeff_match = re.search(r'k 15: (-?\d+)', line)
                if coeff_match:
                    current_frame['mdct_coefficients']['coefficients'][2] = int(coeff_match.group(1))
            
            elif "l3_sb_sample[0][1][0]: first 8 bands:" in line:
                sample_match = re.search(r'first 8 bands: \[(-?\d+)', line)
                if sample_match:
                    current_frame['mdct_coefficients']['l3_sb_sample'] = [
                        int(sample_match.group(1))
                    ]
            
            elif "ch=0, gr=0: xrmax=" in line:
                xrmax_match = re.search(r'xrmax=(-?\d+)', line)
                if xrmax_match:
                    current_frame['quantization']['xrmax'] = int(xrmax_match.group(1))
            
            elif "ch=0, gr=0: max_bits=" in line:
                bits_match = re.search(r'max_bits=(-?\d+)', line)
                if bits_match:
                    current_frame['quantization']['max_bits'] = int(bits_match.group(1))
            
            elif "ch=0, gr=0: part2_3_length=" in line:
                length_match = re.search(r'part2_3_length=(-?\d+)', line)
                if length_match:
                    current_frame['quantization']['part2_3_length'] = int(length_match.group(1))
            
            elif "ch=0, gr=0: quantizerStepSize=" in line:
                step_match = re.search(r'quantizerStepSize=(-?\d+), global_gain=(-?\d+)', line)
                if step_match:
                    current_frame['quantization']['quantizer_step_size'] = int(step_match.group(1))
                    current_frame['quantization']['global_gain'] = int(step_match.group(2))
            
            elif "padding=" in line and "bits_per_frame=" in line and "slot_lag=" in line:
                padding_match = re.search(r'padding=(-?\d+)', line)
                bits_match = re.search(r'bits_per_frame=(-?\d+)', line)
                lag_match = re.search(r'slot_lag=(-?\d+\.?\d*)', line)
                
                if padding_match:
                    current_frame['bitstream']['padding'] = int(padding_match.group(1))
                if bits_match:
                    current_frame['bitstream']['bits_per_frame'] = int(bits_match.group(1))
                if lag_match:
                    current_frame['bitstream']['slot_lag'] = float(lag_match.group(1))
            
            elif "written=" in line and "bytes" in line:
                written_match = re.search(r'written=(-?\d+)', line)
                if written_match:
                    current_frame['bitstream']['written'] = int(written_match.group(1))
    
    # 转换为排序列表
    frame_list = []
    for frame_num in sorted(frames.keys()):
        frame_list.append(frames[frame_num])
    
    return frame_list

def parse_shine_debug_output(debug_output):
    """解析Shine调试输出，优先使用JSON格式"""
    
    # 检查输出是否包含JSON
    is_json_output = any(line.strip().startswith('{"type":') for line in debug_output.split('\n'))
    
    if is_json_output:
        print("  使用JSON调试输出解析")
        return parse_json_debug_output(debug_output)
    else:
        print("  使用文本调试输出解析（备用）")
        return parse_text_debug_output(debug_output)

def generate_test_data_structure(config, wav_metadata, mp3_file, frames):
    """生成测试数据结构"""
    
    # 计算文件大小和哈希值
    file_size = os.path.getsize(mp3_file) if os.path.exists(mp3_file) else 0
    file_hash = calculate_sha256(mp3_file) if os.path.exists(mp3_file) else ""
    
    # 根据通道数确定立体声模式
    stereo_mode = 3 if wav_metadata["channels"] == 1 else 0  # 3=单声道, 0=立体声
    
    test_data = {
        "metadata": {
            "name": f"test_case_{config['name']}_{wav_metadata['sample_rate']}hz_{wav_metadata['channels']}ch_{config['bitrate']}kbps",
            "input_file": config["audio_file"],
            "expected_output_size": file_size,
            "expected_hash": file_hash,
            "created_at": datetime.now().isoformat() + "Z",
            "description": config["description"],
            "generated_by": "Shine参考实现，带JSON调试输出"
        },
        "config": {
            "sample_rate": wav_metadata["sample_rate"],
            "channels": wav_metadata["channels"],
            "bitrate": config["bitrate"],
            "stereo_mode": stereo_mode,
            "mpeg_version": 3  # MPEG-I
        },
        "frames": frames[:config["frames"]]  # 限制到指定的帧数
    }
    
    return test_data

def main():
    """主函数，生成所有参考测试数据"""
    
    print("MP3编码器参考数据生成器")
    print("=" * 50)
    print("使用Shine编码器的JSON调试输出生成参考数据")
    print()
    
    # 创建输出目录
    output_dir = Path("tests/pipeline_data")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    success_count = 0
    
    for config in TEST_CONFIGS:
        print(f"处理: {config['name']}")
        print(f"音频文件: {config['audio_file']}")
        print(f"比特率: {config['bitrate']}kbps，帧数: {config['frames']}")
        
        # 检查音频文件是否存在
        if not os.path.exists(config["audio_file"]):
            print(f"⚠ 跳过 {config['name']} - 找不到音频文件")
            continue
        
        # 读取WAV元数据
        wav_metadata = read_wav_metadata(config["audio_file"])
        if not wav_metadata:
            print(f"⚠ 跳过 {config['name']} - 无法读取WAV元数据")
            continue
        
        print(f"WAV信息: {wav_metadata['channels']}声道, {wav_metadata['sample_rate']}Hz")
        
        # 使用Shine生成MP3并捕获调试输出
        mp3_filename = f"{config['name']}.mp3"
        debug_output, mp3_file = run_shine_with_json_debug(
            config["audio_file"], mp3_filename, config["bitrate"], config["frames"]
        )
        
        if debug_output is None or mp3_file is None:
            print(f"✗ 为 {config['name']} 生成MP3失败")
            continue
        
        # 解析调试数据
        frames = parse_shine_debug_output(debug_output)
        
        if not frames:
            print(f"✗ 为 {config['name']} 提取调试数据失败")
            continue
        
        # 生成测试数据结构
        test_data = generate_test_data_structure(config, wav_metadata, mp3_file, frames)
        
        # 保存测试数据
        json_file = output_dir / f"{config['name']}.json"
        with open(json_file, 'w', encoding='utf-8') as f:
            json.dump(test_data, f, indent=2, ensure_ascii=False)
        
        print(f"✓ 生成测试数据: {json_file}")
        print(f"  输出大小: {test_data['metadata']['expected_output_size']} 字节")
        print(f"  SHA256: {test_data['metadata']['expected_hash'][:16]}...")
        print(f"  提取帧数: {len(frames)}")
        
        # 打印样本数据用于验证
        if frames:
            frame1 = frames[0]
            print(f"  第1帧样本数据:")
            print(f"    MDCT系数: {frame1['mdct_coefficients']['coefficients']}")
            print(f"    l3_sb_sample: {frame1['mdct_coefficients']['l3_sb_sample']}")
            print(f"    xrmax: {frame1['quantization']['xrmax']}")
            print(f"    global_gain: {frame1['quantization']['global_gain']}")
            print(f"    padding: {frame1['bitstream']['padding']}")
            print(f"    written: {frame1['bitstream']['written']}")
        
        success_count += 1
        print()
    
    print("=" * 50)
    print(f"生成了 {success_count}/{len(TEST_CONFIGS)} 个参考数据文件")
    
    if success_count > 0:
        print("\n下一步:")
        print("1. 运行集成测试: cargo test test_complete_encoding_pipeline")
        print("2. 验证Rust实现与Shine参考数据匹配")
        print("3. 所有测试应该通过，哈希值完全一致")
    
    return success_count > 0

if __name__ == "__main__":
    if main():
        sys.exit(0)
    else:
        sys.exit(1)