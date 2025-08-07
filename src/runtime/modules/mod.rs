use crate::parser::ZanoValue;
use crate::runtime::{ZanoFunction, ZanoRuntime};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Built-in modules - all functionality is implemented in this file for now

pub struct ModuleSystem {
    modules: Arc<RwLock<HashMap<String, ZanoValue>>>,
}

impl ModuleSystem {
    pub fn new() -> Self {
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn init(&self, _runtime: &ZanoRuntime) -> Result<()> {
        // Initialize console module
        let console_module = self.create_console_module();
        self.modules.write().await.insert("console".to_string(), console_module);
        
        // Initialize fs module
        let fs_module = self.create_fs_module();
        self.modules.write().await.insert("fs".to_string(), fs_module);
        
        // Initialize http module
        let http_module = self.create_http_module();
        self.modules.write().await.insert("http".to_string(), http_module);
        
        // Initialize path module
        let path_module = self.create_path_module();
        self.modules.write().await.insert("path".to_string(), path_module);
        
        Ok(())
    }
    
    pub async fn get_module(&self, name: &str) -> Option<ZanoValue> {
        self.modules.read().await.get(name).cloned()
    }
    
    fn create_console_module(&self) -> ZanoValue {
        let mut console_obj = HashMap::new();
        
        // console.log
        console_obj.insert(
            "log".to_string(),
            ZanoValue::Function("console_log".to_string()),
        );
        
        // console.error
        console_obj.insert(
            "error".to_string(),
            ZanoValue::Function("console_error".to_string()),
        );
        
        // console.warn
        console_obj.insert(
            "warn".to_string(),
            ZanoValue::Function("console_warn".to_string()),
        );
        
        ZanoValue::Object(console_obj)
    }
    
    fn create_fs_module(&self) -> ZanoValue {
        let mut fs_obj = HashMap::new();
        
        // fs.readFile
        fs_obj.insert(
            "readFile".to_string(),
            ZanoValue::Function("fs_read_file".to_string()),
        );
        
        // fs.writeFile
        fs_obj.insert(
            "writeFile".to_string(),
            ZanoValue::Function("fs_write_file".to_string()),
        );
        
        // fs.exists
        fs_obj.insert(
            "exists".to_string(),
            ZanoValue::Function("fs_exists".to_string()),
        );
        
        ZanoValue::Object(fs_obj)
    }
    
    fn create_http_module(&self) -> ZanoValue {
        let mut http_obj = HashMap::new();
        
        // http.createServer
        http_obj.insert(
            "createServer".to_string(),
            ZanoValue::Function("http_create_server".to_string()),
        );
        
        // http.request
        http_obj.insert(
            "request".to_string(),
            ZanoValue::Function("http_request".to_string()),
        );
        
        ZanoValue::Object(http_obj)
    }
    
    fn create_path_module(&self) -> ZanoValue {
        let mut path_obj = HashMap::new();
        
        // path.join
        path_obj.insert(
            "join".to_string(),
            ZanoValue::Function("path_join".to_string()),
        );
        
        // path.dirname
        path_obj.insert(
            "dirname".to_string(),
            ZanoValue::Function("path_dirname".to_string()),
        );
        
        // path.basename
        path_obj.insert(
            "basename".to_string(),
            ZanoValue::Function("path_basename".to_string()),
        );
        
        ZanoValue::Object(path_obj)
    }
}

// Built-in function implementations
pub struct ConsoleLog;

#[async_trait]
impl ZanoFunction for ConsoleLog {
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue> {
        let messages: Vec<String> = args.iter().map(|arg| zano_value_to_string(arg)).collect();
        println!("{}", messages.join(" "));
        Ok(ZanoValue::Undefined)
    }
}

fn zano_value_to_string(value: &ZanoValue) -> String {
    match value {
        ZanoValue::String(s) => s.clone(),
        ZanoValue::Number(n) => n.to_string(),
        ZanoValue::Boolean(b) => b.to_string(),
        ZanoValue::Null => "null".to_string(),
        ZanoValue::Undefined => "undefined".to_string(),
        ZanoValue::Array(arr) => {
            let items: Vec<String> = arr.iter().map(zano_value_to_string).collect();
            format!("[{}]", items.join(", "))
        },
        ZanoValue::Object(obj) => {
            let items: Vec<String> = obj.iter().map(|(k, v)| {
                format!("{}: {}", k, zano_value_to_string(v))
            }).collect();
            format!("{{{}}}", items.join(", "))
        },
        ZanoValue::Function(name) => format!("function {}", name),
    }
}

pub struct ConsoleError;

#[async_trait]
impl ZanoFunction for ConsoleError {
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue> {
        let messages: Vec<String> = args.iter().map(|arg| zano_value_to_string(arg)).collect();
        eprintln!("{}", messages.join(" "));
        Ok(ZanoValue::Undefined)
    }
}

pub struct ConsoleWarn;

#[async_trait]
impl ZanoFunction for ConsoleWarn {
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue> {
        let messages: Vec<String> = args.iter().map(|arg| zano_value_to_string(arg)).collect();
        println!("WARN: {}", messages.join(" "));
        Ok(ZanoValue::Undefined)
    }
}

pub struct FsReadFile;

#[async_trait]
impl ZanoFunction for FsReadFile {
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue> {
        if let Some(ZanoValue::String(path)) = args.first() {
            match tokio::fs::read_to_string(path).await {
                Ok(content) => Ok(ZanoValue::String(content)),
                Err(e) => Err(anyhow::anyhow!("Failed to read file: {}", e)),
            }
        } else {
            Err(anyhow::anyhow!("readFile requires a string path"))
        }
    }
}

pub struct FsWriteFile;

#[async_trait]
impl ZanoFunction for FsWriteFile {
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue> {
        if args.len() >= 2 {
            if let (Some(ZanoValue::String(path)), Some(ZanoValue::String(content))) = 
                (args.get(0), args.get(1)) {
                match tokio::fs::write(path, content).await {
                    Ok(_) => Ok(ZanoValue::Undefined),
                    Err(e) => Err(anyhow::anyhow!("Failed to write file: {}", e)),
                }
            } else {
                Err(anyhow::anyhow!("writeFile requires path and content strings"))
            }
        } else {
            Err(anyhow::anyhow!("writeFile requires two arguments"))
        }
    }
}

pub struct HttpCreateServer;

#[async_trait]
impl ZanoFunction for HttpCreateServer {
    async fn call(&self, _args: Vec<ZanoValue>) -> Result<ZanoValue> {
        // This would create an HTTP server
        // For now, just return a placeholder
        Ok(ZanoValue::String("HTTP Server Created".to_string()))
    }
}

pub struct PathJoin;

#[async_trait]
impl ZanoFunction for PathJoin {
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue> {
        let paths: Result<Vec<&str>, _> = args.iter().map(|arg| {
            match arg {
                ZanoValue::String(s) => Ok(s.as_str()),
                _ => Err(anyhow::anyhow!("path.join requires string arguments")),
            }
        }).collect();
        
        match paths {
            Ok(path_parts) => {
                let joined = path_parts.iter().collect::<std::path::PathBuf>()
                    .to_string_lossy()
                    .to_string();
                Ok(ZanoValue::String(joined))
            }
            Err(e) => Err(e),
        }
    }
}

pub struct FsExists;

#[async_trait]
impl ZanoFunction for FsExists {
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue> {
        if let Some(ZanoValue::String(path)) = args.first() {
            let exists = tokio::fs::metadata(path).await.is_ok();
            Ok(ZanoValue::Boolean(exists))
        } else {
            Err(anyhow::anyhow!("exists requires a string path"))
        }
    }
}

pub struct HttpRequest;

#[async_trait]
impl ZanoFunction for HttpRequest {
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue> {
        if let Some(ZanoValue::String(url)) = args.first() {
            match reqwest::get(url).await {
                Ok(response) => {
                    match response.text().await {
                        Ok(text) => Ok(ZanoValue::String(text)),
                        Err(e) => Err(anyhow::anyhow!("Failed to read response: {}", e)),
                    }
                }
                Err(e) => Err(anyhow::anyhow!("HTTP request failed: {}", e)),
            }
        } else {
            Err(anyhow::anyhow!("request requires a URL string"))
        }
    }
}

pub struct PathDirname;

#[async_trait]
impl ZanoFunction for PathDirname {
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue> {
        if let Some(ZanoValue::String(path)) = args.first() {
            let path_buf = std::path::Path::new(path);
            if let Some(parent) = path_buf.parent() {
                Ok(ZanoValue::String(parent.to_string_lossy().to_string()))
            } else {
                Ok(ZanoValue::String(".".to_string()))
            }
        } else {
            Err(anyhow::anyhow!("dirname requires a string path"))
        }
    }
}

pub struct PathBasename;

#[async_trait]
impl ZanoFunction for PathBasename {
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue> {
        if let Some(ZanoValue::String(path)) = args.first() {
            let path_buf = std::path::Path::new(path);
            if let Some(name) = path_buf.file_name() {
                Ok(ZanoValue::String(name.to_string_lossy().to_string()))
            } else {
                Ok(ZanoValue::String(path.clone()))
            }
        } else {
            Err(anyhow::anyhow!("basename requires a string path"))
        }
    }
}

pub struct RequireFunction {
    module_system: ModuleSystem,
}

impl RequireFunction {
    pub fn new(module_system: ModuleSystem) -> Self {
        Self { module_system }
    }
}

#[async_trait]
impl ZanoFunction for RequireFunction {
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue> {
        if let Some(ZanoValue::String(module_name)) = args.first() {
            if let Some(module) = self.module_system.get_module(module_name).await {
                Ok(module)
            } else {
                Err(anyhow::anyhow!("Module not found: {}", module_name))
            }
        } else {
            Err(anyhow::anyhow!("require requires a module name string"))
        }
    }
}