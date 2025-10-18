import { Action, ActionPanel, Icon, List, Toast, showToast } from "@raycast/api";
import { useCallback } from "react";

import { SearchHistoryItems } from "../hooks/useSearchHistory";
import { SearchHistoryItem, SearchResult } from "../types/types";
import { formatDefinitionsAsMarkdown, getShortDefinition } from "../utils/definitionParser";

export default function SearchResultItem({
  searchResult,
  addToHistory,
  removeFromHistory,
  historyItem,
  showingDetail = true,
  onToggleDetail,
  onSwitchLanguage,
  language,
}: {
  addToHistory: (result: SearchHistoryItem) => void;
  removeFromHistory: (result: SearchHistoryItem) => void;
  historyItem?: boolean;
  searchResult: SearchResult;
  showingDetail?: boolean;
  onToggleDetail?: () => void;
  onSwitchLanguage?: () => void;
  language?: "de-en" | "en-de";
}) {
  const onChoose = useCallback(
    () => addToHistory(SearchHistoryItems.resultItem(searchResult)),
    [searchResult, addToHistory],
  );

  const languageLabel = searchResult.language === "de-en" ? "🇩🇪→🇬🇧" : "🇬🇧→🇩🇪";
  const matchInfo =
    searchResult.edit_distance > 0
      ? ` (fuzzy: ~${searchResult.edit_distance})`
      : searchResult.edit_distance === 0
        ? " (exact)"
        : "";

  // Handle backward compatibility with old format
  const definitions = searchResult.definitions || [];
  const shortDefinition = getShortDefinition(definitions);
  const markdown = formatDefinitionsAsMarkdown(searchResult.word, definitions, searchResult.language);
  const allDefinitionsText = definitions.join("\n\n---\n\n");

  const props: Partial<List.Item.Props> = showingDetail
    ? {
        detail: <List.Item.Detail markdown={markdown} />,
      }
    : {
        subtitle: shortDefinition,
        accessories: [
          {
            text: `${languageLabel}${matchInfo}`,
          },
        ],
      };

  return (
    <List.Item
      title={searchResult.word}
      {...props}
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
              title="Copy All Definitions"
              content={allDefinitionsText}
              shortcut={{ modifiers: ["cmd", "shift"], key: "." }}
            />
            <Action.CopyToClipboard
              title="Copy Word + Definitions"
              content={`${searchResult.word}:\n${allDefinitionsText}`}
              shortcut={{ modifiers: ["cmd", "opt"], key: "." }}
            />
          </ActionPanel.Section>
          {onToggleDetail && (
            <ActionPanel.Section>
              <Action
                title="Toggle Detail"
                icon={showingDetail ? Icon.List : Icon.Eye}
                shortcut={{ modifiers: ["cmd"], key: "d" }}
                onAction={onToggleDetail}
              />
              {onSwitchLanguage && language && (
                <Action
                  title={`Switch to ${language === "de-en" ? "English→German" : "German→English"}`}
                  icon={Icon.Switch}
                  shortcut={{ modifiers: ["cmd"], key: "l" }}
                  onAction={onSwitchLanguage}
                />
              )}
            </ActionPanel.Section>
          )}
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
