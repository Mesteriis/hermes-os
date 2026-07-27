import { configDefaults, defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import { lstatSync, readFileSync } from 'node:fs'
import { isAbsolute, resolve } from 'node:path'

const DEVELOPMENT_GATEWAY_TARGET = 'http://127.0.0.1:9444'
const DEVELOPMENT_BROWSER_ORIGIN = 'http://127.0.0.1:5173'
const DEVELOPMENT_PROXY_PROOF_HEADER = 'x-hermes-development-proxy-proof'
const FORWARDED_HEADERS = [
	'forwarded',
	'x-forwarded-for',
	'x-forwarded-host',
	'x-forwarded-proto',
	'cf-connecting-ip',
	'true-client-ip',
	'x-real-ip',
]

function loadDevelopmentGatewayProxy() {
	const target = process.env.HERMES_DEV_GATEWAY_TARGET
	const proofFile = process.env.HERMES_DEV_GATEWAY_PROOF_FILE
	if (target === undefined && proofFile === undefined) {
		return undefined
	}
	if (target !== DEVELOPMENT_GATEWAY_TARGET || proofFile === undefined || !isAbsolute(proofFile)) {
		throw new Error('Hermes development Gateway proxy configuration is invalid')
	}
	const metadata = lstatSync(proofFile)
	if (!metadata.isFile() || metadata.isSymbolicLink() || (metadata.mode & 0o077) !== 0) {
		throw new Error('Hermes development Gateway proxy proof file is invalid')
	}
	const proof = readFileSync(proofFile, 'utf8')
	if (!/^[0-9a-fA-F]{64}$/.test(proof)) {
		throw new Error('Hermes development Gateway proxy proof is invalid')
	}
	return { proof, target }
}

const developmentGateway = loadDevelopmentGatewayProxy()

function developmentGatewayProxy(gateway: { proof: string; target: string }) {
	return {
		target: gateway.target,
		changeOrigin: false,
		xfwd: false,
		configure(proxy: {
			on(event: 'proxyReq', listener: (request: {
				removeHeader(name: string): void
				setHeader(name: string, value: string): void
			}) => void): void
		}) {
			proxy.on('proxyReq', (request) => {
				for (const header of FORWARDED_HEADERS) {
					request.removeHeader(header)
				}
				request.removeHeader(DEVELOPMENT_PROXY_PROOF_HEADER)
				request.setHeader('host', '127.0.0.1:5173')
				request.setHeader('origin', DEVELOPMENT_BROWSER_ORIGIN)
				request.setHeader(DEVELOPMENT_PROXY_PROOF_HEADER, gateway.proof)
			})
		},
	}
}

export default defineConfig({
	plugins: [vue()],
	resolve: {
		alias: {
			'@': resolve(__dirname, 'src')
		}
	},
	server: {
		host: '127.0.0.1',
		port: 5173,
		strictPort: true,
		proxy: developmentGateway === undefined
			? undefined
			: {
				'^/hermes\\.': developmentGatewayProxy(developmentGateway),
				'/api/realtime/v1/events': developmentGatewayProxy(developmentGateway),
				'/healthz': developmentGatewayProxy(developmentGateway),
				'/readyz': developmentGatewayProxy(developmentGateway),
			},
	},
	build: {
		outDir: 'dist',
		chunkSizeWarningLimit: 1536,
		rollupOptions: {
			onwarn(warning, defaultHandler) {
				const isIgnoredAnnotationWarning = warning.message.includes('INVALID_ANNOTATION') &&
					warning.message.includes('@vueuse/core');
				if (!isIgnoredAnnotationWarning) {
					defaultHandler(warning);
				}
			}
		}
	},
	test: {
		exclude: [...configDefaults.exclude, 'tests/visual/**']
	}
})
