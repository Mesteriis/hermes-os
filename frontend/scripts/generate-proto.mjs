import { mkdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const __dirname = dirname(fileURLToPath(import.meta.url))
const frontendRoot = resolve(__dirname, '..')
const repoRoot = resolve(frontendRoot, '..')
const protoRoot = join(repoRoot, 'contracts', 'proto')
const gatewayProtoRoot = join(repoRoot, 'backend', 'src', 'api', 'gateway', 'contracts', 'proto')
const outputDir = join(frontendRoot, 'src', 'gen')
const pluginPath = join(frontendRoot, 'node_modules', '.bin', 'protoc-gen-es')
const protoFiles = [
  join(protoRoot, 'hermes', 'common', 'v1', 'common.proto'),
  join(protoRoot, 'hermes', 'events', 'v1', 'event_envelope.proto'),
  join(protoRoot, 'hermes', 'signal_hub', 'v1', 'signal_hub.proto'),
  join(protoRoot, 'hermes', 'communications', 'v1', 'communications.proto'),
  join(gatewayProtoRoot, 'hermes', 'gateway', 'v1', 'client_realtime.proto'),
  join(gatewayProtoRoot, 'hermes', 'gateway', 'v1', 'browser_session.proto'),
  join(gatewayProtoRoot, 'hermes', 'gateway', 'v1', 'client_bootstrap.proto')
]

mkdirSync(outputDir, { recursive: true })

const result = spawnSync(
  'protoc',
  [
    `-I${protoRoot}`,
    `-I${gatewayProtoRoot}`,
    `--plugin=protoc-gen-es=${pluginPath}`,
    `--es_out=${outputDir}`,
    '--es_opt',
    'target=ts',
    ...protoFiles
  ],
  {
    cwd: frontendRoot,
    stdio: 'inherit'
  }
)

if (result.status !== 0) {
  process.exit(result.status ?? 1)
}
