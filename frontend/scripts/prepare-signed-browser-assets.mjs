import { cpSync, existsSync, lstatSync, mkdirSync, readFileSync, statSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'

const MAX_TOTAL_BYTES = 4 * 1024 * 1024
const MAX_JS_BYTES = 1536 * 1024
const MAX_CSS_BYTES = 768 * 1024

const [distArgument, outputArgument] = process.argv.slice(2)
if (!distArgument || !outputArgument || process.argv.length !== 4) {
	throw new Error('usage: prepare-signed-browser-assets.mjs <vite-dist> <output-directory>')
}

const dist = resolve(distArgument)
const output = resolve(outputArgument)
const entry = resolve(dist, 'index.html')
if (!existsSync(entry) || lstatSync(entry).isSymbolicLink()) throw new Error('Vite entry document is unavailable')
if (existsSync(output)) throw new Error('signed browser asset output must not exist')

const pending = ['index.html']
const included = new Set()
let total = 0
while (pending.length > 0) {
	const relativePath = pending.pop()
	if (!relativePath || included.has(relativePath)) continue
	if (!validRelativePath(relativePath)) throw new Error('browser asset reference is invalid')
	const source = resolve(dist, relativePath)
	if (!source.startsWith(`${dist}/`) && source !== entry) throw new Error('browser asset escapes Vite dist')
	if (!existsSync(source) || lstatSync(source).isSymbolicLink() || !lstatSync(source).isFile()) {
		throw new Error(`browser asset is unavailable: ${relativePath}`)
	}
	const size = statSync(source).size
	if (size === 0 || (relativePath.endsWith('.js') && size > MAX_JS_BYTES) || (relativePath.endsWith('.css') && size > MAX_CSS_BYTES)) {
		throw new Error(`browser asset size is invalid: ${relativePath}`)
	}
	total += size
	if (total > MAX_TOTAL_BYTES) throw new Error('signed browser asset inventory exceeds 4 MiB')
	included.add(relativePath)
	const contents = readFileSync(source, 'utf8')
	assertLocalStaticReferences(contents, relativePath)
	for (const dependency of references(contents)) pending.push(dependency)
}

mkdirSync(output, { recursive: true, mode: 0o700 })
for (const relativePath of [...included].sort()) {
	const target = resolve(output, relativePath)
	mkdirSync(dirname(target), { recursive: true, mode: 0o700 })
	cpSync(resolve(dist, relativePath), target, { errorOnExist: true, force: false, verbatimSymlinks: true })
}
writeFileSync(resolve(output, 'inventory.json'), `${JSON.stringify({ files: [...included].sort(), totalBytes: total })}\n`, { encoding: 'utf8', mode: 0o600, flag: 'wx' })
process.stdout.write(`signed browser inventory: ${included.size} files, ${total} bytes\n`)

function references(source) {
	const matches = source.matchAll(/(?:\/|["'`])((?:assets\/)[A-Za-z0-9._/-]+)(?=["'`)\s,])/g)
	return [...matches].map((match) => match[1])
}

function validRelativePath(value) {
	return value === 'index.html' || (value.startsWith('assets/') && value.split('/').every((part) => part && part !== '.' && part !== '..'))
}

function assertLocalStaticReferences(source, relativePath) {
	if (/api\.iconify\.design|api\.unisvg\.com|api\.simplesvg\.com|code\.iconify\.design/.test(source)) {
		throw new Error(`browser asset uses a remote icon source: ${relativePath}`)
	}
	if (/<(?:script|link|img|source|audio|video)\b[^>]*(?:src|href)=["']https?:\/\//i.test(source)) {
		throw new Error(`browser asset uses a remote static source: ${relativePath}`)
	}
}
