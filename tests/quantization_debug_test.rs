//! Quantization debugging tests
//!
//! This module tests the quantization module specifically to debug
//! the count1 calculation issue.

use rust_mp3_encoder::quantization::GranuleInfo;

#[test]
fn test_calculate_run_length_manual() {
    println!("\n🔍 Testing calculate_run_length logic manually with known quantized values");
    
    // We'll manually implement the calculate_run_length logic to verify it works correctly
    
    let test_cases = [
        ("All zeros", [0i32; 576]),
        ("Single 1 at end", {
            let mut arr = [0i32; 576];
            arr[575] = 1;
            arr
        }),
        ("Pattern of 0s and 1s", {
            let mut arr = [0i32; 576];
            // Set some 0s and 1s in the last few positions
            arr[572] = 1;
            arr[573] = 0;
            arr[574] = 1;
            arr[575] = 0;
            arr
        }),
        ("Mixed values", {
            let mut arr = [0i32; 576];
            // Set some values > 1 to break count1 region
            arr[570] = 2;  // This should stop count1 counting
            arr[571] = 0;
            arr[572] = 1;
            arr[573] = 0;
            arr[574] = 1;
            arr[575] = 0;
            arr
        }),
    ];
    
    for (name, quantized) in test_cases.iter() {
        println!("\n--- Testing case: {} ---", name);
        
        // Manually implement the calculate_run_length logic to verify
        let mut i = 576;
        
        // Count trailing zero pairs - following shine's logic exactly
        while i > 1 {
            if quantized[i - 1] == 0 && quantized[i - 2] == 0 {
                i -= 2;
            } else {
                break;
            }
        }
        
        println!("After zero pair removal: i = {}", i);
        
        // Count quadruples (count1 region) - following shine's logic exactly
        let mut count1 = 0;
        while i > 3 {
            // CRITICAL: Follow shine's exact logic - check if values are <= 1
            if quantized[i - 1] <= 1 && quantized[i - 2] <= 1 &&
               quantized[i - 3] <= 1 && quantized[i - 4] <= 1 {
                count1 += 1;
                i -= 4;
            } else {
                break;
            }
        }
        
        let big_values = i >> 1;
        
        println!("Manual calculation results:");
        println!("  big_values: {}", big_values);
        println!("  count1: {}", count1);
        
        // Verify the logic
        if *name == "All zeros" {
            if count1 == 0 && big_values == 0 {
                println!("✓ All zeros case calculated correctly");
            } else {
                println!("❌ All zeros case incorrect: expected count1=0, big_values=0");
                println!("   This indicates the calculate_run_length logic is wrong");
            }
        } else if *name == "Mixed values" {
            // For mixed values with a 2 in position 570, count1 should be limited
            if count1 < 144 {
                println!("✓ Mixed values case looks reasonable: count1={}", count1);
            } else {
                println!("❌ Mixed values case incorrect: count1 should be < 144, got {}", count1);
            }
        }
        
        // Show some details about the quantized array for debugging
        if *name == "All zeros" {
            // Check if the array is really all zeros
            let non_zero_count = quantized.iter().filter(|&&x| x != 0).count();
            println!("  Non-zero values in array: {}", non_zero_count);
        }
    }
}

#[test]
fn test_shine_calc_runlen_logic() {
    println!("\n🔍 Testing shine's calc_runlen logic step by step");
    
    // Test the exact logic from shine's calc_runlen function
    // This is the C code we're trying to replicate:
    /*
    void calc_runlen(int ix[GRANULE_SIZE], gr_info *cod_info) {
      int i;
      int rzero = 0;

      for (i = GRANULE_SIZE; i > 1; i -= 2)
        if (!ix[i - 1] && !ix[i - 2])
          rzero++;
        else
          break;

      cod_info->count1 = 0;
      for (; i > 3; i -= 4)
        if (ix[i - 1] <= 1 && ix[i - 2] <= 1 && ix[i - 3] <= 1 && ix[i - 4] <= 1)
          cod_info->count1++;
        else
          break;

      cod_info->big_values = i >> 1;
    }
    */
    
    // Test case 1: All zeros (this is the problematic case)
    let ix = [0i32; 576];
    
    println!("Testing all-zero array with shine's exact logic:");
    
    let mut i = 576;
    let mut rzero = 0;
    
    // First loop: count trailing zero pairs
    while i > 1 {
        if ix[i - 1] == 0 && ix[i - 2] == 0 {
            rzero += 1;
            i -= 2;
        } else {
            break;
        }
    }
    
    println!("After first loop (zero pairs): i={}, rzero={}", i, rzero);
    
    // Second loop: count quadruples
    let mut count1 = 0;
    while i > 3 {
        if ix[i - 1] <= 1 && ix[i - 2] <= 1 && ix[i - 3] <= 1 && ix[i - 4] <= 1 {
            count1 += 1;
            i -= 4;
        } else {
            break;
        }
    }
    
    let big_values = i >> 1;
    
    println!("Final results:");
    println!("  i: {}", i);
    println!("  rzero: {}", rzero);
    println!("  count1: {}", count1);
    println!("  big_values: {}", big_values);
    
    // For all zeros, the expected result should be:
    // - All 576 coefficients are zero
    // - First loop removes all 288 zero pairs, leaving i=0
    // - Second loop never executes because i=0 is not > 3
    // - Therefore count1=0, big_values=0
    
    if count1 == 0 && big_values == 0 {
        println!("✓ Shine's logic produces correct result for all zeros");
    } else {
        println!("❌ Shine's logic produces incorrect result");
        println!("   Expected: count1=0, big_values=0");
        println!("   Got: count1={}, big_values={}", count1, big_values);
    }
}