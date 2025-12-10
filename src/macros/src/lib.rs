//! Procedural macros for hyperparameter crate.
//!
//! This crate provides the `with_params!` macro for managing parameter scopes.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{quote, ToTokens};
use syn::visit::Visit;
use syn::{parse_macro_input, Expr, Ident, Token};
use syn::parse::{Parse, ParseStream, Result};

/// Get the path to the hyperparameter crate
fn crate_path() -> TokenStream2 {
    match crate_name("hyperparameter") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(#ident)
        }
        Err(_) => quote!(::hyperparameter),
    }
}

/// Compute xxhash64 at compile time for a key string
fn xxhash64(key: &str) -> u64 {
    xxhash_rust::xxh64::xxh64(key.as_bytes(), 42)
}

/// A dotted key like `a.b.c`
#[derive(Debug, Clone)]
struct DottedKey {
    segments: Vec<Ident>,
}

impl DottedKey {
    fn to_string_key(&self) -> String {
        self.segments
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }
}

impl Parse for DottedKey {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut segments = vec![input.parse::<Ident>()?];
        while input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            segments.push(input.parse::<Ident>()?);
        }
        Ok(DottedKey { segments })
    }
}

/// A set statement: `set a.b.c = expr;`
struct SetStatement {
    key: DottedKey,
    value: Expr,
}

impl Parse for SetStatement {
    fn parse(input: ParseStream) -> Result<Self> {
        // Already consumed 'set' keyword
        let key: DottedKey = input.parse()?;
        input.parse::<Token![=]>()?;
        let value: Expr = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(SetStatement { key, value })
    }
}

/// A get statement: `get name = a.b.c or default;`
struct GetStatement {
    name: Ident,
    key: DottedKey,
    default: Expr,
}

impl Parse for GetStatement {
    fn parse(input: ParseStream) -> Result<Self> {
        // Already consumed 'get' keyword
        let name: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let key: DottedKey = input.parse()?;
        
        // Parse 'or' keyword
        let or_ident: Ident = input.parse()?;
        if or_ident != "or" {
            return Err(syn::Error::new(or_ident.span(), "expected 'or'"));
        }
        
        let default: Expr = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(GetStatement { name, key, default })
    }
}

/// A params statement: `params scope_expr;`
struct ParamsStatement {
    scope: Expr,
}

impl Parse for ParamsStatement {
    fn parse(input: ParseStream) -> Result<Self> {
        // Already consumed 'params' keyword
        let scope: Expr = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(ParamsStatement { scope })
    }
}

/// Represents a single item in the with_params block
enum BlockItem {
    Set(SetStatement),
    Get(GetStatement),
    Params(ParamsStatement),
    Code(TokenStream2),
}

/// The parsed content of with_params! macro
struct WithParamsInput {
    items: Vec<BlockItem>,
}

impl Parse for WithParamsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = Vec::new();
        
        while !input.is_empty() {
            // Check for @set, @get, or @params syntax
            if input.peek(Token![@]) {
                let fork = input.fork();
                fork.parse::<Token![@]>()?; // peek '@'
                
                if fork.peek(Ident) {
                    let ident: Ident = fork.parse()?;
                    
                    if ident == "set" {
                        input.parse::<Token![@]>()?; // consume '@'
                        input.parse::<Ident>()?; // consume 'set'
                        let set_stmt: SetStatement = input.parse()?;
                        items.push(BlockItem::Set(set_stmt));
                        continue;
                    }
                    
                    if ident == "get" {
                        input.parse::<Token![@]>()?; // consume '@'
                        input.parse::<Ident>()?; // consume 'get'
                        let get_stmt: GetStatement = input.parse()?;
                        items.push(BlockItem::Get(get_stmt));
                        continue;
                    }
                    
                    if ident == "params" {
                        input.parse::<Token![@]>()?; // consume '@'
                        input.parse::<Ident>()?; // consume 'params'
                        let params_stmt: ParamsStatement = input.parse()?;
                        items.push(BlockItem::Params(params_stmt));
                        continue;
                    }
                }
                // If @ is followed by something other than set/get/params, 
                // treat it as normal code (fall through)
            }
            
            // Check for params keyword (still supports params without @)
            if input.peek(Ident) {
                let ident: Ident = input.fork().parse()?;
                
                if ident == "params" {
                    input.parse::<Ident>()?; // consume 'params'
                    let params_stmt: ParamsStatement = input.parse()?;
                    items.push(BlockItem::Params(params_stmt));
                    continue;
                }
            }
            
            // Otherwise, collect tokens until we see '@set', '@get', '@params', 'params', or end
            let mut code_tokens = TokenStream2::new();
            while !input.is_empty() {
                // Check if next is @set, @get, or @params
                if input.peek(Token![@]) {
                    let fork = input.fork();
                    fork.parse::<Token![@]>()?;
                    if fork.peek(Ident) {
                        if let Ok(ident) = fork.parse::<Ident>() {
                            if ident == "set" || ident == "get" || ident == "params" {
                                break;
                            }
                        }
                    }
                }
                
                // Check if next is params keyword
                if input.peek(Ident) {
                    let fork = input.fork();
                    if let Ok(ident) = fork.parse::<Ident>() {
                        if ident == "params" {
                            break;
                        }
                    }
                }
                
                // Parse one token tree
                let tt: proc_macro2::TokenTree = input.parse()?;
                code_tokens.extend(std::iter::once(tt));
            }
            
            if !code_tokens.is_empty() {
                items.push(BlockItem::Code(code_tokens));
            }
        }
        
        Ok(WithParamsInput { items })
    }
}

/// Visitor to detect .await in token stream
struct AwaitVisitor {
    has_await: bool,
}

impl AwaitVisitor {
    fn new() -> Self {
        Self { has_await: false }
    }
}

impl<'ast> Visit<'ast> for AwaitVisitor {
    fn visit_expr_await(&mut self, _: &'ast syn::ExprAwait) {
        self.has_await = true;
    }
}

/// Check if the token stream contains .await
fn contains_await(tokens: &TokenStream2) -> bool {
    let token_str = tokens.to_string();
    // Quick string check first
    if !token_str.contains(".await") && !token_str.contains(". await") {
        return false;
    }
    
    // Try to parse and visit for more accurate detection
    if let Ok(expr) = syn::parse2::<syn::File>(quote! { fn __check() { #tokens } }) {
        let mut visitor = AwaitVisitor::new();
        visitor.visit_file(&expr);
        return visitor.has_await;
    }
    
    // Fallback to string check
    true
}

/// Extract the last expression from a block
fn extract_last_expr(items: &[BlockItem]) -> Option<TokenStream2> {
    // Find the last code block
    let last_code = items.iter().rev().find_map(|item| {
        if let BlockItem::Code(code) = item {
            Some(code.clone())
        } else {
            None
        }
    })?;
    
    // First try to parse as a single expression (common case)
    if let Ok(expr) = syn::parse2::<syn::Expr>(last_code.clone()) {
        return Some(expr.to_token_stream());
    }
    
    // Try to parse as a block and extract the last expression
    if let Ok(block) = syn::parse2::<syn::Block>(last_code.clone()) {
        if let Some(last_stmt) = block.stmts.last() {
            // Only extract expression statements (not local declarations)
            if let syn::Stmt::Expr(expr, _) = last_stmt {
                return Some(expr.to_token_stream());
            }
        }
    }
    
    // Fallback: return the entire last code block
    Some(last_code)
}

/// Check if an expression likely returns a Future by analyzing its structure
/// This is a heuristic - we can't know actual types at macro expansion time
fn likely_returns_future(expr: &TokenStream2) -> bool {
    // Try to parse and analyze the expression structure first (most accurate)
    if let Ok(parsed) = syn::parse2::<syn::Expr>(expr.clone()) {
        match parsed {
            // Async closure - definitely returns Future
            syn::Expr::Closure(closure) => {
                if closure.asyncness.is_some() {
                    return true;
                }
            }
            // Function calls - be more aggressive in async context
            syn::Expr::Call(call) => {
                if let syn::Expr::Path(path) = &*call.func {
                    let full_path: String = path.path.segments.iter()
                        .map(|s| s.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");
                    
                    // Exclude known sync functions
                    if full_path.contains("thread::spawn") 
                        || full_path.contains("std::thread")
                        || full_path.contains("Vec::new")
                        || full_path.contains("String::new")
                        || full_path.contains("HashMap::new")
                        || full_path.contains("println!")
                        || full_path.contains("eprintln!")
                        || full_path.contains("format!") {
                        return false;
                    }
                    
                    // Exclude JoinHandle (users might want the handle, not the result)
                    if full_path.contains("JoinHandle") || full_path.contains("tokio::spawn") {
                        return false;
                    }
                    
                    let func_name = path.path.segments.last()
                        .map(|s| s.ident.to_string().to_lowercase())
                        .unwrap_or_default();
                    
                    // More comprehensive async function patterns
                    let async_func_patterns = [
                        "fetch", "request", "send", "receive",
                        "connect", "listen", "accept",
                        "timeout", "sleep", "delay", "wait",
                        "download", "upload", "load", "save",
                        "read", "write", "get", "post", "put", "delete",
                        "async", "await", "future",
                    ];
                    
                    for pattern in &async_func_patterns {
                        if func_name == *pattern || func_name.starts_with(pattern) || func_name.ends_with(pattern) {
                            return true;
                        }
                    }
                    
                    // If we're in an async context and it's a function call without .await,
                    // and it's not a known sync function, it might return Future
                    // This is a heuristic - user can always add explicit .await if needed
                    // We'll be conservative and only match if function name suggests async
                }
            }
            // Method calls - check method name
            syn::Expr::MethodCall(method) => {
                let method_name = method.method.to_string().to_lowercase();
                
                // Exclude methods that return handles
                if method_name == "spawn" || method_name.contains("handle") {
                    return false;
                }
                
                let async_method_patterns = [
                    "fetch", "request", "send", "receive",
                    "read_async", "write_async", "load_async", "save_async",
                    "get_async", "post_async", "put_async", "delete_async",
                    "connect", "listen", "accept",
                    "await", "into_future",
                ];
                
                for pattern in &async_method_patterns {
                    if method_name == *pattern || method_name.starts_with(pattern) {
                        return true;
                    }
                }
            }
            // Async block - definitely returns Future
            syn::Expr::Async(..) => {
                return true;
            }
            _ => {}
        }
    }
    
    // Fallback: string-based pattern matching (less accurate but catches edge cases)
    let expr_str = expr.to_string();
    
    // Check for explicit async patterns (definitive)
    let explicit_async_patterns = [
        "async {",
        "async move {",
        "tokio::join!",
        "tokio::try_join!",
        "futures::",
        "Future::",
    ];
    
    for pattern in &explicit_async_patterns {
        if expr_str.contains(pattern) {
            return true;
        }
    }
    
    false
}

/// Check if an expression should NOT be auto-awaited (e.g., JoinHandle)
fn should_not_auto_await(expr: &TokenStream2) -> bool {
    let expr_str = expr.to_string();
    
    // Types that implement IntoFuture but users typically want the handle itself
    let no_await_patterns = [
        "JoinHandle",
        "tokio::spawn",
        "tokio::task::spawn",
        "std::thread::spawn",
        "thread::spawn",
    ];
    
    for pattern in &no_await_patterns {
        if expr_str.contains(pattern) {
            return true;
        }
    }
    
    // Check parsed structure
    if let Ok(parsed) = syn::parse2::<syn::Expr>(expr.clone()) {
        match parsed {
            syn::Expr::Call(call) => {
                if let syn::Expr::Path(path) = &*call.func {
                    let full_path: String = path.path.segments.iter()
                        .map(|s| s.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");
                    
                    if full_path.contains("spawn") || full_path.contains("JoinHandle") {
                        return true;
                    }
                }
            }
            syn::Expr::MethodCall(method) => {
                let method_name = method.method.to_string().to_lowercase();
                if method_name == "spawn" || method_name.contains("handle") {
                    return true;
                }
            }
            _ => {}
        }
    }
    
    false
}

/// Wrap an expression with .await if it likely returns a Future
fn maybe_add_await(expr: TokenStream2) -> TokenStream2 {
    // Don't auto-await if it already has .await
    let expr_str = expr.to_string();
    if expr_str.contains(".await") {
        return expr;
    }
    
    // Don't auto-await if it's a type that shouldn't be awaited
    if should_not_auto_await(&expr) {
        return expr;
    }
    
    // Check if it likely returns a Future
    if likely_returns_future(&expr) {
        // Wrap with .await
        quote! {
            (#expr).await
        }
    } else {
        expr
    }
}

/// Generate code for a set statement
fn generate_set(set: &SetStatement, hp: &TokenStream2) -> TokenStream2 {
    let key_str = set.key.to_string_key();
    let key_hash = xxhash64(&key_str);
    let value = &set.value;
    
    quote! {
        #hp::with_current_storage(|__hp_s| {
            __hp_s.put_with_hash(#key_hash, #key_str, #value);
        });
    }
}

/// Generate code for a get statement
fn generate_get(get: &GetStatement, hp: &TokenStream2) -> TokenStream2 {
    let name = &get.name;
    let key_str = get.key.to_string_key();
    let key_hash = xxhash64(&key_str);
    let default = &get.default;
    
    quote! {
        let #name = #hp::with_current_storage(|__hp_s| {
            __hp_s.get_or_else(#key_hash, #default)
        });
    }
}

/// Generate the synchronous version of with_params
fn generate_sync(items: &[BlockItem], hp: &TokenStream2) -> TokenStream2 {
    // Check if there's a params statement at the beginning
    let (params_setup, remaining_items) = extract_params_setup(items);
    
    let mut body = TokenStream2::new();
    
    for item in remaining_items {
        let code = match item {
            BlockItem::Set(set) => generate_set(set, hp),
            BlockItem::Get(get) => generate_get(get, hp),
            BlockItem::Params(_) => {
                // Additional params statements create nested scopes
                quote! {}
            }
            BlockItem::Code(code) => code.clone(),
        };
        body.extend(code);
    }
    
    if let Some(scope_expr) = params_setup {
        // With external ParamScope
        quote! {{
            let mut __hp_ps = #scope_expr;
            let __hp_guard = __hp_ps.enter_guard();
            let __hp_result = { #body };
            drop(__hp_guard);
            __hp_result
        }}
    } else {
        // Without external ParamScope
        quote! {{
            #hp::with_current_storage(|__hp_s| __hp_s.enter());
            
            struct __HpGuard;
            impl Drop for __HpGuard {
                fn drop(&mut self) {
                    #hp::with_current_storage(|__hp_s| { __hp_s.exit(); });
                }
            }
            let __hp_guard = __HpGuard;
            
            let __hp_result = { #body };
            
            drop(__hp_guard);
            __hp_result
        }}
    }
}

/// Generate the asynchronous version of with_params
/// Automatically handles Future return types by awaiting them
fn generate_async(items: &[BlockItem], hp: &TokenStream2) -> TokenStream2 {
    // Check if there's a params statement at the beginning
    let (params_setup, remaining_items) = extract_params_setup(items);
    
    // Extract the last expression for auto-await detection
    // In async context, we're aggressive: if it's a function/method call or async block
    // without .await and not explicitly excluded, we'll auto-await it
    let last_expr = extract_last_expr(&remaining_items);
    let should_auto_await = last_expr.as_ref().map(|e| {
        // Don't auto-await if explicitly excluded (e.g., JoinHandle)
        if should_not_auto_await(e) {
            return false;
        }
        
        // Check if it already has .await
        let expr_str = e.to_string();
        if expr_str.contains(".await") {
            return false;
        }
        
        // In async context, be aggressive: auto-await function/method calls and async blocks
        if let Ok(parsed) = syn::parse2::<syn::Expr>(e.clone()) {
            match parsed {
                syn::Expr::Call(_) | syn::Expr::MethodCall(_) | syn::Expr::Async(_) => {
                    // Assume these return Future in async context
                    return true;
                }
                syn::Expr::Closure(closure) => {
                    if closure.asyncness.is_some() {
                        return true;
                    }
                }
                _ => {
                    // For other expressions, use heuristic
                    return likely_returns_future(e);
                }
            }
        }
        
        false
    }).unwrap_or(false);
    
    let mut body = TokenStream2::new();
    let mut last_code_idx = None;
    
    // First pass: find the last code block index
    for (idx, item) in remaining_items.iter().enumerate() {
        if matches!(item, BlockItem::Code(_)) {
            last_code_idx = Some(idx);
        }
    }
    
    // Build body, auto-awaiting the last expression if needed
    for (idx, item) in remaining_items.iter().enumerate() {
        let is_last_code = last_code_idx == Some(idx) && should_auto_await;
        
        let code = match item {
            BlockItem::Set(set) => generate_set(set, hp),
            BlockItem::Get(get) => generate_get(get, hp),
            BlockItem::Params(_) => quote! {},
            BlockItem::Code(code) => {
                if is_last_code {
                    // This is the last code block and we should auto-await
                    // First try as a single expression (common case like `fetch_data()`)
                    if let Ok(expr) = syn::parse2::<syn::Expr>(code.clone()) {
                        let expr_tokens = expr.to_token_stream();
                        let expr_str = expr_tokens.to_string();
                        
                        if !expr_str.contains(".await") {
                            maybe_add_await(expr_tokens)
                        } else {
                            code.clone()
                        }
                    } else if let Ok(mut block) = syn::parse2::<syn::Block>(code.clone()) {
                        // Try as a block and modify the last expression
                        if let Some(syn::Stmt::Expr(expr, _)) = block.stmts.last_mut() {
                            let expr_tokens = expr.to_token_stream();
                            let expr_str = expr_tokens.to_string();
                            
                            if !expr_str.contains(".await") {
                                let awaited_expr = maybe_add_await(expr_tokens);
                                
                                if let Ok(new_expr) = syn::parse2::<syn::Expr>(awaited_expr) {
                                    *expr = new_expr;
                                    block.to_token_stream()
                                } else {
                                    code.clone()
                                }
                            } else {
                                code.clone()
                            }
                        } else {
                            code.clone()
                        }
                    } else {
                        code.clone()
                    }
                } else {
                    code.clone()
                }
            }
        };
        body.extend(code);
    }
    
    if let Some(scope_expr) = params_setup {
        // With external ParamScope - need to enter it and bind to async
        quote! {{
            let mut __hp_ps = #scope_expr;
            let __hp_guard = __hp_ps.enter_guard();
            #hp::bind(async move { #body }).await
        }}
    } else {
        // Without external ParamScope
        quote! {{
            // Capture current storage and create a new one for the async task
            let __hp_storage = #hp::with_current_storage(|__hp_s| {
                __hp_s.clone_for_async()
            });
            
            #hp::storage_scope(
                ::std::cell::RefCell::new(__hp_storage),
                async {
                    #hp::with_current_storage(|__hp_s| __hp_s.enter());
                    
                    struct __HpGuard;
                    impl Drop for __HpGuard {
                        fn drop(&mut self) {
                            #hp::with_current_storage(|__hp_s| { __hp_s.exit(); });
                        }
                    }
                    let __hp_guard = __HpGuard;
                    
                    let __hp_result = { #body };
                    
                    drop(__hp_guard);
                    __hp_result
                }
            ).await
        }}
    }
}

/// Extract params statement if it's the first item
fn extract_params_setup(items: &[BlockItem]) -> (Option<TokenStream2>, &[BlockItem]) {
    if let Some(BlockItem::Params(params)) = items.first() {
        let scope = &params.scope;
        (Some(quote! { #scope }), &items[1..])
    } else {
        (None, items)
    }
}

/// The main `with_params!` procedural macro.
///
/// # Example
/// ```ignore
/// // Basic usage
/// with_params! {
///     @set a.b = 1;
///     @set c.d = 2.0;
///     
///     @get val = a.b or 0;
///     
///     process(val)
/// }
///
/// // With external ParamScope
/// with_params! {
///     params config.param_scope();
///     
///     @get val = some.key or "default".to_string();
///     println!("{}", val);
/// }
/// ```
#[proc_macro]
pub fn with_params(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as WithParamsInput);
    let hp = crate_path();
    
    // Collect all code tokens to check for await
    let mut all_code = TokenStream2::new();
    for item in &input.items {
        match item {
            BlockItem::Code(code) => all_code.extend(code.clone()),
            BlockItem::Set(set) => all_code.extend(set.value.to_token_stream()),
            BlockItem::Get(get) => all_code.extend(get.default.to_token_stream()),
            BlockItem::Params(params) => all_code.extend(params.scope.to_token_stream()),
        }
    }
    
    // Check for explicit .await (most reliable indicator)
    let has_explicit_await = contains_await(&all_code);
    
    // Check if last expression likely returns Future (heuristic-based)
    let last_expr = extract_last_expr(&input.items);
    let likely_future = last_expr.as_ref()
        .map(|e| likely_returns_future(e))
        .unwrap_or(false);
    
    // Use async version if:
    // 1. Has explicit .await (definitive), OR
    // 2. Last expression likely returns Future (heuristic)
    // 
    // Note: We prioritize explicit .await for accuracy, but also check
    // for Future-returning patterns to catch cases where user forgot .await
    let use_async = has_explicit_await || likely_future;
    
    let output = if use_async {
        // Generate async version - will handle Future return types
        generate_async(&input.items, &hp)
    } else {
        // Generate sync version
        generate_sync(&input.items, &hp)
    };
    
    output.into()
}

/// The `get_param!` macro for getting a parameter with compile-time key hashing.
///
/// # Example
/// ```ignore
/// let val: i64 = get_param!(a.b.c, 0);
/// let name: String = get_param!(user.name, "default".to_string());
/// ```
#[proc_macro]
pub fn get_param(input: TokenStream) -> TokenStream {
    let input2: TokenStream2 = input.into();
    let input_str = input2.to_string();
    let hp = crate_path();
    
    // Parse: key, default [, help]
    // Find commas to split - we need at least key and default
    let parts: Vec<&str> = input_str.splitn(2, ',').collect();
    if parts.len() < 2 {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected: get_param!(key.path, default) or get_param!(key.path, default, \"help\")"
        ).to_compile_error().into();
    }
    
    let key_str = parts[0].trim().replace(' ', "");
    let rest = parts[1].trim();
    
    // Check if there's a help string (third argument)
    // For now, just take everything after the first comma as the default
    // A more sophisticated parser could handle the help string
    let default_str = if let Some(comma_pos) = rest.rfind(',') {
        // Check if the part after the last comma looks like a string literal
        let after_comma = rest[comma_pos + 1..].trim();
        if after_comma.starts_with('"') {
            // Has help string, use the part before as default
            rest[..comma_pos].trim()
        } else {
            rest
        }
    } else {
        rest
    };
    
    let key_hash = xxhash64(&key_str);
    
    // Parse default as expression
    let default: TokenStream2 = default_str.parse().unwrap_or_else(|_| {
        let s = default_str;
        quote! { #s }
    });
    
    let output = quote! {
        #hp::with_current_storage(|__hp_s| {
            __hp_s.get_or_else(#key_hash, #default)
        })
    };
    
    output.into()
}
