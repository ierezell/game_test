use anyhow::{Error as E, Result, anyhow};
use candle_core::{DType, Device, Tensor};
use candle_examples::token_output_stream::TokenOutputStream;
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::{
    llama::{Cache as LlamaCache, Config as LlamaConfig, Llama},
    mistral::{Config as MistralConfig, Model as Mistral},
    phi::{Config as PhiConfig, Model as Phi},
    phi3::{Config as Phi3Config, Model as Phi3},
    quantized_llama::ModelWeights as QuantizedLlama,
    quantized_mistral::Model as QuantizedMistral,
    quantized_phi::ModelWeights as QuantizedPhi,
    quantized_phi3::ModelWeights as QuantizedPhi3,
    quantized_qwen2::ModelWeights as QuantizedQwen2,
    qwen2::{Config as Qwen2Config, Model as Qwen2},
};
use std::io::Write;

use candle_transformers::quantized_var_builder::VarBuilder as QVarBuilder;
use hf_hub::{Repo, RepoType, api::sync::Api};
use ndarray::{Array, CowArray, IxDyn};
use ort::{Environment, SessionBuilder};
use serde_json::Value;
use std::path::Path;
use tokenizers::Tokenizer;
use tracing::debug;

#[derive(Debug, Clone, PartialEq)]
pub enum ModelArchitecture {
    Llama,
    Mistral,
    Phi,
    Phi3,
    Qwen2,

    Unknown(String),
}

impl ModelArchitecture {
    pub fn from_config(config: &Value) -> Self {
        if let Some(arch) = config.get("architectures").and_then(|a| a.as_array()) {
            if let Some(arch_str) = arch.first().and_then(|s| s.as_str()) {
                match arch_str.to_lowercase().as_str() {
                    s if s.contains("llama") => return Self::Llama,
                    s if s.contains("mistral") => return Self::Mistral,
                    s if s.contains("phi") && s.contains("3") => return Self::Phi3,
                    s if s.contains("phi") => return Self::Phi,

                    s if s.contains("qwen") => return Self::Qwen2,
                    _ => return Self::Unknown(arch_str.to_string()),
                }
            }
        }

        // Check model_type field
        if let Some(model_type) = config.get("model_type").and_then(|t| t.as_str()) {
            match model_type.to_lowercase().as_str() {
                "llama" => return Self::Llama,
                "mistral" => return Self::Mistral,
                "phi" => return Self::Phi,
                "phi3" => return Self::Phi3,

                "qwen2" => return Self::Qwen2,
                _ => return Self::Unknown(model_type.to_string()),
            }
        }

        // Fall back to name-based detection
        if let Some(name) = config.get("_name_or_path").and_then(|n| n.as_str()) {
            let name_lower = name.to_lowercase();
            if name_lower.contains("llama") {
                return Self::Llama;
            } else if name_lower.contains("mistral") {
                return Self::Mistral;
            } else if name_lower.contains("phi-3") || name_lower.contains("phi3") {
                return Self::Phi3;
            } else if name_lower.contains("phi") {
                return Self::Phi;
            } else if name_lower.contains("qwen") {
                return Self::Qwen2;
            }
        }

        Self::Unknown("unknown".to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelFormat {
    SafeTensors,
    QuantizedGguf,
    QuantizedGgml,
    Onnx,
}

impl ModelFormat {
    pub fn detect_from_files(files: &[String]) -> Self {
        for file in files {
            let file_lower = file.to_lowercase();
            if file_lower.ends_with(".gguf") {
                return Self::QuantizedGguf;
            } else if file_lower.ends_with(".ggml") || file_lower.contains(".bin") {
                return Self::QuantizedGgml;
            } else if file_lower.ends_with(".onnx") {
                return Self::Onnx;
            }
        }
        // Default to SafeTensors if no specific format detected
        Self::SafeTensors
    }
}

pub enum UnifiedModel {
    Phi(Phi),
    Phi3(Phi3),
    Mistral(Mistral),
    Llama(Llama, LlamaCache),
    Qwen2(Qwen2),
    QuantizedPhi(QuantizedPhi),
    QuantizedPhi3(QuantizedPhi3),
    QuantizedMistral(QuantizedMistral),
    QuantizedLlama(QuantizedLlama),
    QuantizedQwen2(QuantizedQwen2),

    Onnx(OnnxModel),
}

impl UnifiedModel {
    pub fn forward(&mut self, xs: &Tensor, pos: usize) -> candle_core::Result<Tensor> {
        match self {
            Self::Phi(m) => m.forward(xs),
            Self::Phi3(m) => m.forward(xs, pos),
            Self::Mistral(m) => m.forward(xs, pos),
            Self::Llama(m, cache) => m.forward(xs, pos, cache),
            Self::Qwen2(m) => m.forward(xs, pos, None),

            Self::QuantizedPhi(m) => m.forward(xs, pos),
            Self::QuantizedPhi3(m) => m.forward(xs, pos),
            Self::QuantizedMistral(m) => m.forward(xs, pos),
            Self::QuantizedLlama(m) => m.forward(xs, pos),
            Self::QuantizedQwen2(m) => m.forward(xs, pos),

            Self::Onnx(m) => m.forward(xs, pos),
        }
    }

    pub fn clear_kv_cache(&mut self) {
        match self {
            Self::Phi(m) => m.clear_kv_cache(),
            Self::Phi3(m) => m.clear_kv_cache(),
            Self::Mistral(m) => m.clear_kv_cache(),
            Self::Llama(_, _cache) => {}
            Self::Qwen2(m) => m.clear_kv_cache(),
            Self::QuantizedPhi(_) => {}
            Self::QuantizedPhi3(_) => {}
            Self::QuantizedMistral(m) => m.clear_kv_cache(),
            Self::QuantizedLlama(_) => {}
            Self::QuantizedQwen2(_) => {}

            Self::Onnx(_) => {}
        }
    }
}

pub struct OnnxModel {
    session: ort::Session,
    device: Device,
}

impl OnnxModel {
    pub fn load(model_path: &Path, device: &Device) -> Result<Self> {
        // Initialize ORT environment
        let env = std::sync::Arc::new(
            Environment::builder()
                .with_name("yolo-ai")
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to create ORT environment: {}", e))?,
        );

        let session = SessionBuilder::new(&env)
            .map_err(|e| anyhow::anyhow!("Failed to create session builder: {}", e))?
            .with_optimization_level(ort::GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow::anyhow!("Failed to set optimization level: {}", e))?
            .with_intra_threads(num_cpus::get() as i16)
            .map_err(|e| anyhow::anyhow!("Failed to set intra threads: {}", e))?
            .with_model_from_file(model_path)
            .map_err(|e| anyhow::anyhow!("Failed to load model from file: {}", e))?;

        Ok(Self {
            session,
            device: device.clone(),
        })
    }

    pub fn forward(&mut self, xs: &Tensor, _pos: usize) -> candle_core::Result<Tensor> {
        let xs_cpu = xs.to_device(&Device::Cpu)?;
        let shape = xs_cpu.shape();

        // ONNX models typically expect int64 input for token IDs
        let input_data: Vec<i64> = if shape.rank() == 2 {
            xs_cpu
                .flatten_all()?
                .to_vec1::<u32>()?
                .into_iter()
                .map(|x| x as i64)
                .collect()
        } else {
            xs_cpu
                .to_vec1::<u32>()?
                .into_iter()
                .map(|x| x as i64)
                .collect()
        };

        let input_shape = if shape.rank() == 2 {
            vec![shape.dims()[0], shape.dims()[1]]
        } else {
            vec![1, input_data.len()]
        };

        let array = Array::from_shape_vec(IxDyn(&input_shape), input_data)
            .map_err(|e| candle_core::Error::Msg(format!("Failed to create ndarray: {}", e)))?;
        let cow_array = CowArray::from(array);
        let input_tensor = ort::Value::from_array(self.session.allocator(), &cow_array)
            .map_err(|e| candle_core::Error::Msg(format!("ONNX input error: {}", e)))?;

        let outputs = self
            .session
            .run(vec![input_tensor])
            .map_err(|e| candle_core::Error::Msg(format!("ONNX run error: {}", e)))?;

        if let Some(output) = outputs.get(0) {
            let output_data = output
                .try_extract::<f32>()
                .map_err(|e| candle_core::Error::Msg(format!("ONNX output error: {}", e)))?;

            let output_slice = output_data
                .view()
                .to_slice()
                .ok_or_else(|| candle_core::Error::Msg("Failed to get output slice".to_string()))?;

            let output_tensor = Tensor::from_slice(output_slice, shape, &self.device)?;
            Ok(output_tensor)
        } else {
            Err(candle_core::Error::Msg(
                "No output from ONNX model".to_string(),
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct AutoModelConfig {
    pub max_new_tokens: usize,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub repeat_penalty: f32,
    pub repeat_last_n: usize,
    pub seed: u64,
}

impl Default for AutoModelConfig {
    fn default() -> Self {
        Self {
            max_new_tokens: 100,
            temperature: Some(0.8),
            top_p: Some(0.9),
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            seed: 42,
        }
    }
}

pub struct AutoModel {
    model: UnifiedModel,
    device: Device,
    tokenizer: TokenOutputStream,
    logits_processor: LogitsProcessor,
    config: AutoModelConfig,
    model_id: String,
    architecture: ModelArchitecture,
    format: ModelFormat,
}

impl AutoModel {
    pub fn from_pretrained(model_id: &str) -> Result<Self> {
        let device = Self::auto_device()?;
        Self::from_pretrained_with_device(model_id, &device)
    }

    pub fn from_pretrained_with_device(model_id: &str, device: &Device) -> Result<Self> {
        let api = Api::new()?;
        let repo = api.repo(Repo::with_revision(
            model_id.to_string(),
            RepoType::Model,
            "main".to_string(),
        ));

        let config_path = repo.get("config.json").ok();
        let architecture = if let Some(config_path) = &config_path {
            let config_data = std::fs::read_to_string(config_path)?;
            let config: Value = serde_json::from_str(&config_data)?;
            ModelArchitecture::from_config(&config)
        } else {
            Self::detect_architecture_from_name(model_id)
        };

        let files = Self::list_model_files(&repo)?;
        let format = ModelFormat::detect_from_files(&files);

        // Try to get tokenizer, with multiple fallback options
        let tokenizer = if let Ok(tokenizer_path) = repo.get("tokenizer.json") {
            Tokenizer::from_file(tokenizer_path).map_err(E::msg)?
        } else if let Ok(tokenizer_path) = repo.get("tokenizer.model") {
            Tokenizer::from_file(tokenizer_path).map_err(E::msg)?
        } else {
            return Err(anyhow!(
                "No tokenizer file found (tokenizer.json or tokenizer.model)"
            ));
        };

        // Load model based on format and architecture
        let model = Self::load_model(
            &repo,
            &architecture,
            &format,
            device,
            config_path.as_deref(),
        )?;

        let config = AutoModelConfig::default();
        let logits_processor = LogitsProcessor::new(config.seed, config.temperature, config.top_p);

        Ok(Self {
            model,
            device: device.clone(),
            tokenizer: TokenOutputStream::new(tokenizer),
            logits_processor,
            config,
            model_id: model_id.to_string(),
            architecture,
            format,
        })
    }

    /// Auto-detect optimal device with GPU optimizations
    fn auto_device() -> Result<Device> {
        // Try CUDA first with optimizations
        if let Ok(device) = Self::try_cuda_optimized() {
            return Ok(device);
        }

        // Try Metal on macOS
        #[cfg(target_os = "macos")]
        {
            if let Ok(device) = Device::new_metal(0) {
                return Ok(device);
            }
        }

        Ok(Device::Cpu)
    }

    fn try_cuda_optimized() -> Result<Device> {
        let device = Device::new_cuda(0)?;

        // Test GPU with F32 operations for consistency
        let test_size = (512, 512);
        let dtype = DType::F32; // Use F32 consistently

        let a = Tensor::randn(0.0, 1.0, test_size, &device)?.to_dtype(dtype)?;
        let b = Tensor::randn(0.0, 1.0, test_size, &device)?.to_dtype(dtype)?;

        // Test matrix multiplication performance with F32
        let start = std::time::Instant::now();
        let _result = a.matmul(&b)?;
        let gpu_time = start.elapsed();

        debug!("GPU F32 matmul test: {:?}", gpu_time);

        // Ensure CUDA context is properly initialized
        let _sync = device.synchronize()?;
        Ok(device)
    }

    fn detect_architecture_from_name(model_id: &str) -> ModelArchitecture {
        let model_lower = model_id.to_lowercase();
        if model_lower.contains("llama") {
            ModelArchitecture::Llama
        } else if model_lower.contains("mistral") {
            ModelArchitecture::Mistral
        } else if model_lower.contains("phi-3") || model_lower.contains("phi3") {
            ModelArchitecture::Phi3
        } else if model_lower.contains("phi") {
            ModelArchitecture::Phi
        } else if model_lower.contains("qwen") {
            ModelArchitecture::Qwen2
        } else {
            ModelArchitecture::Unknown(model_id.to_string())
        }
    }

    fn list_model_files(repo: &hf_hub::api::sync::ApiRepo) -> Result<Vec<String>> {
        // Try to get repo info to list files
        let repo_info = repo.info()?;
        let files: Vec<String> = repo_info
            .siblings
            .iter()
            .map(|sibling| sibling.rfilename.clone())
            .collect();
        Ok(files)
    }

    fn load_model(
        repo: &hf_hub::api::sync::ApiRepo,
        architecture: &ModelArchitecture,
        format: &ModelFormat,
        device: &Device,
        config_path: Option<&Path>,
    ) -> Result<UnifiedModel> {
        match format {
            ModelFormat::SafeTensors => {
                Self::load_safetensors_model(repo, architecture, device, config_path)
            }
            ModelFormat::QuantizedGguf => Self::load_gguf_model(repo, architecture, device),
            ModelFormat::QuantizedGgml => Self::load_ggml_model(repo, architecture, device),

            ModelFormat::Onnx => Self::load_onnx_model(repo, device),
        }
    }

    fn load_safetensors_model(
        repo: &hf_hub::api::sync::ApiRepo,
        architecture: &ModelArchitecture,
        device: &Device,
        config_path: Option<&Path>,
    ) -> Result<UnifiedModel> {
        let config_path =
            config_path.ok_or_else(|| anyhow!("config.json required for SafeTensors models"))?;

        // Optimize dtype based on device - use F32 to avoid RoPE dtype issues
        let dtype = if device.is_cuda() {
            DType::F32 // Use F32 for now to avoid RoPE mixed dtype issues
        } else {
            DType::F32
        };

        // Try to load either split model or single file
        let filenames = if let Ok(filenames) =
            candle_examples::hub_load_safetensors(repo, "model.safetensors.index.json")
        {
            // Split model with index
            filenames
        } else {
            // Single file model
            vec![repo.get("model.safetensors")?]
        };

        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&filenames, dtype, device)? };

        let config_data = std::fs::read_to_string(config_path)?;

        match architecture {
            ModelArchitecture::Phi => {
                let config: PhiConfig = serde_json::from_str(&config_data)?;
                let model = Phi::new(&config, vb)?;
                Ok(UnifiedModel::Phi(model))
            }
            ModelArchitecture::Phi3 => {
                let config: Phi3Config = serde_json::from_str(&config_data)?;
                let model = Phi3::new(&config, vb)?;
                Ok(UnifiedModel::Phi3(model))
            }
            ModelArchitecture::Mistral => {
                let config: MistralConfig = serde_json::from_str(&config_data)?;
                let model = Mistral::new(&config, vb)?;
                Ok(UnifiedModel::Mistral(model))
            }
            ModelArchitecture::Llama => {
                // Create cache for Llama models
                let _config_content = std::fs::read_to_string(config_path)?;
                let _config_value: Value = serde_json::from_str(&_config_content)?;

                // Create a basic config - we'll use defaults since the Config doesn't implement Deserialize
                let config = LlamaConfig::config_7b_v2(false);
                let cache = LlamaCache::new(false, DType::F32, &config, device)?;
                let model = Llama::load(vb, &config)?;
                Ok(UnifiedModel::Llama(model, cache))
            }
            ModelArchitecture::Qwen2 => {
                let config: Qwen2Config = serde_json::from_str(&config_data)?;
                let model = Qwen2::new(&config, vb)?;
                Ok(UnifiedModel::Qwen2(model))
            }
            ModelArchitecture::Unknown(name) => Err(anyhow!(
                "Unsupported architecture for SafeTensors: {}",
                name
            )),
        }
    }

    fn load_gguf_model(
        repo: &hf_hub::api::sync::ApiRepo,
        architecture: &ModelArchitecture,
        device: &Device,
    ) -> Result<UnifiedModel> {
        // Find GGUF file
        let repo_info = repo.info()?;
        let gguf_files: Vec<_> = repo_info
            .siblings
            .iter()
            .filter(|s| s.rfilename.ends_with(".gguf"))
            .collect();

        if gguf_files.is_empty() {
            return Err(anyhow!("No GGUF files found"));
        }

        let gguf_path = repo.get(&gguf_files[0].rfilename)?;
        let mut file = std::fs::File::open(&gguf_path)?;
        let model = candle_core::quantized::gguf_file::Content::read(&mut file)?;

        match architecture {
            ModelArchitecture::Llama => {
                let weights = QuantizedLlama::from_gguf(model, &mut file, device)?;
                Ok(UnifiedModel::QuantizedLlama(weights))
            }
            ModelArchitecture::Mistral => {
                let vb = QVarBuilder::from_gguf(&gguf_path, device)?;
                let config = MistralConfig::config_7b_v0_1(true); // Default config
                let model = QuantizedMistral::new(&config, vb)?;
                Ok(UnifiedModel::QuantizedMistral(model))
            }
            ModelArchitecture::Phi => {
                let weights = QuantizedPhi::from_gguf(model, &mut file, device)?;
                Ok(UnifiedModel::QuantizedPhi(weights))
            }
            ModelArchitecture::Phi3 => {
                let weights = QuantizedPhi3::from_gguf(false, model, &mut file, device)?;
                Ok(UnifiedModel::QuantizedPhi3(weights))
            }
            ModelArchitecture::Qwen2 => {
                let weights = QuantizedQwen2::from_gguf(model, &mut file, device)?;
                Ok(UnifiedModel::QuantizedQwen2(weights))
            }
            ModelArchitecture::Unknown(name) => {
                Err(anyhow!("Unsupported architecture for GGUF: {}", name))
            }
        }
    }

    fn load_ggml_model(
        repo: &hf_hub::api::sync::ApiRepo,
        architecture: &ModelArchitecture,
        device: &Device,
    ) -> Result<UnifiedModel> {
        // Find GGML file
        let repo_info = repo.info()?;
        let ggml_files: Vec<_> = repo_info
            .siblings
            .iter()
            .filter(|s| s.rfilename.ends_with(".bin") || s.rfilename.ends_with(".ggml"))
            .collect();

        if ggml_files.is_empty() {
            return Err(anyhow!("No GGML files found"));
        }

        let ggml_path = repo.get(&ggml_files[0].rfilename)?;
        let mut file = std::fs::File::open(&ggml_path)?;
        let model = candle_core::quantized::ggml_file::Content::read(&mut file, device)?;

        match architecture {
            ModelArchitecture::Llama => {
                let weights = QuantizedLlama::from_ggml(model, 1)?; // Default GQA
                Ok(UnifiedModel::QuantizedLlama(weights))
            }
            _ => Err(anyhow!("GGML format only supported for Llama currently")),
        }
    }

    fn load_onnx_model(repo: &hf_hub::api::sync::ApiRepo, device: &Device) -> Result<UnifiedModel> {
        let repo_info = repo.info()?;
        let onnx_files: Vec<_> = repo_info
            .siblings
            .iter()
            .filter(|s| s.rfilename.ends_with(".onnx"))
            .collect();

        if onnx_files.is_empty() {
            return Err(anyhow!("No ONNX files found"));
        }

        let onnx_path = repo.get(&onnx_files[0].rfilename)?;
        let model = OnnxModel::load(onnx_path.as_path(), device)?;
        Ok(UnifiedModel::Onnx(model))
    }

    /// Generate with custom configuration
    pub fn generate_with_config(&mut self, prompt: &str, config: &AutoModelConfig) -> Result<()> {
        self.config = config.clone();

        // Update logits processor with new config
        self.logits_processor = LogitsProcessor::new(config.seed, config.temperature, config.top_p);

        self.run(prompt, config.max_new_tokens)
    }

    /// Get model information
    pub fn info(&self) -> String {
        format!(
            "AutoModel: {} | Architecture: {:?} | Format: {:?} | Device: {:?}",
            self.model_id, self.architecture, self.format, self.device
        )
    }

    /// Main generation loop with GPU optimizations
    fn run(&mut self, prompt: &str, sample_len: usize) -> Result<()> {
        self.model.clear_kv_cache();
        self.tokenizer.clear();

        let mut tokens = self
            .tokenizer
            .tokenizer()
            .encode(prompt, true)
            .map_err(E::msg)?
            .get_ids()
            .to_vec();

        // Print prompt tokens with safety check
        for &t in tokens.iter() {
            if let Some(t) = self.tokenizer.next_token(t)? {
                print!("{t}");
                std::io::stdout().flush()?;
            }
        }

        // Generation loop with optimizations and safety checks
        let mut generated_tokens = 0usize;
        let eos_token = self.get_eos_token();
        let start_gen = std::time::Instant::now();

        for index in 0..sample_len {
            // Safety check for timeout
            if start_gen.elapsed().as_secs() > 60 {
                println!("\n⚠️ Generation timeout after 60 seconds");
                break;
            }

            // Optimize context window for GPU memory
            let context_size = if index > 0 {
                1 // Only use last token for subsequent generations
            } else {
                tokens.len().min(2048) // Limit initial context for memory efficiency
            };

            let start_pos = tokens.len().saturating_sub(context_size);
            let ctxt = &tokens[start_pos..];

            // Create input tensor with optimal memory layout
            let input = Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;

            // Forward pass with GPU optimizations and error handling
            let logits = match self.model.forward(&input, start_pos) {
                Ok(logits) => logits,
                Err(e) => {
                    println!("\n❌ Forward pass failed: {}", e);
                    break;
                }
            };

            let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;

            // Apply repetition penalty efficiently
            let logits = if self.config.repeat_penalty == 1.0 {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(self.config.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.config.repeat_penalty,
                    &tokens[start_at..],
                )?
            };

            // Sample next token
            let next_token = self.logits_processor.sample(&logits)?;
            tokens.push(next_token);
            generated_tokens += 1;

            // Check for end-of-sequence
            if next_token == eos_token {
                break;
            }

            // Output token with debug info
            if let Some(t) = self.tokenizer.next_token(next_token)? {
                print!("{t}");
                std::io::stdout().flush()?;
            }

            // Debug output every few tokens
            if generated_tokens % 3 == 0 {
                println!(" [{}]", generated_tokens);
                std::io::stdout().flush()?;
            }
        }

        let dt = start_gen.elapsed();
        if let Some(rest) = self.tokenizer.decode_rest().map_err(E::msg)? {
            print!("{rest}");
        }
        std::io::stdout().flush()?;

        let tokens_per_sec = generated_tokens as f64 / dt.as_secs_f64();
        println!(
            "\n{} tokens generated ({:.2} token/s) on {:?}",
            generated_tokens, tokens_per_sec, self.device
        );

        Ok(())
    }

    fn get_eos_token(&self) -> u32 {
        self.tokenizer
            .get_token("</s>")
            .or_else(|| self.tokenizer.get_token("<|endoftext|>"))
            .or_else(|| self.tokenizer.get_token("<|im_end|>"))
            .unwrap_or(2) // Common EOS token ID
    }
}
