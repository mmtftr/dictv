# Setup Instructions for dictv Raycast Extension

## Quick Setup

1. **Install Node.js dependencies**:
   ```bash
   cd raycast-extension
   npm install
   ```

2. **Add the command icon**:
   - Create or download a 512x512 PNG icon
   - Save it as `assets/command-icon.png`
   - Suggestion: Use a book or dictionary emoji (📖) converted to PNG

3. **Ensure dictv server is running**:
   ```bash
   # In the dictv project root
   dictv serve --port 3000
   ```

4. **Import the extension in Raycast**:
   - Open Raycast Settings (⌘,)
   - Go to Extensions tab
   - Click the "+" button
   - Select "Add Script Directory" or "Import Extension"
   - Choose the `raycast-extension` folder
   - The extension will build and become available

## Creating a Command Icon

### Option 1: Emoji to PNG
Use an online converter like:
- https://emoji-to-png.com/
- Select 📖 (book) or 📚 (books) emoji
- Export as 512x512 PNG
- Save as `assets/command-icon.png`

### Option 2: SF Symbols (macOS)
- Open SF Symbols app
- Search for "book" or "character.book.closed"
- Export at 512x512
- Save as `assets/command-icon.png`

### Option 3: Simple SVG to PNG
Create a simple SVG and convert:
```svg
<svg width="512" height="512" xmlns="http://www.w3.org/2000/svg">
  <rect width="512" height="512" fill="#4A90E2"/>
  <text x="50%" y="50%" font-size="300" text-anchor="middle" dy=".3em">📖</text>
</svg>
```

## Verify Installation

1. Open Raycast (⌥Space)
2. Type "dict" or "Search Dictionary"
3. The dictv extension should appear
4. Test a search query

## Troubleshooting

### Extension doesn't appear
- Run `npm run dev` to check for TypeScript errors
- Ensure all dependencies are installed
- Check Raycast extension logs

### Can't connect to server
- Verify dictv server is running: `curl http://localhost:3000/health`
- Check server URL in extension preferences

### Missing icon error
- Ensure `assets/command-icon.png` exists
- Icon must be exactly 512x512 pixels
- Use PNG format only

## Development

```bash
# Watch mode for development
npm run dev

# Build for distribution
npm run build

# Lint code
npm run lint
```

## Next Steps

After setup, you can:
- Customize preferences in Raycast Settings
- Add custom keyboard shortcuts
- Modify search behavior in the source code
- Extend with additional features (e.g., pronunciation, examples)

