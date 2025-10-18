export interface SearchState {
  results: SearchResult[];
  isLoading: boolean;
}

export interface SearchResult {
  word: string;
  definition: string;
  language: "de-en" | "en-de";
  edit_distance: number;
  score: number;
}

export interface SearchResponse {
  results: SearchResult[];
  query_time_ms: number;
  total_results: number;
}

export interface Preferences {
  serverUrl: string;
  defaultLanguage: "de-en" | "en-de";
  defaultSearchMode: "exact" | "fuzzy" | "prefix";
  maxDistance: string;
  saveToHistoryOnClear: boolean;
}

export type SearchMode = "exact" | "fuzzy" | "prefix";
export type Language = "de-en" | "en-de";

export type SearchHistoryItem = (SearchResult & { type: "result" }) | { type: "query"; query: string };

export type SearchHistory = SearchHistoryItem[];

