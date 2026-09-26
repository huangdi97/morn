import '@testing-library/jest-dom';
import { vi } from 'vitest';

// jsdom has no fetch base URL, so relative paths (e.g. /api/status) throw
// "Invalid URL" inside undici. Component tests only need the network boundary
// to respond; stub fetch at that boundary instead of hitting a real server.
globalThis.fetch = vi.fn(async () =>
  new Response(JSON.stringify({ status: 'ok' }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  })
) as unknown as typeof fetch;
