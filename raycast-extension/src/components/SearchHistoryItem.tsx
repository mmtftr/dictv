import { Action, ActionPanel, Icon, LaunchType, List, Toast, showToast } from "@raycast/api";

import Command from "../search";
import { SearchHistoryItem } from "../types/types";
import SearchResultItem from "./SearchResultItem";

export default function SearchHistoryItemComponent({
  searchHistoryItem,
  addToHistory,
  removeFromHistory,
}: {
  searchHistoryItem: SearchHistoryItem;
  addToHistory: (result: SearchHistoryItem) => void;
  removeFromHistory: (result: SearchHistoryItem) => void;
}) {
  if (searchHistoryItem.type === "result") {
    return (
      <SearchResultItem
        historyItem
        searchResult={searchHistoryItem}
        removeFromHistory={removeFromHistory}
        addToHistory={addToHistory}
      />
    );
  } else {
    return (
      <List.Item
        title={searchHistoryItem.query}
        icon={Icon.MagnifyingGlass}
        actions={
          <ActionPanel>
            <ActionPanel.Section>
              <Action.Push
                title="Search Again"
                icon={Icon.MagnifyingGlass}
                target={
                  <Command
                    launchContext={{
                      searchText: searchHistoryItem.query,
                    }}
                    launchType={LaunchType.UserInitiated}
                    arguments={{}}
                  />
                }
                onPush={() => {
                  addToHistory(searchHistoryItem);
                }}
              />
            </ActionPanel.Section>
            <ActionPanel.Section>
              <Action
                title="Remove from History"
                icon={Icon.Trash}
                shortcut={{ modifiers: ["cmd"], key: "delete" }}
                onAction={() => {
                  removeFromHistory(searchHistoryItem);
                  showToast({
                    title: "Removed from history",
                    style: Toast.Style.Success,
                  });
                }}
              />
            </ActionPanel.Section>
          </ActionPanel>
        }
      />
    );
  }
}

