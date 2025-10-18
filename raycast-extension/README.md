# dictv Raycast Extension

Search German-English dictionary with fuzzy matching directly from Raycast.

## Features

- **Fast Dictionary Lookup**: Instant search results from your local dictv server
- **Multiple Search Modes**:
  - Fuzzy search (handles typos and spelling variations)
  - Exact search (precise matches only)
  - Prefix search (autocomplete-style)
- **Bidirectional**: Switch between German→English and English→German
- **Search History**: Keep track of your searches and results
- **Keyboard Shortcuts**: Fast navigation and actions

## Prerequisites

You need to have the dictv server running locally:

1. Install dictv: `cargo install --path /path/to/dictv`
2. Import dictionaries:
   ```bash
   dictv import --download freedict-deu-eng
   dictv import --download freedict-eng-deu
   ```
3. Start the server: `dictv serve --port 3000`

## Installation

1. Clone this extension or copy the `raycast-extension` folder
2. In Raycast, go to Extensions → Add Extension → Import Extension
3. Select the `raycast-extension` folder
4. Install dependencies: `npm install`
5. The extension will be available in Raycast

## Usage

### Search Dictionary

- Open Raycast
- Type `dict` or `Dictionary` to open the search command
- Start typing your query
- Use ⌘L to switch between German→English and English→German
- Use the dropdown to change search mode (fuzzy/exact/prefix)
- Press Enter to copy the word
- Press ⌘. to copy the word, ⌘⇧. for definition

### View Search History

- Open Raycast
- Type `Dictionary History`
- Browse your previous searches
- Click on any item to search again
- Press ⌘Delete to remove items from history

## Configuration

Go to Extension Preferences (⌘,) to configure:

- **Server URL**: Default is `http://localhost:3000`
- **Default Language Direction**: Choose German→English or English→German
- **Default Search Mode**: Choose fuzzy, exact, or prefix
- **Fuzzy Match Distance**: Set to 1 (stricter) or 2 (more lenient)
- **Save to History on Clear**: Automatically save searches when cleared

## Keyboard Shortcuts

### In Search View
- `⌘.` - Copy word
- `⌘⇧.` - Copy definition
- `⌘⌥.` - Copy both word and definition
- `⌘L` - Switch language direction
- `⌘↵` - Add to history
- `⌘⌫` - Remove from history (history items only)

### In History View
- `⌘⌫` - Remove from history

## Troubleshooting

### "Could not perform search"

Make sure the dictv server is running:
```bash
dictv serve --port 3000
```

Check the server URL in extension preferences matches your setup.

### No results found

- Try using fuzzy search mode
- Check if you have imported the correct dictionary direction
- Verify dictionaries are imported: `dictv stats`

### Slow search results

- Reduce fuzzy match distance to 1
- Use exact or prefix search mode for faster results
- Check dictv server performance with `dictv query "test"`

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run dev

# Build for production
npm run build

# Lint code
npm run lint
```

## License

MIT

