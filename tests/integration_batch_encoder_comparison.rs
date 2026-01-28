//! 批量编码器对比集成测试
//!
//! 这个测试模块对tests/audio目录下的所有WAV文件进行批量编码对比，
//! 验证Rust版本编码器与Shine参考实现的一致性。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// 编码器类型
#[derive(Debug, Clone, Copy, PartialEq)]
enum EncoderType {
    Rust,
    Shine,
}

/// 编码结果
#[derive(Debug)]
struct EncodingResult {
    success: bool,
    duration_ms: u128,
    output_size: u64,
    error_message: Option<String>,
}

/// 文件对比结果
#[derive(Debug)]
struct ComparisonResult {
    input_file: PathBuf,
    input_size: u64,
    rust_result: EncodingResult,
    shine_result: EncodingResult,
    size_difference: i64,
    size_difference_percent: f64,
}

/// 批量测试统计
#[derive(Debug)]
struct BatchTestStats {
    total_files: usize,
    rust_success_count: usize,
    shine_success_count: usize,
    both_success_count: usize,
    total_rust_time_ms: u128,
    total_shine_time_ms: u128,
    average_size_difference_percent: f64,
    max_size_difference_percent: f64,
    identical_files_count: usize,
}

/// 查找编码器可执行文件
fn find_encoders() -> (Option<PathBuf>, Option<PathBuf>) {
    let rust_paths = [
        "target/release/shine-rs-cli.exe",
        "target/debug/shine-rs-cli.exe",
        "target/release/shine-rs-cli",
        "target/debug/shine-rs-cli",
    ];
    
    let shine_paths = [
        "ref/shine/shineenc.exe",
        "ref/shine/build/shineenc.exe",
        "ref/shine/shineenc",
        "ref/shine/build/shineenc",
    ];
    
    let rust_exe = rust_paths.iter()
        .map(PathBuf::from)
        .find(|p| p.exists());
    
    let shine_exe = shine_paths.iter()
        .map(PathBuf::from)
        .find(|p| p.exists());
    
    (rust_exe, shine_exe)
}

/// 运行编码器
fn run_encoder(
    encoder_path: &Path,
    input_file: &Path,
    output_file: &Path,
    options: &[&str],
) -> EncodingResult {
    let start_time = Instant::now();
    
    let mut cmd = Command::new(encoder_path);
    cmd.args(options)
       .arg(input_file)
       .arg(output_file);
    
    match cmd.output() {
        Ok(output) => {
            let duration = start_time.elapsed().as_millis();
            let success = output.status.success();
            
            let output_size = if success && output_file.exists() {
                fs::metadata(output_file).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };
            
            let error_message = if !success {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            } else {
                None
            };
            
            EncodingResult {
                success,
                duration_ms: duration,
                output_size,
                error_message,
            }
        }
        Err(e) => EncodingResult {
            success: false,
            duration_ms: start_time.elapsed().as_millis(),
            output_size: 0,
            error_message: Some(e.to_string()),
        },
    }
}

/// 获取文件大小
fn get_file_size(path: &Path) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

/// 对比单个文件的编码结果
fn compare_file_encoding(
    input_file: &Path,
    rust_exe: &Path,
    shine_exe: &Path,
    options: &[&str],
) -> ComparisonResult {
    let input_size = get_file_size(input_file);
    
    // 生成输出文件名，包含编码选项以避免冲突
    let base_name = input_file.file_stem().unwrap().to_string_lossy();
    let output_dir = input_file.parent().unwrap();
    
    // 根据选项生成唯一的文件名后缀
    let options_suffix = if options.is_empty() {
        "default".to_string()
    } else {
        options.join("_").replace("-", "")
    };
    
    let rust_output = output_dir.join(format!("{}_rust_{}.mp3", base_name, options_suffix));
    let shine_output = output_dir.join(format!("{}_shine_{}.mp3", base_name, options_suffix));
    
    // 清理可能存在的旧文件
    let _ = fs::remove_file(&rust_output);
    let _ = fs::remove_file(&shine_output);
    
    // 运行编码器
    let rust_result = run_encoder(rust_exe, input_file, &rust_output, options);
    let shine_result = run_encoder(shine_exe, input_file, &shine_output, options);
    
    // 计算大小差异
    let size_difference = rust_result.output_size as i64 - shine_result.output_size as i64;
    let size_difference_percent = if shine_result.output_size > 0 {
        (size_difference.abs() as f64 / shine_result.output_size as f64) * 100.0
    } else {
        0.0
    };
    
    ComparisonResult {
        input_file: input_file.to_path_buf(),
        input_size,
        rust_result,
        shine_result,
        size_difference,
        size_difference_percent,
    }
}

/// 计算批量测试统计
fn calculate_batch_stats(results: &[ComparisonResult]) -> BatchTestStats {
    let total_files = results.len();
    let rust_success_count = results.iter().filter(|r| r.rust_result.success).count();
    let shine_success_count = results.iter().filter(|r| r.shine_result.success).count();
    let both_success_count = results.iter()
        .filter(|r| r.rust_result.success && r.shine_result.success)
        .count();
    
    let total_rust_time_ms: u128 = results.iter()
        .map(|r| r.rust_result.duration_ms)
        .sum();
    
    let total_shine_time_ms: u128 = results.iter()
        .map(|r| r.shine_result.duration_ms)
        .sum();
    
    let successful_results: Vec<_> = results.iter()
        .filter(|r| r.rust_result.success && r.shine_result.success)
        .collect();
    
    let average_size_difference_percent = if !successful_results.is_empty() {
        successful_results.iter()
            .map(|r| r.size_difference_percent)
            .sum::<f64>() / successful_results.len() as f64
    } else {
        0.0
    };
    
    let max_size_difference_percent = successful_results.iter()
        .map(|r| r.size_difference_percent)
        .fold(0.0, f64::max);
    
    let identical_files_count = successful_results.iter()
        .filter(|r| r.size_difference == 0)
        .count();
    
    BatchTestStats {
        total_files,
        rust_success_count,
        shine_success_count,
        both_success_count,
        total_rust_time_ms,
        total_shine_time_ms,
        average_size_difference_percent,
        max_size_difference_percent,
        identical_files_count,
    }
}

/// 查找所有WAV文件
fn find_wav_files(audio_dir: &Path) -> Vec<PathBuf> {
    let mut wav_files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(audio_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wav") {
                wav_files.push(path);
            }
        }
    }
    
    wav_files.sort();
    wav_files
}

/// 批量编码器对比测试
fn run_batch_comparison_test(bitrate: Option<u32>, joint_stereo: bool) -> BatchTestStats {
    // 查找编码器
    let (rust_exe, shine_exe) = find_encoders();
    
    let rust_exe = rust_exe.expect("找不到Rust编码器，请运行: cargo build --release");
    let shine_exe = shine_exe.expect("找不到Shine编码器，请构建Shine");
    
    // 查找音频文件
    let audio_dir = Path::new("tests/audio");
    let wav_files = find_wav_files(audio_dir);
    
    assert!(!wav_files.is_empty(), "在tests/audio目录中找不到WAV文件");
    
    // 构建编码选项
    let mut options = Vec::new();
    if let Some(br) = bitrate {
        options.push("-b");
        options.push(Box::leak(br.to_string().into_boxed_str()));
    }
    if joint_stereo {
        options.push("-j");
    }
    
    // 批量编码对比
    let mut results = Vec::new();
    
    for wav_file in &wav_files {
        println!("测试文件: {}", wav_file.display());
        
        let result = compare_file_encoding(
            wav_file,
            &rust_exe,
            &shine_exe,
            &options,
        );
        
        // 打印单个文件结果
        if result.rust_result.success && result.shine_result.success {
            println!("  ✅ 编码成功");
            println!("    Rust:  {}ms, {} bytes", 
                     result.rust_result.duration_ms, 
                     result.rust_result.output_size);
            println!("    Shine: {}ms, {} bytes", 
                     result.shine_result.duration_ms, 
                     result.shine_result.output_size);
            println!("    大小差异: {} bytes ({:.2}%)", 
                     result.size_difference, 
                     result.size_difference_percent);
        } else {
            if !result.rust_result.success {
                println!("  ❌ Rust编码失败: {:?}", result.rust_result.error_message);
            }
            if !result.shine_result.success {
                println!("  ❌ Shine编码失败: {:?}", result.shine_result.error_message);
            }
        }
        
        results.push(result);
    }
    
    calculate_batch_stats(&results)
}

#[test]
fn test_batch_encoding_comparison_default() {
    let stats = run_batch_comparison_test(None, false);
    
    println!("\n=== 批量编码对比测试结果 (默认设置) ===");
    println!("总文件数: {}", stats.total_files);
    println!("Rust成功: {}/{} ({:.1}%)", 
             stats.rust_success_count, 
             stats.total_files,
             stats.rust_success_count as f64 / stats.total_files as f64 * 100.0);
    println!("Shine成功: {}/{} ({:.1}%)", 
             stats.shine_success_count, 
             stats.total_files,
             stats.shine_success_count as f64 / stats.total_files as f64 * 100.0);
    println!("双方成功: {}/{} ({:.1}%)", 
             stats.both_success_count, 
             stats.total_files,
             stats.both_success_count as f64 / stats.total_files as f64 * 100.0);
    
    if stats.both_success_count > 0 {
        println!("\n性能对比:");
        println!("  总Rust时间:  {}ms", stats.total_rust_time_ms);
        println!("  总Shine时间: {}ms", stats.total_shine_time_ms);
        
        if stats.total_rust_time_ms > 0 && stats.total_shine_time_ms > 0 {
            let speedup = stats.total_shine_time_ms as f64 / stats.total_rust_time_ms as f64;
            if speedup > 1.0 {
                println!("  Rust比Shine快 {:.1}x", speedup);
            } else {
                println!("  Rust比Shine慢 {:.1}x", 1.0 / speedup);
            }
        }
        
        println!("\n文件大小对比:");
        println!("  平均差异: {:.2}%", stats.average_size_difference_percent);
        println!("  最大差异: {:.2}%", stats.max_size_difference_percent);
        println!("  完全相同: {}/{} ({:.1}%)", 
                 stats.identical_files_count,
                 stats.both_success_count,
                 stats.identical_files_count as f64 / stats.both_success_count as f64 * 100.0);
    }
    
    // 断言：至少80%的文件应该成功编码
    assert!(
        stats.rust_success_count as f64 / stats.total_files as f64 >= 0.8,
        "Rust编码器成功率过低: {}/{}",
        stats.rust_success_count,
        stats.total_files
    );
    
    assert!(
        stats.shine_success_count as f64 / stats.total_files as f64 >= 0.8,
        "Shine编码器成功率过低: {}/{}",
        stats.shine_success_count,
        stats.total_files
    );
    
    // 断言：对于成功编码的文件，平均大小差异应该小于5%
    if stats.both_success_count > 0 {
        assert!(
            stats.average_size_difference_percent < 5.0,
            "平均文件大小差异过大: {:.2}%",
            stats.average_size_difference_percent
        );
    }
}

#[test]
fn test_batch_encoding_comparison_192kbps() {
    let stats = run_batch_comparison_test(Some(192), false);
    
    println!("\n=== 批量编码对比测试结果 (192kbps) ===");
    println!("总文件数: {}", stats.total_files);
    println!("双方成功: {}/{}", stats.both_success_count, stats.total_files);
    
    // 断言：至少70%的文件应该成功编码（高比特率可能对某些文件有限制）
    assert!(
        stats.both_success_count as f64 / stats.total_files as f64 >= 0.7,
        "192kbps编码成功率过低: {}/{}",
        stats.both_success_count,
        stats.total_files
    );
    
    if stats.both_success_count > 0 {
        assert!(
            stats.average_size_difference_percent < 5.0,
            "192kbps平均文件大小差异过大: {:.2}%",
            stats.average_size_difference_percent
        );
    }
}

#[test]
fn test_batch_encoding_comparison_joint_stereo() {
    let stats = run_batch_comparison_test(Some(128), true);
    
    println!("\n=== 批量编码对比测试结果 (128kbps Joint Stereo) ===");
    println!("总文件数: {}", stats.total_files);
    println!("双方成功: {}/{}", stats.both_success_count, stats.total_files);
    
    // 断言：至少70%的文件应该成功编码
    assert!(
        stats.both_success_count as f64 / stats.total_files as f64 >= 0.7,
        "Joint Stereo编码成功率过低: {}/{}",
        stats.both_success_count,
        stats.total_files
    );
    
    if stats.both_success_count > 0 {
        assert!(
            stats.average_size_difference_percent < 5.0,
            "Joint Stereo平均文件大小差异过大: {:.2}%",
            stats.average_size_difference_percent
        );
    }
}

#[test]
#[ignore] // 标记为ignore，因为这是一个长时间运行的测试
fn test_batch_encoding_comparison_comprehensive() {
    println!("\n=== 综合批量编码对比测试 ===");
    
    let test_configs = [
        (Some(64), false, "64kbps Stereo"),
        (Some(128), false, "128kbps Stereo"),
        (Some(192), false, "192kbps Stereo"),
        (Some(128), true, "128kbps Joint Stereo"),
        (Some(192), true, "192kbps Joint Stereo"),
    ];
    
    let mut all_results = HashMap::new();
    
    for (bitrate, joint_stereo, description) in &test_configs {
        println!("\n--- 测试配置: {} ---", description);
        let stats = run_batch_comparison_test(*bitrate, *joint_stereo);
        
        println!("成功率: {:.1}%", 
                 stats.both_success_count as f64 / stats.total_files as f64 * 100.0);
        println!("平均大小差异: {:.2}%", stats.average_size_difference_percent);
        
        all_results.insert(description, stats);
    }
    
    // 综合断言：所有配置的平均成功率应该大于70%
    let overall_success_rate: f64 = all_results.values()
        .map(|stats| stats.both_success_count as f64 / stats.total_files as f64)
        .sum::<f64>() / all_results.len() as f64;
    
    assert!(
        overall_success_rate >= 0.7,
        "综合编码成功率过低: {:.1}%",
        overall_success_rate * 100.0
    );
    
    println!("\n=== 综合测试总结 ===");
    println!("总体平均成功率: {:.1}%", overall_success_rate * 100.0);
}