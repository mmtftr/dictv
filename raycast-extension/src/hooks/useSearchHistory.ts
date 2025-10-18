import { getPreferenceValues } from "@raycast/api";
import { useCachedState } from "@raycast/utils";
import { useCallback, useEffect, useMemo, useRef } from "react";

import { Preferences, SearchHistory, SearchHistoryItem, SearchResult } from "../types/types";

const equals = (a: SearchHistoryItem, b: SearchHistoryItem) => {
  if (a.type === "query" && b.type === "query") {
    return a.query === b.query;
  }

  if (a.type === "result" && b.type === "result") {
    return a.word === b.word && a.language === b.language;
  }

  return false;
};

const appendToHistory = (history: SearchHistory, search: SearchHistoryItem): SearchHistory => {
  // Remove duplicates and push to the front
  history = history.filter((item) => !equals(item, search));
  history.splice(0, 0, search);

  // Limit history to 100 items
  if (history.length > 100) {
    history = history.slice(0, 100);
  }

  return history;
};

export const SearchHistoryItems = {
  resultItem: (result: SearchResult): SearchHistoryItem => ({
    type: "result",
    ...result,
  }),
  queryItem: (query: string): SearchHistoryItem => ({
    type: "query",
    query,
  }),
};

export const useSearchHistory = (currentSearch: string) => {
  const preferences = useMemo(() => getPreferenceValues<Preferences>(), []);

  const currentSearchRef = useRef<string>(currentSearch);
  useEffect(() => {
    if (preferences.saveToHistoryOnClear && currentSearchRef.current && !currentSearch) {
      setHistory((history) => appendToHistory(history, SearchHistoryItems.queryItem(currentSearchRef.current)));
    }

    if (!preferences.saveToHistoryOnClear) {
      // Smart save: only save meaningful queries
      const currentQuery = currentSearch;
      const previousQuery = currentSearchRef.current;

      // Replace the last history item if it's the same as the previous query and the current query is a superset
      if (
        history.length > 0 &&
        history[0].type === "query" &&
        history[0].query === previousQuery &&
        currentQuery.includes(previousQuery)
      ) {
        setHistory((history) => {
          const newHistory = [...history];
          newHistory[0] = SearchHistoryItems.queryItem(currentQuery);
          return newHistory;
        });
      }

      // The user is likely deleting characters, so avoid modifying the history
      if (history.length > 0 && history[0].type === "query" && history[0].query.startsWith(currentQuery)) {
        // Do nothing
      } else if (currentQuery && currentQuery !== previousQuery) {
        // There is a new query not subset of the previous query and not superset of the previous query
        // Append the new query to the history
        setHistory((history) => appendToHistory(history, SearchHistoryItems.queryItem(currentQuery)));
      }
    }

    currentSearchRef.current = currentSearch;
  }, [currentSearch, preferences.saveToHistoryOnClear]);

  const [history, setHistory] = useCachedState("history", [] as SearchHistory);

  const addToHistory = useCallback((chosenItem: SearchHistoryItem) => {
    setHistory((history) => appendToHistory(history, chosenItem));
  }, []);

  const removeFromHistory = useCallback((chosenItem: SearchHistoryItem) => {
    setHistory((history) => history.filter((item) => !equals(item, chosenItem)));
  }, []);

  return { addToHistory, removeFromHistory, history };
};

