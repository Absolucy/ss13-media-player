// SPDX-License-Identifier: Zlib
use wasm_bindgen::JsValue;
use web_sys::js_sys::{Object, Reflect};

pub trait ObjectHelperThing: Sized {
	fn set<Key, Value>(self, key: Key, value: Value) -> Self
	where
		Key: Into<JsValue>,
		Value: Into<JsValue>,
	{
		self.set_in_place(key, value);
		self
	}

	fn set_in_place<Key, Value>(&self, key: Key, value: Value) -> &Self
	where
		Key: Into<JsValue>,
		Value: Into<JsValue>;
}

impl ObjectHelperThing for Object {
	fn set_in_place<Key, Value>(&self, key: Key, value: Value) -> &Self
	where
		Key: Into<JsValue>,
		Value: Into<JsValue>,
	{
		let key = key.into();
		let value = value.into();
		let _ = Reflect::set(self, &key, &value);
		self
	}
}
