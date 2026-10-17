# ADR-034 — Product surfaces use declarative UI slots before arbitrary client plugins

Status: Accepted for v11.5 convergence.

## Context

Composable systems eventually need composable user interfaces. DeepSeek
Harness demonstrates a strong client-side pattern with a second Cordis plugin
tree and UI slots. Morn also already exposes Domain SDK `ui_extension`
declarations, but the old reference UI mostly gated hardcoded BioLab panels by
domain name.

Loading arbitrary third-party React/JavaScript into Workbench would widen the
trust boundary, introduce client dependency/version coupling and let a domain
pack create product state outside canonical Morn APIs.

## Decision

1. v11.5 defines typed, declarative `UiExtensionSpec` slots for Workbench,
   Studio, Console and Hub.
2. Extensions declare surface, slot, title, safe renderer, optional in-app data
   endpoint, actions, optional Profile requirement and priority.
3. Data/actions must target in-app `/api/` routes; arbitrary remote JavaScript
   URLs are rejected.
4. UI extensions are projections only. They do not own Work/business truth.
5. Real authority remains enforced server-side; an action button's
   `authority_semantic` is metadata, not an authorization grant.
6. The reference Workbench consumes the UI extension registry generically.
7. A future trusted client-plugin runtime may use Cordis or another client
   composition system behind the same slot contract, but v11.5 does not need
   arbitrary client code to achieve domain composability.

## Consequences

- Domain packs can contribute surfaces without hardcoding every domain in Core.
- Zero-domain builds return no domain extensions.
- UI supply-chain/security remains narrower than an unrestricted web-plugin
  model.
- The DSH client-Cordis design is reused as an architectural lesson (slot
  composition) without copying its entire client runtime into Morn.
