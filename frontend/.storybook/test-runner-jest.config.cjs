const path = require('node:path')
const { getJestConfig } = require('@storybook/test-runner')

const testRunnerConfig = getJestConfig()
const frontendRoot = path.resolve(__dirname, '..')

module.exports = {
	...testRunnerConfig,
	rootDir: frontendRoot
}
