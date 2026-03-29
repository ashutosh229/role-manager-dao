#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Env, String,
};


// ─────────────────────────────────────────────────────────────────────────────
//  Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Spins up a fresh Env, registers the contract, and returns both.
fn setup() -> (Env, RoleManagerDAOClient<'static>) {
    let env = Env::default();
    let contract_id = env.register_contract(None, RoleManagerDAO);
    let client = RoleManagerDAOClient::new(&env, &contract_id);
    (env, client)
}

/// Convenience: create a role proposal and return the proposal_id.
fn make_proposal(
    client:  &RoleManagerDAOClient,
    env:     &Env,
    name:    &str,
    desc:    &str,
    target:  &Address,
) -> u64 {
    client.create_role_proposal(
        &String::from_str(env, name),
        &String::from_str(env, desc),
        target,
    )
}

/// Convenience: cast `for_votes` votes in favour then `against_votes` votes against.
fn cast_votes(
    client:      &RoleManagerDAOClient,
    proposal_id: u64,
    for_votes:   u64,
    against_votes: u64,
) {
    for _ in 0..for_votes {
        client.vote_on_proposal(&proposal_id, &true);
    }
    for _ in 0..against_votes {
        client.vote_on_proposal(&proposal_id, &false);
    }
}


// ─────────────────────────────────────────────────────────────────────────────
//  1. create_role_proposal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_create_role_proposal_returns_id_one() {
    let (env, client) = setup();
    let target = Address::generate(&env);

    let proposal_id = make_proposal(&client, &env, "Treasurer", "Manages DAO funds", &target);

    assert_eq!(proposal_id, 1, "First proposal ID must be 1");
}

#[test]
fn test_create_role_proposal_increments_id() {
    let (env, client) = setup();
    let target = Address::generate(&env);

    let id1 = make_proposal(&client, &env, "Treasurer", "desc1", &target);
    let id2 = make_proposal(&client, &env, "Auditor",   "desc2", &target);
    let id3 = make_proposal(&client, &env, "Moderator", "desc3", &target);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

#[test]
fn test_create_role_proposal_proposal_is_active() {
    let (env, client) = setup();
    let target = Address::generate(&env);

    let pid = make_proposal(&client, &env, "Treasurer", "Manages DAO funds", &target);
    let proposal = client.view_proposal(&pid);

    assert_eq!(proposal.status, ProposalStatus::Active);
}

#[test]
fn test_create_role_proposal_role_skeleton_is_inactive() {
    let (env, client) = setup();
    let target = Address::generate(&env);

    let pid = make_proposal(&client, &env, "Treasurer", "Manages DAO funds", &target);
    let proposal = client.view_proposal(&pid);
    let role = client.view_role(&proposal.role_id);

    assert!(!role.is_active,   "Role must not be active before execution");
    assert!(!role.is_assigned, "Role must not be assigned before execution");
}

#[test]
fn test_create_role_proposal_stores_correct_name_and_description() {
    let (env, client) = setup();
    let target = Address::generate(&env);

    let pid = make_proposal(&client, &env, "Treasurer", "Manages DAO funds", &target);
    let proposal = client.view_proposal(&pid);
    let role = client.view_role(&proposal.role_id);

    assert_eq!(role.name,        String::from_str(&env, "Treasurer"));
    assert_eq!(role.description, String::from_str(&env, "Manages DAO funds"));
}

#[test]
fn test_create_role_proposal_target_address_stored() {
    let (env, client) = setup();
    let target = Address::generate(&env);

    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);
    let proposal = client.view_proposal(&pid);

    assert_eq!(proposal.target, target);
}

#[test]
fn test_create_role_proposal_updates_dao_stats() {
    let (env, client) = setup();
    let target = Address::generate(&env);

    make_proposal(&client, &env, "Treasurer", "desc", &target);
    let stats = client.view_dao_stats();

    assert_eq!(stats.total_proposals,  1);
    assert_eq!(stats.active_proposals, 1);
    assert_eq!(stats.total_roles,      1);
}

#[test]
fn test_create_multiple_proposals_dao_stats_accurate() {
    let (env, client) = setup();
    let target = Address::generate(&env);

    make_proposal(&client, &env, "Role A", "desc", &target);
    make_proposal(&client, &env, "Role B", "desc", &target);
    make_proposal(&client, &env, "Role C", "desc", &target);

    let stats = client.view_dao_stats();
    assert_eq!(stats.total_proposals,  3);
    assert_eq!(stats.active_proposals, 3);
    assert_eq!(stats.total_roles,      3);
}

#[test]
fn test_create_role_proposal_votes_initialised_to_zero() {
    let (env, client) = setup();
    let target = Address::generate(&env);

    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);
    let proposal = client.view_proposal(&pid);

    assert_eq!(proposal.votes_for,     0);
    assert_eq!(proposal.votes_against, 0);
}

#[test]
fn test_create_role_proposal_proposal_type_is_create_role() {
    let (env, client) = setup();
    let target = Address::generate(&env);

    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);
    let proposal = client.view_proposal(&pid);

    assert_eq!(proposal.proposal_type, ProposalType::CreateRole);
}

#[test]
fn test_create_role_proposal_timestamp_recorded() {
    let env = Env::default();
    env.ledger().set(LedgerInfo {
        timestamp: 1_700_000_000,
        ..Default::default()
    });
    let contract_id = env.register_contract(None, RoleManagerDAO);
    let client = RoleManagerDAOClient::new(&env, &contract_id);
    let target = Address::generate(&env);

    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);
    let role = client.view_role(&client.view_proposal(&pid).role_id);

    assert_eq!(role.created_at, 1_700_000_000);
}


// ─────────────────────────────────────────────────────────────────────────────
//  2. vote_on_proposal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_vote_for_increments_votes_for() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    client.vote_on_proposal(&pid, &true);
    let proposal = client.view_proposal(&pid);

    assert_eq!(proposal.votes_for,     1);
    assert_eq!(proposal.votes_against, 0);
}

#[test]
fn test_vote_against_increments_votes_against() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    client.vote_on_proposal(&pid, &false);
    let proposal = client.view_proposal(&pid);

    assert_eq!(proposal.votes_for,     0);
    assert_eq!(proposal.votes_against, 1);
}

#[test]
fn test_proposal_passes_with_majority_for_votes() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    // 2 for, 1 against → majority for, quorum = 3 → Passed
    cast_votes(&client, pid, 2, 1);
    let proposal = client.view_proposal(&pid);

    assert_eq!(proposal.status, ProposalStatus::Passed);
}

#[test]
fn test_proposal_rejected_with_majority_against_votes() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    // 1 for, 2 against → majority against → Rejected
    cast_votes(&client, pid, 1, 2);
    let proposal = client.view_proposal(&pid);

    assert_eq!(proposal.status, ProposalStatus::Rejected);
}

#[test]
fn test_proposal_stays_active_below_quorum() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    // Only 2 votes cast — below quorum of 3
    cast_votes(&client, pid, 2, 0);
    let proposal = client.view_proposal(&pid);

    assert_eq!(proposal.status, ProposalStatus::Active);
}

#[test]
fn test_active_proposals_decrements_after_passing() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    make_proposal(&client, &env, "Treasurer", "desc", &target);
    let pid = make_proposal(&client, &env, "Auditor", "desc", &target);

    // Pass second proposal
    cast_votes(&client, pid, 2, 1);

    let stats = client.view_dao_stats();
    // Two proposals created; one resolved → one still active
    assert_eq!(stats.active_proposals, 1);
}

#[test]
fn test_active_proposals_decrements_after_rejection() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    cast_votes(&client, pid, 0, 3); // all against

    let stats = client.view_dao_stats();
    assert_eq!(stats.active_proposals, 0);
}

#[test]
#[should_panic(expected = "Proposal is not active for voting")]
fn test_vote_on_passed_proposal_panics() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    cast_votes(&client, pid, 2, 1); // now Passed
    client.vote_on_proposal(&pid, &true); // should panic
}

#[test]
#[should_panic(expected = "Proposal is not active for voting")]
fn test_vote_on_rejected_proposal_panics() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    cast_votes(&client, pid, 1, 2); // now Rejected
    client.vote_on_proposal(&pid, &false); // should panic
}

#[test]
fn test_vote_counts_accumulate_correctly_before_quorum() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    client.vote_on_proposal(&pid, &true);
    client.vote_on_proposal(&pid, &true);

    let proposal = client.view_proposal(&pid);
    assert_eq!(proposal.votes_for,     2);
    assert_eq!(proposal.votes_against, 0);
    assert_eq!(proposal.status,        ProposalStatus::Active);
}

#[test]
fn test_mixed_votes_before_quorum_stay_active() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    client.vote_on_proposal(&pid, &true);
    client.vote_on_proposal(&pid, &false);

    let proposal = client.view_proposal(&pid);
    assert_eq!(proposal.votes_for,     1);
    assert_eq!(proposal.votes_against, 1);
    assert_eq!(proposal.status,        ProposalStatus::Active);
}


// ─────────────────────────────────────────────────────────────────────────────
//  3. execute_proposal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_execute_proposal_marks_proposal_executed() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    cast_votes(&client, pid, 2, 1);
    client.execute_proposal(&pid);

    let proposal = client.view_proposal(&pid);
    assert_eq!(proposal.status, ProposalStatus::Executed);
}

#[test]
fn test_execute_proposal_activates_role() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    cast_votes(&client, pid, 2, 1);
    client.execute_proposal(&pid);

    let role_id = client.view_proposal(&pid).role_id;
    let role = client.view_role(&role_id);

    assert!(role.is_active,   "Role must be active after execution");
    assert!(role.is_assigned, "Role must be assigned after execution");
}

#[test]
fn test_execute_proposal_increments_total_members() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    cast_votes(&client, pid, 2, 1);
    client.execute_proposal(&pid);

    let stats = client.view_dao_stats();
    assert_eq!(stats.total_members, 1);
}

#[test]
fn test_execute_two_proposals_increments_members_twice() {
    let (env, client) = setup();
    let target = Address::generate(&env);

    let pid1 = make_proposal(&client, &env, "Treasurer", "desc", &target);
    let pid2 = make_proposal(&client, &env, "Auditor",   "desc", &target);

    cast_votes(&client, pid1, 2, 1);
    cast_votes(&client, pid2, 3, 0);

    client.execute_proposal(&pid1);
    client.execute_proposal(&pid2);

    let stats = client.view_dao_stats();
    assert_eq!(stats.total_members, 2);
}

#[test]
#[should_panic(expected = "Proposal has not passed; cannot execute")]
fn test_execute_active_proposal_panics() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    // No votes cast — still Active
    client.execute_proposal(&pid);
}

#[test]
#[should_panic(expected = "Proposal has not passed; cannot execute")]
fn test_execute_rejected_proposal_panics() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    cast_votes(&client, pid, 0, 3); // Rejected
    client.execute_proposal(&pid);
}

#[test]
#[should_panic(expected = "Proposal has not passed; cannot execute")]
fn test_execute_already_executed_proposal_panics() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    cast_votes(&client, pid, 2, 1);
    client.execute_proposal(&pid);
    client.execute_proposal(&pid); // second call must panic
}

#[test]
fn test_full_lifecycle_create_vote_execute() {
    let (env, client) = setup();
    let target = Address::generate(&env);

    // Step 1 – create
    let pid = make_proposal(&client, &env, "Treasurer", "Manages DAO treasury", &target);
    assert_eq!(client.view_proposal(&pid).status, ProposalStatus::Active);

    // Step 2 – vote (2 for, 1 against → Passed)
    cast_votes(&client, pid, 2, 1);
    assert_eq!(client.view_proposal(&pid).status, ProposalStatus::Passed);

    // Step 3 – execute
    client.execute_proposal(&pid);
    assert_eq!(client.view_proposal(&pid).status, ProposalStatus::Executed);

    let role_id = client.view_proposal(&pid).role_id;
    let role = client.view_role(&role_id);
    assert!(role.is_active);
    assert!(role.is_assigned);

    let stats = client.view_dao_stats();
    assert_eq!(stats.total_proposals,  1);
    assert_eq!(stats.total_roles,      1);
    assert_eq!(stats.total_members,    1);
    assert_eq!(stats.active_proposals, 0);
}


// ─────────────────────────────────────────────────────────────────────────────
//  4. view_dao_stats
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_view_dao_stats_initial_values_all_zero() {
    let (_, client) = setup();
    let stats = client.view_dao_stats();

    assert_eq!(stats.total_roles,      0);
    assert_eq!(stats.total_members,    0);
    assert_eq!(stats.total_proposals,  0);
    assert_eq!(stats.active_proposals, 0);
}

#[test]
fn test_view_dao_stats_reflects_multiple_proposals() {
    let (env, client) = setup();
    let target = Address::generate(&env);

    make_proposal(&client, &env, "Role A", "desc", &target);
    make_proposal(&client, &env, "Role B", "desc", &target);

    let stats = client.view_dao_stats();
    assert_eq!(stats.total_proposals,  2);
    assert_eq!(stats.active_proposals, 2);
    assert_eq!(stats.total_roles,      2);
}


// ─────────────────────────────────────────────────────────────────────────────
//  5. view_proposal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "Proposal not found")]
fn test_view_nonexistent_proposal_panics() {
    let (_, client) = setup();
    client.view_proposal(&999);
}

#[test]
fn test_view_proposal_returns_correct_proposal_id() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);
    let proposal = client.view_proposal(&pid);

    assert_eq!(proposal.proposal_id, pid);
}


// ─────────────────────────────────────────────────────────────────────────────
//  6. view_role
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "Role not found")]
fn test_view_nonexistent_role_panics() {
    let (_, client) = setup();
    client.view_role(&999);
}

#[test]
fn test_view_role_returns_correct_role_id() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);
    let role_id = client.view_proposal(&pid).role_id;
    let role = client.view_role(&role_id);

    assert_eq!(role.role_id, role_id);
}

#[test]
fn test_role_holder_matches_target_after_execution() {
    let (env, client) = setup();
    let target = Address::generate(&env);
    let pid = make_proposal(&client, &env, "Treasurer", "desc", &target);

    cast_votes(&client, pid, 2, 1);
    client.execute_proposal(&pid);

    let role_id = client.view_proposal(&pid).role_id;
    let role = client.view_role(&role_id);

    assert_eq!(role.holder, target);
}