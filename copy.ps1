$base_dir = $env:MONKESTATION_REPO ?? "C:/Users/Lucy/Code/SS13/MonkeStation"
$assets_dir = Join-Path -Path "${base_dir}" -ChildPath "monkestation/code/modules/media/assets"

$assets = @{
	"media_player.js" = "web/media_player.js"
	"media_player.html" = "web/media_player.html"
	"media_player_wasm.js" = "pkg/media_player.js"
	"media_player.wasm" = "pkg/media_player_bg.wasm"
}

wasm-pack build --release --target no-modules --no-pack

foreach ($file_name in $assets.Keys) {
	$origin_path = $assets[$file_name]
	$target_path = Join-Path -Path "${assets_dir}" -ChildPath "${file_name}"

	$origin_hash = Get-FileHash -Path "${origin_path}" -Algorithm MD5
    $target_hash = Get-FileHash -Path "${target_path}" -Algorithm MD5

	if ($origin_hash.Hash -eq $target_hash.Hash) {
		echo "${origin_path} is unchanged"
	} else {
		rm "${target_path}" -ErrorAction Ignore
		cp "${origin_path}" "${target_path}"
		echo "Copied ${origin_path} to ${target_path}"
	}
}
