export function topic(name: string, params?: Record<string, any>) {
	let url = `byond://?src=media:href;type=${encodeURIComponent(name)}`;
	if (params) {
		url += `;params=${encodeURIComponent(JSON.stringify(params))}`;
	}
	window.location.href = url;
}
