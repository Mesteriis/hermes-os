import { configDefaults, defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
	plugins: [vue()],
	resolve: {
		alias: {
			'@': resolve(__dirname, 'src')
		}
	},
	server: {
		port: 5173
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
