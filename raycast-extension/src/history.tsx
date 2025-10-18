import { List } from "@raycast/api";

import SearchHistoryItemComponent from "./components/SearchHistoryItem";
import { useSearchHistory } from "./hooks/useSearchHistory";

export default function Command() {
  const { history, addToHistory, removeFromHistory } = useSearchHistory("");

  return (
    <List searchBarPlaceholder="Filter history...">
      <List.Section title="Search History" subtitle={history.length + ""}>
        {history.map((item, idx) => (
          <SearchHistoryItemComponent
            key={idx}
            searchHistoryItem={item}
            removeFromHistory={removeFromHistory}
            addToHistory={addToHistory}
          />
        ))}
      </List.Section>
      {history.length === 0 && (
        <List.EmptyView
          title="No search history"
          description="Start searching to build your history"
        />
      )}
    </List>
  );
}

