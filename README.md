# Role Manager DAO

## Table of Contents
- [Project Title](#project-title)
- [Project Description](#project-description)
- [Project Vision](#project-vision)
- [Key Features](#key-features)
- [Future Scope](#future-scope)

---

## Project Title

**Role Manager DAO** — Decentralized Role-Based Governance System

---

## Project Description

Role Manager DAO is a decentralized governance framework built in **Rust** using the **Soroban SDK**, designed to manage roles, permissions, and access control within a blockchain ecosystem. The system replaces centralized authority with transparent, community-driven decision-making: every role change is gated behind an on-chain proposal and a DAO vote, ensuring fairness, auditability, and resistance to manipulation.

The smart contract exposes three core governance functions (`create_role_proposal`, `vote_on_proposal`, `execute_proposal`) and a set of read-only view helpers, making it minimal, composable, and easy to audit.

### Contract Functions

| Function | Actor | Description |
|---|---|---|
| `create_role_proposal` | Any participant | Opens a governance proposal to create a new role; persists a role skeleton and a `ProposalStatus::Active` proposal on-chain. Returns the new `proposal_id`. |
| `vote_on_proposal` | DAO member | Casts a `for` or `against` vote. Once ≥ 3 votes are cast, simple majority determines `Passed` or `Rejected`. |
| `execute_proposal` | Any caller | Executes a `Passed` proposal, activating the role change on-chain and marking the proposal `Executed`. |
| `view_dao_stats` | Read-only | Returns aggregate DAO statistics (`total_roles`, `total_members`, `total_proposals`, `active_proposals`). |
| `view_proposal` | Read-only | Returns the full `Proposal` struct for a given `proposal_id`. |
| `view_role` | Read-only | Returns the full `Role` struct for a given `role_id`. |

### Core Data Structures

```
ApprovalStatus / DAOStats  — aggregate platform counters
Proposal                   — governance proposal with type, votes, status, and target
Role                       — on-chain role entry with holder, active flag, and metadata
ProposalType               — enum: CreateRole | AssignRole | RevokeRole
ProposalStatus             — enum: Active | Passed | Rejected | Executed
```

### How It Works — Step-by-Step

```
1. A participant calls create_role_proposal(name, description, target)
      └─► A Role skeleton (is_active = false) and an Active Proposal are stored on-chain.

2. DAO members call vote_on_proposal(proposal_id, true/false) one or more times.
      └─► After ≥ 3 votes, the contract resolves the proposal to Passed or Rejected.

3. Any caller invokes execute_proposal(proposal_id) on a Passed proposal.
      └─► The Role is activated (is_active = true, is_assigned = true) and the
          proposal is marked Executed. DAO stats are updated.
```

---

## Project Vision

The vision behind Role Manager DAO is to provide a **trustless, transparent, and community-governed alternative** to traditional Role-Based Access Control (RBAC) systems. In conventional systems, a single administrator assigns roles — creating a single point of failure and a potential vector for abuse.

Role Manager DAO shifts this power to the community. Every role assignment, revocation, or creation must pass through an on-chain proposal and survive a democratic vote before it takes effect. This design guarantees that no single actor can unilaterally change the permission landscape of a protocol.

Beyond access control, the project serves as a **governance primitive** — a reusable building block that other dApps, DAOs, DeFi protocols, and NFT communities can integrate to add structured, auditable role management without rebuilding governance logic from scratch.

---

## Key Features

- **Decentralized Role Management**  
  Roles are created, assigned, and revoked exclusively through DAO proposals. No privileged admin key exists; the community is the authority.

- **On-Chain Governance Engine**  
  Proposals move through a well-defined lifecycle (`Active → Passed/Rejected → Executed`). All state transitions are recorded immutably on the ledger.

- **Simple Quorum Voting**  
  A proposal resolves once at least 3 votes are cast and a simple majority (votes_for > votes_against) is achieved, making governance lightweight yet meaningful.

- **Typed Proposal System**  
  Three proposal types (`CreateRole`, `AssignRole`, `RevokeRole`) keep governance intentions explicit and auditable.

- **Transparent Statistics**  
  `view_dao_stats` exposes real-time aggregate counters — total roles, total members, total and active proposals — giving anyone a live health snapshot of the DAO.

- **Security-Focused Design**  
  Written in Rust with Soroban SDK, the contract benefits from Rust's memory-safety guarantees, no heap allocations (`#![no_std]`), and deterministic execution — eliminating whole classes of vulnerabilities present in other smart contract environments.

- **Modular & Composable**  
  The contract stores role and proposal data under clean enum-keyed namespaces (`RoleBook`, `ProposalBook`), making it straightforward to extend with new proposal types or integrate into larger dApp architectures.

- **Full Auditability**  
  Every action emits a `log!` event and mutates only well-defined storage keys, giving block explorers and audit tools a clear trail of every governance decision.

---

## Future Scope

The current implementation establishes the foundational governance layer. The following enhancements are planned for subsequent iterations:

1. **Signer-Based Access Control**  
   Integrate Soroban's `Address::require_auth()` to bind votes and executions to authenticated wallet addresses, preventing Sybil attacks and duplicate votes.

2. **Weighted Voting & Token-Gated Governance**  
   Allow governance token holders to cast votes proportional to their stake, moving from one-address-one-vote to token-weighted governance for more representative outcomes.

3. **Role Hierarchy & Inheritance**  
   Introduce parent-child role relationships so that permissions granted to a parent role cascade to all child roles, enabling fine-grained, layered access control.

4. **Time-Locked Proposals**  
   Add a `deadline` field to proposals so that voting windows automatically close after a configurable on-chain time period, preventing stale proposals from lingering indefinitely.

5. **Multi-Signature Role Execution**  
   Require M-of-N admin signatures to execute high-impact proposals (e.g., granting super-admin roles), adding an extra layer of security for sensitive operations.

6. **Proposal Delegation**  
   Let token holders delegate their voting power to trusted representatives, enabling participation even when members are inactive.

7. **Cross-Contract Role Queries**  
   Expose a standardized interface so external contracts can query whether a given address holds a specific role, enabling Role Manager DAO to serve as a shared permission oracle across an entire protocol suite.

8. **Frontend Dashboard**  
   Build a React/Next.js dApp that visualises live DAO stats, lists active proposals, and allows community members to vote through a wallet-connected UI — making governance accessible to non-technical participants.

9. **Role Expiry & Renewal**  
   Introduce optional expiry timestamps on roles so that temporary permissions (e.g., event moderator, seasonal treasury manager) automatically lapse without requiring a revocation proposal.

10. **Governance Analytics & Reporting**  
    Emit structured events for every state transition to power off-chain analytics dashboards that track voter participation rates, proposal pass rates, and role churn over time.
