#!/usr/bin/env node
/*
 * Copyright (C) 2026 Fluxer Contributors
 *
 * Seeds secrets + a VAPID keypair into a Fluxer config.json. Unlike the dev
 * bootstrap (scripts/dev_bootstrap.sh), production / Docker deployments have no
 * auto-seed step — run this once after copying a template.
 *
 * Usage:
 *   node scripts/seed_config.mjs [config_path] [--from <template>] [--force]
 *
 *   config_path   target file (default: config/config.json)
 *   --from <t>    if target is missing, copy this template first
 *                 (default: config/config.localhost.template.json)
 *   --force       re-generate even values that look already set
 *
 * Only fills fields that are empty or a placeholder (e.g. "GENERATE", "YOUR_*",
 * "GENERATE_A_64_CHAR_HEX_SECRET"). Existing real values are preserved.
 */

import {randomBytes, generateKeyPairSync} from 'node:crypto';
import {existsSync, copyFileSync, readFileSync, writeFileSync} from 'node:fs';
import {resolve} from 'node:path';

const args = process.argv.slice(2);
const force = args.includes('--force');
const fromIdx = args.indexOf('--from');
const template = fromIdx !== -1 ? args[fromIdx + 1] : 'config/config.localhost.template.json';
const positional = args.filter((a, i) => !a.startsWith('--') && args[i - 1] !== '--from');
const target = resolve(positional[0] ?? 'config/config.json');

const PLACEHOLDER = /^(|GENERATE.*|YOUR_.*|.*_HERE|changeme)$/i;
function needs(v) {
	return force || v === undefined || v === null || (typeof v === 'string' && PLACEHOLDER.test(v));
}
function hex(bytes) {
	return randomBytes(bytes).toString('hex');
}
function setIfNeeded(obj, key, gen, label) {
	if (obj && typeof obj === 'object' && key in obj && !needs(obj[key])) return false;
	if (!obj || typeof obj !== 'object') return false;
	obj[key] = gen();
	changed.push(label);
	return true;
}
function vapidKeypair() {
	const {privateKey, publicKey} = generateKeyPairSync('ec', {namedCurve: 'prime256v1'});
	const pub = publicKey.export({format: 'jwk'});
	const priv = privateKey.export({format: 'jwk'});
	const raw = Buffer.concat([
		Buffer.from([0x04]),
		Buffer.from(pub.x, 'base64url'),
		Buffer.from(pub.y, 'base64url'),
	]);
	return {public_key: raw.toString('base64url'), private_key: priv.d};
}

if (!existsSync(target)) {
	const tpl = resolve(template);
	if (!existsSync(tpl)) {
		console.error(`Target ${target} missing and template ${tpl} not found.`);
		process.exit(1);
	}
	copyFileSync(tpl, target);
	console.log(`Created ${target} from ${template}`);
}

const config = JSON.parse(readFileSync(target, 'utf8'));
const changed = [];

const s3 = config.s3 ?? {};
setIfNeeded(s3, 'access_key_id', () => hex(16), 's3.access_key_id');
setIfNeeded(s3, 'secret_access_key', () => hex(32), 's3.secret_access_key');

const svc = config.services ?? {};
setIfNeeded(svc.media_proxy ?? {}, 'secret_key', () => hex(32), 'services.media_proxy.secret_key');
if (svc.media_proxy) setIfNeeded(svc.media_proxy, 'secret_key', () => hex(32), 'services.media_proxy.secret_key');
if (svc.admin) {
	setIfNeeded(svc.admin, 'secret_key_base', () => hex(32), 'services.admin.secret_key_base');
	setIfNeeded(svc.admin, 'oauth_client_secret', () => hex(32), 'services.admin.oauth_client_secret');
}
// Schema requires marketing.secret_key_base even when marketing is DISABLED, so seed it
// whenever the marketing object exists — not only when enabled.
if (svc.marketing) {
	setIfNeeded(svc.marketing, 'secret_key_base', () => hex(32), 'services.marketing.secret_key_base');
}
if (svc.gateway) {
	setIfNeeded(svc.gateway, 'admin_reload_secret', () => hex(32), 'services.gateway.admin_reload_secret');
}
if (svc.queue) {
	setIfNeeded(svc.queue, 'secret', () => hex(32), 'services.queue.secret');
}
// NATS auth_token intentionally left as-is (empty == no-auth on the internal network).

const auth = config.auth ?? {};
setIfNeeded(auth, 'sudo_mode_secret', () => hex(32), 'auth.sudo_mode_secret');
setIfNeeded(auth, 'connection_initiation_secret', () => hex(32), 'auth.connection_initiation_secret');
if (auth.vapid && (force || needs(auth.vapid.public_key) || needs(auth.vapid.private_key))) {
	const kp = vapidKeypair();
	auth.vapid.public_key = kp.public_key;
	auth.vapid.private_key = kp.private_key;
	changed.push('auth.vapid');
}

const search = config.integrations?.search;
if (search) setIfNeeded(search, 'api_key', () => hex(32), 'integrations.search.api_key');

// LiveKit voice: generate a key/secret pair (must match config/livekit.yaml).
const voice = config.integrations?.voice;
if (voice) {
	setIfNeeded(voice, 'api_key', () => `fluxer_${hex(6)}`, 'integrations.voice.api_key');
	setIfNeeded(voice, 'api_secret', () => hex(32), 'integrations.voice.api_secret');
}

if (changed.length === 0) {
	console.log('Nothing to seed — all secrets already set (use --force to regenerate).');
} else {
	writeFileSync(target, `${JSON.stringify(config, null, '\t')}\n`);
	console.log(`Seeded ${changed.length} field(s) in ${target}:`);
	for (const c of changed) console.log(`  - ${c}`);
}

// Render config/livekit.yaml from the template so its key/secret/webhook stay in sync
// with integrations.voice. Only when voice is enabled and a template is present.
if (voice?.enabled) {
	const lkTpl = resolve('config/livekit.template.yaml');
	const lkOut = resolve('config/livekit.yaml');
	if (existsSync(lkTpl)) {
		const rendered = readFileSync(lkTpl, 'utf8')
			.replaceAll('{{LIVEKIT_API_KEY}}', voice.api_key ?? '')
			.replaceAll('{{LIVEKIT_API_SECRET}}', voice.api_secret ?? '')
			.replaceAll('{{WEBHOOK_URL}}', voice.webhook_url ?? 'http://fluxer_server:8080/api/webhooks/livekit');
		if (!existsSync(lkOut) || force) {
			writeFileSync(lkOut, rendered);
			console.log(`Rendered ${lkOut} from livekit.template.yaml (voice enabled).`);
		} else {
			console.log(`Left existing ${lkOut} untouched (use --force to re-render).`);
		}
	}
}
