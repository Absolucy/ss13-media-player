// SPDX-License-Identifier: Zlib
#[macro_use]
extern crate log;

mod error;
pub(crate) mod topic;
pub(crate) mod util;

use crate::util::ObjectHelperThing;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
	AudioContext, DistanceModelType, Event, HtmlAudioElement, PannerNode, PanningModelType,
	js_sys::Object,
};

#[wasm_bindgen]
pub struct SpatialAudioPlayer {
	context: AudioContext,
	audio_element: HtmlAudioElement,
	panner: PannerNode,
	waiting_to_play: Rc<Cell<bool>>,
}

impl SpatialAudioPlayer {
	pub fn serialize_state(&self) -> Object {
		Object::new()
			.set("url", self.audio_element.src())
			.set("muted", self.audio_element.muted())
			.set("volume", self.audio_element.volume())
			.set("current_time", self.audio_element.current_time())
			.set("distance", self.serialize_distance())
			.set("position", self.serialize_position())
	}

	fn serialize_distance(&self) -> Object {
		Object::new()
			.set("ref_distance", self.panner.ref_distance())
			.set("max_distance", self.panner.max_distance())
	}

	fn serialize_position(&self) -> Object {
		Object::new()
			.set("x", self.panner.position_x().value())
			.set("y", self.panner.position_y().value())
			.set("z", self.panner.position_z().value())
	}
}

#[wasm_bindgen]
impl SpatialAudioPlayer {
	#[wasm_bindgen(constructor)]
	pub fn new() -> Result<SpatialAudioPlayer, JsValue> {
		error::set_once();
		console_log::init_with_level(log::Level::Debug).expect("failed to init console logger");

		let waiting_to_play = Rc::new(Cell::new(false));

		let context = AudioContext::new()?;

		let audio_element = web_sys::window()
			.expect("no window???")
			.document()
			.expect("no document???")
			.get_element_by_id("meow")
			.expect("no audio player???")
			.dyn_into::<HtmlAudioElement>()?;

		let canplay: Closure<dyn FnMut(_)> = Closure::wrap(Box::new({
			let waiting_to_play = waiting_to_play.clone();
			move |event: Event| {
				debug!("canplay event, waiting_to_play={}", waiting_to_play.get());
				if waiting_to_play.get() {
					debug!("we're waiting to play!");
					let audio = event
						.current_target()
						.expect("failed to get current_target")
						.dyn_into::<HtmlAudioElement>()
						.expect("current_target was not HtmlAudioElement");
					if audio.ready_state() >= 3 {
						debug!("ready_state >= 3, we're playing now");
						let _ = audio.play().expect("failed to play audio");
						waiting_to_play.set(false);
					}
				}
			}
		}) as Box<dyn FnMut(_)>);
		audio_element
			.add_event_listener_with_callback("canplay", canplay.as_ref().unchecked_ref())
			.expect("failed to add canplay event listener");
		canplay.forget();

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
			waiting_to_play,
		})
	}

	#[wasm_bindgen]
	pub fn set_url(&self, url: &str) -> Result<(), JsValue> {
		self.stop()?;

		self.audio_element.set_src(url);
		self.audio_element.set_preload("auto");
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
		if self.audio_element.src().trim().is_empty() {
			return Ok(());
		}
		self.waiting_to_play.set(true);
		if self.audio_element.ready_state() >= 3 {
			self.audio_element.play().map(JsFuture::from)?.await?;
			if !self.audio_element.paused() {
				self.waiting_to_play.set(false);
				debug!("audio successfully playing");
			} else {
				debug!("audio failed to play (2), loading");
				self.audio_element.load();
			}
		} else {
			debug!("audio failed to play (1), loading");
			self.audio_element.load();
		}
		Ok(())
	}

	#[wasm_bindgen]
	pub fn pause(&self) -> Result<(), JsValue> {
		self.audio_element.pause()?;
		Ok(())
	}

	#[wasm_bindgen]
	pub fn stop(&self) -> Result<(), JsValue> {
		self.waiting_to_play.set(false);
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

	#[wasm_bindgen]
	pub fn current_state(&self) -> JsValue {
		self.serialize_state().into()
	}

	#[wasm_bindgen]
	pub fn verify_playing(&self) {
		let waiting_to_play = &self.waiting_to_play;
		let audio = &self.audio_element;
		if waiting_to_play.get() && audio.paused() {
			debug!("check_playing: we should be playing, but aren't");
			if audio.ready_state() >= 3 {
				debug!("check_playing: ready state >= 3, trying to play");
				let _ = audio.play().expect("failed to play audio");
				if !audio.paused() {
					debug!("check_playing: successfully played");
					waiting_to_play.set(false);
				}
			} else {
				debug!("check_playing: not ready, trying to load");
				audio.load();
				if audio.ready_state() >= 3 {
					debug!("check_playing: we loaded, trying to play");
					let _ = audio.play().expect("failed to play audio");
					if !audio.paused() {
						debug!("check_playing: successfully played (2)");
						waiting_to_play.set(false);
					}
				}
			}
		}
	}
}
