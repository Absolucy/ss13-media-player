import { Howl, type HowlOptions } from "howler";
import { topic } from "./byond";

declare global {
	function play(
		url: string,
		volume?: number,
		format?: string | string[],
		x?: number,
		y?: number,
		z?: number,
		balance?: number,
	): void;
	function stop(): void;
	function pause(): void;
	function set_volume(volume: number): void;
	function set_time(seconds: number, force?: boolean): void;
	function set_position(x: number, y: number, z: number): void;
	function set_panning(panning?: number): void;

	var audio: Howl | null;
	var load_time: number | null;
	var start_time: number | null;
	var offset: number;
	var cleared: boolean;
	var sent_playing: boolean;
}

audio = null;
load_time = null;
start_time = null;
offset = 0;
cleared = false;
sent_playing = false;

function is_in_margin_of_error(new_time: number): boolean {
	if (!audio || !start_time || new_time <= 0 || offset <= 0) return false;
	const position = audio.seek();
	const diff = Math.abs(position - new_time);
	console.debug(
		`load_time = ${load_time}\nstart_time = ${start_time}\nnew_time = ${new_time}\nposition = ${position}\noffset = ${offset}\ndiff = ${diff}`,
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
	audio?.unload();
	audio = null;
	sent_playing = false;
	load_time = null;
	start_time = null;
	offset = 0;
	send_clear();
}

function send_playing(url: string) {
	cleared = false;
	if (!sent_playing) {
		sent_playing = true;
		topic("playing", { url });
	}
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
	if (audio) {
		audio.unload();
		audio = null;
		sent_playing = false;
		load_time = null;
		start_time = null;
		offset = 0;
	}
	const options: HowlOptions = {
		src: [url],
		volume: volume === undefined ? 1 : volume / 100,
		html5: true,
		preload: true,
		format,
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
		onend: () => full_clear("onend"),
		onloaderror: () => full_clear("onloaderror"),
		onplayerror: () => full_clear("onplayerror"),
	};
	try {
		const audio = (window.audio = new Howl(options));
		const sounds = (audio as any)._sounds;
		if (sounds) {
			for (const sound of sounds) {
				const node = sound._node;
				if (node) node.crossOrigin = "anonymous";
			}
		}
		load_time = Date.now();
		audio.pos(x, y, z);
		audio.stereo(balance);
		audio.play();
	} catch (error) {
		console.error("Failed to play audio", error);
		stop();
	}
};

window.stop = () => {
	full_clear("stop");
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
