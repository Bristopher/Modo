// SPDX-License-Identifier: AGPL-3.0-or-later

export interface HttpClientMetrics {
	counter(params: {name: string; dimensions?: Record<string, string>; value?: number}): void;
	histogram(params: {name: string; dimensions?: Record<string, string>; valueMs: number}): void;
}

export interface HttpClientTelemetry {
	metrics?: HttpClientMetrics;
}
