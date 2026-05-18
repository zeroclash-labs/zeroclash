//! JavaScript script execution for the enhance pipeline.
//! Uses the Boa JavaScript engine for safe, sandboxed script evaluation.

use super::use_lowercase;
use anyhow::{Result, bail};
use parking_lot::Mutex;
use serde_json::Value as JsonValue;
use serde_yaml_ng::Mapping;
use std::sync::Arc;
use std::time::Duration;

const MAX_OUTPUTS: usize = 1000;
const MAX_OUTPUT_SIZE: usize = 1024 * 1024;
const MAX_JSON_SIZE: usize = 10 * 1024 * 1024;
const SCRIPT_TIMEOUT: Duration = Duration::from_secs(5);

/// Execute a JavaScript script against a clash config YAML mapping.
/// Returns the modified config and a log of console outputs.
pub async fn use_script(
    script: String,
    config: Mapping,
    profile_name: String,
) -> Result<(Mapping, Vec<(String, String)>)> {
    let handle = tokio::task::spawn_blocking(move || {
        use_script_sync(&script, &config, &profile_name)
    });

    match tokio::time::timeout(SCRIPT_TIMEOUT, handle).await {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => Err(anyhow::anyhow!("script panicked: {e}")),
        Err(_) => Err(anyhow::anyhow!("script timed out after 5s")),
    }
}

fn use_script_sync(
    script: &str,
    config: &Mapping,
    name: &str,
) -> Result<(Mapping, Vec<(String, String)>)> {
    use boa_engine::Context;
    use boa_engine::Source;
    use boa_engine::JsString;
    use boa_engine::JsValue;
    use boa_engine::native_function::NativeFunction;

    let mut context = Context::default();
    context
        .runtime_limits_mut()
        .set_loop_iteration_limit(10_000_000);

    let outputs = Arc::new(Mutex::new(vec![]));
    let total_size = Arc::new(Mutex::new(0usize));

    let outputs_clone = Arc::clone(&outputs);
    let total_size_clone = Arc::clone(&total_size);

    // Register __verge_log__ for console.log/info/error etc.
    let _ = context.register_global_builtin_callable(
        "__verge_log__".into(),
        2,
        // SAFETY: The closure only accesses Arc<Mutex<>> which is Send+Sync safe.
        // No mutable static state or FFI boundaries are crossed.
        unsafe { NativeFunction::from_closure(
            move |_: &JsValue, args: &[JsValue], context: &mut Context| {
                let level = args.first().ok_or_else(|| {
                    boa_engine::JsError::from_opaque(JsString::from("Missing level").into())
                })?;
                let level = level.to_string(context)?.to_std_string().map_err(|_| {
                    boa_engine::JsError::from_opaque(JsString::from("Invalid level").into())
                })?;

                let data = args.get(1).ok_or_else(|| {
                    boa_engine::JsError::from_opaque(JsString::from("Missing data").into())
                })?;
                let data = data.to_string(context)?.to_std_string().map_err(|_| {
                    boa_engine::JsError::from_opaque(JsString::from("Invalid data").into())
                })?;

                if outputs_clone.lock().len() >= MAX_OUTPUTS {
                    return Err(boa_engine::JsError::from_opaque(
                        JsString::from("Max outputs exceeded").into(),
                    ));
                }
                let mut size = total_size_clone.lock();
                let new_size = *size + level.len() + data.len();
                if new_size > MAX_OUTPUT_SIZE {
                    return Err(boa_engine::JsError::from_opaque(
                        JsString::from("Max output size exceeded").into(),
                    ));
                }
                *size = new_size;
                drop(size);
                outputs_clone.lock().push((level, data));
                Ok(JsValue::undefined())
            },
        ) },
    );

    // Set up console object
    let _ = context.eval(Source::from_bytes(
        r#"var console = Object.freeze({
        log(data){__verge_log__("log",JSON.stringify(data,null,2))},
        info(data){__verge_log__("info",JSON.stringify(data,null,2))},
        error(data){__verge_log__("error",JSON.stringify(data,null,2))},
        debug(data){__verge_log__("debug",JSON.stringify(data,null,2))},
        warn(data){__verge_log__("warn",JSON.stringify(data,null,2))},
      });"#,
    ));

    let config = use_lowercase(config);
    let config_str = serde_json::to_string(&config)?;
    if config_str.len() > MAX_JSON_SIZE {
        bail!("Config size exceeds maximum");
    }

    let safe_name = name.replace('\\', "\\\\").replace('\'', "\\'");
    if safe_name.len() > 1024 {
        bail!("Name too long");
    }

    let code = format!(
        r"try{{
        {script};
        JSON.stringify(main({config_str},'{safe_name}')||'')
      }} catch(err) {{
        `__error_flag__ ${{err.toString()}}`
      }}"
    );

    match context.eval(Source::from_bytes(code.as_str())) {
        Ok(result) => {
            if !result.is_string() {
                bail!("main() should return an object");
            }
            let result = result
                .to_string(&mut context)
                .map_err(|e| anyhow::anyhow!("Failed to convert result: {e}"))?;
            let result = result
                .to_std_string()
                .map_err(|_| anyhow::anyhow!("Failed to convert result string"))?;

            if result.len() > MAX_JSON_SIZE {
                bail!("Script result exceeds max size");
            }

            match serde_json::from_str::<Mapping>(result.trim_matches('"')) {
                Ok(config) => {
                    Ok((use_lowercase(&config), outputs.lock().to_vec()))
                }
                Err(e) => {
                    outputs.lock().push(("exception".into(), format!("Script parse error: {e}")));
                    Ok((config.clone(), outputs.lock().to_vec()))
                }
            }
        }
        Err(e) => {
            bail!("Script execution failed: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_basic() {
        let script = r#"
        function main(config) {
          if (Array.isArray(config.rules)) {
            config.rules = [...config.rules, "added-rule"];
          }
          console.log("test log");
          return config;
        }
        "#;

        let config: Mapping = serde_yaml_ng::from_str("rules:\n  - rule1\n  - rule2").unwrap();
        let result = use_script_sync(script, &config, "test");
        assert!(result.is_ok());
        let (cfg, logs) = result.unwrap();
        assert!(logs.iter().any(|(l, _)| l == "log"));
        let rules = cfg.get("rules").unwrap().as_sequence().unwrap();
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[2].as_str().unwrap(), "added-rule");
    }
}
