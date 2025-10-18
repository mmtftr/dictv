/**
 * Parse DICTD format definitions into readable markdown
 */

export function parseDefinition(definition: string): string {
  let parsed = definition;

  // Remove leading/trailing whitespace
  parsed = parsed.trim();

  // Extract and format pronunciation /.../
  parsed = parsed.replace(/\/([^/]+)\//g, (match, pronunciation) => {
    return `\n\n**Pronunciation:** /${pronunciation}/\n\n`;
  });

  // Extract and format grammatical information <...>
  parsed = parsed.replace(/<([^>]+)>/g, (match, grammar) => {
    // Clean up grammar markers
    const cleanGrammar = grammar
      .replace(/,\s*/g, ", ")
      .replace(/\bneut\b/g, "neuter")
      .replace(/\bmasc\b/g, "masculine")
      .replace(/\bfem\b/g, "feminine")
      .replace(/\bn\b/g, "noun")
      .replace(/\bv\b/g, "verb")
      .replace(/\badj\b/g, "adjective")
      .replace(/\badv\b/g, "adverb")
      .replace(/\bsg\b/g, "singular")
      .replace(/\bpl\b/g, "plural")
      .replace(/\btrans\b/g, "transitive")
      .replace(/\bintrans\b/g, "intransitive");
    return `*[${cleanGrammar}]*`;
  });

  // Handle square brackets [ugs.], [adm.], etc. - these are usage labels
  parsed = parsed.replace(/\[([^\]]+)\]/g, (match, label) => {
    return `*[${label}]*`;
  });

  // Extract and format synonyms (in curly braces)
  let synonyms: string[] = [];
  parsed = parsed.replace(/Synonyms?:\s*\{([^}]+)\}/g, (match, syns) => {
    synonyms.push(...syns.split(",").map((s: string) => s.trim()));
    return "";
  });
  parsed = parsed.replace(/\{([^}]+)\}/g, (match, syn) => {
    if (!synonyms.includes(syn.trim())) {
      synonyms.push(syn.trim());
    }
    return "";
  });

  // Extract "see:" references
  let seeAlso: string[] = [];
  parsed = parsed.replace(/see:\s*\{([^}]+)\}/g, (match, refs) => {
    seeAlso.push(...refs.split(",").map((s: string) => s.trim()));
    return "";
  });

  // Extract and format quoted examples "..."
  let examples: string[] = [];
  parsed = parsed.replace(/"([^"]+)"\s*-\s*([^"]+?)(?="|$)/g, (match, german, english) => {
    examples.push(`"${german.trim()}" → ${english.trim()}`);
    return "";
  });

  // Extract Note: sections
  let notes: string[] = [];
  parsed = parsed.replace(/Note:\s*([^.]+)/g, (match, note) => {
    notes.push(note.trim());
    return "";
  });

  // Clean up multiple spaces and newlines
  parsed = parsed.replace(/\s+/g, " ").trim();

  // Remove leftover separator characters
  parsed = parsed.replace(/\s*,\s*,\s*/g, ", ");
  parsed = parsed.replace(/\s+,/g, ",");
  parsed = parsed.replace(/,\s+/g, ", ");

  // Build the formatted output
  let output = parsed;

  // Add examples section if we have any
  if (examples.length > 0) {
    output += "\n\n**Examples:**\n";
    examples.forEach((ex) => {
      output += `- ${ex}\n`;
    });
  }

  // Add synonyms section if we have any
  if (synonyms.length > 0) {
    output += `\n**Synonyms:** ${synonyms.join(", ")}`;
  }

  // Add see also section if we have any
  if (seeAlso.length > 0) {
    output += `\n\n**See also:** ${seeAlso.join(", ")}`;
  }

  // Add notes section if we have any
  if (notes.length > 0) {
    output += "\n\n**Notes:**\n";
    notes.forEach((note) => {
      output += `- ${note}\n`;
    });
  }

  return output;
}

export function formatDefinitionsAsMarkdown(word: string, definitions: string[] | undefined, language: string): string {
  const languageLabel = language === "de-en" ? "German → English" : "English → German";

  let markdown = `# ${word}\n\n`;
  markdown += `**${languageLabel}**\n\n`;

  if (!definitions || definitions.length === 0) {
    markdown += "*No definition available*";
    return markdown;
  }

  if (definitions.length === 1) {
    markdown += parseDefinition(definitions[0]);
  } else {
    definitions.forEach((def, index) => {
      markdown += `## Definition ${index + 1}\n\n`;
      markdown += parseDefinition(def) + "\n\n";
    });
  }

  return markdown;
}

export function getShortDefinition(definitions: string[] | undefined): string {
  if (!definitions || definitions.length === 0) return "No definition";

  const firstDef = definitions[0];
  if (!firstDef) return "No definition";

  // Take first line or first sentence
  const firstLine = firstDef.split("\n")[0];
  const firstSentence = firstLine.split(/[.;]/)[0];

  // Limit length
  const maxLength = 100;
  if (firstSentence.length > maxLength) {
    return firstSentence.substring(0, maxLength) + "...";
  }

  // Add count if multiple definitions
  if (definitions.length > 1) {
    return `${firstSentence} (+${definitions.length - 1} more)`;
  }

  return firstSentence;
}
