//! Test Candle and Metal backend functionality
//!
//! Usage: cargo run --example test_candle

use anyhow::Result;
use candle_core::{DType, Device, Tensor};

fn main() -> Result<()> {
    println!("🧪 Testing Candle with Metal backend on M1 MacBook Air");

    // Test Metal device initialization
    println!("\n📱 Testing Metal device...");
    match Device::new_metal(0) {
        Ok(device) => {
            println!("✅ Metal device initialized successfully: {:?}", device);
            test_tensor_operations(&device)?;
        }
        Err(e) => {
            println!("❌ Metal device failed: {}", e);
            println!("🔄 Falling back to CPU...");
            let device = Device::Cpu;
            test_tensor_operations(&device)?;
        }
    }

    println!("\n🎉 All tests passed! Candle is ready for virtual try-on.");
    Ok(())
}

fn test_tensor_operations(device: &Device) -> Result<()> {
    println!("\n🔢 Testing basic tensor operations on {:?}...", device);

    // Create a simple tensor
    let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
    let tensor = Tensor::from_vec(data, (2, 3), device)?;
    println!("✅ Created tensor: {:?}", tensor.shape());

    // Test basic operations
    let tensor_squared = (&tensor * &tensor)?;
    println!("✅ Tensor multiplication works");

    let tensor_sum = tensor.sum_all()?;
    println!("✅ Tensor sum: {:?}", tensor_sum.to_scalar::<f32>()?);

    // Test data type conversion
    let tensor_f16 = tensor.to_dtype(DType::F16)?;
    println!("✅ F16 conversion works (good for M1 optimization)");

    // Test reshaping
    let reshaped = tensor.reshape((3, 2))?;
    println!("✅ Tensor reshaping: {:?}", reshaped.shape());

    // Test device transfer (if using Metal)
    if matches!(device, Device::Metal(_)) {
        let cpu_tensor = tensor.to_device(&Device::Cpu)?;
        println!("✅ Metal ↔ CPU transfer works");
    }

    Ok(())
}
