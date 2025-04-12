// SPDX-License-Identifier: Zlib
//! this is literally just the console_error_panic_hook crate, but it uses
//! log::error! instead, so we can have fancy colored output.

use crate::util::ObjectHelperThing;
use log::error;
use std::panic::PanicHookInfo;
use wasm_bindgen::prelude::*;
use web_sys::js_sys::Object;

#[wasm_bindgen]
extern "C" {
	type Error;

	#[wasm_bindgen(constructor)]
	fn new() -> Error;

	#[wasm_bindgen(structural, method, getter)]
	fn stack(error: &Error) -> String;
}

fn hook_impl(info: &PanicHookInfo) {
	let mut msg = info.to_string();

	// Add the error stack to our message.
	//
	// This ensures that even if the `console` implementation doesn't
	// include stacks for `console.error`, the stack is still available
	// for the user. Additionally, Firefox's console tries to clean up
	// stack traces, and ruins Rust symbols in the process
	// (https://bugzilla.mozilla.org/show_bug.cgi?id=1519569) but since
	// it only touches the logged message's associated stack, and not
	// the message's contents, by including the stack in the message
	// contents we make sure it is available to the user.
	msg.push_str("\n\nStack:\n\n");
	let e = Error::new();
	let stack = e.stack();
	msg.push_str(&stack);

	// Safari's devtools, on the other hand, _do_ mess with logged
	// messages' contents, so we attempt to break their heuristics for
	// doing that by appending some whitespace.
	// https://github.com/rustwasm/console_error_panic_hook/issues/7
	msg.push_str("\n\n");

	// Finally, log the panic with `console.error`!
	error!("{}", msg);

	let error_params = Object::new().set("message", msg.trim());
	if let Some(location) = info.location() {
		error_params.set_in_place("source", location.file());
		error_params.set_in_place("line", location.line());
		error_params.set_in_place("col", location.column());
	}
	crate::topic::byond_topic("error", &error_params);
}

fn hook(info: &PanicHookInfo) {
	hook_impl(info);
}

#[inline]
pub(crate) fn set_once() {
	use std::sync::Once;
	static SET_HOOK: Once = Once::new();
	SET_HOOK.call_once(|| {
		std::panic::set_hook(Box::new(hook));
	});
}
