// SPDX-License-Identifier: Zlib
const { SpatialAudioPlayer } = wasm_bindgen;

const READY_DELAY = 500;
const UPDATE_STATE_DELAY = 500;

var MediaPlayer = (window.MediaPlayer = {});
var Byond = (window.Byond = {});
Byond.topic = (type, params) => {
	let url = `byond://?src=media:href;type=${encodeURIComponent(type)}`;
	if (params) {
		url += `;params=${encodeURIComponent(JSON.stringify(params))}`;
	}
	location.href = url;
};

MediaPlayer.player = null;
MediaPlayer.last_state = null;
MediaPlayer.update_state_timeout = null;
MediaPlayer._update_state = () => {
	const new_state = MediaPlayer.player.current_state();
	if (!Object.is(new_state, MediaPlayer.last_state)) {
		MediaPlayer.last_state = new_state;
		Byond.topic("state", new_state);
	}
};
MediaPlayer.update_state = () => {
	if (MediaPlayer.update_state_timeout) {
		clearTimeout(MediaPlayer.update_state_timeout);
		MediaPlayer.update_state_timeout = null;
	}
	MediaPlayer.update_state_timeout = setTimeout(() => {
		MediaPlayer._update_state();
		MediaPlayer.update_state_timeout = null;
	}, UPDATE_STATE_DELAY);
};

async function setup() {
	await wasm_bindgen("media_player.wasm");
	MediaPlayer.player = new SpatialAudioPlayer();
	setTimeout(() => {
		Byond.topic("ready");
		MediaPlayer._update_state();
	}, READY_DELAY);
}

window.onerror = (message, source, line, col, error) => {
	Byond.topic("error", {
		message: message,
		source: source,
		line: line,
		col: col,
		error: error,
	});
	return true;
};

window.onunhandledrejection = (error) => {
	let msg = "UnhandledRejection";
	if (error.reason) {
		msg += `: ${error.reason.message || error.reason.description || error.reason}`;
		if (error.reason.stack) {
			error.reason.stack = `UnhandledRejection: ${error.reason.stack}`;
		}
	}
	window.onerror(msg, null, null, null, error.reason);
};

document.onreadystatechange = () => {
	if (document.readyState === "complete") {
		setup();
	}
};

window.set_url = (url) => {
	MediaPlayer.player.set_url(url);
	MediaPlayer.update_state();
};

window.set_position = (x, y) => {
	MediaPlayer.player.set_position(x, y);
	MediaPlayer.update_state();
};

window.set_ref_distance = (distance) => {
	MediaPlayer.player.set_ref_distance(distance);
	MediaPlayer.update_state();
};

window.set_max_distance = (distance) => {
	MediaPlayer.player.set_max_distance(distance);
	MediaPlayer.update_state();
};

window.set_time = (time) => {
	MediaPlayer.player.set_time(time);
	MediaPlayer.update_state();
};

window.set_volume = (volume) => {
	MediaPlayer.player.set_volume(volume);
	MediaPlayer.update_state();
};

window.play = (url, volume) => {
	if (url) {
		MediaPlayer.player.set_url(url);
	}
	if (volume) {
		MediaPlayer.player.set_volume(volume);
	}
	MediaPlayer.player.play().finally(() => {
		MediaPlayer.update_state();
	});
};

window.pause = () => {
	MediaPlayer.player.pause();
	MediaPlayer.update_state();
};

window.stop = () => {
	MediaPlayer.player.stop();
	MediaPlayer.update_state();
};
