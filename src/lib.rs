// SPDX-License-Identifier: Zlib
mod error;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioContext, DistanceModelType, HtmlAudioElement, PannerNode, PanningModelType};

#[wasm_bindgen]
pub struct SpatialAudioPlayer {
	context: AudioContext,
	audio_element: HtmlAudioElement,
	panner: PannerNode,
}

#[wasm_bindgen]
impl SpatialAudioPlayer {
	#[wasm_bindgen(constructor)]
	pub fn new() -> Result<SpatialAudioPlayer, JsValue> {
		error::set_once();
		console_log::init_with_level(log::Level::Debug).expect("failed to init console logger");

		let context = AudioContext::new()?;

		let audio_element = web_sys::window()
			.expect("no window???")
			.document()
			.expect("no document???")
			.get_element_by_id("meow")
			.expect("no audio player???")
			.dyn_into::<HtmlAudioElement>()?;

		let source = context.create_media_element_source(&audio_element)?;

		let panner = context.create_panner().expect("failed to create panner");
		panner.set_panning_model(PanningModelType::Hrtf);
		panner.set_distance_model(DistanceModelType::Inverse);
		panner.set_ref_distance(1.0);
		panner.set_max_distance(10.0);
		panner.set_rolloff_factor(1.0);
		panner.set_cone_inner_angle(360.0);
		panner.set_cone_outer_angle(360.0);
		panner.set_cone_outer_gain(0.0);

		source.connect_with_audio_node(&panner)?;
		panner.connect_with_audio_node(&context.destination())?;

		Ok(SpatialAudioPlayer {
			context,
			audio_element,
			panner,
		})
	}

	#[wasm_bindgen]
	pub fn set_url(&self, url: &str) -> Result<(), JsValue> {
		self.stop()?;

		self.audio_element.set_src(url);
		self.audio_element.set_cross_origin(Some("anonymous"));
		self.audio_element.set_current_time(0.0);
		self.audio_element.set_muted(false);

		Ok(())
	}

	#[wasm_bindgen]
	pub fn set_position(&self, x: f32, y: f32) -> Result<(), JsValue> {
		let current_time = self.context.current_time();

		self.panner
			.position_x()
			.set_value_at_time(x, current_time)?;
		self.panner
			.position_y()
			.set_value_at_time(y, current_time)?;
		self.panner
			.position_z()
			.set_value_at_time(0.0, current_time)?;

		Ok(())
	}

	#[wasm_bindgen]
	pub fn set_ref_distance(&self, distance: f64) {
		self.panner.set_ref_distance(distance);
	}

	#[wasm_bindgen]
	pub fn set_max_distance(&self, distance: f64) {
		self.panner.set_max_distance(distance);
	}

	#[wasm_bindgen]
	pub fn set_time(&self, time: f64) {
		self.audio_element.set_current_time(time);
	}

	#[wasm_bindgen]
	pub fn set_volume(&self, volume: f64) -> Result<(), JsValue> {
		self.audio_element.set_volume(volume / 100.0);
		Ok(())
	}

	#[wasm_bindgen]
	pub async fn play(&self) -> Result<(), JsValue> {
		self.audio_element.play().map(JsFuture::from)?.await?;
		Ok(())
	}

	#[wasm_bindgen]
	pub fn pause(&self) -> Result<(), JsValue> {
		self.audio_element.pause()?;
		Ok(())
	}

	#[wasm_bindgen]
	pub fn stop(&self) -> Result<(), JsValue> {
		self.audio_element.pause()?;
		self.audio_element.set_muted(true);
		self.audio_element.set_current_time(0.0);
		self.audio_element.set_volume(1.0);
		self.panner.position_x().set_value_at_time(0.0, 0.0)?;
		self.panner.position_y().set_value_at_time(0.0, 0.0)?;
		self.panner.position_z().set_value_at_time(0.0, 0.0)?;
		self.audio_element.set_src("");
		Ok(())
	}
}
