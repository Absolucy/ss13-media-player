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
	);
	function stop();
	function pause();
	function set_volume(volume: number);
	function set_time(seconds: number);
	function set_position(x: number, y: number, z: number);
	function set_panning(panning?: number);

	var audio: Howl | null;
	var cleared: boolean;
}

audio = null;
cleared = false;

function send_clear() {
	if (cleared) return;
	topic("clear");
	cleared = true;
}

function full_clear(debug_msg?: string) {
	if (debug_msg) console.debug("full_clear:", debug_msg);
	send_clear();
	audio = null;
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
			audio.play();
		},
		onplay: () => send_playing(url),
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
		audio.load();
	} catch (error) {
		console.error("Failed to play audio", error);
		stop();
	}
};

window.stop = () => {
	send_clear();
	try {
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

window.set_time = (seconds: number) => {
	audio?.seek(seconds);
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
