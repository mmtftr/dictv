import { Action, ActionPanel, Icon, List, Toast, showToast } from "@raycast/api";
import { useCallback } from "react";

import { SearchHistoryItems } from "../hooks/useSearchHistory";
import { SearchHistoryItem, SearchResult } from "../types/types";

export default function SearchResultItem({
  searchResult,
  addToHistory,
  removeFromHistory,
  historyItem,
}: {
  addToHistory: (result: SearchHistoryItem) => void;
  removeFromHistory: (result: SearchHistoryItem) => void;
  historyItem?: boolean;
  searchResult: SearchResult;
}) {
  const onChoose = useCallback(
    () => addToHistory(SearchHistoryItems.resultItem(searchResult)),
    [searchResult, addToHistory],
  );

  const languageLabel = searchResult.language === "de-en" ? "🇩🇪→🇬🇧" : "🇬🇧→🇩🇪";
  const matchInfo =
    searchResult.edit_distance > 0 ? ` (fuzzy: ~${searchResult.edit_distance})` : searchResult.edit_distance === 0 ? " (exact)" : "";

  return (
    <List.Item
      title={searchResult.word}
      subtitle={searchResult.definition}
      accessories={[
        {
          text: `${languageLabel}${matchInfo}`,
        },
      ]}
      actions={
        <ActionPanel>
          <ActionPanel.Section>
            <Action.CopyToClipboard
              onCopy={onChoose}
              title="Copy Word"
              content={searchResult.word}
              shortcut={{ modifiers: ["cmd"], key: "." }}
            />
            <Action.CopyToClipboard
              title="Copy Definition"
              content={searchResult.definition}
              shortcut={{ modifiers: ["cmd", "shift"], key: "." }}
            />
            <Action.CopyToClipboard
              title="Copy Both"
              content={`${searchResult.word}: ${searchResult.definition}`}
              shortcut={{ modifiers: ["cmd", "opt"], key: "." }}
            />
          </ActionPanel.Section>
          <ActionPanel.Section>
            <Action
              title="Add to History"
              icon={Icon.Plus}
              shortcut={{ modifiers: ["cmd"], key: "enter" }}
              onAction={() => {
                addToHistory(SearchHistoryItems.resultItem(searchResult));
                showToast({
                  title: "Added to history",
                  style: Toast.Style.Success,
                });
              }}
            />
            {historyItem && (
              <Action
                title="Remove from History"
                icon={Icon.Trash}
                shortcut={{ modifiers: ["cmd"], key: "delete" }}
                onAction={() => {
                  removeFromHistory(SearchHistoryItems.resultItem(searchResult));
                  showToast({
                    title: "Removed from history",
                    style: Toast.Style.Success,
                  });
                }}
              />
            )}
          </ActionPanel.Section>
        </ActionPanel>
      }
    />
  );
}

