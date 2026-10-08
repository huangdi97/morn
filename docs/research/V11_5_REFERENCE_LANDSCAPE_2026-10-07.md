# Morn v11.5 External Reference Landscape — 2026-10-07

Status: research/design input. This document records external product/runtime
facts and Morn design consequences. It is not customer evidence and does not
upgrade any production claim.

Research refresh: 2026-10-08.

## Executive conclusion

The current market is converging on highly composable agent runtimes:

- DeepSeek Harness makes its agent stack plugin-composable on Cordis.
- Pi/PI-Desktop separates a lightweight agent engine from a privileged host,
  persistent workspace and permission layer.
- AgentScope 2.0 now absorbs the former AgentScope Runtime capabilities
  (sandboxing, Agent-as-a-Service, observability) into the main framework and
  adds current pipeline/SOP/A2A patterns; the standalone Runtime repository is
  being archived.
- DeepSeek DSec treats execution as an elastic fleet of heterogeneous sandbox
  backends rather than one universal sandbox.
- Paper2Agent turns scientific artifacts into MCP/skill-style executable
  interfaces.
- Alibaba Model Studio exposes versioned reusable Agent configurations and
  runtime sessions, and can create ephemeral agents inside workflows.

Morn should reuse these mechanisms but should not collapse itself into any one
agent framework. The differentiating responsibility is durable, governed Work
from intent through independently accepted real-world outcome.

## 1. DeepSeek Harness / Cordis

Sources:

- https://github.com/deepseek-ai/deepseek-harness
- https://github.com/deepseek-ai/deepseek-harness/blob/master/vendor/cordis/README.md
- https://github.com/deepseek-ai/deepseek-harness/blob/master/AGENTS.md

Observed design:

- DSH describes itself as “Everything is a Plugin”.
- Cordis supplies Context/Service dependency injection, scoped plugin lifetime,
  reversible registration effects and configuration-driven loading.
- DSH model adapters, tools, session log and agent loop are plugin-composed.
- DSH is still explicitly a developer preview with compatibility-breaking
  changes expected.
- Session/runtime persistence evolves by explicit generations/versions rather
  than silently rewriting committed historical generations.

Morn decision:

**Reuse Cordis as the reference node-local composition runtime. Do not make
Cordis the Work database or the Morn protocol.**

Cordis answers “what software services are live in this process/context?” Morn
must still answer “what Work exists, what authority applied, what actually
happened in the external world, and was the outcome accepted?”

A Cordis plugin reload is therefore an E0 software lifecycle event. It cannot
serve as business rollback for an E2/E3 external effect.

Fresh implementation signal (2026-10-08): a current DSH discussion reports an
internal peer-dependency mismatch where one DSH package expects Cordis 4.0.2
while the HMR plugin expects ~4.0.4. This is not treated as a Morn bug, but it is
additional evidence for keeping Morn's exact-pinned Cordis host and DSH process
boundary separate instead of forcing both systems into one shared dependency
tree. Source:
https://github.com/deepseek-ai/deepseek-harness/discussions/7610

## 2. DeepSeek Harness as a harness provider

Source:

- https://github.com/deepseek-ai/deepseek-harness

Morn should integrate DSH through its public process/SDK boundary rather than
forking or reimplementing the harness. DSH may internally provide richer model,
tool, browser, plan, subagent and session behavior than Morn's
`HarnessProvider` contract.

The Morn contract intentionally stays smaller:

```text
mount scope
start session
send work input
inspect
stream normalized evidence
optional lifecycle controls when actually supported
terminate / receive execution evidence
```

A DSH session never becomes canonical Work. DSH events are executor evidence,
not AcceptedOutcome.

The current public DSH SDK surface is pre-stable, so provider version/digest is
pinned in ExecutionBinding and upgrades create new bindings rather than mutating
an active Attempt.

## 3. Pi / PI-Desktop

Sources:

- https://github.com/vastsa/PI-Desktop
- https://github.com/vastsa/PI-Desktop/blob/main/docs/spec/01-product/00-overview.md

PI-Desktop explicitly separates:

```text
React renderer
 -> Electron orchestration
 -> Rust Host Core: permissions/files/SQLite/secrets
 -> pi Agent Sidecar: model + agent loop
```

Its product also distinguishes Agent / Plan / Goal. In Goal mode the user locks
an objective and acceptance criteria while the agent chooses the path.

Morn decision:

- Pi can be a lightweight HarnessProvider.
- The Host-Core-versus-agent-loop separation is a strong reference for keeping
  secrets, permissions and privileged execution outside model memory.
- Goal + acceptance is closer to Morn than chat/session-centric design, but
  Morn generalizes the executor beyond coding agents and keeps Work/Outcome/
  Acceptance durable across provider/session replacement.
- PI-Desktop plugins remain user-trusted code and are not equivalent to an
  operating-system security sandbox; Morn therefore requires independent
  ExecutionEnvironment guarantees.

## 4. DeepSeek Elastic Compute (DSec)

Source:

- https://arxiv.org/abs/2609.22978

DSec exposes heterogeneous execution backends — FnCall, container, microVM and
full VM — behind a unified elastic infrastructure and preserves state across
long agentic rollouts.

Morn decision:

Do not treat “sandbox” as one implementation or one ordinal label.

Morn separates:

```text
ExecutionClass
+ ExecutionGuarantee[]
+ EnvironmentProvider
+ runtime/workload identity
+ credential handles
```

A remote executor is not automatically stronger than a microVM. Filesystem
confinement is not evidence of network egress policy. Site admission must prove
the exact guarantee vector required by the Profile.

DSec-like infrastructure is therefore a future execution provider, not a Morn
kernel dependency.

## 5. AgentScope 2.0 / former AgentScope Runtime

Sources:

- https://github.com/agentscope-ai/agentscope
- https://github.com/agentscope-ai/agentscope-runtime

Freshness note (2026-10-08): the standalone AgentScope Runtime repository now
carries an archive/migration notice. Its sandboxing, Agent-as-a-Service and
observability capabilities have been folded into AgentScope 2.0. AgentScope 2.0
also exposes Pipeline/TeamPipeline, experimental SOP execution, A2A support and
model routing.

Morn decision:

Treat AgentScope **2.0**, not the archived standalone Runtime repository, as the
active provider/framework reference. It can satisfy agent execution, pipeline,
A2A and deployment concerns behind Morn provider contracts, while Morn
continues to own Work, Authority, binding identity, external-effect truth,
Outcome and Acceptance.

This update further supports a provider-family abstraction: framework projects
will merge, split and deprecate runtime packages over time, so Morn must bind to
versioned provider contracts rather than making any vendor repository part of
its semantic constitution.

## 6. Paper2Agent

Source:

- https://github.com/jmiao24/Paper2Agent

Paper2Agent coordinates specialist agents to transform a paper plus associated
code into MCP servers and skills, with verification and delivery steps.

Morn decision:

Generalize the idea into `Artifact2Capability`, but keep a stronger supply
chain:

```text
Artifact
 -> Declared candidate
 -> Observed/evaluated
 -> Qualified
 -> content-addressed Release
 -> Profile conformance
 -> SiteAdmission
```

Paper text alone does not become an executable capability. Executable
paper-derived candidates require reviewed extraction and real code bindings.
Generation never implies production admission.

## 7. Alibaba Model Studio / Managed Agents

Sources:

- https://help.aliyun.com/zh/model-studio/agent-api/
- https://help.aliyun.com/zh/model-studio/new-single-agent-application
- https://help.aliyun.com/zh/model-studio/managed-agents-quick-start
- https://help.aliyun.com/zh/model-studio/workflow/agent-create-node

Important observed semantics:

- An Alibaba Model Studio Agent is a reusable configuration of model, system
  prompt, toolkits and skills.
- Updating the configuration increments a version; session creation pins the
  current Agent version so existing sessions do not silently change.
- Agent 2.0 unifies knowledge and MCP as tools and performs autonomous planning.
- Workflow can dynamically create an Agent that exists only for the workflow
  execution instead of creating a permanent standalone Agent application.

Morn decision:

This strongly supports separating **reusable template/configuration** from
**runtime execution instance**. Morn generalizes that pattern as:

```text
SolutionPackage (reusable blueprint)
 -> WorkResource (runtime business instance)
 -> ExecutionBinding (exact provider/capability/runtime pin)
```

Morn deliberately does not make “Agent” the top-level reusable product object,
because many Work instances need zero agents or a mixed Workcell.

## 8. Why Morn is not built directly on DSH, Pi or AgentScope

Any one of those projects can be an excellent execution substrate, but adopting
one as the semantic root would couple Morn's business identity to that
framework's session/task/plugin lifecycle.

That creates five problems:

1. provider/session replacement can accidentally rewrite business identity;
2. non-agent executors become second-class;
3. external-effect ambiguity is forced into an agent task status;
4. capability qualification/site admission becomes runtime-specific;
5. historical outcome/acceptance semantics inherit pre-stable runtime APIs.

Morn instead keeps the smallest non-bypassable semantic/control layer and makes
the richer runtimes replaceable providers.

## 9. What is actually stable

The stable part is **not a specific Rust crate, Cordis version, harness, model,
workflow engine, UI, database or Factory implementation**.

The stable interoperability contract is the meaning and separation of:

```text
Work
Capability
Authority
ExecutionBinding
Attempt
Receipt / Reconciliation
ObservedOutcome
Acceptance
Provenance / History
Profile guarantees
```

Implementations may be replaced. Published contract versions and historical
interpretation may evolve only explicitly and non-destructively.

## 10. Current Morn product equation

```text
Morn
= small protocol + durable Work control plane
+ replaceable composition runtime
+ replaceable provider fabric
+ governed capability supply chain
+ guarantee profiles
+ Workbench / Studio / Console / Hub
```

Product experiences such as Factory, Research, BioLab, Personal, “Digital
Employee” or “Agent Factory” are compositions/profiles/packages over this base,
not new kernels.

The user's simple creation path is therefore:

```text
Describe goal / import artifact
 -> Studio compiles
 -> review + approve SolutionPackage
 -> choose Profile + optional site
 -> instantiate Work
 -> resolve minimum sufficient Workcell
 -> satisfy authority/trust gates
 -> execute
 -> observe
 -> accept outcome
```

This preserves the simplicity of an “agent factory” experience while avoiding
an Agent-first architecture.


## 2026 control-plane convergence: ServiceNow / UiPath / Siemens

The 2026 market makes one positioning mistake especially dangerous: Morn must
not claim that "an AI/agent control plane" by itself is unique.

ServiceNow AI Control Tower now explicitly discovers, observes, governs,
secures and measures AI systems, agents, workflows and MCP servers across
third-party environments. Its surrounding platform connects governance to
workflow/action infrastructure and value measurement.

UiPath similarly positions one governed orchestration plane across agents,
robots, APIs and humans, with long-running cross-system work and inherited
identity/audit/compliance controls.

Siemens' industrial direction combines industrial agents with orchestration and
a governed execution layer between AI and real machinery. Its public
architecture emphasizes deterministic/auditable coordination, human authority
and validation before commands reach industrial control.

### Consequence for Morn

Therefore Morn's differentiator cannot be:

- "we have agents";
- "we support MCP/A2A";
- "we have a control tower";
- "we can mix humans and agents";
- "we have industrial AI orchestration";
- "we govern models/tools".

Those are increasingly table stakes.

The sharper Morn thesis is **Work-first outcome control**:

```text
intent/situation
 -> versioned Work
 -> minimum-sufficient capability composition
 -> explicit Authority
 -> pinned ExecutionBinding
 -> Attempt / external-effect truth
 -> Receipt / Reconciliation
 -> source-grounded Outcome
 -> independent Acceptance
 -> value evidence
```

This moves the center of gravity away from "inventory and govern AI assets" and
away from "run agent workflows" toward preserving business meaning and
consequence across replaceable executors.

### Factory consequence

For Factory, Morn should not market itself as a replacement MES, PLC layer or
generic "industrial agent platform". The first wedge is a governed brownfield
Work control layer above existing systems of record and below human operational
accountability.

Agents, solvers, deterministic programs and humans may participate, but the
factory Work survives any executor replacement. Production-write profiles
remain a separate future trust level, not an automatic next step.


## 11. A2A v1.x and the new CLI

Sources:

- https://a2a-protocol.org/latest/
- https://github.com/a2aproject/A2A/blob/main/docs/specification.md
- https://a2a-protocol.org/latest/blog/2026/10/01/introducing-a2a-cli/

Current public A2A documentation identifies the 1.x line as the stable released
protocol. The project now also provides an official CLI that lets ordinary
shell/CI/coding-harness processes discover an Agent Card, submit work and
follow an A2A Task without themselves becoming agent frameworks.

Morn consequence:

A2A is increasingly suitable as a **remote executor boundary**, including for
non-agent Morn controllers. That reduces the need for a Morn-specific remote
agent RPC. It does not change the semantic boundary:

```text
A2A Task / Message / Artifact
    = remote executor protocol state/evidence

Morn Work / Binding / Outcome / Acceptance
    = business control-plane truth
```

The concrete A2A protocol version is pinned in the provider/interface binding,
not embedded into Morn Work identity.

## 12. MCP 2026 authorization hardening

Sources:

- https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/docs/specification/2026-07-28/basic/authorization/index.mdx
- https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/docs/specification/2026-07-28/basic/authorization/authorization-server-discovery.mdx

The 2026 HTTP authorization specification requires protected-resource metadata
and resource/audience-bound access-token use when authorization is enabled.
Tokens must be intended for the concrete protected resource and must not be
blindly passed through to another MCP server.

Morn consequence:

- MCP tool metadata remains capability-interface metadata, not Authority.
- HTTP MCP credential requests carry an explicit resource/audience reference.
- secret material remains behind CredentialProvider opaque handles;
- the Action/Connector enforcement boundary resolves credentials after
  Authority/Profile checks;
- STDIO/local MCP remains a distinct trust shape and should obtain credentials
  from its execution environment rather than copying them into prompts.

This is why Morn keeps identity/credential/enforcement semantics outside the
Harness even when the Harness has first-class MCP support.

## 13. DSH/Cordis packaging volatility validates the isolation boundary

Sources:

- https://github.com/deepseek-ai/deepseek-harness/discussions/7603
- https://github.com/deepseek-ai/deepseek-harness/discussions/7610
- https://www.npmjs.com/package/@deepseek-ai/cordis

September 2026 community reports document dependency-resolution conflicts
between DSH release-candidate packages and different Cordis package versions.
At the same time, `@deepseek-ai/cordis` continues to publish quickly.

This is not evidence that Cordis is unsuitable. It is evidence that Morn should
avoid making DSH's internal package tree its architectural ABI.

Morn therefore keeps these decisions:

1. the reference Cordis host pins an exact Cordis version;
2. only the dedicated composition-runtime adapter imports Cordis;
3. DSH runs out-of-process through a public provider boundary;
4. an active ExecutionBinding records the provider/runtime version/digest;
5. a DSH/Cordis upgrade creates a new future binding or explicit migration,
   never a silent reinterpretation of an active Attempt.

The reference host currently pins `@deepseek-ai/cordis@4.0.4`. This is an
engineering pin, not a semantic claim that 4.0.4 is permanently canonical.


## 2026-10-08 Pi RPC verification

Current Pi documentation was rechecked before implementing the transport seam.
The maintained public RPC mode is `pi --mode rpc --no-session`, with strict
JSONL framing over stdin/stdout. Commands support correlation ids; relevant
boundary operations include `prompt`, `get_state` and `abort`.
A successful prompt response describes only accepted/queued/handled disposition.
`agent_end` can be followed by retry/compaction/follow-up activity;
`agent_settled` is the stronger session-level idle signal.

Architecture consequence: Morn may use Pi as a real out-of-process provider
without making Pi session state canonical Work state. The reference Rust client
is in `crates/morn-harness/src/pi_rpc.rs`. Actual binary/model/credential smoke
remains environment-dependent.
