import { Context, Service } from '@deepseek-ai/cordis'

/**
 * Cordis is used only as the node-local composition runtime. Business Work,
 * bindings, receipts, outcomes and acceptance stay in the durable Morn control
 * plane and are intentionally absent from this host.
 */
export class MornCompositionHost {
  constructor() {
    this.root = new Context()
    this.slots = new Map()
  }

  async mount(slot, provider) {
    if (!slot || typeof slot !== 'string') throw new TypeError('slot must be a non-empty string')
    if (!provider || typeof provider !== 'object') throw new TypeError('provider must be an object')

    const previous = this.slots.get(slot)
    if (previous) {
      await previous.fiber.dispose()
      this.slots.delete(slot)
    }

    const value = provider
    class ProviderSlot extends Service {
      constructor(ctx) {
        super(ctx, slot)
        this.provider = value
      }
    }

    const fiber = await this.root.plugin(ProviderSlot)
    this.slots.set(slot, { fiber, provider })
    return { slot, providerId: provider.id ?? slot }
  }

  get(slot) {
    return this.root.get(slot)?.provider
  }

  async unmount(slot) {
    const mounted = this.slots.get(slot)
    if (!mounted) return false
    await mounted.fiber.dispose()
    this.slots.delete(slot)
    return true
  }

  async dispose() {
    await this.root.fiber.dispose()
    this.slots.clear()
  }
}
