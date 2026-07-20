import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join, resolve } from 'node:path'

const root = resolve(process.argv[2] ?? 'dist')
const forbiddenRuntimeSources = [
  'api.iconify.design',
  'api.unisvg.com',
  'api.simplesvg.com',
  'code.iconify.design',
]

for (const file of files(root)) {
  const source = readFileSync(file, 'utf8')
  if (forbiddenRuntimeSources.some((host) => source.includes(host))) {
    throw new Error(`client bundle contains an external static asset source: ${file}`)
  }
  if (/<(?:script|link|img|source|audio|video)\b[^>]*(?:src|href)=["']https?:\/\//i.test(source)) {
    throw new Error(`client bundle contains an external static asset reference: ${file}`)
  }
}

process.stdout.write('local client asset policy passed\n')

function files(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) return files(path)
    return statSync(path).isFile() ? [path] : []
  })
}
