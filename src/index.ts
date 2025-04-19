import { topic } from "./byond";
import { Howl, type HowlOptions } from "howler";

declare global {
	function play(
		url: string,
		volume?: number,
		format?: string | string[],
		x?: number,
		y?: number,
		z?: number,
		balance?: number,
	);
	function stop();
	function pause();
	function set_volume(volume: number);
	function set_time(seconds: number, force?: boolean);
	function set_position(x: number, y: number, z: number);
	function set_panning(panning?: number);

	var audio: Howl | null;
	var load_time: number | null;
	var start_time: number | null;
	var offset: number;
	var cleared: boolean;
}

audio = null;
load_time = null;
start_time = null;
offset = 0;
cleared = false;

function is_in_margin_of_error(new_time: number): boolean {
	if (!start_time || new_time <= 0 || offset <= 0) return false;
	const elapsed = (Date.now() - start_time) / 1000;
	const diff = Math.abs(elapsed - new_time);
	console.debug(
		`load_time = ${load_time}\nstart_time = ${start_time}\nnew_time = ${new_time}\nelapsed = ${elapsed}\noffset = ${offset}\ndiff = ${diff}`,
	);
	return diff <= offset;
}

function send_clear() {
	if (cleared) return;
	topic("clear");
	cleared = true;
}

function full_clear(debug_msg?: string) {
	if (debug_msg) console.debug("full_clear:", debug_msg);
	send_clear();
	audio = null;
	load_time = null;
	start_time = null;
	offset = 0;
}

function send_playing(url: string) {
	cleared = false;
	topic("playing", { url });
}

window.play = (
	url: string,
	volume?: number,
	format?: string | string[],
	x = 0,
	y = 0,
	z = 0,
	balance = 0,
) => {
	if (audio) stop();
	const options: HowlOptions = {
		src: [url],
		volume: volume ? volume / 100 : 1,
		html5: true,
		preload: false,
		format,
		onload: () => {
			console.debug("onload");
			if (!audio) return;
			audio.pos(x, y, z);
			audio.stereo(balance);
			audio.play();
		},
		onplay: () => {
			start_time = Date.now();
			send_playing(url);
			if (load_time !== null) {
				offset = Math.ceil((start_time - load_time) / 1000) * 2;
			}
			console.debug(
				`--- onplay ---\nstart_time=${start_time}\nload_time=${load_time}\noffset=${offset}`,
			);
		},
		onstop: () => full_clear("onstop"),
		onend: () => full_clear("onend"),
		onloaderror: () => full_clear("onloaderror"),
		onplayerror: () => full_clear("onplayerror"),
	};
	try {
		const audio = (window.audio = new Howl(options));
		//console.log((audio as any)._sounds);
		const sounds = (audio as any)._sounds;
		if (sounds) {
			for (const sound of sounds) {
				const node = sound._node;
				if (node) node.crossOrigin = "anonymous";
			}
		}
		load_time = Date.now();
		audio.load();
	} catch (error) {
		console.error("Failed to play audio", error);
		stop();
	}
};

window.stop = () => {
	send_clear();
	try {
		load_time = null;
		start_time = null;
		offset = 0;
		audio?.unload();
	} catch (error) {
		console.error("Failed to stop audio", error);
	} finally {
		audio = null;
	}
};

window.pause = () => {
	audio?.pause();
};

window.set_volume = (volume: number) => {
	audio?.volume(volume / 100);
};

window.set_time = (seconds: number, force?: boolean) => {
	// Avoid causing stuttering, when possible.
	if (force || !is_in_margin_of_error(seconds)) audio?.seek(seconds);
};

window.set_position = (x: number, y: number, z: number) => {
	audio?.pos(x, y, z);
};

window.set_panning = (panning?: number) => {
	audio?.stereo(panning || 0);
};

document.onreadystatechange = () => {
	if (document.readyState === "complete") {
		topic("ready");
	}
};
