#!/usr/bin/env bash

# SPDX-License-Identifier: AGPL-3.0-or-later

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT_DIR}"

WORKSPACE_DIR="$(cd "${ROOT_DIR}/.." && pwd)"

for tool in awk cargo curl date ffmpeg file grep mktemp paste sed seq sort tail tr uname xargs; do
	if ! command -v "${tool}" >/dev/null 2>&1; then
		printf 'missing required tool: %s\n' "${tool}" >&2
		exit 1
	fi
done

BENCH_ITERATIONS="${BENCH_ITERATIONS:-2}"
BENCH_ONLINE="${BENCH_ONLINE:-1}"
BENCH_REAL_VIDEO="${BENCH_REAL_VIDEO:-1}"
BENCH_FULL="${BENCH_FULL:-0}"
BENCH_PARALLEL="${BENCH_PARALLEL:-1}"
BENCH_CONCURRENCY="${BENCH_CONCURRENCY:-4}"
BENCH_CONCURRENT_REQUESTS="${BENCH_CONCURRENT_REQUESTS:-8}"
BENCH_NATIVE_TRANSFORMS="${BENCH_NATIVE_TRANSFORMS:-4}"
BENCH_EXTERNAL="${BENCH_EXTERNAL:-1}"
BENCH_COMPAT_MATRIX="${BENCH_COMPAT_MATRIX:-1}"
BENCH_PORT="${BENCH_PORT:-19110}"
BENCH_RESULTS_DIR="${BENCH_RESULTS_DIR:-${ROOT_DIR}/bench-results}"
BENCH_CACHE_DIR="${BENCH_CACHE_DIR:-${ROOT_DIR}/.benchmark-cache/media}"
BENCH_DOWNLOAD_TIMEOUT="${BENCH_DOWNLOAD_TIMEOUT:-180}"
BENCH_STRICT_MIME="${BENCH_STRICT_MIME:-0}"

RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)"
RUN_DIR="${BENCH_RESULTS_DIR}/${RUN_ID}"
WORK_DIR="$(mktemp -d /tmp/fluxer-media-bench.XXXXXX)"
STORAGE_ROOT="${WORK_DIR}/storage"
OUTPUT_DIR="${RUN_DIR}/outputs"
CSV="${RUN_DIR}/results.csv"
PARALLEL_CSV="${RUN_DIR}/parallel-results.csv"
SYSTEM_INFO="${RUN_DIR}/system.txt"
PROXY_LOG="${RUN_DIR}/proxy.log"
PROXY_PID=""
SERVER_URL="http://127.0.0.1:${BENCH_PORT}"
SECRET_KEY="benchmark-secret"

cleanup() {
	if [[ -n "${PROXY_PID}" ]]; then
		kill "${PROXY_PID}" 2>/dev/null || true
		wait "${PROXY_PID}" 2>/dev/null || true
	fi
	rm -rf "${WORK_DIR}"
}
trap cleanup EXIT

mkdir -p "${RUN_DIR}" "${OUTPUT_DIR}" "${BENCH_CACHE_DIR}"
printf 'case,source,kind,input_format,operation,iteration,status,time_total_ms,size_download_bytes,content_type,output_file,validation\n' >"${CSV}"
printf 'case,source,kind,input_format,operation,requests,concurrency,successes,total_time_ms,requests_per_second,avg_latency_ms,total_bytes,validation\n' >"${PARALLEL_CSV}"

note() {
	printf '[bench] %s\n' "$*" >&2
}

has_encoder() {
	ffmpeg -hide_banner -encoders 2>/dev/null | awk '{print $2}' | grep -qx "$1"
}

has_muxer() {
	ffmpeg -hide_banner -muxers 2>/dev/null | awk '{print $2}' | grep -qx "$1"
}

wait_for_url() {
	local url="$1"
	for _ in $(seq 1 160); do
		if curl -fsS "${url}" >/dev/null 2>&1; then
			return 0
		fi
		sleep 0.25
	done
	printf 'timed out waiting for %s\n' "${url}" >&2
	return 1
}

header_value() {
	local header_file="$1"
	local name="$2"
	sed -n "s/^[[:space:]]*${name}:[[:space:]]*//Ip" "${header_file}" | tail -n 1 | tr -d '\r'
}

mime_matches() {
	local actual="$1"
	local expected="$2"
	[[ -z "${expected}" || "${actual,,}" == "${expected,,}"* ]]
}

validate_output() {
	local file_path="$1"
	local expected_mime="$2"
	local content_type="$3"
	if [[ ! -s "${file_path}" ]]; then
		printf 'empty-output'
		return
	fi
	if ! mime_matches "${content_type}" "${expected_mime}"; then
		printf 'mime-mismatch:%s' "${content_type:-missing}"
		return
	fi
	case "${expected_mime}" in
		image/webp)
			file "${file_path}" | grep -q 'Web/P image' && printf 'ok' || printf 'file-mismatch'
			;;
		image/jpeg)
			file "${file_path}" | grep -q 'JPEG image data' && printf 'ok' || printf 'file-mismatch'
			;;
		image/png)
			file "${file_path}" | grep -q 'PNG image data' && printf 'ok' || printf 'file-mismatch'
			;;
		image/gif)
			file "${file_path}" | grep -q 'GIF image data' && printf 'ok' || printf 'file-mismatch'
			;;
		image/avif)
			file "${file_path}" | grep -Eqi 'AVIF|ISO Media' && printf 'ok' || printf 'file-mismatch'
			;;
		video/mp4)
			file "${file_path}" | grep -Eqi 'ISO Media|MP4' && printf 'ok' || printf 'file-mismatch'
			;;
		video/webm)
			file "${file_path}" | grep -qi 'WebM' && printf 'ok' || printf 'file-mismatch'
			;;
		audio/ogg)
			file "${file_path}" | grep -Eqi 'Ogg|Vorbis|Opus' && printf 'ok' || printf 'file-mismatch'
			;;
		audio/wav)
			file "${file_path}" | grep -Eqi 'WAVE|WAV|RIFF' && printf 'ok' || printf 'file-mismatch'
			;;
		text/plain)
			printf 'ok'
			;;
		*)
			printf 'ok'
			;;
	esac
}

safe_name() {
	printf '%s' "$1" | sed 's/[^A-Za-z0-9_.-]/_/g'
}

write_system_info() {
	{
		printf 'run_id=%s\n' "${RUN_ID}"
		printf 'date_utc=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
		printf 'uname=%s\n' "$(uname -a)"
		printf 'rustc=%s\n' "$(rustc --version)"
		printf 'ffmpeg=%s\n' "$(ffmpeg -hide_banner -version | sed -n '1p')"
		printf 'bench_iterations=%s\n' "${BENCH_ITERATIONS}"
		printf 'bench_online=%s\n' "${BENCH_ONLINE}"
		printf 'bench_real_video=%s\n' "${BENCH_REAL_VIDEO}"
		printf 'bench_full=%s\n' "${BENCH_FULL}"
		printf 'bench_parallel=%s\n' "${BENCH_PARALLEL}"
		printf 'bench_external=%s\n' "${BENCH_EXTERNAL}"
		printf 'bench_compat_matrix=%s\n' "${BENCH_COMPAT_MATRIX}"
		printf 'bench_concurrency=%s\n' "${BENCH_CONCURRENCY}"
		printf 'bench_concurrent_requests=%s\n' "${BENCH_CONCURRENT_REQUESTS}"
		printf 'bench_native_transforms=%s\n' "${BENCH_NATIVE_TRANSFORMS}"
		printf 'encoders='
		ffmpeg -hide_banner -encoders 2>/dev/null | awk '{print $2}' | grep -E '^(libx264|libvpx-vp9|libwebp|libwebp_anim|libaom-av1|gif|png|mjpeg)$' | sort | paste -sd ',' - || true
		printf '\n'
	} >"${SYSTEM_INFO}"
}

copy_cdn() {
	local key="$1"
	local source="$2"
	mkdir -p "$(dirname "${STORAGE_ROOT}/cdn/${key}")"
	cp "${source}" "${STORAGE_ROOT}/cdn/${key}"
}

copy_upload() {
	local key="$1"
	local source="$2"
	mkdir -p "$(dirname "${STORAGE_ROOT}/uploads/${key}")"
	cp "${source}" "${STORAGE_ROOT}/uploads/${key}"
}

download_fixture() {
	local name="$1"
	local url="$2"
	local max_bytes="$3"
	local dest="$4"
	local cached="${BENCH_CACHE_DIR}/${name}"
	if [[ -s "${cached}" ]]; then
		cp "${cached}" "${dest}"
		return 0
	fi
	if [[ "${BENCH_ONLINE}" != "1" ]]; then
		note "skipping ${name}; not cached and BENCH_ONLINE=0"
		return 1
	fi
	local tmp="${cached}.tmp"
	note "downloading ${name} into ${BENCH_CACHE_DIR}"
	if ! curl -fL --retry 0 --connect-timeout 10 --max-time "${BENCH_DOWNLOAD_TIMEOUT}" --max-filesize "${max_bytes}" -o "${tmp}" "${url}"; then
		rm -f "${tmp}"
		note "skipping ${name}; download failed"
		return 1
	fi
	mv "${tmp}" "${cached}"
	cp "${cached}" "${dest}"
}

generate_fixtures() {
	mkdir -p "${WORK_DIR}/fixtures" "${STORAGE_ROOT}/cdn/attachments/bench" "${STORAGE_ROOT}/uploads"
	if ! has_encoder libx264; then
		printf 'missing required ffmpeg encoder: libx264\n' >&2
		exit 1
	fi
	printf '0123456789abcdef\n' >"${WORK_DIR}/fixtures/range.txt"
	copy_cdn "attachments/bench/text/range.txt" "${WORK_DIR}/fixtures/range.txt"

	note "generating synthetic high-resolution images"
	ffmpeg -hide_banner -loglevel error -y \
		-f lavfi -i testsrc2=size=3840x2160:rate=1:duration=1 \
		-frames:v 1 "${WORK_DIR}/fixtures/synthetic-4k.png"
	copy_cdn "attachments/bench/images/synthetic-4k.png" "${WORK_DIR}/fixtures/synthetic-4k.png"
	copy_cdn "avatars/42/benchpng" "${WORK_DIR}/fixtures/synthetic-4k.png"
	copy_upload "upload-image.png" "${WORK_DIR}/fixtures/synthetic-4k.png"

	ffmpeg -hide_banner -loglevel error -y \
		-f lavfi -i testsrc2=size=3840x2160:rate=1:duration=1 \
		-frames:v 1 -q:v 2 "${WORK_DIR}/fixtures/synthetic-4k.jpg"
	copy_cdn "attachments/bench/images/synthetic-4k.jpg" "${WORK_DIR}/fixtures/synthetic-4k.jpg"

	if has_encoder libwebp; then
		ffmpeg -hide_banner -loglevel error -y \
			-f lavfi -i testsrc2=size=3840x2160:rate=1:duration=1 \
			-frames:v 1 -c:v libwebp -q:v 80 "${WORK_DIR}/fixtures/synthetic-4k.webp"
		copy_cdn "attachments/bench/images/synthetic-4k.webp" "${WORK_DIR}/fixtures/synthetic-4k.webp"
	fi

	if has_encoder libaom-av1 && has_muxer avif; then
		ffmpeg -hide_banner -loglevel error -y \
			-f lavfi -i testsrc2=size=1920x1080:rate=1:duration=1 \
			-frames:v 1 -c:v libaom-av1 -still-picture 1 -cpu-used 8 -crf 36 -b:v 0 \
			"${WORK_DIR}/fixtures/synthetic-1080p.avif" || true
		if [[ -s "${WORK_DIR}/fixtures/synthetic-1080p.avif" ]]; then
			copy_cdn "attachments/bench/images/synthetic-1080p.avif" "${WORK_DIR}/fixtures/synthetic-1080p.avif"
		fi
	fi

	note "generating synthetic animated GIF"
	local gif_size="1280x720"
	if [[ "${BENCH_FULL}" == "1" ]]; then
		gif_size="1920x1080"
	fi
	ffmpeg -hide_banner -loglevel error -y \
		-f lavfi -i "testsrc2=size=${gif_size}:rate=15:duration=2" \
		-vf "fps=15" "${WORK_DIR}/fixtures/synthetic-${gif_size}.gif"
	copy_cdn "attachments/bench/gifs/synthetic-${gif_size}.gif" "${WORK_DIR}/fixtures/synthetic-${gif_size}.gif"

	note "generating synthetic videos"
	ffmpeg -hide_banner -loglevel error -y \
		-f lavfi -i testsrc2=size=1920x1080:rate=30:duration=2 \
		-f lavfi -i sine=frequency=440:duration=2 \
		-c:v libx264 -preset veryfast -pix_fmt yuv420p -c:a aac -movflags +faststart \
		"${WORK_DIR}/fixtures/synthetic-1080p.mp4"
	copy_cdn "attachments/bench/videos/synthetic-1080p.mp4" "${WORK_DIR}/fixtures/synthetic-1080p.mp4"
	copy_upload "upload-video.mp4" "${WORK_DIR}/fixtures/synthetic-1080p.mp4"

	if [[ "${BENCH_FULL}" == "1" ]]; then
		ffmpeg -hide_banner -loglevel error -y \
			-f lavfi -i testsrc2=size=3840x2160:rate=30:duration=2 \
			-f lavfi -i sine=frequency=660:duration=2 \
			-c:v libx264 -preset ultrafast -pix_fmt yuv420p -c:a aac -movflags +faststart \
			"${WORK_DIR}/fixtures/synthetic-4k.mp4"
		copy_cdn "attachments/bench/videos/synthetic-4k.mp4" "${WORK_DIR}/fixtures/synthetic-4k.mp4"
	fi

	if has_encoder libvpx-vp9; then
		ffmpeg -hide_banner -loglevel error -y \
			-f lavfi -i testsrc2=size=1280x720:rate=30:duration=2 \
			-c:v libvpx-vp9 -deadline realtime -cpu-used 8 -b:v 2M -an \
			"${WORK_DIR}/fixtures/synthetic-720p.webm"
		copy_cdn "attachments/bench/videos/synthetic-720p.webm" "${WORK_DIR}/fixtures/synthetic-720p.webm"
	fi

	if [[ "${BENCH_COMPAT_MATRIX}" == "1" ]]; then
		note "generating synthetic video compatibility matrix"
		ffmpeg -hide_banner -loglevel error -y \
			-f lavfi -i testsrc2=size=1920x1080:rate=30:duration=2 \
			-c:v libx264 -preset veryfast -pix_fmt yuv420p -f mov \
			"${WORK_DIR}/fixtures/compat-h264-1080p.mov"
		copy_cdn "attachments/bench/compat/videos/h264-1080p.mov" "${WORK_DIR}/fixtures/compat-h264-1080p.mov"

		ffmpeg -hide_banner -loglevel error -y \
			-f lavfi -i testsrc2=size=1920x1080:rate=30:duration=2 \
			-c:v libx264 -preset veryfast -pix_fmt yuv420p -f matroska \
			"${WORK_DIR}/fixtures/compat-h264-1080p.mkv"
		copy_cdn "attachments/bench/compat/videos/h264-1080p.mkv" "${WORK_DIR}/fixtures/compat-h264-1080p.mkv"

		if has_encoder libx265; then
			ffmpeg -hide_banner -loglevel error -y \
				-f lavfi -i testsrc2=size=1920x1080:rate=30:duration=2 \
				-c:v libx265 -preset ultrafast -x265-params log-level=error -pix_fmt yuv420p -f mov \
				"${WORK_DIR}/fixtures/compat-hevc-1080p.mov"
			copy_cdn "attachments/bench/compat/videos/hevc-1080p.mov" "${WORK_DIR}/fixtures/compat-hevc-1080p.mov"
		fi

		if has_encoder prores_ks; then
			ffmpeg -hide_banner -loglevel error -y \
				-f lavfi -i testsrc2=size=1920x1080:rate=30:duration=1 \
				-c:v prores_ks -profile:v 0 -pix_fmt yuv422p10le -f mov \
				"${WORK_DIR}/fixtures/compat-prores-1080p.mov"
			copy_cdn "attachments/bench/compat/videos/prores-1080p.mov" "${WORK_DIR}/fixtures/compat-prores-1080p.mov"
		fi

		ffmpeg -hide_banner -loglevel error -y \
			-f lavfi -i testsrc2=size=1280x720:rate=30:duration=2 \
			-c:v mpeg4 -pix_fmt yuv420p -f avi \
			"${WORK_DIR}/fixtures/compat-mpeg4-720p.avi"
		copy_cdn "attachments/bench/compat/videos/mpeg4-720p.avi" "${WORK_DIR}/fixtures/compat-mpeg4-720p.avi"

		ffmpeg -hide_banner -loglevel error -y \
			-f lavfi -i testsrc2=size=1280x720:rate=30:duration=2 \
			-c:v mpeg2video -pix_fmt yuv420p -f mpegts \
			"${WORK_DIR}/fixtures/compat-mpeg2-720p.ts"
		copy_cdn "attachments/bench/compat/videos/mpeg2-720p.ts" "${WORK_DIR}/fixtures/compat-mpeg2-720p.ts"
	fi

	note "generating synthetic audio"
	ffmpeg -hide_banner -loglevel error -y \
		-f lavfi -i sine=frequency=440:duration=30 \
		-c:a pcm_s16le "${WORK_DIR}/fixtures/synthetic-30s.wav"
	copy_cdn "attachments/bench/audio/synthetic-30s.wav" "${WORK_DIR}/fixtures/synthetic-30s.wav"

	if download_fixture "picsum-id10-4k.jpg" \
		"https://picsum.photos/id/10/3840/2160.jpg" \
		20000000 "${WORK_DIR}/fixtures/real-photo-4k.jpg"; then
		copy_cdn "attachments/bench/online/real-photo-4k.jpg" "${WORK_DIR}/fixtures/real-photo-4k.jpg"
	fi

	if download_fixture "google-webp-gallery-1.png" \
		"https://www.gstatic.com/webp/gallery3/1.png" \
		10000000 "${WORK_DIR}/fixtures/google-webp-gallery-1.png"; then
		copy_cdn "attachments/bench/online/google-webp-gallery-1.png" "${WORK_DIR}/fixtures/google-webp-gallery-1.png"
	fi

	if download_fixture "google-webp-gallery-1.webp" \
		"https://www.gstatic.com/webp/gallery3/1_webp_ll.webp" \
		10000000 "${WORK_DIR}/fixtures/google-webp-gallery-1.webp"; then
		copy_cdn "attachments/bench/online/google-webp-gallery-1.webp" "${WORK_DIR}/fixtures/google-webp-gallery-1.webp"
	fi

	if download_fixture "google-webp-animated-1.gif" \
		"https://www.gstatic.com/webp/animated/1.gif" \
		20000000 "${WORK_DIR}/fixtures/google-webp-animated-1.gif"; then
		copy_cdn "attachments/bench/online/google-webp-animated-1.gif" "${WORK_DIR}/fixtures/google-webp-animated-1.gif"
	fi

	if [[ "${BENCH_FULL}" == "1" ]]; then
		if download_fixture "fronalpstock-big.jpg" \
			"https://upload.wikimedia.org/wikipedia/commons/3/3f/Fronalpstock_big.jpg" \
			60000000 "${WORK_DIR}/fixtures/fronalpstock-big.jpg"; then
			copy_cdn "attachments/bench/online/fronalpstock-big.jpg" "${WORK_DIR}/fixtures/fronalpstock-big.jpg"
		fi

		if download_fixture "rotating-earth.gif" \
			"https://upload.wikimedia.org/wikipedia/commons/e/e1/Rotating_earth_animated.gif" \
			80000000 "${WORK_DIR}/fixtures/rotating-earth.gif"; then
			copy_cdn "attachments/bench/online/rotating-earth.gif" "${WORK_DIR}/fixtures/rotating-earth.gif"
		fi
	fi

	if [[ "${BENCH_REAL_VIDEO}" == "1" ]]; then
		if download_fixture "sample-cat-1920x1080-mov" \
			"https://disk.sample.cat/samples/mov/1416529-hd_1920_1080_30fps.mov" \
			40000000 "${WORK_DIR}/fixtures/sample-cat-1080p.mov"; then
			copy_cdn "attachments/bench/online/sample-cat-1080p.mov" "${WORK_DIR}/fixtures/sample-cat-1080p.mov"
		fi
		if download_fixture "mdn-flower.mp4" \
			"https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.mp4" \
			20000000 "${WORK_DIR}/fixtures/mdn-flower.mp4"; then
			copy_cdn "attachments/bench/online/mdn-flower.mp4" "${WORK_DIR}/fixtures/mdn-flower.mp4"
		fi
		if download_fixture "mdn-flower.webm" \
			"https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.webm" \
			20000000 "${WORK_DIR}/fixtures/mdn-flower.webm"; then
			copy_cdn "attachments/bench/online/mdn-flower.webm" "${WORK_DIR}/fixtures/mdn-flower.webm"
		fi
		if download_fixture "big-buck-bunny-720p-10s.mp4" \
			"https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/720/Big_Buck_Bunny_720_10s_1MB.mp4" \
			20000000 "${WORK_DIR}/fixtures/big-buck-bunny-720p-10s.mp4"; then
			copy_cdn "attachments/bench/online/big-buck-bunny-720p-10s.mp4" "${WORK_DIR}/fixtures/big-buck-bunny-720p-10s.mp4"
		fi
		if [[ "${BENCH_FULL}" == "1" ]]; then
			if download_fixture "wikimedia-big-buck-bunny-medium-ogv" \
				"https://commons.wikimedia.org/wiki/Special:Redirect/file/Big_Buck_Bunny_medium.ogv" \
				180000000 "${WORK_DIR}/fixtures/big-buck-bunny-medium.ogv"; then
				copy_cdn "attachments/bench/online/big-buck-bunny-medium.ogv" "${WORK_DIR}/fixtures/big-buck-bunny-medium.ogv"
			fi
		fi
	fi

	if download_fixture "maple-leaf-ragq.ogg" \
		"https://commons.wikimedia.org/wiki/Special:Redirect/file/Maple_Leaf_RagQ.ogg" \
		20000000 "${WORK_DIR}/fixtures/maple-leaf-ragq.ogg"; then
		copy_cdn "attachments/bench/online/maple-leaf-ragq.ogg" "${WORK_DIR}/fixtures/maple-leaf-ragq.ogg"
	fi

	if [[ "${BENCH_FULL}" == "1" ]]; then
		if download_fixture "candles-green-heads-and-skulls.ogg" \
			"https://commons.wikimedia.org/wiki/Special:Redirect/file/018-_Candles_Green,_Heads_and_Skulls.ogg" \
			160000000 "${WORK_DIR}/fixtures/candles-green-heads-and-skulls.ogg"; then
			copy_cdn "attachments/bench/online/candles-green-heads-and-skulls.ogg" "${WORK_DIR}/fixtures/candles-green-heads-and-skulls.ogg"
		fi
	fi
}

start_proxy() {
	note "building release binary"
	(cd "${WORKSPACE_DIR}" && cargo build --release -p fluxer-media-proxy)
	note "starting proxy on ${SERVER_URL}"
	env \
		"FLUXER_MEDIA_PROXY_SECRET_KEY=${SECRET_KEY}" \
		"FLUXER_MEDIA_PROXY_STORAGE_BACKEND=local" \
		"FLUXER_MEDIA_PROXY_STORAGE_ROOT=${STORAGE_ROOT}" \
		"FLUXER_MEDIA_PROXY_MAX_NATIVE_TRANSFORMS=${BENCH_NATIVE_TRANSFORMS}" \
		"FLUXER_MEDIA_PROXY_HOST=127.0.0.1" \
		"FLUXER_MEDIA_PROXY_PORT=${BENCH_PORT}" \
		"${WORKSPACE_DIR}/target/release/fluxer-media-proxy" >"${PROXY_LOG}" 2>&1 &
	PROXY_PID="$!"
	wait_for_url "${SERVER_URL}/_health"
}

run_case() {
	local case_name="$1"
	local source="$2"
	local kind="$3"
	local input_format="$4"
	local operation="$5"
	local url="$6"
	local expected_status="$7"
	local expected_mime="$8"
	local extension="$9"
	local extra_header="${10:-}"
	local request_args=()
	if [[ -n "${extra_header}" ]]; then
		request_args=(-H "${extra_header}")
	fi

	local output_base
	output_base="$(safe_name "${case_name}")"
	note "benchmarking ${case_name}"
	curl -fsS "${request_args[@]}" "${url}" -o /dev/null >/dev/null 2>&1 || true
	for iteration in $(seq 1 "${BENCH_ITERATIONS}"); do
		local headers="${WORK_DIR}/${output_base}-${iteration}.headers"
		local output="${OUTPUT_DIR}/${output_base}-${iteration}.${extension}"
		local metrics status seconds bytes ms content_type validation
		metrics="$(curl -sS "${request_args[@]}" -D "${headers}" -o "${output}" -w '%{http_code} %{time_total} %{size_download}' "${url}")"
		status="$(printf '%s' "${metrics}" | awk '{print $1}')"
		seconds="$(printf '%s' "${metrics}" | awk '{print $2}')"
		bytes="$(printf '%s' "${metrics}" | awk '{print $3}')"
		ms="$(awk -v s="${seconds}" 'BEGIN { printf "%.3f", s * 1000 }')"
		content_type="$(header_value "${headers}" "content-type")"
		validation="$(validate_output "${output}" "${expected_mime}" "${content_type}")"
		printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
			"${case_name}" "${source}" "${kind}" "${input_format}" "${operation}" "${iteration}" "${status}" "${ms}" "${bytes}" "${content_type}" "${output}" "${validation}" >>"${CSV}"
		if [[ "${status}" != "${expected_status}" ]]; then
			printf 'case %s returned HTTP %s, expected %s\n' "${case_name}" "${status}" "${expected_status}" >&2
			return 1
		fi
		if [[ "${BENCH_STRICT_MIME}" == "1" && "${validation}" != "ok" ]]; then
			printf 'case %s validation failed: %s\n' "${case_name}" "${validation}" >&2
			return 1
		fi
	done
}

run_post_case() {
	local case_name="$1"
	local source="$2"
	local kind="$3"
	local input_format="$4"
	local operation="$5"
	local url="$6"
	local payload="$7"
	local expected_status="$8"
	local expected_mime="$9"
	local extension="${10}"

	local output_base
	output_base="$(safe_name "${case_name}")"
	note "benchmarking ${case_name}"
	curl -fsS -X POST -H "Authorization: Bearer ${SECRET_KEY}" -H 'Content-Type: application/json' --data "${payload}" "${url}" -o /dev/null >/dev/null 2>&1 || true
	for iteration in $(seq 1 "${BENCH_ITERATIONS}"); do
		local headers="${WORK_DIR}/${output_base}-${iteration}.headers"
		local output="${OUTPUT_DIR}/${output_base}-${iteration}.${extension}"
		local metrics status seconds bytes ms content_type validation
		metrics="$(curl -sS -X POST -H "Authorization: Bearer ${SECRET_KEY}" -H 'Content-Type: application/json' --data "${payload}" -D "${headers}" -o "${output}" -w '%{http_code} %{time_total} %{size_download}' "${url}")"
		status="$(printf '%s' "${metrics}" | awk '{print $1}')"
		seconds="$(printf '%s' "${metrics}" | awk '{print $2}')"
		bytes="$(printf '%s' "${metrics}" | awk '{print $3}')"
		ms="$(awk -v s="${seconds}" 'BEGIN { printf "%.3f", s * 1000 }')"
		content_type="$(header_value "${headers}" "content-type")"
		validation="$(validate_output "${output}" "${expected_mime}" "${content_type}")"
		printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
			"${case_name}" "${source}" "${kind}" "${input_format}" "${operation}" "${iteration}" "${status}" "${ms}" "${bytes}" "${content_type}" "${output}" "${validation}" >>"${CSV}"
		if [[ "${status}" != "${expected_status}" ]]; then
			printf 'case %s returned HTTP %s, expected %s\n' "${case_name}" "${status}" "${expected_status}" >&2
			return 1
		fi
		if [[ "${BENCH_STRICT_MIME}" == "1" && "${validation}" != "ok" ]]; then
			printf 'case %s validation failed: %s\n' "${case_name}" "${validation}" >&2
			return 1
		fi
	done
}

run_if_exists() {
	local storage_path="$1"
	shift
	if [[ -s "${STORAGE_ROOT}/cdn/${storage_path}" ]]; then
		run_case "$@"
	else
		note "skipping $1; missing ${storage_path}"
	fi
}

external_url() {
	local upstream="$1"
	cargo run --quiet --manifest-path "${WORKSPACE_DIR}/Cargo.toml" --package fluxer-dev -- \
		media-proxy sign-external-url \
		--secret-key "${SECRET_KEY}" \
		--server-url "${SERVER_URL}" \
		"${upstream}"
}

run_parallel_case() {
	local case_name="$1"
	local source="$2"
	local kind="$3"
	local input_format="$4"
	local operation="$5"
	local url="$6"
	local expected_status="$7"
	local expected_mime="$8"
	local extension="$9"
	if [[ "${BENCH_PARALLEL}" != "1" ]]; then
		return 0
	fi

	local output_base tmp start_ns end_ns total_ms successes total_bytes avg_latency rps validation
	output_base="$(safe_name "${case_name}")"
	tmp="${WORK_DIR}/${output_base}-parallel.txt"
	note "parallel benchmark ${case_name} (${BENCH_CONCURRENT_REQUESTS} requests, ${BENCH_CONCURRENCY} concurrent)"
	start_ns="$(date +%s%N)"
	if ! seq 1 "${BENCH_CONCURRENT_REQUESTS}" | xargs -P "${BENCH_CONCURRENCY}" -I '{}' sh -c '
		headers="$1/'"${output_base}"'-parallel-{}.headers"
		output="$2/'"${output_base}"'-parallel-{}.'"${extension}"'"
		curl -sS -D "${headers}" -o "${output}" -w "%{http_code} %{time_total} %{size_download}\n" "$3"
	' sh "${WORK_DIR}" "${OUTPUT_DIR}" "${url}" >"${tmp}"; then
		note "one or more parallel curl workers failed for ${case_name}"
	fi
	end_ns="$(date +%s%N)"
	total_ms="$(awk -v start="${start_ns}" -v end="${end_ns}" 'BEGIN { printf "%.3f", (end - start) / 1000000 }')"
	successes="$(awk -v status="${expected_status}" '$1 == status { count++ } END { print count + 0 }' "${tmp}")"
	total_bytes="$(awk '{ sum += $3 } END { print sum + 0 }' "${tmp}")"
	avg_latency="$(awk '{ sum += $2; count++ } END { if (count == 0) print "0.000"; else printf "%.3f", (sum / count) * 1000 }' "${tmp}")"
	rps="$(awk -v req="${BENCH_CONCURRENT_REQUESTS}" -v ms="${total_ms}" 'BEGIN { if (ms == 0) print "0.000"; else printf "%.3f", req / (ms / 1000) }')"
	local first_output="${OUTPUT_DIR}/${output_base}-parallel-1.${extension}"
	local first_headers="${WORK_DIR}/${output_base}-parallel-1.headers"
	validation="$(validate_output "${first_output}" "${expected_mime}" "$(header_value "${first_headers}" "content-type")")"
	printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
		"${case_name}" "${source}" "${kind}" "${input_format}" "${operation}" "${BENCH_CONCURRENT_REQUESTS}" "${BENCH_CONCURRENCY}" "${successes}" "${total_ms}" "${rps}" "${avg_latency}" "${total_bytes}" "${validation}" >>"${PARALLEL_CSV}"
	if [[ "${successes}" != "${BENCH_CONCURRENT_REQUESTS}" ]]; then
		printf 'parallel case %s had %s/%s successful responses\n' "${case_name}" "${successes}" "${BENCH_CONCURRENT_REQUESTS}" >&2
		return 1
	fi
	if [[ "${BENCH_STRICT_MIME}" == "1" && "${validation}" != "ok" ]]; then
		printf 'parallel case %s validation failed: %s\n' "${case_name}" "${validation}" >&2
		return 1
	fi
}

run_benchmarks() {
	run_case "range_text_full" "synthetic" "stream" "txt" "full_stream" \
		"${SERVER_URL}/attachments/bench/text/range.txt" "200" "application/octet-stream" "txt"
	run_case "range_text_partial" "synthetic" "stream" "txt" "range_stream" \
		"${SERVER_URL}/attachments/bench/text/range.txt" "206" "application/octet-stream" "txt" "Range: bytes=2-7"
	run_case "synthetic_audio_wav_full" "synthetic" "audio" "wav" "full_stream" \
		"${SERVER_URL}/attachments/bench/audio/synthetic-30s.wav" "200" "audio/wav" "wav"
	run_case "synthetic_audio_wav_range" "synthetic" "audio" "wav" "range_stream" \
		"${SERVER_URL}/attachments/bench/audio/synthetic-30s.wav" "206" "audio/wav" "wav" "Range: bytes=0-65535"
	run_case "synthetic_mp4_1080p_full" "synthetic" "video" "mp4" "full_stream" \
		"${SERVER_URL}/attachments/bench/videos/synthetic-1080p.mp4" "200" "video/mp4" "mp4"
	run_case "synthetic_mp4_1080p_range" "synthetic" "video" "mp4" "range_stream" \
		"${SERVER_URL}/attachments/bench/videos/synthetic-1080p.mp4" "206" "video/mp4" "mp4" "Range: bytes=0-65535"

	run_case "synthetic_png_4k_to_webp_512" "synthetic" "image" "png" "resize_to_webp_512" \
		"${SERVER_URL}/attachments/bench/images/synthetic-4k.png?width=512&format=webp&quality=high" "200" "image/webp" "webp"
	run_case "synthetic_png_4k_to_jpeg_2048" "synthetic" "image" "png" "resize_to_jpeg_2048" \
		"${SERVER_URL}/attachments/bench/images/synthetic-4k.png?width=2048&format=jpeg&quality=high" "200" "image/jpeg" "jpg"
	run_case "synthetic_jpeg_4k_to_webp_1024" "synthetic" "image" "jpeg" "resize_to_webp_1024" \
		"${SERVER_URL}/attachments/bench/images/synthetic-4k.jpg?width=1024&format=webp&quality=high" "200" "image/webp" "webp"
	run_case "avatar_png_4k_to_webp_512" "synthetic" "avatar" "png" "avatar_square_webp_512" \
		"${SERVER_URL}/avatars/42/benchpng.png?size=512&format=webp" "200" "image/webp" "webp"

	run_if_exists "attachments/bench/images/synthetic-4k.webp" \
		"synthetic_webp_4k_to_png_1024" "synthetic" "image" "webp" "resize_to_png_1024" \
		"${SERVER_URL}/attachments/bench/images/synthetic-4k.webp?width=1024&format=png" "200" "image/png" "png"
	run_if_exists "attachments/bench/images/synthetic-1080p.avif" \
		"synthetic_avif_1080p_to_webp_768" "synthetic" "image" "avif" "resize_to_webp_768" \
		"${SERVER_URL}/attachments/bench/images/synthetic-1080p.avif?width=768&format=webp" "200" "image/webp" "webp"

	run_case "synthetic_gif_to_gif_animated_512" "synthetic" "gif" "gif" "animated_resize_to_gif_512" \
		"${SERVER_URL}/attachments/bench/gifs/$(cd "${STORAGE_ROOT}/cdn/attachments/bench/gifs" && ls synthetic-*.gif | sort | tail -n 1)?width=512&format=webp&animated=true" "200" "image/gif" "gif"
	run_case "synthetic_gif_to_webp_static_512" "synthetic" "gif" "gif" "static_resize_to_webp_512" \
		"${SERVER_URL}/attachments/bench/gifs/$(cd "${STORAGE_ROOT}/cdn/attachments/bench/gifs" && ls synthetic-*.gif | sort | tail -n 1)?width=512&format=webp" "200" "image/webp" "webp"

	run_case "synthetic_mp4_1080p_to_jpeg" "synthetic" "video" "mp4" "thumbnail_to_jpeg" \
		"${SERVER_URL}/attachments/bench/videos/synthetic-1080p.mp4?format=jpeg" "200" "image/jpeg" "jpg"
	run_case "synthetic_mp4_1080p_to_webp" "synthetic" "video" "mp4" "thumbnail_to_webp" \
		"${SERVER_URL}/attachments/bench/videos/synthetic-1080p.mp4?format=webp" "200" "image/webp" "webp"
	run_if_exists "attachments/bench/videos/synthetic-720p.webm" \
		"synthetic_webm_720p_to_jpeg" "synthetic" "video" "webm" "thumbnail_to_jpeg" \
		"${SERVER_URL}/attachments/bench/videos/synthetic-720p.webm?format=jpeg" "200" "image/jpeg" "jpg"
	run_if_exists "attachments/bench/videos/synthetic-4k.mp4" \
		"synthetic_mp4_4k_to_jpeg" "synthetic" "video" "mp4" "thumbnail_to_jpeg" \
		"${SERVER_URL}/attachments/bench/videos/synthetic-4k.mp4?format=jpeg" "200" "image/jpeg" "jpg"

	if [[ "${BENCH_COMPAT_MATRIX}" == "1" ]]; then
		run_if_exists "attachments/bench/compat/videos/h264-1080p.mov" \
			"compat_h264_mov_1080p_to_webp" "synthetic" "video" "mov" "thumbnail_to_webp" \
			"${SERVER_URL}/attachments/bench/compat/videos/h264-1080p.mov?format=webp" "200" "image/webp" "webp"
		run_if_exists "attachments/bench/compat/videos/hevc-1080p.mov" \
			"compat_hevc_mov_1080p_to_webp" "synthetic" "video" "mov" "thumbnail_to_webp" \
			"${SERVER_URL}/attachments/bench/compat/videos/hevc-1080p.mov?format=webp" "200" "image/webp" "webp"
		run_if_exists "attachments/bench/compat/videos/prores-1080p.mov" \
			"compat_prores_mov_1080p_to_webp" "synthetic" "video" "mov" "thumbnail_to_webp" \
			"${SERVER_URL}/attachments/bench/compat/videos/prores-1080p.mov?format=webp" "200" "image/webp" "webp"
		run_if_exists "attachments/bench/compat/videos/h264-1080p.mkv" \
			"compat_h264_mkv_1080p_to_webp" "synthetic" "video" "mkv" "thumbnail_to_webp" \
			"${SERVER_URL}/attachments/bench/compat/videos/h264-1080p.mkv?format=webp" "200" "image/webp" "webp"
		run_if_exists "attachments/bench/compat/videos/mpeg4-720p.avi" \
			"compat_mpeg4_avi_720p_to_webp" "synthetic" "video" "avi" "thumbnail_to_webp" \
			"${SERVER_URL}/attachments/bench/compat/videos/mpeg4-720p.avi?format=webp" "200" "image/webp" "webp"
		run_if_exists "attachments/bench/compat/videos/mpeg2-720p.ts" \
			"compat_mpeg2_ts_720p_to_webp" "synthetic" "video" "ts" "thumbnail_to_webp" \
			"${SERVER_URL}/attachments/bench/compat/videos/mpeg2-720p.ts?format=webp" "200" "image/webp" "webp"
	fi

	run_if_exists "attachments/bench/online/real-photo-4k.jpg" \
		"real_photo_jpeg_4k_to_webp_1024" "online" "image" "jpeg" "resize_to_webp_1024" \
		"${SERVER_URL}/attachments/bench/online/real-photo-4k.jpg?width=1024&format=webp&quality=high" "200" "image/webp" "webp"
	run_if_exists "attachments/bench/online/real-photo-4k.jpg" \
		"real_photo_jpeg_4k_to_jpeg_2048" "online" "image" "jpeg" "resize_to_jpeg_2048" \
		"${SERVER_URL}/attachments/bench/online/real-photo-4k.jpg?width=2048&format=jpeg&quality=high" "200" "image/jpeg" "jpg"
	run_if_exists "attachments/bench/online/google-webp-gallery-1.png" \
		"real_google_png_to_webp_1024" "online" "image" "png" "resize_to_webp_1024" \
		"${SERVER_URL}/attachments/bench/online/google-webp-gallery-1.png?width=1024&format=webp&quality=high" "200" "image/webp" "webp"
	run_if_exists "attachments/bench/online/google-webp-gallery-1.webp" \
		"real_google_webp_to_jpeg_1024" "online" "image" "webp" "resize_to_jpeg_1024" \
		"${SERVER_URL}/attachments/bench/online/google-webp-gallery-1.webp?width=1024&format=jpeg&quality=high" "200" "image/jpeg" "jpg"
	run_if_exists "attachments/bench/online/google-webp-animated-1.gif" \
		"real_google_gif_to_gif_animated_512" "online" "gif" "gif" "animated_resize_to_gif_512" \
		"${SERVER_URL}/attachments/bench/online/google-webp-animated-1.gif?width=512&format=webp&animated=true" "200" "image/gif" "gif"
	run_if_exists "attachments/bench/online/google-webp-animated-1.gif" \
		"real_google_gif_to_webp_static_512" "online" "gif" "gif" "static_resize_to_webp_512" \
		"${SERVER_URL}/attachments/bench/online/google-webp-animated-1.gif?width=512&format=webp" "200" "image/webp" "webp"
	run_if_exists "attachments/bench/online/fronalpstock-big.jpg" \
		"real_fronalpstock_jpeg_to_webp_1024" "online-full" "image" "jpeg" "resize_to_webp_1024" \
		"${SERVER_URL}/attachments/bench/online/fronalpstock-big.jpg?width=1024&format=webp&quality=high" "200" "image/webp" "webp"
	run_if_exists "attachments/bench/online/rotating-earth.gif" \
		"real_rotating_earth_gif_to_gif_animated_512" "online-full" "gif" "gif" "animated_resize_to_gif_512" \
		"${SERVER_URL}/attachments/bench/online/rotating-earth.gif?width=512&format=webp&animated=true" "200" "image/gif" "gif"

	run_if_exists "attachments/bench/online/mdn-flower.mp4" \
		"real_mdn_flower_mp4_to_jpeg" "online" "video" "mp4" "thumbnail_to_jpeg" \
		"${SERVER_URL}/attachments/bench/online/mdn-flower.mp4?format=jpeg" "200" "image/jpeg" "jpg"
	run_if_exists "attachments/bench/online/sample-cat-1080p.mov" \
		"real_sample_cat_mov_1080p_to_webp" "online" "video" "mov" "thumbnail_to_webp" \
		"${SERVER_URL}/attachments/bench/online/sample-cat-1080p.mov?format=webp" "200" "image/webp" "webp"
	run_if_exists "attachments/bench/online/mdn-flower.webm" \
		"real_mdn_flower_webm_to_webp" "online" "video" "webm" "thumbnail_to_webp" \
		"${SERVER_URL}/attachments/bench/online/mdn-flower.webm?format=webp" "200" "image/webp" "webp"
	run_if_exists "attachments/bench/online/big-buck-bunny-720p-10s.mp4" \
		"real_big_buck_bunny_720p_mp4_to_jpeg" "online" "video" "mp4" "thumbnail_to_jpeg" \
		"${SERVER_URL}/attachments/bench/online/big-buck-bunny-720p-10s.mp4?format=jpeg" "200" "image/jpeg" "jpg"
	run_if_exists "attachments/bench/online/big-buck-bunny-medium.ogv" \
		"real_big_buck_bunny_ogv_to_webp" "online-full" "video" "ogv" "thumbnail_to_webp" \
		"${SERVER_URL}/attachments/bench/online/big-buck-bunny-medium.ogv?format=webp" "200" "image/webp" "webp"
	run_if_exists "attachments/bench/online/maple-leaf-ragq.ogg" \
		"real_maple_leaf_ogg_full" "online" "audio" "ogg" "full_stream" \
		"${SERVER_URL}/attachments/bench/online/maple-leaf-ragq.ogg" "200" "audio/ogg" "ogg"
	run_if_exists "attachments/bench/online/maple-leaf-ragq.ogg" \
		"real_maple_leaf_ogg_range" "online" "audio" "ogg" "range_stream" \
		"${SERVER_URL}/attachments/bench/online/maple-leaf-ragq.ogg" "206" "audio/ogg" "ogg" "Range: bytes=0-65535"
	run_if_exists "attachments/bench/online/candles-green-heads-and-skulls.ogg" \
		"real_long_audio_ogg_range" "online-full" "audio" "ogg" "range_stream" \
		"${SERVER_URL}/attachments/bench/online/candles-green-heads-and-skulls.ogg" "206" "audio/ogg" "ogg" "Range: bytes=0-1048575"

	if [[ "${BENCH_EXTERNAL}" == "1" && "${BENCH_ONLINE}" == "1" ]]; then
		if external_photo_url="$(external_url "https://picsum.photos/id/10/3840/2160.jpg")"; then
			run_case "external_real_photo_jpeg_full" "online" "external" "jpeg" "external_full_stream" \
				"${external_photo_url}" "200" "image/jpeg" "jpg"
			run_case "external_real_photo_jpeg_range" "online" "external" "jpeg" "external_range_stream" \
				"${external_photo_url}" "206" "image/jpeg" "jpg" "Range: bytes=0-65535"
		else
			note "skipping external cases; external URL signer unavailable"
		fi
		if external_audio_url="$(external_url "https://commons.wikimedia.org/wiki/Special:Redirect/file/Maple_Leaf_RagQ.ogg")"; then
			run_case "external_real_audio_ogg_range" "online" "external" "ogg" "external_range_stream" \
				"${external_audio_url}" "206" "audio/ogg" "ogg" "Range: bytes=0-65535"
		fi
		if external_long_video_url="$(external_url "https://commons.wikimedia.org/wiki/Special:Redirect/file/Big_Buck_Bunny_medium.ogv")"; then
			run_case "external_long_video_ogv_range" "online" "external" "ogv" "external_range_stream" \
				"${external_long_video_url}" "206" "video/ogg" "ogv" "Range: bytes=0-1048575"
		fi
	fi

	run_post_case "upload_image_metadata_png_4k" "synthetic" "metadata" "png" "metadata_upload" \
		"${SERVER_URL}/_metadata" '{"version":2,"type":"upload","upload_filename":"upload-image.png","filename":"upload-image.png","nsfw":"allow"}' "200" "application/json" "json"
	run_post_case "upload_video_thumbnail_mp4_1080p" "synthetic" "thumbnail" "mp4" "thumbnail_upload" \
		"${SERVER_URL}/_thumbnail" '{"upload_filename":"upload-video.mp4"}' "200" "image/webp" "webp"
	run_post_case "upload_video_frames_mp4_1080p" "synthetic" "frames" "mp4" "frames_upload" \
		"${SERVER_URL}/_frames" '{"version":2,"type":"upload","upload_filename":"upload-video.mp4","nsfw":"allow"}' "200" "application/json" "json"

	run_parallel_case "parallel_png_4k_to_webp_512" "synthetic" "image" "png" "resize_to_webp_512" \
		"${SERVER_URL}/attachments/bench/images/synthetic-4k.png?width=512&format=webp&quality=high" "200" "image/webp" "webp"
	run_parallel_case "parallel_mp4_1080p_to_jpeg" "synthetic" "video" "mp4" "thumbnail_to_jpeg" \
		"${SERVER_URL}/attachments/bench/videos/synthetic-1080p.mp4?format=jpeg" "200" "image/jpeg" "jpg"
}

print_summary() {
	printf '\nSerial latency summary (ms):\n'
	printf "%-46s %8s %10s %10s %10s\n" "case" "n" "min" "avg" "max"
	awk -F, '
		NR > 1 {
			case_name = $1
			value = $8 + 0
			count[case_name]++
			sum[case_name] += value
			if (!(case_name in min) || value < min[case_name]) min[case_name] = value
			if (!(case_name in max) || value > max[case_name]) max[case_name] = value
			validation[case_name] = validation[case_name] "," $12
		}
		END {
			for (case_name in count) {
				printf "%-46s %8d %10.3f %10.3f %10.3f\n", case_name, count[case_name], min[case_name], sum[case_name] / count[case_name], max[case_name]
			}
		}
	' "${CSV}" | sort

	if [[ "${BENCH_PARALLEL}" == "1" ]]; then
		printf '\nParallel throughput summary:\n'
		awk -F, '
			NR > 1 {
				printf "%-36s requests=%s concurrency=%s successes=%s total_ms=%s rps=%s avg_latency_ms=%s validation=%s\n", $1, $6, $7, $8, $9, $10, $11, $13
			}
		' "${PARALLEL_CSV}"
	fi
	printf '\nWrote results to %s\n' "${RUN_DIR}"
}

write_system_info
generate_fixtures
start_proxy
run_benchmarks
print_summary
