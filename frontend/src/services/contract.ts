import {
  BASE_FEE,
  nativeToScVal,
  Operation,
  scValToNative,
  TransactionBuilder,
  xdr,
} from "@stellar/stellar-sdk";
import { Server } from "@stellar/stellar-sdk/rpc";
import { getNetwork, isConnected, requestAccess, signTransaction } from "@stellar/freighter-api";
import {
  APP_NETWORK,
  CONTRACT_ID,
  DEPLOYMENT_PENDING,
  NETWORK_LABEL,
  NETWORK_PASSPHRASE,
  READ_ACCOUNT,
  RPC_URL,
  STELLAR_EXPERT_CONTRACT_URL,
  STELLAR_EXPERT_TX_URL,
} from "../contractConfig";

export const ROLE_MANUFACTURER = 1;
export const ROLE_INSPECTOR = 2;
export const ROLE_VERIFIER = 4;
export const ROLE_RECYCLER = 8;
export const ROLE_RECALL_AUTHORITY = 16;

export type PassportStatus =
  | "active"
  | "verified"
  | "under_review"
  | "recalled"
  | "recycled";

export type BatteryPassport = {
  serial: string;
  chemistry: string;
  capacityWh: number;
  carbonKg: number;
  batchId: string;
  manufacturer: string;
  owner: string;
  status: PassportStatus;
  inspections: number;
  healthScore: number;
  verifiedBy: string | null;
  recycler: string | null;
  createdAt: number;
  updatedAt: number;
};

export type AuditRecord = {
  serial: string;
  actor: string;
  action: string;
  note: string;
  score: number;
  timestamp: number;
};

export type RegistryStats = {
  totalPassports: number;
  circulatingPassports: number;
  recycledPassports: number;
  verifiedPassports: number;
  recalledPassports: number;
  totalInspections: number;
};

export type RecyclingApproval = {
  serial: string;
  owner: string;
  recycler: string;
  ownerApproved: boolean;
  recyclerApproved: boolean;
  executed: boolean;
  createdAt: number;
  updatedAt: number;
};

export type PlatformConfig = {
  admin: string;
};

export type WriteResult = {
  hash: string;
  explorerUrl: string;
};

type ContractArg = xdr.ScVal;

export const runtime = {
  contractId: CONTRACT_ID,
  rpcUrl: RPC_URL,
  readAccount: READ_ACCOUNT,
  deploymentPending: DEPLOYMENT_PENDING,
  network: APP_NETWORK,
  networkLabel: NETWORK_LABEL,
  contractExplorerUrl: STELLAR_EXPERT_CONTRACT_URL,
};

export function shortAddress(address: string) {
  if (!address) return "";
  if (address.length <= 16) return address;
  return `${address.slice(0, 7)}…${address.slice(-6)}`;
}

export function createExplorerTxUrl(hash: string) {
  return `${STELLAR_EXPERT_TX_URL}/${hash}`;
}

export function statusLabel(status: PassportStatus) {
  return {
    active: "Active",
    verified: "Verified",
    under_review: "Under review",
    recalled: "Recalled",
    recycled: "Recycled",
  }[status];
}

export function statusFromCode(status: number): PassportStatus {
  if (status === 2) return "verified";
  if (status === 3) return "under_review";
  if (status === 4) return "recalled";
  if (status === 5) return "recycled";
  return "active";
}

export function roleNames(mask: number): string[] {
  const roles: string[] = [];
  if (mask & ROLE_MANUFACTURER) roles.push("Manufacturer");
  if (mask & ROLE_INSPECTOR) roles.push("Inspector");
  if (mask & ROLE_VERIFIER) roles.push("Verifier");
  if (mask & ROLE_RECYCLER) roles.push("Recycler");
  if (mask & ROLE_RECALL_AUTHORITY) roles.push("Recall authority");
  return roles;
}

export function mapContractError(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  const contractErrors: Record<string, string> = {
    "#1": "The registry has already been initialized.",
    "#2": "The registry is not initialized yet.",
    "#3": "A passport with this serial number already exists.",
    "#4": "No battery passport was found for this serial number.",
    "#5": "This battery has already been recycled.",
    "#6": "This wallet is not authorized for that action.",
    "#7": "Health score must be between 0 and 100.",
    "#8": "No recycling request was found for this battery.",
    "#9": "This recycling request has already been completed.",
    "#10": "The recycler must approve this request before it can be completed.",
    "#11": "Owner and recycler must be different wallets.",
    "#12": "The selected role is not supported.",
    "#13": "Some information is missing or invalid. Please review the form.",
    "#14": "This battery cannot move to that lifecycle state.",
    "#15": "At least one inspection is required before verification.",
    "#16": "This battery needs a health score of at least 60 before verification.",
    "#17": "This passport is already verified.",
    "#18": "This battery has already been recalled.",
    "#19": "The selected recycler is not an authorized recycler.",
  };
  for (const [code, friendly] of Object.entries(contractErrors)) {
    if (message.includes(`Contract, ${code}`)) return friendly;
  }

  const lower = message.toLowerCase();
  if (lower.includes("contract_id_not_configured")) {
    return `The ${NETWORK_LABEL} contract is not configured yet.`;
  }
  if (lower.includes("read_account_not_configured")) {
    return "Public verification is not configured yet. Add a public read account after deployment.";
  }
  if (lower.includes("freighter_wrong_network")) {
    return `Switch Freighter to ${NETWORK_LABEL}, then try again.`;
  }
  if (lower.includes("freighter")) {
    return `Freighter is unavailable. Install or unlock the wallet and use ${NETWORK_LABEL}.`;
  }
  if (lower.includes("rejected")) return "The transaction was rejected in your wallet.";
  if (lower.includes("insufficient")) return `Your wallet does not have enough XLM for this ${NETWORK_LABEL} transaction.`;
  if (lower.includes("not found")) return "We could not find that battery passport.";
  return message;
}

export async function connectFreighterWallet(): Promise<string> {
  const installed = await isConnected();
  if (installed.error) throw new Error(installed.error.message);
  if (!installed.isConnected) throw new Error("Freighter wallet is not available.");

  const access = await requestAccess();
  if (access.error) throw new Error(access.error.message);
  if (!access.address) throw new Error("Freighter did not return a wallet address.");

  const network = await getNetwork();
  if (network.error) throw new Error(network.error.message);
  if (network.network !== APP_NETWORK || network.networkPassphrase !== NETWORK_PASSPHRASE) {
    throw new Error("FREIGHTER_WRONG_NETWORK");
  }

  return access.address;
}

export async function getPassport(serial: string, source?: string): Promise<BatteryPassport> {
  const native = await simulateRead("get_passport", [stringArg(serial)], source);
  return normalizePassport(native);
}

export async function getStats(source?: string): Promise<RegistryStats> {
  const native = await simulateRead("get_stats", [], source);
  const value = asObject(native);
  return {
    totalPassports: toNumber(value.total_passports),
    circulatingPassports: toNumber(value.circulating_passports),
    recycledPassports: toNumber(value.recycled_passports),
    verifiedPassports: toNumber(value.verified_passports),
    recalledPassports: toNumber(value.recalled_passports),
    totalInspections: toNumber(value.total_inspections),
  };
}

export async function getAuditRecords(serial: string, source?: string): Promise<AuditRecord[]> {
  const native = await simulateRead(
    "get_recent_audits",
    [stringArg(serial), nativeToScVal(20, { type: "u32" })],
    source,
  );
  if (!Array.isArray(native)) throw new Error("Unexpected audit history response.");

  return native.map((item) => {
    const value = asObject(item);
    return {
      serial: String(value.serial ?? serial),
      actor: String(value.actor ?? ""),
      action: String(value.action ?? ""),
      note: String(value.note ?? ""),
      score: toNumber(value.score),
      timestamp: toNumber(value.timestamp),
    };
  }).reverse();
}

export async function getRecyclingApproval(
  serial: string,
  source?: string,
): Promise<RecyclingApproval | null> {
  try {
    const native = await simulateRead("get_recycling_approval", [stringArg(serial)], source);
    const value = asObject(native);
    return {
      serial: String(value.serial ?? serial),
      owner: String(value.owner ?? ""),
      recycler: String(value.recycler ?? ""),
      ownerApproved: Boolean(value.owner_approved),
      recyclerApproved: Boolean(value.recycler_approved),
      executed: Boolean(value.executed),
      createdAt: toNumber(value.created_at),
      updatedAt: toNumber(value.updated_at),
    };
  } catch (error) {
    if (String(error).includes("Contract, #8")) return null;
    throw error;
  }
}

export async function getRoles(account: string, source?: string): Promise<number> {
  return toNumber(await simulateRead("get_roles", [addressArg(account)], source));
}

export async function getConfig(source?: string): Promise<PlatformConfig> {
  const native = await simulateRead("get_config", [], source);
  const value = asObject(native);
  return { admin: String(value.admin ?? "") };
}

export const actions = {
  createPassport(wallet: string, input: {
    serial: string; chemistry: string; capacityWh: number; carbonKg: number; batchId: string;
  }) {
    return submitWrite("create_passport", wallet, [
      addressArg(wallet), stringArg(input.serial), stringArg(input.chemistry),
      u32Arg(input.capacityWh), u32Arg(input.carbonKg), stringArg(input.batchId),
    ]);
  },
  transferOwner(wallet: string, serial: string, newOwner: string) {
    return submitWrite("transfer_owner", wallet, [addressArg(wallet), stringArg(serial), addressArg(newOwner)]);
  },
  addInspection(wallet: string, serial: string, score: number, note: string) {
    return submitWrite("add_inspection", wallet, [addressArg(wallet), stringArg(serial), u32Arg(score), stringArg(note)]);
  },
  verifyPassport(wallet: string, serial: string) {
    return submitWrite("verify_passport", wallet, [addressArg(wallet), stringArg(serial)]);
  },
  flagRecall(wallet: string, serial: string, reason: string) {
    return submitWrite("flag_recall", wallet, [addressArg(wallet), stringArg(serial), stringArg(reason)]);
  },
  requestRecycling(wallet: string, serial: string, recycler: string) {
    return submitWrite("request_recycling", wallet, [addressArg(wallet), stringArg(serial), addressArg(recycler)]);
  },
  approveRecycling(wallet: string, serial: string) {
    return submitWrite("approve_recycling", wallet, [addressArg(wallet), stringArg(serial)]);
  },
  executeRecycling(wallet: string, serial: string) {
    return submitWrite("execute_recycling", wallet, [addressArg(wallet), stringArg(serial)]);
  },
  grantRole(wallet: string, account: string, role: number) {
    return submitWrite("grant_role", wallet, [addressArg(wallet), addressArg(account), u32Arg(role)]);
  },
  revokeRole(wallet: string, account: string, role: number) {
    return submitWrite("revoke_role", wallet, [addressArg(wallet), addressArg(account), u32Arg(role)]);
  },
};

async function simulateRead(functionName: string, args: ContractArg[], source?: string): Promise<unknown> {
  ensureConfigured();
  const readSource = source || READ_ACCOUNT;
  if (!readSource) throw new Error("READ_ACCOUNT_NOT_CONFIGURED");

  const server = new Server(RPC_URL);
  const sourceAccount = await server.getAccount(readSource);
  const transaction = new TransactionBuilder(sourceAccount, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(Operation.invokeContractFunction({ contract: CONTRACT_ID, function: functionName, args }))
    .setTimeout(30)
    .build();

  const simulation = await server.simulateTransaction(transaction);

if ("error" in simulation) {
  throw new Error(simulation.error);
}

if (!simulation.result?.retval) {
  throw new Error("The contract did not return a value.");
}

return scValToNative(simulation.result.retval);
}

async function submitWrite(functionName: string, wallet: string, args: ContractArg[]): Promise<WriteResult> {
  ensureConfigured();
  if (!wallet) throw new Error("Wallet not connected.");
  const walletNetwork = await getNetwork();
  if (walletNetwork.error) throw new Error(walletNetwork.error.message);
  if (walletNetwork.network !== APP_NETWORK || walletNetwork.networkPassphrase !== NETWORK_PASSPHRASE) {
    throw new Error("FREIGHTER_WRONG_NETWORK");
  }

  const server = new Server(RPC_URL);
  const sourceAccount = await server.getAccount(wallet);
  const transaction = new TransactionBuilder(sourceAccount, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(Operation.invokeContractFunction({ contract: CONTRACT_ID, function: functionName, args }))
    .setTimeout(60)
    .build();

  const prepared = await server.prepareTransaction(transaction);
  const signedResponse = await signTransaction(prepared.toXDR(), {
  networkPassphrase: NETWORK_PASSPHRASE,
  address: wallet,
  });
  if (signedResponse.error) throw new Error(signedResponse.error.message);
  if (!signedResponse.signedTxXdr) throw new Error("Freighter did not return a signed transaction.");
  if (signedResponse.signerAddress && signedResponse.signerAddress !== wallet) {
    throw new Error("Freighter signed with a different account than the connected wallet.");
  }
  const signedTransaction = TransactionBuilder.fromXDR(
    signedResponse.signedTxXdr,
    NETWORK_PASSPHRASE,
  );
  const submitted = await server.sendTransaction(signedTransaction);
  if (!submitted.hash || (String(submitted.status) !== "PENDING" && String(submitted.status) !== "DUPLICATE")) {
    throw new Error(`Transaction submission failed: ${String(submitted.status)}`);
  }

  const final = await server.pollTransaction(submitted.hash, { attempts: 20 });
  if (String(final.status) !== "SUCCESS") {
    throw new Error(`Transaction failed: ${String(final.status)}`);
  }

  return { hash: submitted.hash, explorerUrl: createExplorerTxUrl(submitted.hash) };
}

function normalizePassport(native: unknown): BatteryPassport {
  const value = asObject(native);
  return {
    serial: String(value.serial ?? ""),
    chemistry: String(value.chemistry ?? ""),
    capacityWh: toNumber(value.capacity_wh),
    carbonKg: toNumber(value.carbon_kg),
    batchId: String(value.batch_id ?? ""),
    manufacturer: String(value.manufacturer ?? ""),
    owner: String(value.owner ?? ""),
    status: statusFromCode(toNumber(value.status)),
    inspections: toNumber(value.inspections),
    healthScore: toNumber(value.health_score),
    verifiedBy: value.verified_by ? String(value.verified_by) : null,
    recycler: value.recycler ? String(value.recycler) : null,
    createdAt: toNumber(value.created_at),
    updatedAt: toNumber(value.updated_at),
  };
}

function ensureConfigured() {
  if (DEPLOYMENT_PENDING) throw new Error("CONTRACT_ID_NOT_CONFIGURED");
}
function addressArg(value: string): ContractArg {
  if (!/^G[A-Z2-7]{55}$/.test(value.trim())) throw new Error("Enter a valid Stellar wallet address.");
  return nativeToScVal(value.trim(), { type: "address" });
}
function stringArg(value: string): ContractArg {
  const clean = value.trim();
  if (!clean) throw new Error("A required field is empty.");
  return nativeToScVal(clean, { type: "string" });
}
function u32Arg(value: number): ContractArg {
  if (!Number.isInteger(value) || value < 0) throw new Error("Use a valid positive whole number.");
  return nativeToScVal(value, { type: "u32" });
}
function asObject(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Unexpected contract response.");
  }
  return value as Record<string, unknown>;
}
function toNumber(value: unknown): number {
  if (typeof value === "bigint") return Number(value);
  return Number(value ?? 0);
}
