import { FormEvent, useEffect, useMemo, useState } from "react";
import {
  actions,
  AuditRecord,
  BatteryPassport,
  connectFreighterWallet,
  getAuditRecords,
  getConfig,
  getPassport,
  getRecyclingApproval,
  getRoles,
  getStats,
  mapContractError,
  RecyclingApproval,
  RegistryStats,
  ROLE_INSPECTOR,
  ROLE_MANUFACTURER,
  ROLE_RECALL_AUTHORITY,
  ROLE_RECYCLER,
  ROLE_VERIFIER,
  roleNames,
  runtime,
  shortAddress,
  statusLabel,
  WriteResult,
} from "./services/contract";
import "./App.css";

type Notice = { type: "success" | "error" | "info"; message: string; link?: string } | null;
type WorkMode = "inspect" | "transfer" | "recycle" | "access";

const roleOptions = [
  [ROLE_MANUFACTURER, "Manufacturer"],
  [ROLE_INSPECTOR, "Inspector"],
  [ROLE_VERIFIER, "Verifier"],
  [ROLE_RECYCLER, "Recycler"],
  [ROLE_RECALL_AUTHORITY, "Recall authority"],
] as const;

function App() {
  const [wallet, setWallet] = useState("");
  const [roles, setRoles] = useState(0);
  const [admin, setAdmin] = useState("");
  const [query, setQuery] = useState("");
  const [passport, setPassport] = useState<BatteryPassport | null>(null);
  const [audits, setAudits] = useState<AuditRecord[]>([]);
  const [approval, setApproval] = useState<RecyclingApproval | null>(null);
  const [stats, setStats] = useState<RegistryStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [notice, setNotice] = useState<Notice>(null);
  const [mode, setMode] = useState<WorkMode>("inspect");

  const [createForm, setCreateForm] = useState({ serial: "", chemistry: "LFP", capacityWh: "", carbonKg: "", batchId: "" });
  const [inspectionForm, setInspectionForm] = useState({ score: "", note: "" });
  const [newOwner, setNewOwner] = useState("");
  const [recycler, setRecycler] = useState("");
  const [recallReason, setRecallReason] = useState("");
  const [accessForm, setAccessForm] = useState({ account: "", role: ROLE_MANUFACTURER });

  const isAdmin = Boolean(wallet && admin && wallet === admin);
  const isOwner = Boolean(wallet && passport && wallet === passport.owner);
  const isLifecycleClosed = passport?.status === "recycled";
  const canInspect = Boolean(passport && !isLifecycleClosed && (roles & ROLE_INSPECTOR) > 0);
  const canTransfer = Boolean(passport && isOwner && !isLifecycleClosed);
  const canRecycle = Boolean(passport && !isLifecycleClosed && ((roles & ROLE_RECYCLER) > 0 || isOwner));
  const canVerify = Boolean(
    passport &&
    (roles & ROLE_VERIFIER) > 0 &&
    passport.status === "active" &&
    passport.inspections > 0 &&
    passport.healthScore >= 60,
  );
  const canRecall = Boolean(
    passport &&
    (roles & ROLE_RECALL_AUTHORITY) > 0 &&
    passport.status !== "recalled" &&
    passport.status !== "recycled",
  );
  const myRoles = useMemo(() => roleNames(roles), [roles]);
  const availableModes = useMemo<WorkMode[]>(() => {
    const next: WorkMode[] = [];
    if (canInspect) next.push("inspect");
    if (canTransfer) next.push("transfer");
    if (canRecycle) next.push("recycle");
    if (isAdmin) next.push("access");
    return next;
  }, [canInspect, canTransfer, canRecycle, isAdmin]);

  useEffect(() => {
    if (runtime.deploymentPending || !runtime.readAccount) return;
    void Promise.all([getStats(), getConfig()])
      .then(([nextStats, config]) => {
        setStats(nextStats);
        setAdmin(config.admin);
      })
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    if (!passport || availableModes.length === 0) return;
    if (!availableModes.includes(mode)) setMode(availableModes[0]);
  }, [passport, availableModes, mode]);

  async function connectWallet() {
    setNotice(null);
    try {
      const address = await connectFreighterWallet();
      setWallet(address);
      const [nextRoles, config] = await Promise.all([
        getRoles(address, address),
        getConfig(address),
      ]);
      setRoles(nextRoles);
      setAdmin(config.admin);
      setNotice({ type: "success", message: "Wallet connected. Your available actions are ready." });
    } catch (error) {
      setNotice({ type: "error", message: mapContractError(error) });
    }
  }

  async function searchBattery(serial = query, quiet = false) {
    const clean = serial.trim();
    if (!clean) {
      setNotice({ type: "error", message: "Enter a battery serial number first." });
      return;
    }
    if (!quiet) {
      setLoading(true);
      setNotice(null);
    }
    try {
      const source = wallet || undefined;
      const nextPassport = await getPassport(clean, source);
      const [nextAudits, nextApproval] = await Promise.all([
        getAuditRecords(clean, source),
        getRecyclingApproval(clean, source),
      ]);
      setPassport(nextPassport);
      setAudits(nextAudits);
      setApproval(nextApproval);
      setQuery(clean);
    } catch (error) {
      if (!quiet) {
        setPassport(null);
        setAudits([]);
        setApproval(null);
        setNotice({ type: "error", message: mapContractError(error) });
      }
    } finally {
      if (!quiet) setLoading(false);
    }
  }

  async function refreshSelected() {
    if (passport?.serial) await searchBattery(passport.serial, true);
    if (wallet) setRoles(await getRoles(wallet, wallet));
    try { setStats(await getStats(wallet || undefined)); } catch { /* optional */ }
  }

  async function runWrite(
    task: () => Promise<WriteResult>,
    success: string,
    refreshSerial?: string,
    afterSuccess?: () => void,
  ) {
    setLoading(true);
    setNotice({ type: "info", message: "Check your wallet and confirm the transaction." });
    try {
      const result = await task();
      if (refreshSerial) await searchBattery(refreshSerial, true);
      else await refreshSelected();
      afterSuccess?.();
      setNotice({ type: "success", message: success, link: result.explorerUrl });
    } catch (error) {
      setNotice({ type: "error", message: mapContractError(error) });
    } finally {
      setLoading(false);
    }
  }

  function submitInspection() {
    if (!inspectionForm.score.trim()) {
      setNotice({ type: "error", message: "Enter a health score between 0 and 100." });
      return;
    }
    void runWrite(
      () => actions.addInspection(wallet, passport!.serial, Number(inspectionForm.score), inspectionForm.note),
      "Inspection added to the battery lifecycle.",
      undefined,
      () => setInspectionForm({ score: "", note: "" }),
    );
  }

  function submitCreatePassport() {
    const serial = createForm.serial.trim();
    if (!serial || !createForm.chemistry.trim() || !createForm.capacityWh.trim() || !createForm.batchId.trim()) {
      setNotice({ type: "error", message: "Complete the required battery origin information first." });
      return;
    }
    void runWrite(
      () => actions.createPassport(wallet, {
        serial,
        chemistry: createForm.chemistry,
        capacityWh: Number(createForm.capacityWh),
        carbonKg: Number(createForm.carbonKg || 0),
        batchId: createForm.batchId,
      }),
      "Battery passport created.",
      serial,
      () => setCreateForm({ serial: "", chemistry: "LFP", capacityWh: "", carbonKg: "", batchId: "" }),
    );
  }

  function onSearch(event: FormEvent) {
    event.preventDefault();
    void searchBattery();
  }

  const timeline = passport ? buildTimeline(passport, audits) : [];

  return (
    <main className="app-shell">
      <header className="site-header">
        <a className="brand" href="#top" aria-label="Battery Passport home">
          <span className="brand-mark"><span /></span>
          <span>Battery Passport</span>
        </a>
        <nav className="header-actions">
          <a className="text-link" href="#how-it-works">How it works</a>
          {wallet ? (
            <button className="wallet-pill" aria-label="Disconnect wallet" title="Disconnect wallet" onClick={() => { setWallet(""); setRoles(0); }}>
              <span className="live-dot" /> {shortAddress(wallet)}
            </button>
          ) : (
            <button className="secondary-button" onClick={() => void connectWallet()}>Connect wallet</button>
          )}
        </nav>
      </header>

      <section className="hero" id="top">
        <div className="hero-copy">
          <p className="kicker">Trusted battery history, from origin to recycling</p>
          <h1>Know the story behind every battery.</h1>
          <p className="hero-lede">
            Verify origin, ownership, inspections, recalls and recycling history from a single lifecycle record secured on Stellar.
          </p>
          <form className="search-bar" onSubmit={onSearch}>
            <label className="sr-only" htmlFor="serial-search">Battery serial number</label>
            <input id="serial-search" value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Enter battery serial number" />
            <button className="primary-button" disabled={loading}>{loading ? "Checking…" : "Verify battery"}</button>
          </form>
          <div className="trust-row">
            <span><i className="check-icon">✓</i> Public verification</span>
            <span><i className="check-icon">✓</i> Tamper-resistant lifecycle</span>
            <span><i className="check-icon">✓</i> Non-custodial actions</span>
          </div>
        </div>

        <div className="battery-scene" aria-hidden="true">
          <div className="orbit orbit-one" />
          <div className="orbit orbit-two" />
          <div className="battery-body">
            <div className="battery-cap" />
            <div className="battery-fill">
              <span>↻</span>
              <small>lifecycle record</small>
            </div>
            <div className="battery-grid" />
          </div>
          <div className="scene-chip chip-origin">Origin<br/><strong>Recorded</strong></div>
          <div className="scene-chip chip-inspection">Inspection<br/><strong>Verified</strong></div>
          <div className="scene-chip chip-recycle">Recycle<br/><strong>Traceable</strong></div>
        </div>
      </section>

      {notice && (
        <div className={`notice ${notice.type}`} role="status">
          <span>{notice.message}</span>
          {notice.link && <a href={notice.link} target="_blank" rel="noreferrer">View transaction ↗</a>}
        </div>
      )}

      {runtime.deploymentPending && (
        <section className="deployment-note">
          <strong>{runtime.networkLabel} contract is not connected yet.</strong>
          <span>Add the deployed contract ID and public read account for this environment.</span>
        </section>
      )}

      {passport ? (
        <section className="passport-section" id="passport">
          <div className="passport-heading">
            <div>
              <p className="kicker">Battery passport</p>
              <h2>{passport.serial}</h2>
              <p>{passport.chemistry} · {(passport.capacityWh / 1000).toLocaleString()} kWh · Batch {passport.batchId}</p>
            </div>
            <span className={`status-badge ${passport.status}`}>{statusLabel(passport.status)}</span>
          </div>

          <div className="passport-layout">
            <article className="identity-panel">
              <div className="mini-battery">
                <div className="mini-fill" style={{ height: `${Math.max(passport.healthScore, 8)}%` }} />
                <span>{passport.inspections ? `${passport.healthScore}` : "—"}</span>
              </div>
              <div className="identity-copy">
                <span className="subtle-label">Latest health score</span>
                <strong>{passport.inspections ? `${passport.healthScore}/100` : "Not inspected yet"}</strong>
                <p>{passport.inspections} inspection{passport.inspections === 1 ? "" : "s"} recorded</p>
              </div>
              <dl className="passport-facts">
                <div><dt>Reported carbon footprint</dt><dd>{passport.carbonKg} kg CO₂e</dd></div>
                <div><dt>Current owner</dt><dd title={passport.owner}>{shortAddress(passport.owner)}</dd></div>
                <div><dt>Manufacturer</dt><dd title={passport.manufacturer}>{shortAddress(passport.manufacturer)}</dd></div>
              </dl>
              <p className="record-note">Stellar preserves the recorded lifecycle. Source data is supplied by authorized participants.</p>
            </article>

            <article className="lifecycle-panel">
              <div className="section-title"><span>Lifecycle</span><small>Latest activity first</small></div>
              <div className="timeline">
                {timeline.map((item, index) => (
                  <div className="timeline-item" key={`${item.title}-${index}`}>
                    <span className={`timeline-dot ${item.tone}`} />
                    <div><strong>{item.title}</strong><p>{item.detail}</p><small>{formatDate(item.timestamp)}</small></div>
                  </div>
                ))}
              </div>
            </article>
          </div>

          {wallet && (
            <section className="workspace" id="workspace">
              <div className="workspace-intro">
                <div><p className="kicker">Your workspace</p><h2>Actions for this battery</h2></div>
                <div className="workspace-meta">
                  <div className="role-list">{isOwner && <span>Owner</span>}{myRoles.map((role) => <span key={role}>{role}</span>)}</div>
                  {(roles & ROLE_MANUFACTURER) > 0 && (
                    <button className="text-action" onClick={() => { setPassport(null); setAudits([]); setApproval(null); setQuery(""); setNotice(null); }}>+ Create new passport</button>
                  )}
                </div>
              </div>

              {availableModes.length > 0 && (
                <div className="action-rail">
                  {canInspect && <button className={mode === "inspect" ? "active" : ""} onClick={() => setMode("inspect")}>Inspect</button>}
                  {canTransfer && <button className={mode === "transfer" ? "active" : ""} onClick={() => setMode("transfer")}>Ownership</button>}
                  {canRecycle && <button className={mode === "recycle" ? "active" : ""} onClick={() => setMode("recycle")}>Recycling</button>}
                  {isAdmin && <button className={mode === "access" ? "active" : ""} onClick={() => setMode("access")}>Access</button>}
                </div>
              )}

              <div className="action-surface">
                {mode === "inspect" && canInspect && (
                  <div className="compact-form"><div><h3>Add inspection</h3><p>Record the latest health assessment for {passport.serial}.</p></div>
                    <label>Health score<input type="number" min="0" max="100" inputMode="numeric" value={inspectionForm.score} onChange={(e) => setInspectionForm({ ...inspectionForm, score: e.target.value })} placeholder="0–100" /></label>
                    <label>Inspection note<textarea maxLength={256} value={inspectionForm.note} onChange={(e) => setInspectionForm({ ...inspectionForm, note: e.target.value })} placeholder="What did you observe?" /><small className="field-hint">Public lifecycle note — do not include personal or confidential information.</small></label>
                    <button className="primary-button" disabled={loading} onClick={submitInspection}>Add inspection</button>
                  </div>
                )}

                {mode === "transfer" && canTransfer && (
                  <div className="compact-form"><div><h3>Transfer ownership</h3><p>The new owner will become the wallet authorized to transfer or recycle this battery.</p></div>
                    <label>New owner wallet<input value={newOwner} onChange={(e) => setNewOwner(e.target.value)} placeholder="G…" /></label>
                    <button className="primary-button" disabled={loading} onClick={() => void runWrite(() => actions.transferOwner(wallet, passport.serial, newOwner), "Ownership transferred successfully.", undefined, () => setNewOwner(""))}>Transfer ownership</button>
                  </div>
                )}

                {mode === "recycle" && canRecycle && (
                  <div className="recycling-flow">
                    <div className="flow-step"><span>1</span><div><strong>Owner requests</strong><p>Select an authorized recycler.</p></div></div>
                    <div className="flow-line" />
                    <div className={`flow-step ${approval?.recyclerApproved ? "complete" : ""}`}><span>2</span><div><strong>Recycler approves</strong><p>Recycler confirms the handoff.</p></div></div>
                    <div className="flow-line" />
                    <div className={`flow-step ${approval?.executed ? "complete" : ""}`}><span>3</span><div><strong>Lifecycle closes</strong><p>Owner records completed recycling.</p></div></div>
                    {isOwner && !approval && <div className="compact-form inline-form"><label>Authorized recycler<input value={recycler} onChange={(e) => setRecycler(e.target.value)} placeholder="G…" /></label><button className="secondary-button" disabled={loading} onClick={() => void runWrite(() => actions.requestRecycling(wallet, passport.serial, recycler), "Recycling request sent.", undefined, () => setRecycler(""))}>Request recycling</button></div>}
                    {approval && !approval.executed && <div className="recycling-request"><span>Assigned recycler</span><strong title={approval.recycler}>{shortAddress(approval.recycler)}</strong><small>{approval.recyclerApproved ? "Recycler approved — ready for the owner to close the lifecycle." : "Waiting for recycler approval."}</small></div>}
                    {(roles & ROLE_RECYCLER) > 0 && approval?.recycler === wallet && !approval.recyclerApproved && <button className="primary-button" disabled={loading} onClick={() => void runWrite(() => actions.approveRecycling(wallet, passport.serial), "Recycling request approved.")}>Approve recycling request</button>}
                    {isOwner && approval?.recyclerApproved && !approval.executed && <button className="primary-button" disabled={loading} onClick={() => void runWrite(() => actions.executeRecycling(wallet, passport.serial), "Battery marked as recycled.")}>Complete recycling</button>}
                  </div>
                )}

                {mode === "access" && isAdmin && (
                  <div className="compact-form"><div><h3>Manage participant access</h3><p>Only authorized organizations can create, inspect, verify, recall or recycle batteries.</p></div>
                    <label>Wallet address<input value={accessForm.account} onChange={(e) => setAccessForm({ ...accessForm, account: e.target.value })} placeholder="G…" /></label>
                    <label>Role<select value={accessForm.role} onChange={(e) => setAccessForm({ ...accessForm, role: Number(e.target.value) })}>{roleOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
                    <div className="button-row"><button className="primary-button" disabled={loading} onClick={() => void runWrite(() => actions.grantRole(wallet, accessForm.account, accessForm.role), "Access granted.")}>Grant role</button><button className="secondary-button" disabled={loading} onClick={() => void runWrite(() => actions.revokeRole(wallet, accessForm.account, accessForm.role), "Access removed.")}>Remove role</button></div>
                  </div>
                )}

                {availableModes.length === 0 && !canVerify && !canRecall && (
                  <div className="no-actions"><strong>No action is available for this battery right now.</strong><span>Your role may apply at another lifecycle stage. You can still review the public history above.</span></div>
                )}

                {canVerify && (
                  <div className="quick-action"><div><strong>Ready to verify?</strong><span>Requires a passing inspection score.</span></div><button className="secondary-button" disabled={loading} onClick={() => void runWrite(() => actions.verifyPassport(wallet, passport.serial), "Passport verified.")}>Verify passport</button></div>
                )}
                {canRecall && (
                  <div className="danger-action"><label>Recall reason<input maxLength={256} value={recallReason} onChange={(e) => setRecallReason(e.target.value)} placeholder="Reason for recall" /><small className="field-hint">This reason becomes part of the public lifecycle history.</small></label><button disabled={loading} onClick={() => void runWrite(() => actions.flagRecall(wallet, passport.serial, recallReason), "Battery recall recorded.", undefined, () => setRecallReason(""))}>Flag recall</button></div>
                )}
              </div>
            </section>
          )}
        </section>
      ) : null}

      <section className="empty-product" id="how-it-works">
        <div className="section-copy"><p className="kicker">One record, three moments that matter</p><h2>A battery lifecycle that stays understandable.</h2><p>Battery Passport keeps the technical blockchain layer out of the way and turns verified lifecycle events into a simple history anyone can inspect.</p></div>
        <div className="journey">
          <div className="journey-step"><span className="journey-icon origin-icon">01</span><h3>Created at origin</h3><p>Authorized manufacturers register core battery data and ownership.</p></div>
          <div className="journey-path"><span /></div>
          <div className="journey-step"><span className="journey-icon inspect-icon">02</span><h3>Checked over time</h3><p>Authorized inspectors and verifiers add health and trust signals.</p></div>
          <div className="journey-path"><span /></div>
          <div className="journey-step"><span className="journey-icon recycle-icon">03</span><h3>Closed responsibly</h3><p>Owner and recycler both approve the final recycling handoff.</p></div>
        </div>
      </section>

      {wallet && (roles & ROLE_MANUFACTURER) > 0 && !passport && (
        <section className="create-section">
          <div className="section-copy"><p className="kicker">Manufacturer workspace</p><h2>Create a new battery passport.</h2><p>Only essential origin information is recorded. The record is public on-chain and ownership starts with the manufacturer wallet.</p></div>
          <div className="create-form">
            <label>Serial number<input value={createForm.serial} onChange={(e) => setCreateForm({ ...createForm, serial: e.target.value })} maxLength={64} placeholder="BAT-2026-001" /></label>
            <label>Chemistry<input value={createForm.chemistry} onChange={(e) => setCreateForm({ ...createForm, chemistry: e.target.value })} maxLength={32} placeholder="LFP" /></label>
            <label>Capacity (Wh)<input type="number" min="1" inputMode="numeric" value={createForm.capacityWh} onChange={(e) => setCreateForm({ ...createForm, capacityWh: e.target.value })} placeholder="75000" /></label>
            <label>Carbon footprint (kg CO₂e)<input type="number" min="0" inputMode="numeric" value={createForm.carbonKg} onChange={(e) => setCreateForm({ ...createForm, carbonKg: e.target.value })} placeholder="420" /></label>
            <label className="wide">Batch ID<input value={createForm.batchId} onChange={(e) => setCreateForm({ ...createForm, batchId: e.target.value })} maxLength={64} placeholder="BATCH-2026-A" /></label>
            <button className="primary-button wide" disabled={loading} onClick={submitCreatePassport}>Create passport</button>
          </div>
        </section>
      )}

      <footer>
        <div><strong>Battery Passport</strong><span>Lifecycle records secured on Stellar.</span></div>
        <div className="footer-links">{stats && <span>{stats.totalPassports} passports · {stats.totalInspections} inspections</span>}<a href={runtime.contractExplorerUrl} target="_blank" rel="noreferrer">Contract ↗</a></div>
      </footer>
    </main>
  );
}

function buildTimeline(passport: BatteryPassport, audits: AuditRecord[]) {
  if (audits.length) return audits.map((audit) => ({ title: actionLabel(audit.action), detail: audit.note, timestamp: audit.timestamp, tone: audit.action.includes("recall") ? "danger" : audit.action.includes("recycl") ? "green" : "blue" }));
  return [{ title: "Passport created", detail: "Origin record created on Stellar.", timestamp: passport.createdAt, tone: "blue" }];
}

function actionLabel(action: string) {
  return ({ create_passport: "Passport created", transfer_owner: "Ownership transferred", add_inspection: "Inspection recorded", verify_passport: "Passport verified", flag_recall: "Recall issued", request_recycling: "Recycling requested", approve_recycling: "Recycler approved", execute_recycling: "Battery recycled" } as Record<string, string>)[action] || "Lifecycle updated";
}

function formatDate(timestamp: number) {
  if (!timestamp) return "";
  return new Date(timestamp * 1000).toLocaleString(undefined, { year: "numeric", month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" });
}

export default App;
