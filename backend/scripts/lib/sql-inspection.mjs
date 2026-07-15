function stripSqlComments(content) {
  return content
    .replace(/\/\*[\s\S]*?\*\//gu, '')
    .replace(/--.*$/gmu, '');
}

export function sqlReferencedIdentifiers(content) {
  const sql = stripSqlComments(content);
  const identifiers = new Set();
  const statements = [
    /\b(?:create\s+(?:table|schema|materialized\s+view)|alter\s+table|drop\s+(?:table|schema|materialized\s+view)|insert\s+into|update|delete\s+from|references|from|join)\s+(?:if\s+(?:not\s+)?exists\s+)?([a-zA-Z0-9_".]+)/giu,
    /\bcreate\s+(?:unique\s+)?index\s+[a-zA-Z0-9_".]+\s+on\s+([a-zA-Z0-9_".]+)/giu,
  ];

  for (const pattern of statements) {
    for (const match of sql.matchAll(pattern)) identifiers.add(match[1]);
  }
  return [...identifiers];
}
