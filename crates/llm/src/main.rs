mod auto;

use anyhow::Result;
use auto::{AutoModel, AutoModelConfig};
use candle_core::{DType, Device, Tensor};
use tracing_subscriber;

fn try_cuda_device() -> Result<Device> {
    let device = Device::new_cuda(0)?;
    let test_tensor = Tensor::zeros((4, 4), DType::F32, &device)?;
    let _result = (&test_tensor + 1.0)?;
    let test_f32 = Tensor::randn(0.0, 1.0, (128, 128), &device)?;
    let _matmul_f32 = test_f32.matmul(&test_f32.t()?)?;
    device.synchronize()?;
    Ok(device)
}

fn benchmark_device_performance(device: &Device) -> Result<f64> {
    println!("🚀 Benchmarking device performance...");

    let size = (512, 512);
    let dtype = DType::F32; // Use F32 consistently to avoid CUDA driver issues

    let a = Tensor::randn(0.0, 1.0, size, device)?.to_dtype(dtype)?;
    let b = Tensor::randn(0.0, 1.0, size, device)?.to_dtype(dtype)?;

    let warmup_iterations = 5;
    let benchmark_iterations = 10;

    // Warmup
    for _ in 0..warmup_iterations {
        let _result = a.matmul(&b)?;
    }

    if device.is_cuda() {
        device.synchronize()?;
    }

    // Benchmark
    let start = std::time::Instant::now();
    for _ in 0..benchmark_iterations {
        let _result = a.matmul(&b)?;
    }

    if device.is_cuda() {
        device.synchronize()?;
    }

    let elapsed = start.elapsed();
    let ops_per_sec = benchmark_iterations as f64 / elapsed.as_secs_f64();

    println!(
        "📊 Performance: {:.2} matrix ops/sec on {:?}",
        ops_per_sec, device
    );
    Ok(ops_per_sec)
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let device = match try_cuda_device() {
        Ok(device) => {
            let _perf = benchmark_device_performance(&device)?;
            Some(device)
        }
        Err(_) => {
            let cpu_device = Device::Cpu;
            let _perf = benchmark_device_performance(&cpu_device)?;
            None
        }
    };

    println!("\n📦 Testing AutoModel with various model formats...");

    // Test models with different architectures and formats
    let test_models = [
        // SafeTensors format (default HuggingFace)
        ("microsoft/phi-2", "SafeTensors"),
        // Add Qwen model with proper SafeTensors support
        ("Qwen/Qwen3-0.6B", "SafeTensors"),
        // GGUF quantized format - use a model with tokenizer
        ("microsoft/Phi-3-mini-4k-instruct-gguf", "GGUF"),
    ];
    for (model_id, format_name) in test_models {
        println!("\n🔄 Loading: {} ({})", model_id, format_name);

        // Load model with optimized configuration and error handling
        let model_result = match device.as_ref() {
            Some(device) => AutoModel::from_pretrained_with_device(model_id, device),
            None => AutoModel::from_pretrained(model_id),
        };

        let mut model = match model_result {
            Ok(model) => {
                println!("✅ {}", model.info());
                model
            }
            Err(e) => {
                println!("❌ Failed to load {}: {}", model_id, e);
                continue;
            }
        };

        // Conservative config with proper GPU dtype support
        let config = AutoModelConfig {
            max_new_tokens: 10,
            temperature: Some(0.7),
            top_p: Some(0.9),
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            seed: 42,
        };

        let prompts = [
            "The future of AI is",
            "In Rust programming, the best practice for",
        ];

        for prompt in prompts {
            println!("\n💭 Prompt: '{}'", prompt);

            let start_time = std::time::Instant::now();
            match model.generate_with_config(prompt, &config) {
                Ok(_) => {
                    let generation_time = start_time.elapsed();
                    println!("\n⏱️  Generation took: {:?}", generation_time);
                }
                Err(e) => {
                    println!("❌ Generation failed: {}", e);
                }
            }
        }
    }

    Ok(())
}
