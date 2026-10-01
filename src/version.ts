/**
 * Single source of truth for the version shown in the UI.
 *
 * The splash screen and top bar used to hardcode the version string, which
 * silently went stale: the release bumped package.json / tauri.conf.json /
 * Cargo.toml to 2.2.1 while the UI still announced "v2.2.0". Because it is
 * hand-maintained it drifts on every release.
 *
 * Importing it from package.json means the displayed version is whatever the
 * build is actually versioned as -- bump package.json and the UI follows.
 */
import pkg from '../package.json';

export const APP_VERSION: string = pkg.version;