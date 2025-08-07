use crate::parser::{Expression, Statement, ZanoValue, BinaryOp};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

pub mod modules;

#[async_trait]
pub trait ZanoFunction: Send + Sync {
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue>;
}

pub struct ZanoRuntime {
    globals: Arc<RwLock<HashMap<String, ZanoValue>>>,
    functions: Arc<RwLock<HashMap<String, Arc<dyn ZanoFunction>>>>,
    modules: Arc<RwLock<HashMap<String, ZanoValue>>>,
}

impl ZanoRuntime {
    pub async fn new() -> Self {
        let runtime = Self {
            globals: Arc::new(RwLock::new(HashMap::new())),
            functions: Arc::new(RwLock::new(HashMap::new())),
            modules: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Initialize built-ins
        runtime.init_builtins().await;
        
        runtime
    }
    
    async fn init_builtins(&self) {
        use crate::runtime::modules::*;
        
        // Initialize console functions
        self.functions.write().await.insert("console_log".to_string(), Arc::new(ConsoleLog));
        self.functions.write().await.insert("console_error".to_string(), Arc::new(ConsoleError));
        self.functions.write().await.insert("console_warn".to_string(), Arc::new(ConsoleWarn));
        
        // Initialize fs functions
        self.functions.write().await.insert("fs_readFile".to_string(), Arc::new(FsReadFile));
        self.functions.write().await.insert("fs_writeFile".to_string(), Arc::new(FsWriteFile));
        self.functions.write().await.insert("fs_exists".to_string(), Arc::new(FsExists));
        
        // Initialize http functions
        self.functions.write().await.insert("http_createServer".to_string(), Arc::new(HttpCreateServer));
        self.functions.write().await.insert("http_request".to_string(), Arc::new(HttpRequest));
        
        // Initialize path functions
        self.functions.write().await.insert("path_join".to_string(), Arc::new(PathJoin));
        self.functions.write().await.insert("path_dirname".to_string(), Arc::new(PathDirname));
        self.functions.write().await.insert("path_basename".to_string(), Arc::new(PathBasename));
        
        // Create module system
        let module_system = modules::ModuleSystem::new();
        module_system.init(self).await.expect("Failed to initialize modules");
        
        // Add global console object
        let mut console_obj = HashMap::new();
        console_obj.insert("log".to_string(), ZanoValue::Function("console_log".to_string()));
        console_obj.insert("error".to_string(), ZanoValue::Function("console_error".to_string()));
        console_obj.insert("warn".to_string(), ZanoValue::Function("console_warn".to_string()));
        self.globals.write().await.insert("console".to_string(), ZanoValue::Object(console_obj));
        
        // Add require function
        self.functions.write().await.insert("require".to_string(), Arc::new(RequireFunction::new(module_system)));
        self.globals.write().await.insert("require".to_string(), ZanoValue::Function("require".to_string()));
    }
    
    pub async fn execute(&self, statements: Vec<Statement>) -> Result<ZanoValue> {
        let mut last_value = ZanoValue::Undefined;
        
        for statement in statements {
            last_value = self.execute_statement(statement).await?;
        }
        
        Ok(last_value)
    }
    
    fn execute_statement(&self, statement: Statement) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ZanoValue>> + Send + '_>> {
        Box::pin(async move {
        match statement {
            Statement::Expression(expr) => self.evaluate_expression(expr).await,
            Statement::VarDeclaration { name, value, is_const: _ } => {
                let val = if let Some(expr) = value {
                    self.evaluate_expression(expr).await?
                } else {
                    ZanoValue::Undefined
                };
                
                self.globals.write().await.insert(name, val);
                Ok(ZanoValue::Undefined)
            }
            Statement::FunctionDeclaration { name, params, body, is_async: _ } => {
                let func = UserDefinedFunction {
                    params,
                    body,
                    runtime: self.clone(),
                };
                
                self.functions.write().await.insert(name.clone(), Arc::new(func));
                self.globals.write().await.insert(name, ZanoValue::Function("user_defined".to_string()));
                
                Ok(ZanoValue::Undefined)
            }
            Statement::If { condition, then_branch, else_branch } => {
                let condition_value = self.evaluate_expression(condition).await?;
                
                if self.is_truthy(&condition_value) {
                    self.execute_statement(*then_branch).await
                } else if let Some(else_stmt) = else_branch {
                    self.execute_statement(*else_stmt).await
                } else {
                    Ok(ZanoValue::Undefined)
                }
            }
            Statement::Block(statements) => {
                let mut last_value = ZanoValue::Undefined;
                for stmt in statements {
                    last_value = self.execute_statement(stmt).await?;
                }
                Ok(last_value)
            }
            Statement::Return(expr) => {
                if let Some(expression) = expr {
                    self.evaluate_expression(expression).await
                } else {
                    Ok(ZanoValue::Undefined)
                }
            }
            Statement::While { condition, body } => {
                while self.is_truthy(&self.evaluate_expression(condition.clone()).await?) {
                    self.execute_statement((*body).clone()).await?;
                }
                Ok(ZanoValue::Undefined)
            }
            Statement::Try { try_block, catch_param, catch_block } => {
                match self.execute_statement(*try_block).await {
                    Ok(value) => Ok(value),
                    Err(error) => {
                        if let Some(catch_stmt) = catch_block {
                            if let Some(param_name) = catch_param {
                                // Bind error to catch parameter
                                let error_obj = ZanoValue::String(error.to_string());
                                self.globals.write().await.insert(param_name, error_obj);
                            }
                            self.execute_statement(*catch_stmt).await
                        } else {
                            Err(error)
                        }
                    }
                }
            }
            Statement::Throw(expr) => {
                let value = self.evaluate_expression(expr).await?;
                let error_message = match value {
                    ZanoValue::String(s) => s,
                    _ => format!("{:?}", value),
                };
                Err(anyhow::anyhow!("Thrown: {}", error_message))
            }
        }
        })
    }
    
    fn evaluate_expression(&self, expression: Expression) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ZanoValue>> + Send + '_>> {
        Box::pin(async move {
        match expression {
            Expression::Literal(value) => Ok(value),
            Expression::Identifier(name) => {
                if let Some(value) = self.globals.read().await.get(&name) {
                    Ok(value.clone())
                } else {
                    Err(anyhow::anyhow!("Undefined variable: {}", name))
                }
            }
            Expression::Binary { left, operator, right } => {
                let left_val = self.evaluate_expression(*left).await?;
                let right_val = self.evaluate_expression(*right).await?;
                
                self.apply_binary_operator(left_val, operator, right_val)
            }
            Expression::Call { callee, args } => {
                let function_name = match *callee {
                    Expression::Identifier(name) => name,
                    Expression::Member { object, property } => {
                        // Handle member function calls like console.log
                        match *object {
                            Expression::Identifier(obj_name) => {
                                format!("{}_{}", obj_name, property)
                            }
                            _ => return Err(anyhow::anyhow!("Complex member calls not supported yet")),
                        }
                    }
                    _ => return Err(anyhow::anyhow!("Invalid function call")),
                };
                
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.evaluate_expression(arg).await?);
                }
                
                if let Some(func) = self.functions.read().await.get(&function_name) {
                    func.call(arg_values).await
                } else {
                    Err(anyhow::anyhow!("Undefined function: {}", function_name))
                }
            }
            Expression::Member { object, property } => {
                let obj_value = self.evaluate_expression(*object).await?;
                
                match obj_value {
                    ZanoValue::Object(ref map) => {
                        if let Some(value) = map.get(&property) {
                            Ok(value.clone())
                        } else {
                            Ok(ZanoValue::Undefined)
                        }
                    }
                    _ => Ok(ZanoValue::Undefined),
                }
            }
            Expression::Assignment { target, value } => {
                let val = self.evaluate_expression(*value).await?;
                self.globals.write().await.insert(target, val.clone());
                Ok(val)
            }
            Expression::Array(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.evaluate_expression(element).await?);
                }
                Ok(ZanoValue::Array(values))
            }
            Expression::Object(pairs) => {
                let mut obj = HashMap::new();
                for (key, value) in pairs {
                    let val = self.evaluate_expression(value).await?;
                    obj.insert(key, val);
                }
                Ok(ZanoValue::Object(obj))
            }
            Expression::Index { object, index } => {
                let obj_value = self.evaluate_expression(*object).await?;
                let index_value = self.evaluate_expression(*index).await?;
                
                match (obj_value, index_value) {
                    (ZanoValue::Array(ref arr), ZanoValue::Number(n)) => {
                        let idx = n as usize;
                        if idx < arr.len() {
                            Ok(arr[idx].clone())
                        } else {
                            Ok(ZanoValue::Undefined)
                        }
                    }
                    (ZanoValue::Object(ref obj), ZanoValue::String(key)) => {
                        Ok(obj.get(&key).cloned().unwrap_or(ZanoValue::Undefined))
                    }
                    _ => Ok(ZanoValue::Undefined),
                }
            }
            Expression::Await(expr) => {
                // For now, just evaluate the expression
                // In a full implementation, this would handle promises/futures
                self.evaluate_expression(*expr).await
            }
        }
        })
    }
    
    fn apply_binary_operator(&self, left: ZanoValue, op: BinaryOp, right: ZanoValue) -> Result<ZanoValue> {
        match (left, right) {
            (ZanoValue::Number(a), ZanoValue::Number(b)) => {
                let result = match op {
                    BinaryOp::Add => a + b,
                    BinaryOp::Sub => a - b,
                    BinaryOp::Mul => a * b,
                    BinaryOp::Div => a / b,
                    BinaryOp::Mod => a % b,
                    BinaryOp::Equal => return Ok(ZanoValue::Boolean(a == b)),
                    BinaryOp::NotEqual => return Ok(ZanoValue::Boolean(a != b)),
                    BinaryOp::Less => return Ok(ZanoValue::Boolean(a < b)),
                    BinaryOp::Greater => return Ok(ZanoValue::Boolean(a > b)),
                    BinaryOp::LessEqual => return Ok(ZanoValue::Boolean(a <= b)),
                    BinaryOp::GreaterEqual => return Ok(ZanoValue::Boolean(a >= b)),
                    _ => return Err(anyhow::anyhow!("Invalid operation for numbers")),
                };
                Ok(ZanoValue::Number(result))
            }
            (ZanoValue::String(a), ZanoValue::String(b)) => {
                match op {
                    BinaryOp::Add => Ok(ZanoValue::String(format!("{}{}", a, b))),
                    BinaryOp::Equal => Ok(ZanoValue::Boolean(a == b)),
                    BinaryOp::NotEqual => Ok(ZanoValue::Boolean(a != b)),
                    _ => Err(anyhow::anyhow!("Invalid operation for strings")),
                }
            }
            (ZanoValue::String(a), ZanoValue::Number(b)) => {
                match op {
                    BinaryOp::Add => Ok(ZanoValue::String(format!("{}{}", a, b))),
                    _ => Err(anyhow::anyhow!("Invalid operation for string and number")),
                }
            }
            (ZanoValue::Number(a), ZanoValue::String(b)) => {
                match op {
                    BinaryOp::Add => Ok(ZanoValue::String(format!("{}{}", a, b))),
                    _ => Err(anyhow::anyhow!("Invalid operation for number and string")),
                }
            }
            (ZanoValue::Boolean(a), ZanoValue::Boolean(b)) => {
                let result = match op {
                    BinaryOp::And => a && b,
                    BinaryOp::Or => a || b,
                    BinaryOp::Equal => a == b,
                    BinaryOp::NotEqual => a != b,
                    _ => return Err(anyhow::anyhow!("Invalid operation for booleans")),
                };
                Ok(ZanoValue::Boolean(result))
            }
            _ => Err(anyhow::anyhow!("Type mismatch in binary operation")),
        }
    }
    
    fn is_truthy(&self, value: &ZanoValue) -> bool {
        match value {
            ZanoValue::Boolean(b) => *b,
            ZanoValue::Null | ZanoValue::Undefined => false,
            ZanoValue::Number(n) => *n != 0.0,
            ZanoValue::String(s) => !s.is_empty(),
            _ => true,
        }
    }
}

impl Clone for ZanoRuntime {
    fn clone(&self) -> Self {
        Self {
            globals: self.globals.clone(),
            functions: self.functions.clone(),
            modules: self.modules.clone(),
        }
    }
}

struct UserDefinedFunction {
    params: Vec<String>,
    body: Vec<Statement>,
    runtime: ZanoRuntime,
}

#[async_trait]
impl ZanoFunction for UserDefinedFunction {
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue> {
        // Create new scope for function execution
        let function_runtime = self.runtime.clone();
        
        // Bind parameters
        for (i, param) in self.params.iter().enumerate() {
            let value = args.get(i).cloned().unwrap_or(ZanoValue::Undefined);
            function_runtime.globals.write().await.insert(param.clone(), value);
        }
        
        // Execute function body
        function_runtime.execute(self.body.clone()).await
    }
}

struct BuiltinFunction<F> {
    func: F,
}

impl<F> BuiltinFunction<F>
where
    F: Fn(Vec<ZanoValue>) -> Result<ZanoValue> + Send + Sync,
{
    fn new(func: F) -> Self {
        Self { func }
    }
}

#[async_trait]
impl<F> ZanoFunction for BuiltinFunction<F>
where
    F: Fn(Vec<ZanoValue>) -> Result<ZanoValue> + Send + Sync,
{
    async fn call(&self, args: Vec<ZanoValue>) -> Result<ZanoValue> {
        (self.func)(args)
    }
}