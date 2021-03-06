use crate::common::list::ListOptions;
use crate::offline::print::PrintOptions;
use crate::offline::random::RandomOptions;
use crate::offline::restore::RestoreOptions;
use crate::offline::sign::SignOptions;
use crate::*;
use android_logger::Config;
use bitcoin::Network;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use log::{debug, info, Level};
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::str::FromStr;

fn rust_call(c_str: &CStr) -> Result<CString> {
    let str = c_str.to_str()?;
    let value: Value = serde_json::from_str(str)?;
    let datadir = value
        .get("datadir")
        .and_then(|s| s.as_str())
        .ok_or_else(|| Error::Generic("missing datadir".into()))?;
    let network = value
        .get("network")
        .and_then(|s| s.as_str())
        .ok_or_else(|| Error::Generic("missing network".into()))?;
    let network = Network::from_str(network)?;
    let method = value.get("method").and_then(|s| s.as_str());
    let args = value.get("args").unwrap_or(&Value::Null);
    info!(
        "method:{:?} datadir:{} network:{} args:{:?}",
        method, datadir, network, args
    );

    let value = match method {
        Some("random") => {
            let random_opts: RandomOptions = serde_json::from_value(args.clone())?;
            let result = crate::offline::random::create_key(datadir, network, &random_opts)?;
            serde_json::to_value(result)?
        }
        Some("list") => {
            let list_opts: ListOptions = serde_json::from_value(args.clone())?;
            let result = crate::common::list::list(datadir, network, &list_opts)?;
            serde_json::to_value(result)?
        }
        Some("merge_qrs") => {
            let string_values: Vec<String> = serde_json::from_value(args.clone())?;
            let mut values = vec![];
            for string in string_values {
                values.push(hex::decode(&string)?);
            }
            match crate::common::qr::merge_qrs(values) {
                Ok(merged) => hex::encode(merged).into(),
                Err(e) => e.to_json(),
            }
        }
        Some("create_qrs") => {
            let opts: CreateQrOptions = serde_json::from_value(args.clone())?;
            crate::common::qr::create_qrs(&opts)?;
            Value::Null
        }
        Some("sign") => {
            let opts: SignOptions = serde_json::from_value(args.clone())?;
            let result = crate::offline::sign::start(&opts, network)?;
            serde_json::to_value(result)?
        }
        Some("restore") => {
            let opts: RestoreOptions = serde_json::from_value(args.clone())?;
            let result = crate::offline::restore::start(datadir, network, &opts)?;
            serde_json::to_value(result)?
        }
        Some("print") => {
            let opts: PrintOptions = serde_json::from_value(args.clone())?;
            let result = crate::offline::print::start(datadir, network, &opts)?;
            serde_json::to_value(result)?
        }
        _ => {
            let error: Error = "invalid method".into();
            error.to_json()
        }
    };
    let result = serde_json::to_string(&value)?;
    debug!("result: ({})", result);
    Ok(CString::new(result)?)
}

#[no_mangle]
pub extern "C" fn c_call(to: *const c_char) -> *mut c_char {
    android_logger::init_once(Config::default().with_min_level(Level::Debug));
    let input = unsafe { CStr::from_ptr(to) };
    info!("<-- ({:?})", input);
    let output = rust_call(input)
        .unwrap_or_else(|e| CString::new(serde_json::to_vec(&e.to_json()).unwrap()).unwrap());

    info!("--> ({:?})", output);
    output.into_raw()
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn Java_it_casatta_Rust_call(
    env: JNIEnv,
    _: JClass,
    java_pattern: JString,
) -> jstring {
    // Our Java companion code might pass-in "world" as a string, hence the name.
    let world = c_call(
        env.get_string(java_pattern)
            .expect("invalid pattern string")
            .as_ptr(),
    );
    // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
    let world_ptr = CString::from_raw(world);
    let output = env
        .new_string(world_ptr.to_str().unwrap())
        .expect("Couldn't create java string!");

    output.into_inner()
}
