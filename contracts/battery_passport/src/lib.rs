#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, Env, String, Vec,
};

pub const STATUS_ACTIVE: u32 = 1;
pub const STATUS_VERIFIED: u32 = 2;
pub const STATUS_UNDER_REVIEW: u32 = 3;
pub const STATUS_RECALLED: u32 = 4;
pub const STATUS_RECYCLED: u32 = 5;

pub const ROLE_MANUFACTURER: u32 = 1;
pub const ROLE_INSPECTOR: u32 = 2;
pub const ROLE_VERIFIER: u32 = 4;
pub const ROLE_RECYCLER: u32 = 8;
pub const ROLE_RECALL_AUTHORITY: u32 = 16;
pub const ROLE_ALL: u32 =
    ROLE_MANUFACTURER | ROLE_INSPECTOR | ROLE_VERIFIER | ROLE_RECYCLER | ROLE_RECALL_AUTHORITY;

const PASSING_HEALTH_SCORE: u32 = 60;
const MAX_SERIAL_LEN: u32 = 64;
const MAX_CHEMISTRY_LEN: u32 = 32;
const MAX_BATCH_ID_LEN: u32 = 64;
const MAX_NOTE_LEN: u32 = 256;
const TTL_RENEWAL_DIVISOR: u32 = 2;
const MAX_MAINTENANCE_BATCH: u32 = 50;
const MAX_PUBLIC_AUDIT_BATCH: u32 = 20;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformConfig {
    pub admin: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatteryPassport {
    pub serial: String,
    pub chemistry: String,
    pub capacity_wh: u32,
    pub carbon_kg: u32,
    pub batch_id: String,
    pub manufacturer: Address,
    pub owner: Address,
    pub status: u32,
    pub inspections: u32,
    pub health_score: u32,
    pub verified_by: Option<Address>,
    pub recycler: Option<Address>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditRecord {
    pub serial: String,
    pub actor: Address,
    pub action: String,
    pub note: String,
    pub score: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryStats {
    pub total_passports: u32,
    pub circulating_passports: u32,
    pub recycled_passports: u32,
    pub verified_passports: u32,
    pub recalled_passports: u32,
    pub total_inspections: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleApproval {
    pub serial: String,
    pub owner: Address,
    pub recycler: Address,
    pub owner_approved: bool,
    pub recycler_approved: bool,
    pub executed: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Config,
    Stats,
    Role(Address),
    Passport(String),
    AuditCount(String),
    Audit(String, u32),
    RecyclingApproval(String),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PassportError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    SerialAlreadyExists = 3,
    PassportNotFound = 4,
    AlreadyRecycled = 5,
    Unauthorized = 6,
    InvalidScore = 7,
    ApprovalNotFound = 8,
    ApprovalAlreadyExecuted = 9,
    ApprovalMissing = 10,
    SameCounterparty = 11,
    InvalidRole = 12,
    InvalidInput = 13,
    InvalidState = 14,
    InspectionRequired = 15,
    HealthScoreTooLow = 16,
    AlreadyVerified = 17,
    AlreadyRecalled = 18,
    RecyclerNotAuthorized = 19,
}

#[contractevent(topics = ["battery_passport", "initialized"], data_format = "map")]
pub struct RegistryInitializedEvent {
    pub admin: Address,
}

#[contractevent(topics = ["battery_passport", "admin_transferred"], data_format = "map")]
pub struct AdminTransferredEvent {
    pub previous_admin: Address,
    pub new_admin: Address,
}

#[contractevent(topics = ["battery_passport", "role_updated"], data_format = "map")]
pub struct RoleUpdatedEvent {
    pub account: Address,
    pub role: u32,
    pub granted: bool,
}

#[contractevent(topics = ["battery_passport", "created"], data_format = "map")]
pub struct PassportCreatedEvent {
    pub serial: String,
    pub manufacturer: Address,
}

#[contractevent(
    topics = ["battery_passport", "ownership_transferred"],
    data_format = "map"
)]
pub struct OwnershipTransferredEvent {
    pub serial: String,
    pub previous_owner: Address,
    pub new_owner: Address,
}

#[contractevent(topics = ["battery_passport", "inspection_added"], data_format = "map")]
pub struct InspectionAddedEvent {
    pub serial: String,
    pub inspector: Address,
    pub score: u32,
}

#[contractevent(topics = ["battery_passport", "verified"], data_format = "map")]
pub struct PassportVerifiedEvent {
    pub serial: String,
    pub verifier: Address,
}

#[contractevent(topics = ["battery_passport", "recall_flagged"], data_format = "map")]
pub struct RecallFlaggedEvent {
    pub serial: String,
    pub authority: Address,
}

#[contractevent(
    topics = ["battery_passport", "recycling_requested"],
    data_format = "map"
)]
pub struct RecyclingRequestedEvent {
    pub serial: String,
    pub owner: Address,
    pub recycler: Address,
}

#[contractevent(
    topics = ["battery_passport", "recycling_approved"],
    data_format = "map"
)]
pub struct RecyclingApprovedEvent {
    pub serial: String,
    pub recycler: Address,
}

#[contractevent(topics = ["battery_passport", "recycled"], data_format = "map")]
pub struct BatteryRecycledEvent {
    pub serial: String,
    pub owner: Address,
    pub recycler: Address,
}

#[contract]
pub struct BatteryPassportContract;

#[contractimpl]
impl BatteryPassportContract {
    pub fn __constructor(env: Env, admin: Address) {
        let config = PlatformConfig {
            admin: admin.clone(),
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage()
            .instance()
            .set(&DataKey::Stats, &empty_stats());
        bump_instance_ttl(&env);
        write_roles(&env, &admin, ROLE_ALL);

        RegistryInitializedEvent { admin }.publish(&env);
    }

    pub fn transfer_admin(
        env: Env,
        current_admin: Address,
        new_admin: Address,
    ) -> Result<PlatformConfig, PassportError> {
        current_admin.require_auth();
        let mut config = ensure_admin(&env, &current_admin)?;

        if current_admin == new_admin {
            return Err(PassportError::InvalidInput);
        }

        config.admin = new_admin.clone();
        env.storage().instance().set(&DataKey::Config, &config);
        bump_instance_ttl(&env);
        // Admin authority and operational roles are separate. Clear the outgoing
        // bootstrap roles, but do not automatically grant operational roles to the
        // new admin. The new admin can explicitly assign only what is needed.
        write_roles(&env, &current_admin, 0);

        AdminTransferredEvent {
            previous_admin: current_admin,
            new_admin,
        }
        .publish(&env);

        Ok(config)
    }

    pub fn grant_role(
        env: Env,
        admin: Address,
        account: Address,
        role: u32,
    ) -> Result<u32, PassportError> {
        admin.require_auth();
        ensure_admin(&env, &admin)?;
        ensure_valid_role(role)?;

        let roles = read_roles(&env, &account) | role;
        write_roles(&env, &account, roles);

        RoleUpdatedEvent {
            account,
            role,
            granted: true,
        }
        .publish(&env);

        Ok(roles)
    }

    pub fn revoke_role(
        env: Env,
        admin: Address,
        account: Address,
        role: u32,
    ) -> Result<u32, PassportError> {
        admin.require_auth();
        ensure_admin(&env, &admin)?;
        ensure_valid_role(role)?;

        let roles = read_roles(&env, &account) & !role;
        write_roles(&env, &account, roles);

        RoleUpdatedEvent {
            account,
            role,
            granted: false,
        }
        .publish(&env);

        Ok(roles)
    }

    pub fn get_roles(env: Env, account: Address) -> u32 {
        peek_roles(&env, &account)
    }

    pub fn has_role(env: Env, account: Address, role: u32) -> bool {
        is_valid_role(role) && (peek_roles(&env, &account) & role) == role
    }

    pub fn refresh_role_ttl(env: Env, account: Address) -> Result<bool, PassportError> {
        ensure_initialized(&env)?;
        let key = DataKey::Role(account);
        if !env.storage().persistent().has(&key) {
            return Ok(false);
        }

        bump_persistent_ttl(&env, &key);
        Ok(true)
    }

    pub fn create_passport(
        env: Env,
        manufacturer: Address,
        serial: String,
        chemistry: String,
        capacity_wh: u32,
        carbon_kg: u32,
        batch_id: String,
    ) -> Result<BatteryPassport, PassportError> {
        ensure_initialized(&env)?;
        manufacturer.require_auth();
        ensure_role(&env, &manufacturer, ROLE_MANUFACTURER)?;
        validate_passport_input(&serial, &chemistry, capacity_wh, &batch_id)?;

        let key = DataKey::Passport(serial.clone());
        if env.storage().persistent().has(&key) {
            return Err(PassportError::SerialAlreadyExists);
        }

        let timestamp = env.ledger().timestamp();
        let passport = BatteryPassport {
            serial: serial.clone(),
            chemistry,
            capacity_wh,
            carbon_kg,
            batch_id,
            manufacturer: manufacturer.clone(),
            owner: manufacturer.clone(),
            status: STATUS_ACTIVE,
            inspections: 0,
            health_score: 0,
            verified_by: None,
            recycler: None,
            created_at: timestamp,
            updated_at: timestamp,
        };

        env.storage().persistent().set(&key, &passport);
        bump_persistent_ttl(&env, &key);

        let mut stats = read_stats(&env);
        stats.total_passports += 1;
        stats.circulating_passports += 1;
        write_stats(&env, &stats);

        write_audit(
            &env,
            serial.clone(),
            manufacturer.clone(),
            String::from_str(&env, "create_passport"),
            String::from_str(&env, "Battery passport created"),
            0,
        );

        PassportCreatedEvent {
            serial: serial.clone(),
            manufacturer,
        }
        .publish(&env);

        Ok(passport)
    }

    pub fn transfer_owner(
        env: Env,
        current_owner: Address,
        serial: String,
        new_owner: Address,
    ) -> Result<BatteryPassport, PassportError> {
        ensure_initialized(&env)?;
        current_owner.require_auth();

        if current_owner == new_owner {
            return Err(PassportError::InvalidInput);
        }

        let key = DataKey::Passport(serial.clone());
        let mut passport = read_passport(&env, &serial)?;

        if passport.owner != current_owner {
            return Err(PassportError::Unauthorized);
        }
        if passport.status == STATUS_RECYCLED {
            return Err(PassportError::AlreadyRecycled);
        }

        let previous_owner = passport.owner.clone();
        passport.owner = new_owner.clone();
        passport.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&key, &passport);
        bump_persistent_ttl(&env, &key);

        let approval_key = DataKey::RecyclingApproval(serial.clone());
        if env.storage().persistent().has(&approval_key) {
            env.storage().persistent().remove(&approval_key);
        }

        write_audit(
            &env,
            serial.clone(),
            current_owner,
            String::from_str(&env, "transfer_owner"),
            String::from_str(&env, "Ownership transferred"),
            0,
        );

        OwnershipTransferredEvent {
            serial,
            previous_owner,
            new_owner,
        }
        .publish(&env);

        Ok(passport)
    }

    pub fn add_inspection(
        env: Env,
        inspector: Address,
        serial: String,
        score: u32,
        note: String,
    ) -> Result<BatteryPassport, PassportError> {
        ensure_initialized(&env)?;
        inspector.require_auth();
        ensure_role(&env, &inspector, ROLE_INSPECTOR)?;
        validate_score(score)?;
        validate_note(&note)?;

        let key = DataKey::Passport(serial.clone());
        let mut passport = read_passport(&env, &serial)?;

        if passport.status == STATUS_RECYCLED {
            return Err(PassportError::AlreadyRecycled);
        }

        let was_verified = passport.status == STATUS_VERIFIED;
        passport.inspections += 1;
        passport.health_score = score;
        passport.updated_at = env.ledger().timestamp();

        if passport.status != STATUS_RECALLED {
            passport.verified_by = None;
            passport.status = if score < PASSING_HEALTH_SCORE {
                STATUS_UNDER_REVIEW
            } else {
                STATUS_ACTIVE
            };
        }

        env.storage().persistent().set(&key, &passport);
        bump_persistent_ttl(&env, &key);

        let mut stats = read_stats(&env);
        stats.total_inspections += 1;
        if was_verified && passport.status != STATUS_VERIFIED && stats.verified_passports > 0 {
            stats.verified_passports -= 1;
        }
        write_stats(&env, &stats);

        write_audit(
            &env,
            serial.clone(),
            inspector.clone(),
            String::from_str(&env, "add_inspection"),
            note,
            score,
        );

        InspectionAddedEvent {
            serial,
            inspector,
            score,
        }
        .publish(&env);

        Ok(passport)
    }

    pub fn verify_passport(
        env: Env,
        verifier: Address,
        serial: String,
    ) -> Result<BatteryPassport, PassportError> {
        ensure_initialized(&env)?;
        verifier.require_auth();
        ensure_role(&env, &verifier, ROLE_VERIFIER)?;

        let key = DataKey::Passport(serial.clone());
        let mut passport = read_passport(&env, &serial)?;

        match passport.status {
            STATUS_RECYCLED => return Err(PassportError::AlreadyRecycled),
            STATUS_RECALLED => return Err(PassportError::InvalidState),
            STATUS_VERIFIED => return Err(PassportError::AlreadyVerified),
            _ => {}
        }

        if passport.inspections == 0 {
            return Err(PassportError::InspectionRequired);
        }
        if passport.health_score < PASSING_HEALTH_SCORE {
            return Err(PassportError::HealthScoreTooLow);
        }

        passport.status = STATUS_VERIFIED;
        passport.verified_by = Some(verifier.clone());
        passport.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&key, &passport);
        bump_persistent_ttl(&env, &key);

        let mut stats = read_stats(&env);
        stats.verified_passports += 1;
        write_stats(&env, &stats);

        write_audit(
            &env,
            serial.clone(),
            verifier.clone(),
            String::from_str(&env, "verify_passport"),
            String::from_str(&env, "Passport verified"),
            0,
        );

        PassportVerifiedEvent { serial, verifier }.publish(&env);
        Ok(passport)
    }

    pub fn flag_recall(
        env: Env,
        authority: Address,
        serial: String,
        reason: String,
    ) -> Result<BatteryPassport, PassportError> {
        ensure_initialized(&env)?;
        authority.require_auth();
        ensure_role(&env, &authority, ROLE_RECALL_AUTHORITY)?;
        validate_note(&reason)?;

        let key = DataKey::Passport(serial.clone());
        let mut passport = read_passport(&env, &serial)?;

        if passport.status == STATUS_RECYCLED {
            return Err(PassportError::AlreadyRecycled);
        }
        if passport.status == STATUS_RECALLED {
            return Err(PassportError::AlreadyRecalled);
        }

        let was_verified = passport.status == STATUS_VERIFIED;
        passport.status = STATUS_RECALLED;
        passport.verified_by = None;
        passport.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&key, &passport);
        bump_persistent_ttl(&env, &key);

        let mut stats = read_stats(&env);
        stats.recalled_passports += 1;
        if was_verified && stats.verified_passports > 0 {
            stats.verified_passports -= 1;
        }
        write_stats(&env, &stats);

        write_audit(
            &env,
            serial.clone(),
            authority.clone(),
            String::from_str(&env, "flag_recall"),
            reason,
            0,
        );

        RecallFlaggedEvent { serial, authority }.publish(&env);
        Ok(passport)
    }

    pub fn request_recycling(
        env: Env,
        owner: Address,
        serial: String,
        recycler: Address,
    ) -> Result<LifecycleApproval, PassportError> {
        ensure_initialized(&env)?;
        owner.require_auth();

        if owner == recycler {
            return Err(PassportError::SameCounterparty);
        }
        ensure_role(&env, &recycler, ROLE_RECYCLER)
            .map_err(|_| PassportError::RecyclerNotAuthorized)?;

        let passport = read_passport(&env, &serial)?;
        if passport.owner != owner {
            return Err(PassportError::Unauthorized);
        }
        if passport.status == STATUS_RECYCLED {
            return Err(PassportError::AlreadyRecycled);
        }

        let timestamp = env.ledger().timestamp();
        let approval = LifecycleApproval {
            serial: serial.clone(),
            owner: owner.clone(),
            recycler: recycler.clone(),
            owner_approved: true,
            recycler_approved: false,
            executed: false,
            created_at: timestamp,
            updated_at: timestamp,
        };

        let approval_key = DataKey::RecyclingApproval(serial.clone());
        env.storage().persistent().set(&approval_key, &approval);
        bump_persistent_ttl(&env, &approval_key);

        write_audit(
            &env,
            serial.clone(),
            owner.clone(),
            String::from_str(&env, "request_recycling"),
            String::from_str(&env, "Recycling requested by owner"),
            0,
        );

        RecyclingRequestedEvent {
            serial,
            owner,
            recycler,
        }
        .publish(&env);

        Ok(approval)
    }

    pub fn approve_recycling(
        env: Env,
        recycler: Address,
        serial: String,
    ) -> Result<LifecycleApproval, PassportError> {
        ensure_initialized(&env)?;
        recycler.require_auth();
        ensure_role(&env, &recycler, ROLE_RECYCLER)
            .map_err(|_| PassportError::RecyclerNotAuthorized)?;

        let key = DataKey::RecyclingApproval(serial.clone());
        let mut approval = read_recycling_approval(&env, &serial)?;

        if approval.executed {
            return Err(PassportError::ApprovalAlreadyExecuted);
        }
        if approval.recycler != recycler {
            return Err(PassportError::Unauthorized);
        }
        if approval.recycler_approved {
            return Err(PassportError::InvalidState);
        }

        approval.recycler_approved = true;
        approval.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&key, &approval);
        bump_persistent_ttl(&env, &key);

        write_audit(
            &env,
            serial.clone(),
            recycler.clone(),
            String::from_str(&env, "approve_recycling"),
            String::from_str(&env, "Recycler approved recycling request"),
            0,
        );

        RecyclingApprovedEvent { serial, recycler }.publish(&env);
        Ok(approval)
    }

    pub fn execute_recycling(
        env: Env,
        owner: Address,
        serial: String,
    ) -> Result<BatteryPassport, PassportError> {
        ensure_initialized(&env)?;
        owner.require_auth();

        let approval_key = DataKey::RecyclingApproval(serial.clone());
        let mut approval = read_recycling_approval(&env, &serial)?;

        if approval.executed {
            return Err(PassportError::ApprovalAlreadyExecuted);
        }
        if approval.owner != owner {
            return Err(PassportError::Unauthorized);
        }
        if !approval.owner_approved || !approval.recycler_approved {
            return Err(PassportError::ApprovalMissing);
        }
        ensure_role(&env, &approval.recycler, ROLE_RECYCLER)
            .map_err(|_| PassportError::RecyclerNotAuthorized)?;

        let recycler = approval.recycler.clone();
        let passport = recycle_passport(&env, owner.clone(), serial.clone(), recycler.clone())?;

        approval.executed = true;
        approval.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&approval_key, &approval);
        bump_persistent_ttl(&env, &approval_key);

        BatteryRecycledEvent {
            serial,
            owner,
            recycler,
        }
        .publish(&env);

        Ok(passport)
    }

    pub fn get_recycling_approval(
        env: Env,
        serial: String,
    ) -> Result<LifecycleApproval, PassportError> {
        peek_recycling_approval(&env, &serial)
    }

    pub fn get_passport(env: Env, serial: String) -> Result<BatteryPassport, PassportError> {
        peek_passport(&env, &serial)
    }

    pub fn get_stats(env: Env) -> RegistryStats {
        peek_stats(&env)
    }

    pub fn get_audit_count(env: Env, serial: String) -> u32 {
        peek_audit_count(&env, &serial)
    }

    pub fn get_audit(env: Env, serial: String, index: u32) -> Result<AuditRecord, PassportError> {
        let key = DataKey::Audit(serial, index);
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(PassportError::PassportNotFound)
    }

    pub fn get_recent_audits(
        env: Env,
        serial: String,
        limit: u32,
    ) -> Result<Vec<AuditRecord>, PassportError> {
        if limit == 0 || limit > MAX_PUBLIC_AUDIT_BATCH {
            return Err(PassportError::InvalidInput);
        }

        let _ = peek_passport(&env, &serial)?;
        let count = peek_audit_count(&env, &serial);
        let start = count.saturating_sub(limit);
        let mut records = Vec::new(&env);
        let mut index = start;

        while index < count {
            let key = DataKey::Audit(serial.clone(), index);
            if let Some(record) = env.storage().persistent().get::<DataKey, AuditRecord>(&key) {
                records.push_back(record);
            }
            index += 1;
        }

        Ok(records)
    }

    pub fn refresh_passport_ttl(
        env: Env,
        serial: String,
        audit_from: u32,
        audit_limit: u32,
    ) -> Result<u32, PassportError> {
        ensure_initialized(&env)?;
        if audit_limit > MAX_MAINTENANCE_BATCH {
            return Err(PassportError::InvalidInput);
        }

        let _ = read_passport(&env, &serial)?;
        let mut refreshed = 1;
        let audit_count = read_audit_count(&env, &serial);
        if audit_count > 0 {
            refreshed += 1;
        }

        let approval_key = DataKey::RecyclingApproval(serial.clone());
        if env.storage().persistent().has(&approval_key) {
            bump_persistent_ttl(&env, &approval_key);
            refreshed += 1;
        }

        let end = core::cmp::min(audit_count, audit_from.saturating_add(audit_limit));
        let mut index = audit_from;
        while index < end {
            let audit_key = DataKey::Audit(serial.clone(), index);
            if env.storage().persistent().has(&audit_key) {
                bump_persistent_ttl(&env, &audit_key);
                refreshed += 1;
            }
            index += 1;
        }

        Ok(refreshed)
    }

    pub fn get_config(env: Env) -> Result<PlatformConfig, PassportError> {
        peek_config(&env)
    }
}

fn ttl_window(env: &Env) -> (u32, u32) {
    let extend_to = env.storage().max_ttl();
    let threshold = extend_to / TTL_RENEWAL_DIVISOR;
    (threshold, extend_to)
}

fn bump_instance_ttl(env: &Env) {
    let (threshold, extend_to) = ttl_window(env);
    env.storage().instance().extend_ttl(threshold, extend_to);
}

fn bump_persistent_ttl(env: &Env, key: &DataKey) {
    let (threshold, extend_to) = ttl_window(env);
    env.storage()
        .persistent()
        .extend_ttl(key, threshold, extend_to);
}

fn peek_config(env: &Env) -> Result<PlatformConfig, PassportError> {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(PassportError::NotInitialized)
}

fn peek_roles(env: &Env, account: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::Role(account.clone()))
        .unwrap_or(0)
}

fn peek_stats(env: &Env) -> RegistryStats {
    env.storage()
        .instance()
        .get(&DataKey::Stats)
        .unwrap_or(empty_stats())
}

fn peek_passport(env: &Env, serial: &String) -> Result<BatteryPassport, PassportError> {
    env.storage()
        .persistent()
        .get(&DataKey::Passport(serial.clone()))
        .ok_or(PassportError::PassportNotFound)
}

fn peek_recycling_approval(env: &Env, serial: &String) -> Result<LifecycleApproval, PassportError> {
    env.storage()
        .persistent()
        .get(&DataKey::RecyclingApproval(serial.clone()))
        .ok_or(PassportError::ApprovalNotFound)
}

fn peek_audit_count(env: &Env, serial: &String) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::AuditCount(serial.clone()))
        .unwrap_or(0)
}

fn ensure_initialized(env: &Env) -> Result<PlatformConfig, PassportError> {
    let config = env
        .storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(PassportError::NotInitialized)?;
    bump_instance_ttl(env);
    Ok(config)
}

fn ensure_admin(env: &Env, admin: &Address) -> Result<PlatformConfig, PassportError> {
    let config = ensure_initialized(env)?;
    if &config.admin != admin {
        return Err(PassportError::Unauthorized);
    }
    Ok(config)
}

fn is_valid_role(role: u32) -> bool {
    role == ROLE_MANUFACTURER
        || role == ROLE_INSPECTOR
        || role == ROLE_VERIFIER
        || role == ROLE_RECYCLER
        || role == ROLE_RECALL_AUTHORITY
}

fn ensure_valid_role(role: u32) -> Result<(), PassportError> {
    if is_valid_role(role) {
        Ok(())
    } else {
        Err(PassportError::InvalidRole)
    }
}

fn read_roles(env: &Env, account: &Address) -> u32 {
    let key = DataKey::Role(account.clone());
    let roles = env.storage().persistent().get(&key);
    if roles.is_some() {
        bump_persistent_ttl(env, &key);
    }
    roles.unwrap_or(0)
}

fn write_roles(env: &Env, account: &Address, roles: u32) {
    let key = DataKey::Role(account.clone());
    if roles == 0 {
        if env.storage().persistent().has(&key) {
            env.storage().persistent().remove(&key);
        }
        return;
    }

    env.storage().persistent().set(&key, &roles);
    bump_persistent_ttl(env, &key);
}

fn ensure_role(env: &Env, account: &Address, role: u32) -> Result<(), PassportError> {
    ensure_valid_role(role)?;
    if (read_roles(env, account) & role) == role {
        Ok(())
    } else {
        Err(PassportError::Unauthorized)
    }
}

fn validate_passport_input(
    serial: &String,
    chemistry: &String,
    capacity_wh: u32,
    batch_id: &String,
) -> Result<(), PassportError> {
    if serial.len() == 0
        || serial.len() > MAX_SERIAL_LEN
        || chemistry.len() == 0
        || chemistry.len() > MAX_CHEMISTRY_LEN
        || batch_id.len() == 0
        || batch_id.len() > MAX_BATCH_ID_LEN
        || capacity_wh == 0
    {
        return Err(PassportError::InvalidInput);
    }
    Ok(())
}

fn validate_score(score: u32) -> Result<(), PassportError> {
    if score > 100 {
        Err(PassportError::InvalidScore)
    } else {
        Ok(())
    }
}

fn validate_note(note: &String) -> Result<(), PassportError> {
    if note.len() == 0 || note.len() > MAX_NOTE_LEN {
        Err(PassportError::InvalidInput)
    } else {
        Ok(())
    }
}

fn empty_stats() -> RegistryStats {
    RegistryStats {
        total_passports: 0,
        circulating_passports: 0,
        recycled_passports: 0,
        verified_passports: 0,
        recalled_passports: 0,
        total_inspections: 0,
    }
}

fn read_stats(env: &Env) -> RegistryStats {
    let stats = env
        .storage()
        .instance()
        .get(&DataKey::Stats)
        .unwrap_or(empty_stats());
    bump_instance_ttl(env);
    stats
}

fn write_stats(env: &Env, stats: &RegistryStats) {
    env.storage().instance().set(&DataKey::Stats, stats);
    bump_instance_ttl(env);
}

fn read_passport(env: &Env, serial: &String) -> Result<BatteryPassport, PassportError> {
    let key = DataKey::Passport(serial.clone());
    let passport = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(PassportError::PassportNotFound)?;
    bump_persistent_ttl(env, &key);
    Ok(passport)
}

fn read_recycling_approval(env: &Env, serial: &String) -> Result<LifecycleApproval, PassportError> {
    let key = DataKey::RecyclingApproval(serial.clone());
    let approval = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(PassportError::ApprovalNotFound)?;
    bump_persistent_ttl(env, &key);
    Ok(approval)
}

fn read_audit_count(env: &Env, serial: &String) -> u32 {
    let key = DataKey::AuditCount(serial.clone());
    let count = env.storage().persistent().get(&key);
    if count.is_some() {
        bump_persistent_ttl(env, &key);
    }
    count.unwrap_or(0)
}

fn write_audit(
    env: &Env,
    serial: String,
    actor: Address,
    action: String,
    note: String,
    score: u32,
) {
    let index = read_audit_count(env, &serial);
    let record = AuditRecord {
        serial: serial.clone(),
        actor,
        action,
        note,
        score,
        timestamp: env.ledger().timestamp(),
    };

    let audit_key = DataKey::Audit(serial.clone(), index);
    let count_key = DataKey::AuditCount(serial);
    env.storage().persistent().set(&audit_key, &record);
    env.storage().persistent().set(&count_key, &(index + 1));
    bump_persistent_ttl(env, &audit_key);
    bump_persistent_ttl(env, &count_key);
}

fn recycle_passport(
    env: &Env,
    owner: Address,
    serial: String,
    recycler: Address,
) -> Result<BatteryPassport, PassportError> {
    let key = DataKey::Passport(serial.clone());
    let mut passport = read_passport(env, &serial)?;

    if passport.owner != owner {
        return Err(PassportError::Unauthorized);
    }
    if passport.status == STATUS_RECYCLED {
        return Err(PassportError::AlreadyRecycled);
    }

    let previous_status = passport.status;
    passport.status = STATUS_RECYCLED;
    passport.recycler = Some(recycler.clone());
    passport.verified_by = None;
    passport.updated_at = env.ledger().timestamp();
    env.storage().persistent().set(&key, &passport);
    bump_persistent_ttl(env, &key);

    let mut stats = read_stats(env);
    stats.recycled_passports += 1;
    if stats.circulating_passports > 0 {
        stats.circulating_passports -= 1;
    }
    if previous_status == STATUS_VERIFIED && stats.verified_passports > 0 {
        stats.verified_passports -= 1;
    }
    if previous_status == STATUS_RECALLED && stats.recalled_passports > 0 {
        stats.recalled_passports -= 1;
    }
    write_stats(env, &stats);

    write_audit(
        env,
        serial,
        owner,
        String::from_str(env, "execute_recycling"),
        String::from_str(env, "Battery recycled after owner and recycler approval"),
        0,
    );

    Ok(passport)
}

#[cfg(test)]
mod test;
