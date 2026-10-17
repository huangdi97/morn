import assert from 'node:assert/strict'
import test from 'node:test'

import { MornCompositionHost } from '../src/index.mjs'

test('provider replacement is explicit and node-local', async () => {
  const host = new MornCompositionHost()
  await host.mount('harness', { id: 'dsh' })
  assert.equal(host.get('harness').id, 'dsh')

  await host.mount('harness', { id: 'pi' })
  assert.equal(host.get('harness').id, 'pi')

  // The composition host intentionally has no canonical Work/Outcome state.
  assert.equal(host.get('work'), undefined)
  assert.equal(host.get('outcome'), undefined)

  await host.dispose()
})

test('unmount removes a service without inventing business failure', async () => {
  const host = new MornCompositionHost()
  await host.mount('solver', { id: 'ortools' })
  assert.equal(await host.unmount('solver'), true)
  assert.equal(host.get('solver'), undefined)
  assert.equal(await host.unmount('solver'), false)
  await host.dispose()
})
