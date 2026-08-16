#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

struct Fixture {
    env: Env,
    contract_id: Address,
    admin: Address,
    manufacturer: Address,
    inspector: Address,
    verifier: Address,
    recall_authority: Address,
    recycler: Address,
    serial: String,
}

impl Fixture {
    fn client(&self) -> BatteryPassportContractClient<'_> {
        BatteryPassportContractClient::new(&self.env, &self.contract_id)
    }
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(
        BatteryPassportContract,
        BatteryPassportContractArgs::__constructor(&admin),
    );
    let client = BatteryPassportContractClient::new(&env, &contract_id);

    let manufacturer = Address::generate(&env);
    let inspector = Address::generate(&env);
    let verifier = Address::generate(&env);
    let recall_authority = Address::generate(&env);
    let recycler = Address::generate(&env);
    let serial = String::from_str(&env, "BATTERY-001");

    client.grant_role(&admin, &manufacturer, &ROLE_MANUFACTURER);
    client.grant_role(&admin, &inspector, &ROLE_INSPECTOR);
    client.grant_role(&admin, &verifier, &ROLE_VERIFIER);
    client.grant_role(&admin, &recall_authority, &ROLE_RECALL_AUTHORITY);
    client.grant_role(&admin, &recycler, &ROLE_RECYCLER);

    client.create_passport(
        &manufacturer,
        &serial,
        &String::from_str(&env, "LFP"),
        &75_000,
        &420,
        &String::from_str(&env, "BATCH-2026-001"),
    );

    Fixture {
        env,
        contract_id,
        admin,
        manufacturer,
        inspector,
        verifier,
        recall_authority,
        recycler,
        serial,
    }
}

#[test]
fn constructor_bootstraps_admin_and_roles() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(
        BatteryPassportContract,
        BatteryPassportContractArgs::__constructor(&admin),
    );
    let client = BatteryPassportContractClient::new(&env, &contract_id);

    let config = client.get_config();

    assert_eq!(config.admin, admin);
    assert_eq!(client.get_roles(&admin), ROLE_ALL);
}

#[test]
fn admin_can_manage_roles() {
    let f = setup();
    let account = Address::generate(&f.env);

    let roles = f.client().grant_role(&f.admin, &account, &ROLE_INSPECTOR);
    assert_eq!(roles, ROLE_INSPECTOR);
    assert!(f.client().has_role(&account, &ROLE_INSPECTOR));

    let roles = f.client().revoke_role(&f.admin, &account, &ROLE_INSPECTOR);
    assert_eq!(roles, 0);
    assert!(!f.client().has_role(&account, &ROLE_INSPECTOR));
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn non_admin_cannot_grant_roles() {
    let f = setup();
    let attacker = Address::generate(&f.env);
    let target = Address::generate(&f.env);

    f.client()
        .grant_role(&attacker, &target, &ROLE_MANUFACTURER);
}

#[test]
fn privileged_actions_require_real_authorization_context() {
    let f = setup();
    let new_owner = Address::generate(&f.env);

    // setup() uses mock_all_auths to build fixture state. Clearing auth entries
    // disables that blanket mock so require_auth must fail for these calls.
    f.env.set_auths(&[]);

    assert!(f
        .client()
        .try_transfer_owner(&f.manufacturer, &f.serial, &new_owner)
        .is_err());
    assert!(f
        .client()
        .try_add_inspection(
            &f.inspector,
            &f.serial,
            &88,
            &String::from_str(&f.env, "Unsigned inspection"),
        )
        .is_err());
    assert!(f
        .client()
        .try_flag_recall(
            &f.recall_authority,
            &f.serial,
            &String::from_str(&f.env, "Unsigned recall"),
        )
        .is_err());
}

#[test]
fn role_ttl_refresh_reports_existing_and_missing_roles() {
    let f = setup();
    let unknown = Address::generate(&f.env);

    assert!(f.client().refresh_role_ttl(&f.inspector));
    assert!(!f.client().refresh_role_ttl(&unknown));
}

#[test]
fn manufacturer_creates_passport() {
    let f = setup();
    let passport = f.client().get_passport(&f.serial);

    assert_eq!(passport.manufacturer, f.manufacturer);
    assert_eq!(passport.owner, f.manufacturer);
    assert_eq!(passport.status, STATUS_ACTIVE);
    assert_eq!(passport.health_score, 0);
    assert_eq!(passport.verified_by, None);

    let stats = f.client().get_stats();
    assert_eq!(stats.total_passports, 1);
    assert_eq!(stats.circulating_passports, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn unapproved_wallet_cannot_create_passport() {
    let f = setup();
    let attacker = Address::generate(&f.env);

    f.client().create_passport(
        &attacker,
        &String::from_str(&f.env, "UNAUTHORIZED-001"),
        &String::from_str(&f.env, "LFP"),
        &50_000,
        &300,
        &String::from_str(&f.env, "BATCH-X"),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn rejects_invalid_passport_metadata() {
    let f = setup();

    f.client().create_passport(
        &f.manufacturer,
        &String::from_str(&f.env, ""),
        &String::from_str(&f.env, "LFP"),
        &50_000,
        &300,
        &String::from_str(&f.env, "BATCH-X"),
    );
}

#[test]
fn owner_can_transfer_battery() {
    let f = setup();
    let new_owner = Address::generate(&f.env);

    let passport = f
        .client()
        .transfer_owner(&f.manufacturer, &f.serial, &new_owner);

    assert_eq!(passport.owner, new_owner);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn non_owner_cannot_transfer_battery() {
    let f = setup();
    let attacker = Address::generate(&f.env);
    let new_owner = Address::generate(&f.env);

    f.client().transfer_owner(&attacker, &f.serial, &new_owner);
}

#[test]
fn inspection_can_move_battery_into_and_out_of_review() {
    let f = setup();

    let failed = f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &45,
        &String::from_str(&f.env, "Cell imbalance detected"),
    );
    assert_eq!(failed.status, STATUS_UNDER_REVIEW);
    assert_eq!(failed.health_score, 45);

    let passed = f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &92,
        &String::from_str(&f.env, "Battery passed follow-up inspection"),
    );
    assert_eq!(passed.status, STATUS_ACTIVE);
    assert_eq!(passed.health_score, 92);
    assert_eq!(passed.inspections, 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn unapproved_wallet_cannot_inspect() {
    let f = setup();
    let attacker = Address::generate(&f.env);

    f.client().add_inspection(
        &attacker,
        &f.serial,
        &90,
        &String::from_str(&f.env, "Fake inspection"),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn rejects_invalid_health_score() {
    let f = setup();

    f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &101,
        &String::from_str(&f.env, "Invalid score"),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn verification_requires_inspection() {
    let f = setup();
    f.client().verify_passport(&f.verifier, &f.serial);
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn verification_requires_passing_health_score() {
    let f = setup();

    f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &40,
        &String::from_str(&f.env, "Battery failed inspection"),
    );
    f.client().verify_passport(&f.verifier, &f.serial);
}

#[test]
fn authorized_verifier_can_verify_passing_battery() {
    let f = setup();

    f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &93,
        &String::from_str(&f.env, "Battery passed inspection"),
    );
    let passport = f.client().verify_passport(&f.verifier, &f.serial);

    assert_eq!(passport.status, STATUS_VERIFIED);
    assert_eq!(passport.verified_by.as_ref(), Some(&f.verifier));
    assert_eq!(f.client().get_stats().verified_passports, 1);
}

#[test]
fn new_passing_inspection_requires_fresh_verification() {
    let f = setup();

    f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &95,
        &String::from_str(&f.env, "Passed initial inspection"),
    );
    f.client().verify_passport(&f.verifier, &f.serial);

    let passport = f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &88,
        &String::from_str(&f.env, "New inspection requires a fresh attestation"),
    );

    assert_eq!(passport.status, STATUS_ACTIVE);
    assert_eq!(passport.verified_by, None);
    assert_eq!(f.client().get_stats().verified_passports, 0);
}

#[test]
fn failed_inspection_removes_verified_status() {
    let f = setup();

    f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &95,
        &String::from_str(&f.env, "Passed initial inspection"),
    );
    f.client().verify_passport(&f.verifier, &f.serial);

    let passport = f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &35,
        &String::from_str(&f.env, "Critical degradation found"),
    );

    assert_eq!(passport.status, STATUS_UNDER_REVIEW);
    assert_eq!(passport.verified_by, None);
    assert_eq!(f.client().get_stats().verified_passports, 0);
}

#[test]
fn recall_is_terminal_until_recycling() {
    let f = setup();

    f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &90,
        &String::from_str(&f.env, "Passed inspection"),
    );
    f.client().verify_passport(&f.verifier, &f.serial);

    let recalled = f.client().flag_recall(
        &f.recall_authority,
        &f.serial,
        &String::from_str(&f.env, "Manufacturer safety recall"),
    );

    assert_eq!(recalled.status, STATUS_RECALLED);
    assert_eq!(recalled.verified_by, None);
    assert_eq!(f.client().get_stats().verified_passports, 0);
    assert_eq!(f.client().get_stats().recalled_passports, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn recalled_battery_cannot_be_verified_again() {
    let f = setup();

    f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &90,
        &String::from_str(&f.env, "Passed inspection"),
    );
    f.client().flag_recall(
        &f.recall_authority,
        &f.serial,
        &String::from_str(&f.env, "Safety recall"),
    );
    f.client().verify_passport(&f.verifier, &f.serial);
}

#[test]
fn recycling_requires_authorized_recycler_and_both_approvals() {
    let f = setup();

    let requested = f
        .client()
        .request_recycling(&f.manufacturer, &f.serial, &f.recycler);
    assert!(requested.owner_approved);
    assert!(!requested.recycler_approved);

    let approved = f.client().approve_recycling(&f.recycler, &f.serial);
    assert!(approved.recycler_approved);

    let recycled = f.client().execute_recycling(&f.manufacturer, &f.serial);
    assert_eq!(recycled.status, STATUS_RECYCLED);
    assert_eq!(recycled.recycler.as_ref(), Some(&f.recycler));

    let stats = f.client().get_stats();
    assert_eq!(stats.circulating_passports, 0);
    assert_eq!(stats.recycled_passports, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")]
fn recycling_request_rejects_unapproved_recycler() {
    let f = setup();
    let unapproved_recycler = Address::generate(&f.env);

    f.client()
        .request_recycling(&f.manufacturer, &f.serial, &unapproved_recycler);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn owner_cannot_execute_recycling_before_recycler_approval() {
    let f = setup();

    f.client()
        .request_recycling(&f.manufacturer, &f.serial, &f.recycler);
    f.client().execute_recycling(&f.manufacturer, &f.serial);
}

#[test]
fn revoked_recycler_cannot_execute_previously_approved_request() {
    let f = setup();

    f.client()
        .request_recycling(&f.manufacturer, &f.serial, &f.recycler);
    f.client().approve_recycling(&f.recycler, &f.serial);
    f.client()
        .revoke_role(&f.admin, &f.recycler, &ROLE_RECYCLER);

    let result = f.client().try_execute_recycling(&f.manufacturer, &f.serial);
    assert!(result.is_err());
}

#[test]
fn ownership_transfer_clears_pending_recycling_request() {
    let f = setup();
    let new_owner = Address::generate(&f.env);

    f.client()
        .request_recycling(&f.manufacturer, &f.serial, &f.recycler);
    f.client()
        .transfer_owner(&f.manufacturer, &f.serial, &new_owner);

    assert!(f.client().try_get_recycling_approval(&f.serial).is_err());
}

#[test]
fn duplicate_recycler_approval_is_rejected() {
    let f = setup();

    f.client()
        .request_recycling(&f.manufacturer, &f.serial, &f.recycler);
    f.client().approve_recycling(&f.recycler, &f.serial);

    assert!(f
        .client()
        .try_approve_recycling(&f.recycler, &f.serial)
        .is_err());
}

#[test]
fn recent_audits_are_bounded_and_return_latest_window() {
    let f = setup();

    f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &91,
        &String::from_str(&f.env, "First inspection"),
    );
    f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &92,
        &String::from_str(&f.env, "Second inspection"),
    );

    let records = f.client().get_recent_audits(&f.serial, &2);
    assert_eq!(records.len(), 2);
    assert_eq!(
        records.get(1).unwrap().note,
        String::from_str(&f.env, "Second inspection")
    );

    assert!(f.client().try_get_recent_audits(&f.serial, &21).is_err());
    assert!(f.client().try_get_recent_audits(&f.serial, &0).is_err());
}

#[test]
fn passport_ttl_refresh_is_bounded_and_callable() {
    let f = setup();

    f.client().add_inspection(
        &f.inspector,
        &f.serial,
        &91,
        &String::from_str(&f.env, "Lifecycle maintenance test"),
    );

    let refreshed = f.client().refresh_passport_ttl(&f.serial, &0, &50);
    assert!(refreshed >= 3);
    assert!(f
        .client()
        .try_refresh_passport_ttl(&f.serial, &0, &51)
        .is_err());
}

#[test]
fn admin_can_be_rotated() {
    let f = setup();
    let new_admin = Address::generate(&f.env);
    let target = Address::generate(&f.env);

    let config = f.client().transfer_admin(&f.admin, &new_admin);
    assert_eq!(config.admin, new_admin);
    assert_eq!(f.client().get_roles(&f.admin), 0);
    assert_eq!(f.client().get_roles(&new_admin), 0);

    f.client()
        .grant_role(&new_admin, &target, &ROLE_MANUFACTURER);
    assert!(f.client().has_role(&target, &ROLE_MANUFACTURER));
}
