// SPDX-License-Identifier: AGPL-3.0-or-later

const TOR_HOSTNAME_PATTERNS: ReadonlyArray<{
	pattern: RegExp;
	label: string;
}> = [
	{pattern: /(^|[.-])tor[.-]?exit(\d+)?([.-]|$)/i, label: 'tor-exit'},
	{pattern: /(^|[.-])exit[.-]?tor([.-]|$)/i, label: 'exit-tor'},
	{pattern: /(^|[.-])torexit(\d+)?([.-]|$)/i, label: 'torexit'},
	{pattern: /(^|[.-])tor[.-]?relay([.-]|$)/i, label: 'tor-relay'},
	{pattern: /(^|[.-])tor[.-]?node(\d+)?([.-]|$)/i, label: 'tor-node'},
	{pattern: /(^|[.-])tor[.-]?srv([.-]|$)/i, label: 'tor-srv'},
	{pattern: /(^|[.-])torsrv(\d+)?([.-]|$)/i, label: 'torsrv'},
	{pattern: /(^|[.-])torproject([.-]|$)/i, label: 'torproject'},
	{pattern: /\.tor\./i, label: 'dotted-tor'},
	{pattern: /(^|[.-])anonymi[sz]er([.-]|$)/i, label: 'anonymiser'},
	{pattern: /(^|[.-])anon[.-]?proxy([.-]|$)/i, label: 'anon-proxy'},
	{pattern: /^tor\d+\./i, label: 'tor-numbered'},
];

export function matchTorHostname(hostname: string): {
	label: string;
} | null {
	for (const {pattern, label} of TOR_HOSTNAME_PATTERNS) {
		if (pattern.test(hostname)) return {label};
	}
	return null;
}
