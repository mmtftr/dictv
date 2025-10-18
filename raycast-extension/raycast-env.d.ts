/// <reference types="@raycast/api">

/* 🚧 🚧 🚧
 * This file is auto-generated from the extension's manifest.
 * Do not modify manually. Instead, update the `package.json` file.
 * 🚧 🚧 🚧 */

/* eslint-disable @typescript-eslint/ban-types */

type ExtensionPreferences = {
  /** Server URL - The URL of the dictv server */
  "serverUrl": string,
  /** Default Language Direction - Default search direction */
  "defaultLanguage": "de-en" | "en-de",
  /** Default Search Mode - Default search mode for queries */
  "defaultSearchMode": "fuzzy" | "exact" | "prefix",
  /** Fuzzy Match Distance - Maximum edit distance for fuzzy matching (1-2) */
  "maxDistance": "1" | "2",
  /** Save to History on Clear - Save the last search query to history when search is cleared */
  "saveToHistoryOnClear": boolean
}

/** Preferences accessible in all the extension's commands */
declare type Preferences = ExtensionPreferences

declare namespace Preferences {
  /** Preferences accessible in the `search` command */
  export type Search = ExtensionPreferences & {}
  /** Preferences accessible in the `history` command */
  export type History = ExtensionPreferences & {}
}

declare namespace Arguments {
  /** Arguments passed to the `search` command */
  export type Search = {}
  /** Arguments passed to the `history` command */
  export type History = {}
}

