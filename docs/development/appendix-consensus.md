# Consensus Algorithms: Distributed Agreement Under Failure

**Purpose**: Algorithms and theoretical frameworks for achieving agreement among distributed processes despite failures, enabling fault-tolerant distributed systems.

**Core Insight**: Consensus is **impossible** in asynchronous systems with even one crash failure (FLP), but **practical** with timeouts, randomization, or synchrony assumptions.

---

## Table of Contents

1. [Foundational Concepts](#foundational-concepts)
2. [FLP Impossibility](#flp-impossibility)
3. [Paxos](#paxos)
4. [Raft](#raft)
5. [Byzantine Consensus](#byzantine-consensus)
6. [Practical Systems](#practical-systems)
7. [Stack-Specific Implementations](#stack-specific-implementations)
8. [Integration Points](#integration-points)

---

## Foundational Concepts

### The Consensus Problem

**Goal**: N processes must agree on a single value, despite failures.

**Requirements** (Traditional):
1. **Termination**: Every correct process eventually decides on a value
2. **Agreement**: All correct processes decide the same value
3. **Validity**: The decided value was proposed by some process

### Failure Models

**Crash Failures** (Benign):
- Process stops executing permanently
- No incorrect behavior before crash
- Examples: power failure, OOM kill

**Byzantine Failures** (Malicious):
- Arbitrary behavior: send wrong messages, lie, collude
- Includes both malicious actors and software bugs
- Requires stronger protocols (BFT)

**Omission Failures**:
- Process fails to send or receive messages
- Includes network partitions

**Timing Failures**:
- Process responds too late
- Relevant in synchronous systems

### System Models

**Asynchronous**:
- No bounds on message delay or processing speed
- No reliable failure detection
- Most realistic but hardest (FLP impossible)

**Synchronous**:
- Known bounds on delays and processing
- Reliable failure detection via timeouts
- Easiest but least realistic

**Partially Synchronous** (Practical):
- Eventually synchronous: bounds hold after some unknown time
- Models real systems (networks eventually recover)
- Algorithms like Paxos, Raft work here

---

## FLP Impossibility

**Theorem** (Fischer, Lynch, Paterson, 1985):

"No deterministic consensus algorithm can guarantee termination in an asynchronous system with even one crash failure."

### FLP Proof Sketch

**Setup**:
- Asynchronous system, N processes
- At most 1 crash failure
- Binary consensus (decide 0 or 1)

**Key Concept**: **Bivalent Configuration**
- Configuration where both 0 and 1 are possible outcomes
- Univalent = only one outcome possible (0-valent or 1-valent)

**Proof Steps**:
1. **Initial bivalent configuration exists**: Some initial state allows either outcome
2. **Bivalent configurations reachable forever**: From any bivalent config, can always reach another bivalent config by delaying one message
3. **No termination**: If always bivalent configs reachable, never reach univalent (decided) state

**Implication**: In pure asynchronous model, can't distinguish crashed process from slow process → can't safely terminate.

### Circumventing FLP

Real systems work because they relax assumptions:

**Randomization**:
- Use randomness to break symmetry
- Terminate with probability 1 (not deterministic guarantee)
- Example: Bitcoin's proof-of-work

**Timeouts** (Partial Synchrony):
- Assume eventual synchrony: bounds hold eventually
- Timeout to detect failures (may be wrong temporarily)
- Examples: Paxos, Raft, PBFT

**Failure Detectors**:
- Abstract failure detection service
- Perfect FD: never wrong (requires synchrony)
- Eventually Perfect FD: eventually accurate (achievable)
- Examples: Chandra-Toueg consensus

**Weakened Termination**:
- Require only "live" processes terminate
- Non-faulty but slow processes may not decide
- Example: Paxos doesn't guarantee termination for all

---

## Paxos

**Developed by**: Leslie Lamport (1989, published 1998)

**Key Idea**: Use **quorum-based** voting with **proposal numbering** to ensure agreement even when processes crash or messages are delayed.

### Single-Decree Paxos (Basic Paxos)

**Goal**: Agree on a single value.

**Roles**:
- **Proposer**: Proposes values
- **Acceptor**: Votes on proposals
- **Learner**: Learns the chosen value

**Phases**:

**Phase 1: Prepare**
1. Proposer selects unique proposal number N
2. Sends `Prepare(N)` to quorum of acceptors
3. Acceptor responds:
   - `Promise(N)` if N > any previous promise
   - Include highest-numbered accepted proposal (if any)

**Phase 2: Accept**
1. If proposer receives promises from quorum:
   - If any acceptor returned accepted value, use highest-numbered one
   - Otherwise, use proposer's own value
2. Send `Accept(N, value)` to quorum
3. Acceptor accepts if N ≥ promised number
4. Respond `Accepted(N, value)`

**Learning**:
- Learner receives `Accepted` from quorum → value is chosen

**Invariant**: If quorums overlap, at most one value can be chosen.

### Multi-Paxos

**Problem**: Basic Paxos requires 2 round trips per decision.

**Solution**: Elect a **leader** (stable proposer) who skips Phase 1 for subsequent proposals.

**Leader Election**:
- Use Phase 1 to establish leadership
- Leader has exclusive proposal rights while stable
- On leader failure, run Phase 1 again

**Log Replication**:
- Apply Paxos to each log slot independently
- Leader coordinates appending entries across replicas
- Achieves replicated state machine

### Paxos Properties

**Safety** (Always):
- **Agreement**: Only one value chosen per instance
- **Validity**: Chosen value was proposed

**Liveness** (Eventually, with stable leader):
- Progress if majority available and leader stable

**Quorum Requirement**:
- Need majority: ⌈(N+1)/2⌉
- Tolerates ⌊N/2⌋ crash failures

### Paxos Variants

**Fast Paxos**:
- Clients send directly to acceptors (skip proposer)
- Requires 3f+1 acceptors for f failures
- Faster when no conflicts

**Cheap Paxos**:
- Subset of acceptors (main quorum) + auxiliary acceptors
- Auxiliary only activated on main failure
- Reduces number of always-active acceptors

**Flexible Paxos**:
- Different quorum sizes for Phase 1 and Phase 2
- Requirement: Q1 ∩ Q2 ≠ ∅ (quorums must overlap)

---

## Raft

**Developed by**: Diego Ongaro and John Ousterhout (2014)

**Goal**: Understandable alternative to Paxos for replicated state machines.

### Raft Overview

**Key Innovation**: Decompose consensus into:
1. **Leader Election**
2. **Log Replication**
3. **Safety**

**Roles**:
- **Leader**: Handles all client requests, replicates log
- **Follower**: Passive, replicate leader's log
- **Candidate**: Participates in leader election

### Leader Election

**Terms**:
- Logical clock, monotonically increasing
- Each term has at most one leader

**Election Process**:
1. Follower timeout → becomes Candidate
2. Candidate increments term, votes for self
3. Requests votes from all peers
4. Peer grants vote if:
   - Candidate's term ≥ peer's term
   - Peer hasn't voted this term
   - Candidate's log at least as up-to-date
5. Candidate wins if receives majority votes
6. Becomes Leader, sends heartbeats

**Split Vote**:
- No candidate gets majority → timeout, retry with new term
- Randomized timeouts reduce split votes

### Log Replication

**Normal Operation**:
1. Client sends command to Leader
2. Leader appends to local log
3. Leader sends `AppendEntries` RPCs to Followers
4. Followers append entry, respond success
5. Leader commits when majority replicated
6. Leader applies to state machine, replies to client
7. Leader piggybacks commitIndex in heartbeats

**Log Matching Property**:
- If two logs have entry with same index and term → all preceding entries identical
- Maintained by Leader's consistency check

**Handling Failures**:
- Follower crash: Leader retries `AppendEntries` indefinitely
- Leader crash: New leader elected, log repaired
- Log inconsistencies: Leader forces followers to match its log

### Safety

**Election Safety**:
- At most one leader per term

**Leader Completeness**:
- If log entry committed in term T, present in logs of leaders for all terms > T

**Log Matching**:
- If two logs contain entry with same index and term, they match up to that point

**State Machine Safety**:
- If server applies log entry at index, no other server applies different entry at same index

**Commitment Rule**:
- Leader can only commit entries from current term
- Once current term entry committed, all prior entries implicitly committed

### Raft vs. Paxos

| Aspect | Paxos | Raft |
|--------|-------|------|
| Structure | Single-decree + Multi-Paxos | Leader-based from start |
| Leader | Optional, emergent | Explicit, necessary |
| Log Gaps | Allowed | Not allowed |
| Commit Rule | Value-based | Log-index-based |
| Understandability | Complex | Simpler |

---

## Byzantine Consensus

**Challenge**: Achieve consensus when some nodes are **Byzantine** (malicious or faulty).

**Requirement**: ≥ 3f+1 nodes to tolerate f Byzantine failures (quorum = 2f+1).

### PBFT (Practical Byzantine Fault Tolerance)

**Developed by**: Castro & Liskov (1999)

**Key Idea**: State machine replication with Byzantine fault tolerance.

**Phases**:

**Pre-Prepare**:
1. Client sends request to Primary
2. Primary assigns sequence number, broadcasts `PrePrepare(v, n, m)` where:
   - v = view number
   - n = sequence number
   - m = request message

**Prepare**:
1. Replica receives PrePrepare, validates
2. Broadcasts `Prepare(v, n, d)` where d = digest of m
3. Replica waits for 2f matching Prepare messages (quorum)
4. "Prepared" certificate established

**Commit**:
1. Replica broadcasts `Commit(v, n, d)`
2. Waits for 2f+1 Commit messages (including self)
3. "Committed" certificate established
4. Execute request, reply to client

**Client**:
- Waits for f+1 matching replies (one from correct replica guaranteed)

**View Change** (Leader Replacement):
- Replica timeout → suspect Primary faulty
- Initiate view change protocol
- New primary elected, state transferred

**PBFT Properties**:
- **Safety**: All correct replicas agree on order
- **Liveness**: Clients eventually receive replies
- **Performance**: ~3 round trips, similar to Paxos

### Tendermint

**Developed by**: Jae Kwon (2014)

**Key Idea**: BFT consensus for blockchain with immediate finality.

**Mechanism**:
- Rounds with designated proposer (rotating)
- Three phases: Propose, Prevote, Precommit
- Two rounds of voting for each decision
- Block finalized when 2f+1 Precommits collected

**Features**:
- **Immediate Finality**: No forks, block final when committed
- **Application Agnostic**: ABCI interface for any application
- **Light Clients**: Efficient verification via Merkle proofs

**Used In**: Cosmos blockchain, many app-chains.

### HotStuff

**Developed by**: Yin, Malkhi, et al. (2018)

**Key Innovation**: **Linear** communication complexity (O(n) per decision, not O(n²) like PBFT).

**Mechanism**:
- **Threshold Signatures**: Aggregate 2f+1 signatures into single signature
- **Three-Phase Voting**: Prepare, Pre-Commit, Commit
- **Chained Structure**: Each phase's QC becomes next phase's justify

**Phases**:
1. **Prepare**: Leader proposes, replicas vote
2. **Pre-Commit**: Leader aggregates votes into QC, replicas vote again
3. **Commit**: Leader aggregates, replicas commit
4. **Decide**: Block finalized

**Advantages**:
- O(n) communication (vs. O(n²) in PBFT)
- Responsive: Progresses at network speed (not fixed timeout)
- Used in LibraBFT (Diem blockchain)

---

## Practical Systems

### Chubby (Google)

**Purpose**: Distributed lock service

**Algorithm**: Paxos (Multi-Paxos)

**Features**:
- Coarse-grained locks (lease-based)
- Small file storage (config, leader election results)
- 5-replica typical deployment

**Use Cases**: GFS, Bigtable (master election, metadata storage)

### ZooKeeper (Apache)

**Purpose**: Coordination service for distributed applications

**Algorithm**: Zab (ZooKeeper Atomic Broadcast) - similar to Paxos/Raft

**Features**:
- Hierarchical namespace (like filesystem)
- Watches for change notifications
- Ephemeral nodes (disappear when client session ends)

**Use Cases**: Kafka (broker coordination), HBase (master election)

### etcd

**Purpose**: Distributed key-value store

**Algorithm**: Raft

**Features**:
- Strong consistency (linearizable reads/writes)
- Watch API for change notifications
- TTL for keys (lease-based)

**Use Cases**: Kubernetes (cluster state), service discovery

### Consul (HashiCorp)

**Purpose**: Service mesh, service discovery, configuration

**Algorithm**: Raft

**Features**:
- Multi-datacenter support (federated Raft clusters)
- Health checking integrated
- DNS and HTTP interfaces

**Use Cases**: Microservice coordination, config management

---

## Stack-Specific Implementations

### Rust: Raft Implementation (Simplified)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Raft node state
#[derive(Debug, Clone, PartialEq)]
enum Role {
    Follower,
    Candidate,
    Leader,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogEntry {
    term: u64,
    index: u64,
    command: String,
}

/// RequestVote RPC
#[derive(Debug, Serialize, Deserialize)]
struct RequestVoteArgs {
    term: u64,
    candidate_id: u64,
    last_log_index: u64,
    last_log_term: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RequestVoteReply {
    term: u64,
    vote_granted: bool,
}

/// AppendEntries RPC
#[derive(Debug, Serialize, Deserialize)]
struct AppendEntriesArgs {
    term: u64,
    leader_id: u64,
    prev_log_index: u64,
    prev_log_term: u64,
    entries: Vec<LogEntry>,
    leader_commit: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppendEntriesReply {
    term: u64,
    success: bool,
    conflict_index: Option<u64>,
}

/// Raft node
struct RaftNode {
    id: u64,
    peers: Vec<u64>,

    // Persistent state
    current_term: u64,
    voted_for: Option<u64>,
    log: Vec<LogEntry>,

    // Volatile state
    commit_index: u64,
    last_applied: u64,
    role: Role,

    // Leader-specific
    next_index: HashMap<u64, u64>,
    match_index: HashMap<u64, u64>,

    // Election timing
    last_heartbeat: Instant,
    election_timeout: Duration,
}

impl RaftNode {
    fn new(id: u64, peers: Vec<u64>) -> Self {
        RaftNode {
            id,
            peers,
            current_term: 0,
            voted_for: None,
            log: vec![],
            commit_index: 0,
            last_applied: 0,
            role: Role::Follower,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            last_heartbeat: Instant::now(),
            election_timeout: Duration::from_millis(150),
        }
    }

    /// Handle RequestVote RPC
    fn handle_request_vote(
        &mut self,
        args: RequestVoteArgs,
    ) -> RequestVoteReply {
        let mut vote_granted = false;

        if args.term > self.current_term {
            self.current_term = args.term;
            self.role = Role::Follower;
            self.voted_for = None;
        }

        if args.term == self.current_term {
            let can_vote = self.voted_for.is_none()
                || self.voted_for == Some(args.candidate_id);

            let log_ok = self.is_log_up_to_date(
                args.last_log_term,
                args.last_log_index,
            );

            if can_vote && log_ok {
                self.voted_for = Some(args.candidate_id);
                vote_granted = true;
                self.last_heartbeat = Instant::now();
            }
        }

        RequestVoteReply {
            term: self.current_term,
            vote_granted,
        }
    }

    /// Check if candidate's log is at least as up-to-date
    fn is_log_up_to_date(
        &self,
        last_term: u64,
        last_index: u64,
    ) -> bool {
        let (my_last_term, my_last_index) = self.last_log_info();

        if last_term != my_last_term {
            last_term > my_last_term
        } else {
            last_index >= my_last_index
        }
    }

    fn last_log_info(&self) -> (u64, u64) {
        if let Some(last) = self.log.last() {
            (last.term, last.index)
        } else {
            (0, 0)
        }
    }

    /// Handle AppendEntries RPC
    fn handle_append_entries(
        &mut self,
        args: AppendEntriesArgs,
    ) -> AppendEntriesReply {
        if args.term > self.current_term {
            self.current_term = args.term;
            self.role = Role::Follower;
            self.voted_for = None;
        }

        let mut success = false;
        let mut conflict_index = None;

        if args.term == self.current_term {
            self.role = Role::Follower;
            self.last_heartbeat = Instant::now();

            // Check log consistency
            if self.log_matches(args.prev_log_index, args.prev_log_term) {
                success = true;

                // Append new entries
                let mut index = args.prev_log_index;
                for entry in args.entries {
                    index += 1;
                    if let Some(existing) =
                        self.log.get_mut(index as usize)
                    {
                        if existing.term != entry.term {
                            // Conflict: delete from here onward
                            self.log.truncate(index as usize);
                            self.log.push(entry.clone());
                        }
                    } else {
                        self.log.push(entry.clone());
                    }
                }

                // Update commit index
                if args.leader_commit > self.commit_index {
                    self.commit_index = std::cmp::min(
                        args.leader_commit,
                        self.log.len() as u64,
                    );
                }
            } else {
                // Find conflict index
                conflict_index = Some(args.prev_log_index);
            }
        }

        AppendEntriesReply {
            term: self.current_term,
            success,
            conflict_index,
        }
    }

    fn log_matches(&self, index: u64, term: u64) -> bool {
        if index == 0 {
            return true;
        }

        if let Some(entry) = self.log.get(index as usize - 1) {
            entry.term == term
        } else {
            false
        }
    }

    /// Start leader election
    fn start_election(&mut self) {
        self.current_term += 1;
        self.role = Role::Candidate;
        self.voted_for = Some(self.id);
        self.last_heartbeat = Instant::now();

        let (last_log_term, last_log_index) = self.last_log_info();

        let args = RequestVoteArgs {
            term: self.current_term,
            candidate_id: self.id,
            last_log_index,
            last_log_term,
        };

        // Send RequestVote RPCs to all peers
        // (In real implementation, send via network)
        println!(
            "Node {} starting election for term {}",
            self.id, self.current_term
        );
    }

    /// Become leader
    fn become_leader(&mut self) {
        self.role = Role::Leader;

        // Initialize leader state
        let next_index = self.log.len() as u64 + 1;
        for peer in &self.peers {
            self.next_index.insert(*peer, next_index);
            self.match_index.insert(*peer, 0);
        }

        println!("Node {} became leader for term {}", self.id, self.current_term);
    }

    /// Check if election timeout elapsed
    fn election_timeout_elapsed(&self) -> bool {
        self.last_heartbeat.elapsed() > self.election_timeout
    }
}

/// Simplified Raft example usage
async fn raft_example() {
    let mut node = RaftNode::new(1, vec![2, 3, 4, 5]);

    // Simulate election timeout
    node.start_election();

    // Simulate receiving votes (in real system, via network)
    let mut votes = 1; // Self-vote

    if votes > node.peers.len() / 2 {
        node.become_leader();
    }
}
```

### TypeScript: Basic Paxos (Conceptual)

```typescript
/**
 * Paxos proposer
 */
class Proposer {
  private proposalNumber = 0;
  private highestAccepted: {
    number: number;
    value: any;
  } | null = null;

  constructor(
    private id: number,
    private acceptors: Acceptor[]
  ) {}

  async propose(value: any): Promise<boolean> {
    // Phase 1: Prepare
    this.proposalNumber++;
    const promisesreceived = await this.sendPrepare(
      this.proposalNumber
    );

    const promises = promisesreceived.filter(p => p !== null);

    // Need majority
    if (promises.length < Math.floor(this.acceptors.length / 2) + 1) {
      return false; // Failed to get quorum
    }

    // Find highest accepted value
    let valueToPropose = value;
    for (const promise of promises) {
      if (
        promise.acceptedNumber !== null &&
        (this.highestAccepted === null ||
          promise.acceptedNumber > this.highestAccepted.number)
      ) {
        this.highestAccepted = {
          number: promise.acceptedNumber,
          value: promise.acceptedValue,
        };
        valueToPropose = promise.acceptedValue;
      }
    }

    // Phase 2: Accept
    const accepted = await this.sendAccept(
      this.proposalNumber,
      valueToPropose
    );

    const acceptCount = accepted.filter(a => a).length;

    // Need majority
    return acceptCount >= Math.floor(this.acceptors.length / 2) + 1;
  }

  private async sendPrepare(
    n: number
  ): Promise<PromiseResponse[]> {
    return Promise.all(
      this.acceptors.map(acceptor =>
        acceptor.receivePrepare(n).catch(() => null)
      )
    );
  }

  private async sendAccept(
    n: number,
    value: any
  ): Promise<boolean[]> {
    return Promise.all(
      this.acceptors.map(acceptor =>
        acceptor.receiveAccept(n, value).catch(() => false)
      )
    );
  }
}

/**
 * Paxos acceptor
 */
class Acceptor {
  private promisedNumber: number | null = null;
  private acceptedNumber: number | null = null;
  private acceptedValue: any = null;

  constructor(private id: number) {}

  async receivePrepare(n: number): Promise<PromiseResponse> {
    if (this.promisedNumber === null || n > this.promisedNumber) {
      this.promisedNumber = n;
      return {
        promised: true,
        acceptedNumber: this.acceptedNumber,
        acceptedValue: this.acceptedValue,
      };
    }

    return { promised: false, acceptedNumber: null, acceptedValue: null };
  }

  async receiveAccept(n: number, value: any): Promise<boolean> {
    if (this.promisedNumber === null || n >= this.promisedNumber) {
      this.promisedNumber = n;
      this.acceptedNumber = n;
      this.acceptedValue = value;
      return true; // Accepted
    }

    return false; // Rejected
  }

  getAcceptedValue(): any {
    return this.acceptedValue;
  }
}

interface PromiseResponse {
  promised: boolean;
  acceptedNumber: number | null;
  acceptedValue: any;
}

/**
 * Example: Run Paxos
 */
async function runPaxos() {
  // Create 5 acceptors
  const acceptors = [
    new Acceptor(1),
    new Acceptor(2),
    new Acceptor(3),
    new Acceptor(4),
    new Acceptor(5),
  ];

  // Create proposer
  const proposer = new Proposer(1, acceptors);

  // Propose value
  const success = await proposer.propose('my-value');

  if (success) {
    console.log('Consensus reached!');
  } else {
    console.log('Failed to reach consensus');
  }
}
```

### PHP: Simple Consensus Coordinator

```php
<?php

namespace Consensus;

/**
 * Simple leader election using database (similar to ZooKeeper)
 */
class LeaderElection
{
    private string $lockKey = 'leader_lock';
    private int $leaseSeconds = 10;

    public function tryBecomeLeader(string $nodeId): bool
    {
        // Try to acquire lock using Redis
        $acquired = Redis::set(
            $this->lockKey,
            $nodeId,
            'NX', // Only set if not exists
            'EX', // Expiration
            $this->leaseSeconds
        );

        return (bool) $acquired;
    }

    public function renewLease(string $nodeId): bool
    {
        $currentLeader = Redis::get($this->lockKey);

        if ($currentLeader === $nodeId) {
            Redis::expire($this->lockKey, $this->leaseSeconds);
            return true;
        }

        return false;
    }

    public function isLeader(string $nodeId): bool
    {
        return Redis::get($this->lockKey) === $nodeId;
    }

    public function getLeader(): ?string
    {
        return Redis::get($this->lockKey) ?: null;
    }
}

/**
 * Quorum-based configuration store
 */
class QuorumStore
{
    private array $nodes;
    private int $quorumSize;

    public function __construct(array $nodes)
    {
        $this->nodes = $nodes;
        $this->quorumSize = (int) floor(count($nodes) / 2) + 1;
    }

    public function write(string $key, mixed $value): bool
    {
        $successes = 0;
        $version = time(); // Simple versioning

        foreach ($this->nodes as $node) {
            if ($this->writeToNode($node, $key, $value, $version)) {
                $successes++;
            }

            if ($successes >= $this->quorumSize) {
                return true; // Quorum reached
            }
        }

        return false;
    }

    public function read(string $key): mixed
    {
        $responses = [];

        foreach ($this->nodes as $node) {
            $response = $this->readFromNode($node, $key);
            if ($response !== null) {
                $responses[] = $response;
            }

            if (count($responses) >= $this->quorumSize) {
                break;
            }
        }

        if (count($responses) < $this->quorumSize) {
            throw new \Exception('Failed to reach quorum for read');
        }

        // Return value with highest version
        usort($responses, fn($a, $b) => $b['version'] <=> $a['version']);

        return $responses[0]['value'];
    }

    private function writeToNode(
        string $node,
        string $key,
        mixed $value,
        int $version
    ): bool {
        // Simulate network write to node
        try {
            $response = Http::post("http://$node/write", [
                'key' => $key,
                'value' => $value,
                'version' => $version,
            ]);

            return $response->successful();
        } catch (\Exception $e) {
            return false;
        }
    }

    private function readFromNode(string $node, string $key): ?array
    {
        try {
            $response = Http::get("http://$node/read", [
                'key' => $key,
            ]);

            if ($response->successful()) {
                return $response->json();
            }
        } catch (\Exception $e) {
            // Node unavailable
        }

        return null;
    }
}
```

---

## Integration Points

### With Consistency Models
- **Consensus provides linearizability**: Agreed value visible to all
- **Log replication**: Consensus on operation order → consistent state
- **Quorum intersection**: Foundation for both consensus and consistency

### With Actor Model
- **Leader election**: Select coordinator actor in actor system
- **Distributed actors**: Consensus for cross-node actor coordination
- **Saga coordination**: Consensus on distributed transaction outcome

### With Streams
- **Kafka**: Log replication via quorum-based replication (Raft-like)
- **Stream partitioning**: Consensus on partition assignment
- **Watermarks**: Consensus on progress in distributed stream processing

---

## Further Reading

### Papers
- Fischer, Lynch, Paterson (1985) - "Impossibility of Distributed Consensus with One Faulty Process"
- Lamport (1998) - "The Part-Time Parliament" (Paxos)
- Ongaro & Ousterhout (2014) - "In Search of an Understandable Consensus Algorithm" (Raft)
- Castro & Liskov (1999) - "Practical Byzantine Fault Tolerance"

### Books
- Van Steen & Tanenbaum - "Distributed Systems" (3rd ed.)
- Kleppmann - "Designing Data-Intensive Applications"

---

**End of Consensus Appendix**
