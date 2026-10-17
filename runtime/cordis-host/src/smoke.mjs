import { MornCompositionHost } from './index.mjs'

const host = new MornCompositionHost()
await host.mount('harness', { id: 'fixture-dsh', kind: 'harness' })
await host.mount('solver', { id: 'fixture-solver', kind: 'solver' })

if (host.get('harness')?.id !== 'fixture-dsh') throw new Error('harness slot did not resolve')
if (host.get('solver')?.id !== 'fixture-solver') throw new Error('solver slot did not resolve')

await host.dispose()
console.log('Morn Cordis composition host smoke OK')
