// SPDX-License-Identifier: AGPL-3.0-or-later

export type PlatformGpuInfoSource = 'metal' | 'dxgi' | 'linux-sysfs';

export interface PlatformGpuDeviceInfo {
	active: boolean;
	vendorId: number;
	deviceId: number;
	vendorName?: string;
	deviceString?: string;
	driverVendor?: string;
	driverVersion?: string;
	dedicatedVideoMemory?: number;
	sharedSystemMemory?: number;
	subsystemVendorId?: number;
	subsystemDeviceId?: number;
	registryId?: string;
	adapterLuid?: string;
	pciPath?: string;
	integrated?: boolean;
	removable?: boolean;
	headless?: boolean;
	source: PlatformGpuInfoSource;
}

export interface PlatformGpuInfo {
	devices: ReadonlyArray<PlatformGpuDeviceInfo>;
	source: PlatformGpuInfoSource;
	error?: string;
}

export declare const getGpuInfo: (() => PlatformGpuInfo) | null;
export declare const loadError: Error | null;
