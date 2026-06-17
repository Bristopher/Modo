// SPDX-License-Identifier: AGPL-3.0-or-later

export interface IGatewayRpcTransport {
	call(method: string, params: Record<string, unknown>): Promise<unknown>;
	destroy(): Promise<void>;
}
