// SPDX-License-Identifier: Zlib
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = Byond, js_name = topic)]
	pub fn byond_topic(name: &str, params: &JsValue);
}
