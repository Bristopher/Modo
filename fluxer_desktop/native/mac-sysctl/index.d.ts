// SPDX-License-Identifier: AGPL-3.0-or-later

export declare const sysctlByNameInt: ((name: string) => Promise<number | null>) | null;
export declare const sysctlByNameString: ((name: string) => Promise<string | null>) | null;
export declare const loadError: Error | null;
