/*
 * Copyright (C) 2026 Fluxer Contributors
 *
 * This file is part of Fluxer.
 *
 * Fluxer is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Fluxer is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with Fluxer. If not, see <https://www.gnu.org/licenses/>.
 */

// Self-host patch: brand + default server for the Modo desktop build. APP_PROTOCOL is the custom
// URL scheme registered with the OS for deep links (must be unique per install). Both STABLE and
// CANARY point at our self-hosted instance; users can still switch servers from the app's settings.
export const APP_PROTOCOL = 'fluxermodo';
export const STABLE_APP_URL = 'https://modo.bigweld.duckdns.org';
export const CANARY_APP_URL = 'https://modo.bigweld.duckdns.org';
export const DEFAULT_WINDOW_WIDTH = 1280;
export const DEFAULT_WINDOW_HEIGHT = 800;
export const MIN_WINDOW_WIDTH = 800;
export const MIN_WINDOW_HEIGHT = 600;
