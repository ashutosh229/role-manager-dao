#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, symbol_short};


// Tracks aggregate DAO statistics
#[contracttype]
#[derive(Clone)]
pub struct DAOStats {
    pub total_roles: u64,      // total roles ever created
    pub total_members: u64,    // total unique members assigned a role
    pub total_proposals: u64,  // total governance proposals submitted
    pub active_proposals: u64, // proposals currently open for voting
}

const DAO_STATS: Symbol = symbol_short!("DAO_STATS");


// Maps a proposal_id to its Proposal struct
#[contracttype]
pub enum ProposalBook {
    Proposal(u64),
}

// Maps a role_id to its Role struct
#[contracttype]
pub enum RoleBook {
    Role(u64),
}

// Counter keys
const COUNT_PROPOSAL: Symbol = symbol_short!("C_PROP");
const COUNT_ROLE:     Symbol = symbol_short!("C_ROLE");


// Proposal types supported by the DAO
#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ProposalType {
    AssignRole,  // propose assigning a role to an address
    RevokeRole,  // propose revoking a role from an address
    CreateRole,  // propose creating a brand-new role
}


// Lifecycle of a proposal
#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ProposalStatus {
    Active,   // voting is open
    Passed,   // quorum reached, approved
    Rejected, // voting closed, not approved
    Executed, // approved and on-chain action applied
}


// A governance proposal
#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub proposal_id:   u64,
    pub proposal_type: ProposalType,
    pub role_id:       u64,            // role being acted upon
    pub target:        Address,        // address receiving / losing the role
    pub votes_for:     u64,
    pub votes_against: u64,
    pub status:        ProposalStatus,
    pub created_at:    u64,
}


// A role entry in the registry
#[contracttype]
#[derive(Clone)]
pub struct Role {
    pub role_id:     u64,
    pub name:        String,
    pub description: String,
    pub is_active:   bool,
    pub created_at:  u64,
    pub holder:      Address,  // current holder of this role instance
    pub is_assigned: bool,     // whether the role is presently assigned
}


// ─────────────────────────────────────────────
//  Contract
// ─────────────────────────────────────────────

#[contract]
pub struct RoleManagerDAO;

#[contractimpl]
impl RoleManagerDAO {

    // ── 1. CREATE ROLE PROPOSAL ──────────────────────────────────────────
    // Any participant can open a proposal to create a new role.
    // Returns the new proposal_id.
    pub fn create_role_proposal(
        env:         Env,
        name:        String,
        description: String,
        target:      Address,  // intended future holder of the role
    ) -> u64 {
        let mut count_proposal: u64 = env
            .storage().instance()
            .get(&COUNT_PROPOSAL)
            .unwrap_or(0);
        count_proposal += 1;

        let mut count_role: u64 = env
            .storage().instance()
            .get(&COUNT_ROLE)
            .unwrap_or(0);
        count_role += 1;

        let time = env.ledger().timestamp();

        // Persist the role skeleton (not yet active – awaits proposal execution)
        let role = Role {
            role_id:     count_role,
            name,
            description,
            is_active:   false,
            created_at:  time,
            holder:      target.clone(),
            is_assigned: false,
        };
        env.storage().instance()
            .set(&RoleBook::Role(count_role), &role);
        env.storage().instance()
            .set(&COUNT_ROLE, &count_role);

        // Persist the proposal
        let proposal = Proposal {
            proposal_id:   count_proposal,
            proposal_type: ProposalType::CreateRole,
            role_id:       count_role,
            target,
            votes_for:     0,
            votes_against: 0,
            status:        ProposalStatus::Active,
            created_at:    time,
        };
        env.storage().instance()
            .set(&ProposalBook::Proposal(count_proposal), &proposal);
        env.storage().instance()
            .set(&COUNT_PROPOSAL, &count_proposal);

        // Update DAO stats
        let mut stats = Self::view_dao_stats(env.clone());
        stats.total_proposals  += 1;
        stats.active_proposals += 1;
        stats.total_roles      += 1;
        env.storage().instance().set(&DAO_STATS, &stats);

        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Proposal created with ID: {}", count_proposal);
        count_proposal
    }


    // ── 2. VOTE ON PROPOSAL ──────────────────────────────────────────────
    // Members vote for (vote_in_favour = true) or against a proposal.
    // A simple majority (votes_for > votes_against) passes the proposal.
    pub fn vote_on_proposal(env: Env, proposal_id: u64, vote_in_favour: bool) {
        let mut proposal = Self::view_proposal(env.clone(), proposal_id);

        // Only active proposals can be voted on
        if proposal.status != ProposalStatus::Active {
            log!(&env, "Proposal {} is not active", proposal_id);
            panic!("Proposal is not active for voting");
        }

        if vote_in_favour {
            proposal.votes_for += 1;
        } else {
            proposal.votes_against += 1;
        }

        // Resolve outcome once at least 3 votes have been cast (simple quorum)
        let total_votes = proposal.votes_for + proposal.votes_against;
        if total_votes >= 3 {
            if proposal.votes_for > proposal.votes_against {
                proposal.status = ProposalStatus::Passed;

                let mut stats = Self::view_dao_stats(env.clone());
                if stats.active_proposals > 0 {
                    stats.active_proposals -= 1;
                }
                env.storage().instance().set(&DAO_STATS, &stats);
            } else {
                proposal.status = ProposalStatus::Rejected;

                let mut stats = Self::view_dao_stats(env.clone());
                if stats.active_proposals > 0 {
                    stats.active_proposals -= 1;
                }
                env.storage().instance().set(&DAO_STATS, &stats);
            }
        }

        env.storage().instance()
            .set(&ProposalBook::Proposal(proposal_id), &proposal);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Vote recorded for proposal: {}", proposal_id);
    }


    // ── 3. EXECUTE PROPOSAL ─────────────────────────────────────────────
    // Executes a Passed proposal, activating the on-chain role change.
    pub fn execute_proposal(env: Env, proposal_id: u64) {
        let mut proposal = Self::view_proposal(env.clone(), proposal_id);

        if proposal.status != ProposalStatus::Passed {
            log!(&env, "Proposal {} has not passed yet", proposal_id);
            panic!("Proposal has not passed; cannot execute");
        }

        let mut role = Self::view_role(env.clone(), proposal.role_id);

        match proposal.proposal_type {
            ProposalType::CreateRole => {
                role.is_active   = true;
                role.is_assigned = true;
            }
            ProposalType::AssignRole => {
                role.holder      = proposal.target.clone();
                role.is_assigned = true;
            }
            ProposalType::RevokeRole => {
                role.is_assigned = false;
            }
        }

        proposal.status = ProposalStatus::Executed;

        env.storage().instance()
            .set(&RoleBook::Role(role.role_id), &role);
        env.storage().instance()
            .set(&ProposalBook::Proposal(proposal_id), &proposal);

        let mut stats = Self::view_dao_stats(env.clone());
        stats.total_members += 1;
        env.storage().instance().set(&DAO_STATS, &stats);

        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Proposal {} executed successfully", proposal_id);
    }


    // ── 4. VIEW HELPERS ─────────────────────────────────────────────────

    // Returns global DAO statistics
    pub fn view_dao_stats(env: Env) -> DAOStats {
        env.storage().instance().get(&DAO_STATS).unwrap_or(DAOStats {
            total_roles:      0,
            total_members:    0,
            total_proposals:  0,
            active_proposals: 0,
        })
    }

    // Returns a proposal by its ID
    pub fn view_proposal(env: Env, proposal_id: u64) -> Proposal {
        env.storage().instance()
            .get(&ProposalBook::Proposal(proposal_id))
            .unwrap_or_else(|| panic!("Proposal not found"))
    }

    // Returns a role by its ID
    pub fn view_role(env: Env, role_id: u64) -> Role {
        env.storage().instance()
            .get(&RoleBook::Role(role_id))
            .unwrap_or_else(|| panic!("Role not found"))
    }
}