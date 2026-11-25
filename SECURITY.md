# Security and privacy

ShadowMind is planned for authorized learning, collaboration and productivity. There is no released supported version yet.

## Mandatory boundaries

Input access requires explicit authorization and applicable participant consent. Respect OS permissions and protected content; fail safely when input is unavailable. Do not implement capture-protection circumvention, assessment monitoring evasion, hidden capture-excluded overlays or unauthorized assessment assistance. No automatic capture at startup or after unexpected termination. No raw screen/audio retention by default.

Treat external inputs and AI output as untrusted. Generated content must not execute commands or autonomously modify the system. Validate IPC, minimize data, restrict provider destinations, store credentials in a supported secure OS store, and require separate approval for external processing and nonzero spend. CI uses mocks.

## Reporting

Report suspected vulnerabilities privately to the repository owner via an agreed private channel; do not publish secrets, personal content or exploitable reproduction details in public issues. If no private channel is configured, ask the owner for one first. Include affected version, impact and sanitized reproduction evidence. The owner triages under the specification's S0–S3 policy. S0/S1 defects block release. Revoke exposed credentials and stop affected processing where necessary.

No runtime security claims are made; controls must be independently tested before release.
