<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Infra change discipline

Infrastructure changes affecting cost, capacity, or availability carry a measured before figure, a projected after figure, and a stated rollback path before execution; redeploy mechanisms are verified against the intended configuration source first, because platforms can replay stale stored config; services are stopped only on operator instruction or evidenced intent, never on inference.

## Narrative

Three July 2026 incidents define this precept. A platform redeploy replayed
the platform's stored configuration and silently scaled CI runners from one
replica back to four — a cost regression the operator caught by screenshot.
A crashed product service was stopped on the inference that it was being
retired, when the operator wanted it improved as a shipping product. And a
claimed cost reduction, once burn rates were actually measured, turned out
to be a 38% increase.

The language here is deliberately class-of-tool, per precept-authoring: any
deploy platform can hold stored state that diverges from the checked-in
configuration, so the discipline binds to the shape (verify which config a
redeploy replays) rather than to a vendor's API name. The before/after
figure requirement is what turns "we cut costs" from a narrative into a
measurement; the rollback statement is what makes the change reversible in
practice rather than in principle.

## Requires

- MUST state the measured current cost or capacity, the projected post-change figure, and the exact rollback command before executing a cost- or capacity-affecting change
- MUST verify which configuration source a redeploy will apply — checked-in file versus platform-stored state — before triggering it
- MUST re-measure after the change and report before, after, and delta rather than asserting the intended effect
- MUST verify dependents before stopping, scaling, or removing any service

## Forbids

- MUST NOT stop or remove a running service on inference about intent — a crash is a finding to report, not a decommission signal
- MUST NOT trigger a redeploy-latest operation without confirming the configuration it will replay matches the intended state
- MUST NOT claim a cost change without a measured figure from the provider's billing surface

## References

- precept:safety
- precept:constitution
- precept:precept_specification
