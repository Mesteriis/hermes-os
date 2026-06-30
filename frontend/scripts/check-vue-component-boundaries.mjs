import { readdir, readFile, stat } from 'node:fs/promises'
import path from 'node:path'

const sourceRoot = path.resolve('src')
const maxReport = 80

const forbiddenImportPaths = [
  '/api/',
  '/connect/',
  '/platform/api',
  '/mappers/',
  '/policies/',
  '/model/',
  '/services/',
  '/helpers/',
  '/lib/',
]

const allowedImportPrefixes = new Set([
  'vue',
  '@iconify/vue',
  'reka-ui',
  'motion-v',
  'vee-validate',
  '@vee-validate/',
  '@tanstack/vue-table',
  '@tanstack/vue-virtual',
  '@tiptap/vue-3',
  '@vue-flow/core',
  '@vueuse/core',
  'date-fns',
  'pinia',
  '@/shared',
])

function isVueFile(filePath) {
  return path.extname(filePath) === '.vue'
}

function toPosix(filePath) {
  return filePath.split(path.sep).join('/')
}

function getScriptSection(source) {
  const start = source.indexOf('<script')
  if (start === -1) return ''
  const openEnd = source.indexOf('>', start)
  if (openEnd === -1) return ''
  const end = source.indexOf('</script>', openEnd)
  if (end === -1) return ''
  return source.slice(openEnd + 1, end)
}

function parseImports(scriptSource) {
  const importStatements = []
  const importRegex = /(^|\n)\s*import[\s\S]*?from\s+["']([^"']+)["']/g
  let match
  while ((match = importRegex.exec(scriptSource)) !== null) {
    const statement = match[0]
    const source = match[2]
    const line = scriptSource.slice(0, match.index).split('\n').length
    const isTypeOnly = /(^|\n)\s*import\s+type\s/.test(statement)
    importStatements.push({ source, line, isTypeOnly, statement })
  }
  return importStatements
}

function splitPath(filePath) {
  const normal = toPosix(filePath)
  const match = normal.match(/\/(domains|integrations)\/([^/]+)\//)
  if (!match) return null
  return { kind: match[1], name: match[2], raw: normal }
}

function resolveImport(filePath, fileDir, source) {
  if (source.startsWith('@/')) {
    return path.resolve(sourceRoot, source.replace(/^@\//, ''))
  }
  if (source.startsWith('./') || source.startsWith('../')) {
    return path.resolve(fileDir, source)
  }
  return null
}

function isPackageImport(source) {
  return !source.startsWith('.') && !source.startsWith('/') && !source.startsWith('@/') && !source.startsWith('..')
}

function isRuntimeTypesImport(source, isTypeOnly) {
  return !isTypeOnly && source.includes('/types/')
}

function isAllowedPackageImport(source) {
  if (allowedImportPrefixes.has(source)) return true
  if (source.startsWith('@iconify/vue')) return true
  if (source.startsWith('reka-ui')) return true
  if (source.startsWith('vue')) return true
  if (source.startsWith('motion-v')) return true
  if (source.startsWith('vee-validate')) return true
  if (source.startsWith('@vee-validate/')) return true
  if (source.startsWith('@tanstack/vue-table')) return true
  if (source.startsWith('@tanstack/vue-virtual')) return true
  if (source.startsWith('@tiptap/vue-3')) return true
  if (source.startsWith('@vue-flow/core')) return true
  if (source.startsWith('@vueuse/core')) return true
  if (source.startsWith('date-fns')) return true
  if (source.startsWith('pinia')) return true
  return false
}

function hasForbiddenImportPath(source) {
  return forbiddenImportPaths.some((entry) => source.includes(entry))
}

function isForbiddenCrossBoundary(current, imported) {
  if (!current || !imported) return false
  if (current.kind === imported.kind && current.name === imported.name) return false
  if (imported.kind === 'domains' && current.kind === 'integrations') return true
  if (imported.kind === 'integrations' && current.kind === 'domains') return true
  if (current.kind === 'domains' && imported.kind === 'domains') return true
  return true
}

function hasForbiddenCallPatterns(scriptSource) {
  const violations = []
  const checks = [
    { regex: /\bfetch\(/g, message: 'forbidden fetch() in component' },
    { regex: /\bclient\.(get|post|put|delete|patch)\s*\(/g, message: 'direct client.http call in component' },
    { regex: /\bqueryClient\.invalidateQueries\s*\(/g, message: 'queryClient.invalidateQueries in component' },
    { regex: /\bqueryClient\.setQueryData\s*\(/g, message: 'queryClient.setQueryData in component' },
    { regex: /\blocalStorage\./g, message: 'localStorage usage in component' },
    { regex: /\bsessionStorage\./g, message: 'sessionStorage usage in component' },
    { regex: /\bz\.object\b/g, message: 'zod schema declaration in component' },
    { regex: /(?:\.|\b)(map|filter|reduce|sort)\s*\([^)]*\)\s*\.(map|filter|reduce|sort)\s*\(/g, message: 'collection transform chain in component business flow' },
    { regex: /\bnew\s+(Set|Map)\s*\([^)\n]*\.map\s*\(/g, message: 'Set/Map construction from collection transform in component business flow' },
  ]
  for (const { regex, message } of checks) {
    const line = firstMatchLine(scriptSource, regex)
    if (line > 0) {
      violations.push({ line, message })
    }
  }
  return violations
}

function hasForbiddenDomainCollectionTransforms(scriptSource, currentBoundary) {
  if (!currentBoundary) return []
  if (currentBoundary.kind !== 'domains' && currentBoundary.kind !== 'integrations') return []

  const violations = []
  const checks = [
    { regex: /\bflatMap\s*\(/, message: 'flatMap() collection transform in domain/integration component' },
    { regex: /\.map\s*\(/, message: 'map() collection transform in domain/integration component' },
    { regex: /\.filter\s*\(/, message: 'filter() collection transform in domain/integration component' },
    { regex: /\.reduce\s*\(/, message: 'reduce() collection transform in domain/integration component' },
    { regex: /\.sort\s*\(/, message: 'sort() collection transform in domain/integration component' },
    { regex: /\bnew\s+(Set|Map)\s*\(/, message: 'Set/Map construction in domain/integration component' },
  ]

  for (const { regex, message } of checks) {
    const line = firstMatchLine(scriptSource, regex)
    if (line > 0) {
      violations.push({ line, message })
    }
  }

  return violations
}

function hasDirectTanstackHookCall(scriptSource) {
  const importFromTanstack = /from\s+["']@tanstack\/vue-query["']/.test(scriptSource)
  if (!importFromTanstack) return null
  const line = firstMatchLine(scriptSource, /\b(useQuery|useMutation)\s*\(/)
  if (line > 0) return line
  return null
}

function firstMatchLine(source, pattern) {
  const lines = source.split('\n')
  for (let i = 0; i < lines.length; i += 1) {
    pattern.lastIndex = 0
    if (pattern.test(lines[i])) return i + 1
  }
  return -1
}

function collectBoundaryViolations(filePath, source) {
  const violations = []
  const currentBoundary = splitPath(toPosix(filePath))
  const scriptSource = getScriptSection(source)

  const imports = parseImports(scriptSource)
  const fileDir = path.dirname(filePath)

  for (const imported of imports) {
    const sourcePath = imported.source

    if (isPackageImport(sourcePath) && !isAllowedPackageImport(sourcePath)) {
      violations.push({
        file: filePath,
        line: imported.line,
        message: `forbidden package import "${sourcePath}"`,
      })
      continue
    }

    if (!imported.isTypeOnly && hasForbiddenImportPath(sourcePath)) {
      violations.push({
        file: filePath,
        line: imported.line,
        message: `forbidden business import path "${sourcePath}"`,
      })
      continue
    }

    if (isRuntimeTypesImport(sourcePath, imported.isTypeOnly)) {
      violations.push({
        file: filePath,
        line: imported.line,
        message: `runtime import from types path "${sourcePath}"`,
      })
      continue
    }

    const resolved = resolveImport(filePath, fileDir, sourcePath)
    if (resolved) {
      const targetBoundary = splitPath(toPosix(resolved))
      if (!isAllowedShared(resolved) && isForbiddenCrossBoundary(currentBoundary, targetBoundary)) {
        violations.push({
          file: filePath,
          line: imported.line,
          message: `cross-domain/surface import "${sourcePath}"`,
        })
      }
      continue
    }

    if (hasForbiddenImportPath(sourcePath) && !sourcePath.startsWith('@')) {
      violations.push({
        file: filePath,
        line: imported.line,
        message: `forbidden import path "${sourcePath}"`,
      })
    }
  }

  for (const call of hasForbiddenCallPatterns(scriptSource)) {
    violations.push({
      file: filePath,
      line: call.line,
      message: call.message,
    })
  }

  for (const call of hasForbiddenDomainCollectionTransforms(scriptSource, currentBoundary)) {
    violations.push({
      file: filePath,
      line: call.line,
      message: call.message,
    })
  }

  const directHookLine = hasDirectTanstackHookCall(scriptSource)
  if (directHookLine > 0) {
    violations.push({
      file: filePath,
      line: directHookLine,
      message: 'direct @tanstack/vue-query usage (useQuery/useMutation) in component',
    })
  }

  return violations
}

function isAllowedShared(resolvedPath) {
  const normal = toPosix(resolvedPath)
  return normal.includes('/shared/') || normal.includes('/platform/')
}

async function collectVueFiles(root) {
  if (!(await exists(root))) return []
  const entries = await readdir(root, { withFileTypes: true })
  const files = []

  for (const entry of entries) {
    const filePath = path.join(root, entry.name)
    if (entry.isDirectory()) {
      files.push(...(await collectVueFiles(filePath)))
      continue
    }
    if (entry.isFile() && isVueFile(filePath)) {
      files.push(filePath)
    }
  }
  return files
}

async function exists(filePath) {
  try {
    await stat(filePath)
    return true
  } catch (error) {
    if (error && typeof error === 'object' && 'code' in error && error.code === 'ENOENT') return false
    throw error
  }
}

async function main() {
  const files = await collectVueFiles(sourceRoot)
  const allViolations = []

  for (const filePath of files) {
    const source = await readFile(filePath, 'utf8')
    const fileViolations = collectBoundaryViolations(filePath, source)
    allViolations.push(...fileViolations)
  }

  if (allViolations.length > 0) {
    console.error('Vue component boundaries check failed.')
    const ordered = allViolations
      .sort((a, b) => a.file.localeCompare(b.file) || a.line - b.line)
      .slice(0, maxReport)
    for (const violation of ordered) {
      console.error(`${toPosix(violation.file)}:${violation.line}: ${violation.message}`)
    }
    if (allViolations.length > maxReport) {
      console.error(`... and ${allViolations.length - maxReport} additional violation(s).`)
    }
    process.exit(1)
  }

  console.log('Vue component boundaries check passed: no violations found.')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  await main()
}
