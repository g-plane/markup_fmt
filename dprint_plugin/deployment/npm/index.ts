import * as path from 'node:path'
import * as URL from 'node:url'

export function getPath() {
    return path.join(path.dirname(URL.fileURLToPath(import.meta.url)), 'plugin.wasm')
}

export type * from './config.js'
