import { Icon, LaunchProps, List } from "@raycast/api";
import { useState } from "react";

import SearchResultItem from "./components/SearchResultItem";
import useSearch from "./hooks/useSearch";
import { useSearchHistory } from "./hooks/useSearchHistory";
import { SearchMode } from "./types/types";

export default function Command({ launchContext }: LaunchProps) {
  const { searchText: initialSearchText = "" } = (launchContext as { searchText: string }) || {};
  const {
    state,
    setSearchText: search,
    searchText,
    searchMode,
    setSearchMode,
    language,
    setLanguage,
  } = useSearch(initialSearchText);

  const { addToHistory, removeFromHistory } = useSearchHistory(searchText);
  const [showingDetail, setShowingDetail] = useState(true);

  const modeIcon = searchMode === "fuzzy" ? "🔍" : searchMode === "exact" ? "🎯" : "📝";
  const langLabel = language === "de-en" ? "🇩🇪→🇬🇧" : "🇬🇧→🇩🇪";

  return (
    <List
      isLoading={state.isLoading}
      searchText={searchText}
      onSearchTextChange={search}
      searchBarPlaceholder={`Search dictionary... (${langLabel})`}
      isShowingDetail={showingDetail}
      searchBarAccessory={
        <List.Dropdown
          tooltip="Search Mode"
          value={searchMode}
          onChange={(newValue) => setSearchMode(newValue as SearchMode)}
        >
          <List.Dropdown.Item title="Fuzzy (handles typos)" value="fuzzy" icon="🔍" />
          <List.Dropdown.Item title="Exact (exact matches)" value="exact" icon="🎯" />
          <List.Dropdown.Item title="Prefix (starts with...)" value="prefix" icon="📝" />
        </List.Dropdown>
      }
    >
      <List.Section title="Results" subtitle={state.results.length + ""}>
        {state.results.map((searchResult, idx) => (
          <SearchResultItem
            key={`${searchResult.word}-${idx}`}
            searchResult={searchResult}
            addToHistory={addToHistory}
            removeFromHistory={removeFromHistory}
            showingDetail={showingDetail}
            onToggleDetail={() => setShowingDetail(!showingDetail)}
            onSwitchLanguage={() => setLanguage(language === "de-en" ? "en-de" : "de-en")}
            language={language}
          />
        ))}
      </List.Section>
      {!state.isLoading && state.results.length === 0 && searchText && (
        <List.EmptyView
          title="No results found"
          description="Try fuzzy search mode or check if the dictv server is running"
          icon={Icon.MagnifyingGlass}
        />
      )}
    </List>
  );
}
