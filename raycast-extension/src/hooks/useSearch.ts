import { getPreferenceValues, showToast, Toast } from "@raycast/api";
import { useFetch } from "@raycast/utils";
import { debounce } from "lodash";
import { useMemo, useState } from "react";

import { Language, Preferences, SearchMode, SearchResponse, SearchResult } from "../types/types";

const useSearch = (initialSearchText: string) => {
  const preferences = getPreferenceValues<Preferences>();
  const [searchText, setSearchText] = useState<string>(initialSearchText);
  const [searchMode, setSearchMode] = useState<SearchMode>(preferences.defaultSearchMode);
  const [language, setLanguage] = useState<Language>(preferences.defaultLanguage);

  const [debouncedSearch, _setDebouncedSearch] = useState<string>(initialSearchText);
  const setDebouncedSearch = useMemo(() => {
    return debounce(_setDebouncedSearch, 100);
  }, [_setDebouncedSearch]);

  const onError = (error: Error) => {
    console.error("search error", error);
    showToast({
      style: Toast.Style.Failure,
      title: "Could not perform search",
      message: String(error),
    });
  };

  // Build search URL
  const searchUrl = useMemo(() => {
    if (!debouncedSearch) return null;

    const params = new URLSearchParams({
      q: debouncedSearch,
      mode: searchMode,
      lang: language,
      max_distance: preferences.maxDistance,
      limit: "20",
    });

    return `${preferences.serverUrl}/search?${params.toString()}`;
  }, [debouncedSearch, searchMode, language, preferences.serverUrl, preferences.maxDistance]);

  const options = {
    parseResponse: parseResponse,
    initialData: [],
    keepPreviousData: true,
    onError,
    execute: !!searchUrl,
  };

  const { isLoading, data } = useFetch(searchUrl || "", options);

  return {
    state: {
      isLoading,
      results: debouncedSearch ? data : [],
    },
    setSearchText: (text: string) => {
      setSearchText(text);
      setDebouncedSearch(text);
    },
    searchText,
    searchMode,
    setSearchMode,
    language,
    setLanguage,
  };
};

async function parseResponse(response: Response): Promise<SearchResult[]> {
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  const json = (await response.json()) as SearchResponse;
  return json.results;
}

export default useSearch;
