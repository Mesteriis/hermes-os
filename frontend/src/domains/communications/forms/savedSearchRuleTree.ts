export type SavedSearchMatchMode = 'all' | 'any'
export type SavedSearchRuleValidation = {
  isValid: boolean
  message: string
}
export type SavedSearchRuleField = 'subject' | 'body' | 'sender' | 'all'
export type SavedSearchRuleOperator = ':' | '='
export type SavedSearchRule = {
  field: SavedSearchRuleField
  operator: SavedSearchRuleOperator
  value: string
}
export type SavedSearchRuleCondition = SavedSearchRule & {
  id: string
  kind: 'rule'
}
export type SavedSearchRuleGroup = {
  id: string
  kind: 'group'
  matchMode: SavedSearchMatchMode
  children: SavedSearchRuleNode[]
}
export type SavedSearchRuleNode = SavedSearchRuleCondition | SavedSearchRuleGroup
export type SavedSearchParsedQuery = {
  plainQuery: string
  rules: SavedSearchRule[]
  matchMode: SavedSearchMatchMode
  tree: SavedSearchRuleGroup
}
export type SavedSearchBuilderState = SavedSearchParsedQuery

export function tokenizeSavedSearchQuery(rawQuery: string): string[] {
  const terms: string[] = []
  let current = ''
  let inQuotes = false
  let quote: string | null = null

  for (const symbol of rawQuery) {
    if ((symbol === '"' || symbol === "'") && (!inQuotes || quote === symbol)) {
      inQuotes = !inQuotes
      quote = inQuotes ? symbol : null
      current += symbol
      continue
    }

    if (symbol.trim() === '' && !inQuotes) {
      if (current.trim()) {
        terms.push(current.trim())
      }
      current = ''
      continue
    }

    current += symbol
  }

  if (current.trim()) terms.push(current.trim())
  return terms
}

export function parseSavedSearchQuery(rawQuery: string): SavedSearchParsedQuery {
  const normalized = rawQuery.trim()
  if (!normalized) {
    return {
      plainQuery: '',
      rules: [],
      matchMode: 'all',
      tree: createSavedSearchRuleGroup('all', [])
    }
  }

  const explicitTree = parseExplicitSavedSearchTree(normalized)
  if (explicitTree) {
    return {
      plainQuery: '',
      rules: flattenSavedSearchRuleTree(explicitTree),
      matchMode: explicitTree.matchMode,
      tree: explicitTree
    }
  }

  const rules: SavedSearchRule[] = []
  const plainQueryTerms: string[] = []
  let matchMode: SavedSearchMatchMode = 'all'

  for (const token of tokenizeSavedSearchQuery(normalized)) {
    const parsedMode = parseSavedSearchMatchMode(token)
    if (parsedMode) {
      matchMode = parsedMode
      continue
    }

    const rule = parseSavedSearchRuleToken(token)
    if (rule) {
      rules.push(rule)
      continue
    }
    plainQueryTerms.push(token)
  }

  return {
    plainQuery: plainQueryTerms.join(' '),
    rules,
    matchMode,
    tree: createSavedSearchRuleGroup(matchMode, rules.map((rule) => createSavedSearchRuleCondition(rule)))
  }
}

export function composeSavedSearchQuery(
  plainQuery: string,
  rules: ReadonlyArray<SavedSearchRule>,
  matchMode: SavedSearchMatchMode = 'all'
): string {
  const ruleTokens = rules
    .map((rule) => {
      const trimmedValue = rule.value.trim()
      if (!trimmedValue) return ''
      return `${rule.field}${rule.operator}${formatSavedSearchRuleValue(trimmedValue)}`
    })
    .filter(Boolean)

  const merged: string[] = []
  if (plainQuery.trim()) merged.push(plainQuery.trim())
  if (ruleTokens.length) merged.push(...ruleTokens)
  if (matchMode === 'any' && merged.length > 0) {
    merged.unshift('mode:any')
  }

  return merged.join(' ')
}

export function normalizeSavedSearchBuilderState(
  rawQuery: string,
  existingRules: ReadonlyArray<SavedSearchRule> = [],
  currentMatchMode: SavedSearchMatchMode = 'all'
): SavedSearchBuilderState {
  const parsed = parseSavedSearchQuery(rawQuery)
  const mergedRules = mergeSavedSearchRules(parsed.rules, existingRules)
  const matchMode = parsed.matchMode === 'any' ? 'any' : currentMatchMode

  return {
    plainQuery: parsed.plainQuery,
    rules: mergedRules,
    matchMode,
    tree: parsed.tree.kind === 'group'
      ? parsed.tree
      : createSavedSearchRuleGroup(matchMode, mergedRules.map((rule) => createSavedSearchRuleCondition(rule)))
  }
}

export function resolveSavedSearchEffectiveQuery(
  rawQuery: string,
  existingRules: ReadonlyArray<SavedSearchRule> = [],
  currentMatchMode: SavedSearchMatchMode = 'all'
): string {
  const normalized = normalizeSavedSearchBuilderState(rawQuery, existingRules, currentMatchMode)
  return composeSavedSearchQuery(normalized.plainQuery, normalized.rules, normalized.matchMode)
}

export function validateSavedSearchRules(
  rules: ReadonlyArray<SavedSearchRule>
): SavedSearchRuleValidation {
  const normalizedRules = rules
    .map((rule) => ({
      field: rule.field,
      operator: rule.operator,
      value: rule.value.trim()
    }))

  const incompleteCount = normalizedRules.filter((rule) => !rule.value).length
  if (incompleteCount > 0) {
    return {
      isValid: false,
      message: incompleteCount === 1
        ? 'Complete or remove the empty rule before saving'
        : 'Complete or remove the empty rules before saving'
    }
  }

  const seen = new Set<string>()
  for (const rule of normalizedRules) {
    const signature = `${rule.field}|${rule.operator}|${rule.value.toLowerCase()}`
    if (seen.has(signature)) {
      return {
        isValid: false,
        message: 'Remove duplicate rules before saving'
      }
    }
    seen.add(signature)
  }

  return {
    isValid: true,
    message: ''
  }
}

export function validateSavedSearchRuleTree(
  group: SavedSearchRuleGroup
): SavedSearchRuleValidation {
  if (group.children.length === 0) {
    return {
      isValid: false,
      message: 'Add at least one rule or group before saving'
    }
  }

  const flattened = flattenSavedSearchRuleTree(group)
  const ruleValidation = validateSavedSearchRules(flattened)
  if (!ruleValidation.isValid) return ruleValidation

  for (const child of group.children) {
    if (child.kind === 'group') {
      const nestedValidation = validateSavedSearchRuleTree(child)
      if (!nestedValidation.isValid) return nestedValidation
    }
  }

  return {
    isValid: true,
    message: ''
  }
}

export function createSavedSearchRuleCondition(
  rule: Partial<SavedSearchRule> = {}
): SavedSearchRuleCondition {
  return {
    id: savedSearchBuilderId('rule'),
    kind: 'rule',
    field: rule.field ?? 'subject',
    operator: rule.operator ?? ':',
    value: rule.value ?? ''
  }
}

export function createSavedSearchRuleGroup(
  matchMode: SavedSearchMatchMode = 'all',
  children: SavedSearchRuleNode[] = []
): SavedSearchRuleGroup {
  return {
    id: savedSearchBuilderId('group'),
    kind: 'group',
    matchMode,
    children
  }
}

export function flattenSavedSearchRuleTree(
  group: SavedSearchRuleGroup
): SavedSearchRule[] {
  return group.children.flatMap((child) => {
    if (child.kind === 'group') return flattenSavedSearchRuleTree(child)
    return [{ field: child.field, operator: child.operator, value: child.value }]
  })
}

export function composeSavedSearchRuleTreeQuery(
  plainQuery: string,
  group: SavedSearchRuleGroup
): string {
  if (group.children.length === 0) {
    return plainQuery.trim()
  }

  const hasNestedGroups = group.children.some((child) => child.kind === 'group')
  if (!hasNestedGroups && group.matchMode === 'all') {
    return composeSavedSearchQuery(plainQuery, flattenSavedSearchRuleTree(group), 'all')
  }

  const plainTerms = plainQuery.trim()
    ? tokenizeSavedSearchQuery(plainQuery).map((term) =>
        createSavedSearchRuleCondition({ field: 'all', operator: ':', value: parseSavedSearchRuleValue(term) })
      )
    : []
  const rootChildren = group.matchMode === 'any'
    ? [...plainTerms, ...group.children]
    : group.children
  const expression = formatSavedSearchRuleGroupExpression({
    ...group,
    children: rootChildren
  })
  if (group.matchMode === 'all' && plainQuery.trim()) {
    return `${plainQuery.trim()} ${expression}`.trim()
  }
  return expression
}

function parseSavedSearchMatchMode(token: string): SavedSearchMatchMode | null {
  const [rawField, rawValue] = token.split(':', 2)
  if (rawValue === undefined) return null
  if (rawField.trim().toLowerCase() !== 'mode') return null

  const value = rawValue.trim().toLowerCase()
  if (value === 'all' || value === 'any') return value
  return null
}

function parseSavedSearchRuleToken(token: string): SavedSearchRule | null {
  const operators: Array<SavedSearchRuleOperator | '=='> = ['==', '=', ':']

  for (const operator of operators) {
    const index = token.indexOf(operator)
    if (index <= 0) continue

    const rawField = token.slice(0, index)
    const normalizedOperator: SavedSearchRuleOperator = operator === '==' ? '=' : operator
    const rawValue = token.slice(index + operator.length)

    const field = parseSavedSearchRuleField(rawField)
    if (!field || !rawValue.trim()) return null

    const value = parseSavedSearchRuleValue(rawValue)
    if (!value) return null

    return { field, operator: normalizedOperator, value }
  }

  return null
}

function parseSavedSearchRuleField(value: string): SavedSearchRuleField | null {
  const normalized = value.trim().toLowerCase()
  if (normalized === 'from') return 'sender'
  if (normalized === 'subject' || normalized === 'body' || normalized === 'sender' || normalized === 'all') {
    return normalized
  }
  return null
}

function parseSavedSearchRuleValue(rawValue: string): string {
  const value = rawValue.trim()
  if (value.length < 2) return value

  const isDouble = value.startsWith('"') && value.endsWith('"')
  const isSingle = value.startsWith("'") && value.endsWith("'")
  if (!isDouble && !isSingle) return value

  return value.slice(1, -1)
}

function formatSavedSearchRuleValue(value: string): string {
  const trimmed = value.trim()
  if (!trimmed) return ''
  const needsQuote = trimmed.includes(' ') || trimmed.includes('"') || trimmed.includes("'")
  if (!needsQuote) return trimmed
  return `"${trimmed.replaceAll('"', '\\"')}"`
}

function mergeSavedSearchRules(
  parsedRules: ReadonlyArray<SavedSearchRule>,
  existingRules: ReadonlyArray<SavedSearchRule>
): SavedSearchRule[] {
  const merged: SavedSearchRule[] = []
  const seen = new Set<string>()

  for (const rule of [...parsedRules, ...existingRules]) {
    const normalized = normalizeSavedSearchRule(rule)
    if (!normalized) continue
    const signature = `${normalized.field}|${normalized.operator}|${normalized.value.toLowerCase()}`
    if (seen.has(signature)) continue
    seen.add(signature)
    merged.push(normalized)
  }

  return merged
}

function normalizeSavedSearchRule(rule: SavedSearchRule): SavedSearchRule | null {
  const value = rule.value.trim()
  if (!value) return null

  return {
    field: rule.field,
    operator: rule.operator,
    value
  }
}

function parseExplicitSavedSearchTree(rawQuery: string): SavedSearchRuleGroup | null {
  const tokens = tokenizeSavedSearchExpression(rawQuery)
  if (!tokens.some((token) => token === '(' || token === ')' || token === 'AND' || token === 'OR')) {
    return null
  }
  const parser = createSavedSearchExpressionParser(tokens)
  const expression = parser.parseExpression()
  if (!expression || parser.hasRemainingTokens()) return null
  return expression
}

function tokenizeSavedSearchExpression(rawQuery: string): string[] {
  const tokens: string[] = []
  let current = ''
  let inQuotes = false
  let quote: string | null = null

  const pushCurrent = () => {
    const normalized = current.trim()
    if (normalized) tokens.push(normalized)
    current = ''
  }

  for (const symbol of rawQuery) {
    if ((symbol === '"' || symbol === "'") && (!inQuotes || quote === symbol)) {
      inQuotes = !inQuotes
      quote = inQuotes ? symbol : null
      current += symbol
      continue
    }

    if (!inQuotes && (symbol === '(' || symbol === ')')) {
      pushCurrent()
      tokens.push(symbol)
      continue
    }

    if (!inQuotes && symbol.trim() === '') {
      pushCurrent()
      continue
    }

    current += symbol
  }

  pushCurrent()
  return tokens
}

function createSavedSearchExpressionParser(tokens: string[]) {
  let index = 0

  function peek(): string | null {
    return tokens[index] ?? null
  }

  function parsePrimary(): SavedSearchRuleGroup | SavedSearchRuleCondition | null {
    const token = peek()
    if (!token) return null
    if (token === '(') {
      index += 1
      const expression = parseExpression()
      if (peek() !== ')') return null
      index += 1
      return expression
    }
    if (token === ')' || token === 'AND' || token === 'OR') return null
    index += 1
    const parsedRule = parseSavedSearchRuleToken(token)
    if (parsedRule) return createSavedSearchRuleCondition(parsedRule)
    const value = parseSavedSearchRuleValue(token)
    if (!value) return null
    return createSavedSearchRuleCondition({ field: 'all', operator: ':', value })
  }

  function parseAndExpression(): SavedSearchRuleGroup | SavedSearchRuleCondition | null {
    const children: SavedSearchRuleNode[] = []
    const first = parsePrimary()
    if (!first) return null
    children.push(first)
    while (peek() === 'AND') {
      index += 1
      const next = parsePrimary()
      if (!next) return null
      children.push(next)
    }
    return children.length === 1 ? children[0] : createSavedSearchRuleGroup('all', children)
  }

  function parseExpression(): SavedSearchRuleGroup | SavedSearchRuleCondition | null {
    const children: SavedSearchRuleNode[] = []
    const first = parseAndExpression()
    if (!first) return null
    children.push(first)
    while (peek() === 'OR') {
      index += 1
      const next = parseAndExpression()
      if (!next) return null
      children.push(next)
    }
    return children.length === 1 ? children[0] : createSavedSearchRuleGroup('any', children)
  }

  return {
    parseExpression(): SavedSearchRuleGroup | null {
      const expression = parseExpression()
      if (!expression) return null
      return expression.kind === 'group'
        ? expression
        : createSavedSearchRuleGroup('all', [expression])
    },
    hasRemainingTokens(): boolean {
      return index < tokens.length
    }
  }
}

function formatSavedSearchRuleGroupExpression(group: SavedSearchRuleGroup): string {
  const separator = group.matchMode === 'any' ? ' OR ' : ' AND '
  return `(${group.children.map(formatSavedSearchRuleNodeExpression).join(separator)})`
}

function formatSavedSearchRuleNodeExpression(node: SavedSearchRuleNode): string {
  if (node.kind === 'group') return formatSavedSearchRuleGroupExpression(node)
  return `${node.field}${node.operator}${formatSavedSearchRuleValue(node.value)}`
}

function savedSearchBuilderId(prefix: 'rule' | 'group'): string {
  return `${prefix}-${Math.random().toString(36).slice(2, 10)}`
}
