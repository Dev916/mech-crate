# Dataflow Models: Deterministic Concurrent Computation

**Purpose**: Mathematical frameworks for modeling computation as data flowing through networks of interconnected processing nodes, enabling deterministic parallelism and compositional reasoning.

**Core Insight**: Computation is data transformation. Explicitly model data dependencies, achieving deterministic parallel execution and compositional system design.

---

## Table of Contents

1. [Foundational Concepts](#foundational-concepts)
2. [Kahn Process Networks](#kahn-process-networks)
3. [Synchronous Dataflow](#synchronous-dataflow)
4. [Flow-Based Programming](#flow-based-programming)
5. [Dataflow Languages and DSLs](#dataflow-languages-and-dsls)
6. [Practical Applications](#practical-applications)
7. [Stack-Specific Implementations](#stack-specific-implementations)
8. [Integration Points](#integration-points)

---

## Foundational Concepts

### What is Dataflow?

**Dataflow**: Computational model where execution is driven by data availability rather than control flow.

**Key Properties**:
1. **Data-Driven Execution**: Operations fire when inputs available
2. **Explicit Dependencies**: Data dependencies visible in graph structure
3. **Implicit Parallelism**: Independent operations execute concurrently
4. **Determinism**: Same inputs → same outputs (in most models)

### Dataflow vs. Control Flow

| Aspect | Control Flow | Dataflow |
|--------|-------------|----------|
| Execution Order | Sequential (program counter) | Data-driven (readiness) |
| Parallelism | Explicit (threads, async) | Implicit (data dependencies) |
| Dependencies | Hidden in code | Explicit in graph |
| Composition | Function calls | Graph wiring |
| Determinism | Not guaranteed | Often guaranteed |

### Dataflow Graph

**Components**:
- **Nodes**: Processing elements (functions, operators)
- **Edges**: Data channels (FIFO queues, streams)
- **Tokens**: Data values flowing through edges

**Execution**:
```
Node fires when:
  - All input channels have required tokens
  - Output channels have space

Firing:
  1. Consume input tokens
  2. Execute computation
  3. Produce output tokens
```

### Benefits of Dataflow

**Advantages**:
- **Automatic Parallelism**: Dependencies explicit, scheduler exploits
- **Determinism**: Reproducible results (critical for testing, debugging)
- **Modularity**: Nodes are composable, reusable black boxes
- **Visual Programming**: Graphs are intuitive representations
- **Optimization**: Static analysis of graph enables compile-time optimization

**Challenges**:
- **State Management**: Handling mutable state requires care
- **Deadlock**: Cycles with insufficient buffering can deadlock
- **Memory Overhead**: Buffering tokens between nodes
- **Debugging**: Harder to trace execution than sequential code

---

## Kahn Process Networks

**Developed by**: Gilles Kahn (1974)

**Key Idea**: Network of **deterministic** sequential processes communicating through **unbounded FIFO channels**. Produces **deterministic** results regardless of execution order.

### KPN Formal Model

**Definition**:
- **Processes**: Sequential computations
- **Channels**: Unbounded FIFO queues
- **Operations**: `read(channel)` (blocking), `write(channel, value)` (non-blocking)

**Determinism Theorem (Kahn 1974)**:

For any KPN:
1. **Determinism**: Output sequences depend only on input sequences, not execution timing
2. **Monotonicity**: Partial results never retracted (outputs append-only)
3. **Continuity**: Finite outputs computed from finite prefixes of inputs

**Proof Sketch**: Process sees input channel as stream. Blocking read ensures process waits for data. Since channels unbounded, writes never block on full buffer. Thus, process execution determined solely by input arrival order, which is fixed.

### KPN Properties

**Determinism**: Given same inputs, KPN produces same outputs regardless of:
- Process scheduling order
- Execution speed of individual processes
- Parallel vs. sequential execution

**Monotonicity**: Once a value output, it's never retracted. Allows incremental computation.

**Deadlock**: KPN can deadlock if:
- Process blocks reading from empty channel, no process will write to it
- Circular dependency with finite buffers (though KPN assumes unbounded)

**Boundedness**: Channel buffer size may grow unboundedly. Implementation challenge.

### Example: Stream Processing Pipeline

```
[Generator] → channel_a → [Filter] → channel_b → [Mapper] → channel_c → [Sink]

Generator:
  for i in 1..∞:
    write(channel_a, i)

Filter:
  while true:
    val = read(channel_a)
    if val % 2 == 0:
      write(channel_b, val)

Mapper:
  while true:
    val = read(channel_b)
    write(channel_c, val * 2)

Sink:
  while true:
    result = read(channel_c)
    print(result)
```

**Determinism**: Given same generator sequence, output always same.

**Parallelism**: Filter, Mapper, Sink can run concurrently.

### Parks' Bounded Buffer Problem

**Problem**: KPN assumes unbounded channels. Real systems have finite memory.

**Parks' Algorithm** (1995): Implements KPN with bounded buffers:
- Maintain minimum buffer sizes to avoid artificial deadlock
- Dynamically expand buffers when necessary
- Detect "real" deadlock (cyclic wait) vs. "buffer deadlock"

**Trade-off**: Bounded buffers may introduce artificial deadlock if too small. Parks' algorithm minimizes this.

### KPN Scheduling

**Goal**: Determine process execution order to maximize throughput, minimize buffer usage.

**Strategies**:
1. **Static Scheduling**: Compile-time order (requires static rates)
2. **Dynamic Scheduling**: Runtime decisions based on data availability
3. **Quasi-Static Scheduling**: Hybrid - static with runtime adaptations

**SDF** (Synchronous Dataflow, next section) enables efficient static scheduling.

---

## Synchronous Dataflow

**Developed by**: Lee & Messerschmitt (1987)

**Key Idea**: Restrict KPN by specifying **fixed** number of tokens consumed/produced per firing. Enables **static scheduling** and **bounded memory**.

### SDF Formal Model

**SDF Actor**:
- **Ports**: Named input/output channels
- **Consumption Rates**: Fixed tokens consumed per port per firing
- **Production Rates**: Fixed tokens produced per port per firing
- **Firing Rule**: Fire when all inputs have enough tokens and outputs have space

**Notation**:
```
Actor A:
  inputs: in1 (consume 2), in2 (consume 1)
  outputs: out1 (produce 3)

Firing: Consumes 2 from in1, 1 from in2, produces 3 to out1
```

### SDF Balance Equations

For SDF graph to execute in **bounded memory** with **consistent** firing:

**Topology Matrix** Γ:
- Rows = channels
- Columns = actors
- Entry Γ[c,a] = (tokens produced by a on c) - (tokens consumed by a on c)

**Balance Equation**: Γq = 0

Where q = firing vector (number of times each actor fires per iteration).

**Example**:
```
[A] --2--> [B] --3--> [C]
     3 tokens/fire  6 tokens/fire

A produces 2, B consumes 2: Balance for A→B channel
B produces 3, C consumes 3: Balance for B→C channel

Firing vector: q_A, q_B, q_C
Balance equations:
  2*q_A - 2*q_B = 0  =>  q_A = q_B
  3*q_B - 3*q_C = 0  =>  q_B = q_C

Solution: q_A = q_B = q_C = k (any integer)
Minimal: q = [1, 1, 1]
```

### Static Scheduling

**Given**: SDF graph with balance equations satisfied

**Goal**: Find execution order that:
1. Respects data dependencies
2. Minimizes buffer sizes
3. Executes in bounded memory

**Algorithm**:
1. Compute firing vector q from Γq = 0
2. Topologically sort actors respecting dependencies
3. Schedule q[a] firings of each actor a in order

**Example Schedule**:
```
Graph: [A(1,2)] → [B(2,3)] → [C(3,1)]

Firing vector: q = [6, 3, 2]  (GCD normalization)

Schedule: A A A A A A B B B C C

Bounded memory: Predictable buffer sizes
```

### SDF Extensions

**Cyclo-Static Dataflow (CSDF)**:
- Rates vary in fixed cycle pattern
- Extends SDF expressiveness while keeping static analysis

**Boolean Dataflow (BDF)**:
- Conditional execution (if-then-else)
- Rates depend on token values (dynamic)
- Loses static schedulability

**Parameterized Dataflow**:
- Rates parameterized by constants
- Static analysis with parameters
- Enables reusable components

### SDF Applications

**Signal Processing**:
- Audio/video codecs
- Wireless communication (modulation, filtering)
- Radar signal processing

**Embedded Systems**:
- Real-time constraints
- Resource-constrained devices
- Deterministic timing

**Streaming Applications**:
- Video streaming pipelines
- Data analytics streams
- Network packet processing

---

## Flow-Based Programming

**Developed by**: J. Paul Morrison (1970s)

**Key Idea**: Applications are **networks of black-box processes** exchanging **data packets** (Information Packets, IPs) over **bounded connections**.

### FBP Concepts

**Components**:
- **Processes**: Reusable, concurrent black boxes
- **Information Packets (IPs)**: Data with ownership semantics
- **Connections**: Bounded FIFO queues between ports
- **Ports**: Named input/output connection points

**Process Lifecycle**:
```
1. Wait for input IPs
2. Process IPs (read, transform, create new)
3. Send output IPs
4. Repeat or terminate
```

### FBP vs. KPN/SDF

| Feature | KPN | SDF | FBP |
|---------|-----|-----|-----|
| Channels | Unbounded | Bounded (static) | Bounded (configurable) |
| Rates | Dynamic | Static | Dynamic |
| Determinism | Guaranteed | Guaranteed | Not guaranteed (depends) |
| Scheduling | Dynamic | Static | Dynamic |
| State | Process-local | Process-local | Process-local + IP data |

### FBP Patterns

**Generator**:
```
Process: DataGenerator
Outputs: out

Loop:
  data = generateData()
  send(out, data)
```

**Filter**:
```
Process: Filter
Inputs: in
Outputs: out

Loop:
  ip = receive(in)
  if predicate(ip):
    send(out, ip)
```

**Transformer**:
```
Process: Mapper
Inputs: in
Outputs: out

Loop:
  ip = receive(in)
  result = transform(ip)
  send(out, result)
```

**Splitter**:
```
Process: Splitter
Inputs: in
Outputs: out1, out2

Loop:
  ip = receive(in)
  if condition(ip):
    send(out1, ip)
  else:
    send(out2, ip)
```

**Merger**:
```
Process: Merger
Inputs: in1, in2
Outputs: out

Loop:
  select:
    case ip1 = receive(in1):
      send(out, ip1)
    case ip2 = receive(in2):
      send(out, ip2)
```

### FBP Example: Text Processing

```
[FileReader] → [Splitter] → [Counter] → [Sorter] → [Writer]
                    ↓
                [Filter] → [Formatter] → [Writer2]

FileReader: Read file line by line, emit IPs
Splitter: Split lines into words, emit word IPs
Counter: Count word frequencies, emit (word, count) IPs
Sorter: Sort by count descending
Writer: Write top 10 to file

Filter: Filter by word length > 3
Formatter: Format as JSON
Writer2: Write filtered results
```

### Information Packets (IPs)

**Ownership**: IP owned by one process at a time. Ownership transfers on send/receive.

**Types**:
- **Normal IP**: Carries data payload
- **Bracket IP**: Mark start/end of substream (hierarchy)
- **Empty IP**: Signal without data (e.g., end-of-stream)

**Bracket IPs**:
```
Process: GroupCounter
Inputs: in
Outputs: out

count = 0
Loop:
  ip = receive(in)
  if ip is OpenBracket:
    count = 0
  elif ip is CloseBracket:
    send(out, count)
  else:
    count += 1
```

---

## Dataflow Languages and DSLs

### Lustre: Synchronous Dataflow Language

**Lustre**: Declarative language for reactive systems (synchronous model).

**Example**:
```lustre
node counter(reset: bool) returns (count: int);
let
  count = 0 -> if reset then 0 else pre(count) + 1;
tel

node edge_detector(x: bool) returns (edge: bool);
let
  edge = false -> x and not pre(x);
tel
```

**Semantics**: Clock-driven. All variables updated simultaneously each tick.

**Applications**: Safety-critical embedded systems (avionics, automotive).

### LabVIEW: Visual Dataflow Programming

**LabVIEW**: Graphical programming environment using dataflow (called G language).

**Features**:
- Visual block diagram (nodes and wires)
- Parallel loops execute concurrently
- Built-in data acquisition and instrument control

**Use Cases**: Test/measurement, embedded control, FPGA programming.

### StreamIt: Stream Processing Language

**StreamIt**: High-level language for streaming applications.

**Constructs**:
```streamit
filter Doubler() {
  work pop 1 push 1 {
    push(2 * pop());
  }
}

pipeline DoublerPipeline {
  add DataSource();
  add Doubler();
  add DataSink();
}
```

**Compiler**: Optimizes for multi-core, generates C code.

---

## Practical Applications

### 1. Video Processing Pipeline

**Problem**: Decode video, apply filters, encode to multiple resolutions.

**Solution**: Dataflow pipeline with parallel branches.

```
[Decoder] → [ColorCorrect] → [Split] → [Resize_720p] → [Encode_720p]
                               ↓
                           [Resize_1080p] → [Encode_1080p]
                               ↓
                           [Resize_4K] → [Encode_4K]

Each node = process
Edges = bounded FIFO queues
Parallel execution of resize/encode stages
```

### 2. Machine Learning Inference Pipeline

**Problem**: Real-time inference on video stream with preprocessing.

**Solution**: SDF graph for deterministic latency.

```
[Camera] → [Preprocess] → [Model Inference] → [Postprocess] → [Display]

Static schedule:
  - Preprocess: 30 FPS (consume 1 frame, produce 1 tensor)
  - Inference: 10 FPS (consume 1 tensor, produce 1 result)
  - Postprocess: 10 FPS (consume 1 result, produce 1 frame)

Buffer between stages to handle rate mismatch
```

### 3. ETL Data Pipeline

**Problem**: Extract, transform, load large datasets.

**Solution**: FBP network with reusable components.

```
[CSVReader] → [Validator] → [Transformer] → [Enricher] → [DBWriter]
                  ↓                            ↑
              [ErrorLog]                  [APIFetch]

Components:
- CSVReader: Generator process
- Validator: Filter invalid records
- Transformer: Map fields
- Enricher: Join with external data (APIFetch)
- DBWriter: Sink process
- ErrorLog: Handle validation failures
```

### 4. Real-Time Audio Effects

**Problem**: Apply audio effects (reverb, EQ) in real-time with low latency.

**Solution**: SDF graph for predictable timing.

```
[AudioInput] → [FFT] → [EQ] → [Reverb] → [IFFT] → [AudioOutput]

SDF rates:
- AudioInput: 1 sample/fire
- FFT: 512 samples in, 512 frequency bins out
- EQ: 512 bins in, 512 bins out
- Reverb: 512 bins in, 512 bins out
- IFFT: 512 bins in, 512 samples out

Static schedule ensures deterministic latency
```

---

## Stack-Specific Implementations

### Rust: KPN with Async Channels

```rust
use tokio::sync::mpsc;
use tokio::task;

/// Kahn Process Network node trait
#[async_trait::async_trait]
trait KPNProcess: Send {
    async fn run(self: Box<Self>);
}

/// Example: Generator process
struct Generator {
    output: mpsc::UnboundedSender<i32>,
    count: usize,
}

#[async_trait::async_trait]
impl KPNProcess for Generator {
    async fn run(self: Box<Self>) {
        for i in 0..self.count {
            // Non-blocking write (unbounded channel)
            self.output.send(i as i32).unwrap();
        }
    }
}

/// Example: Filter process
struct Filter {
    input: mpsc::UnboundedReceiver<i32>,
    output: mpsc::UnboundedSender<i32>,
    predicate: fn(i32) -> bool,
}

#[async_trait::async_trait]
impl KPNProcess for Filter {
    async fn run(mut self: Box<Self>) {
        while let Some(value) = self.input.recv().await {
            // Blocking read
            if (self.predicate)(value) {
                self.output.send(value).unwrap();
            }
        }
    }
}

/// Example: Mapper process
struct Mapper {
    input: mpsc::UnboundedReceiver<i32>,
    output: mpsc::UnboundedSender<i32>,
    transform: fn(i32) -> i32,
}

#[async_trait::async_trait]
impl KPNProcess for Mapper {
    async fn run(mut self: Box<Self>) {
        while let Some(value) = self.input.recv().await {
            let result = (self.transform)(value);
            self.output.send(result).unwrap();
        }
    }
}

/// Build and run KPN
async fn kpn_example() {
    let (gen_tx, gen_rx) = mpsc::unbounded_channel();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel();
    let (map_tx, map_rx) = mpsc::unbounded_channel();

    // Create processes
    let generator: Box<dyn KPNProcess> = Box::new(Generator {
        output: gen_tx,
        count: 100,
    });

    let filter: Box<dyn KPNProcess> = Box::new(Filter {
        input: gen_rx,
        output: filter_tx,
        predicate: |x| x % 2 == 0, // Even numbers only
    });

    let mapper: Box<dyn KPNProcess> = Box::new(Mapper {
        input: filter_rx,
        output: map_tx,
        transform: |x| x * 2,
    });

    // Spawn concurrent tasks
    let gen_handle = task::spawn(async move { generator.run().await });
    let filter_handle = task::spawn(async move { filter.run().await });
    let mapper_handle = task::spawn(async move { mapper.run().await });

    // Collect results
    task::spawn(async move {
        let mut receiver = map_rx;
        while let Some(value) = receiver.recv().await {
            println!("Result: {}", value);
        }
    });

    // Wait for completion
    let _ = tokio::join!(gen_handle, filter_handle, mapper_handle);
}

/// SDF Actor trait
trait SDFActor {
    /// Number of tokens consumed from each input port per firing
    fn consumption_rates(&self) -> Vec<usize>;

    /// Number of tokens produced to each output port per firing
    fn production_rates(&self) -> Vec<usize>;

    /// Fire actor: consume inputs, produce outputs
    fn fire(&mut self, inputs: Vec<Vec<i32>>) -> Vec<Vec<i32>>;
}

/// Example: SDF Upsampler (rate conversion)
struct Upsampler {
    factor: usize,
}

impl SDFActor for Upsampler {
    fn consumption_rates(&self) -> Vec<usize> {
        vec![1] // Consume 1 token per firing
    }

    fn production_rates(&self) -> Vec<usize> {
        vec![self.factor] // Produce 'factor' tokens per firing
    }

    fn fire(&mut self, inputs: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
        let input = &inputs[0];
        let value = input[0];

        // Repeat value 'factor' times
        vec![vec![value; self.factor]]
    }
}

/// SDF Graph executor
struct SDFGraph {
    actors: Vec<Box<dyn SDFActor>>,
    topology: Vec<(usize, usize)>, // (src_actor, dst_actor) edges
    buffers: Vec<Vec<i32>>,         // Buffer for each edge
}

impl SDFGraph {
    fn new(actors: Vec<Box<dyn SDFActor>>, topology: Vec<(usize, usize)>) -> Self {
        let buffers = vec![Vec::new(); topology.len()];
        SDFGraph {
            actors,
            topology,
            buffers,
        }
    }

    /// Execute one iteration of static schedule
    fn execute_iteration(&mut self) {
        // Simplified: fire each actor in order
        // Real implementation would compute firing vector and schedule

        for (actor_idx, actor) in self.actors.iter_mut().enumerate() {
            // Gather inputs from incoming edges
            let mut inputs: Vec<Vec<i32>> = Vec::new();

            for (edge_idx, (src, dst)) in self.topology.iter().enumerate() {
                if *dst == actor_idx {
                    let required = actor.consumption_rates()[0];
                    if self.buffers[edge_idx].len() >= required {
                        let consumed: Vec<i32> = self.buffers[edge_idx]
                            .drain(0..required)
                            .collect();
                        inputs.push(consumed);
                    }
                }
            }

            if !inputs.is_empty() {
                // Fire actor
                let outputs = actor.fire(inputs);

                // Distribute outputs to outgoing edges
                for (edge_idx, (src, _dst)) in self.topology.iter().enumerate() {
                    if *src == actor_idx {
                        self.buffers[edge_idx].extend(&outputs[0]);
                    }
                }
            }
        }
    }
}
```

### TypeScript: FBP with Streams

```typescript
import { Readable, Writable, Transform } from 'stream';

/**
 * Information Packet
 */
interface IP<T> {
  type: 'data' | 'open-bracket' | 'close-bracket';
  data?: T;
}

/**
 * FBP Process base class
 */
abstract class FBPProcess<TIn, TOut> extends Transform {
  constructor() {
    super({ objectMode: true });
  }

  protected abstract process(ip: IP<TIn>): IP<TOut>[] | null;

  _transform(
    chunk: any,
    encoding: string,
    callback: Function
  ): void {
    const ip = chunk as IP<TIn>;
    const outputs = this.process(ip);

    if (outputs) {
      outputs.forEach(out => this.push(out));
    }

    callback();
  }
}

/**
 * Generator process
 */
class Generator extends Readable {
  private count: number = 0;
  private max: number;

  constructor(max: number) {
    super({ objectMode: true });
    this.max = max;
  }

  _read(): void {
    if (this.count < this.max) {
      const ip: IP<number> = {
        type: 'data',
        data: this.count++,
      };
      this.push(ip);
    } else {
      this.push(null); // End of stream
    }
  }
}

/**
 * Filter process
 */
class FilterProcess extends FBPProcess<number, number> {
  constructor(private predicate: (x: number) => boolean) {
    super();
  }

  protected process(ip: IP<number>): IP<number>[] | null {
    if (ip.type === 'data' && ip.data !== undefined) {
      if (this.predicate(ip.data)) {
        return [ip];
      }
    }
    return null;
  }
}

/**
 * Mapper process
 */
class MapperProcess extends FBPProcess<number, number> {
  constructor(private transform: (x: number) => number) {
    super();
  }

  protected process(ip: IP<number>): IP<number>[] | null {
    if (ip.type === 'data' && ip.data !== undefined) {
      return [{
        type: 'data',
        data: this.transform(ip.data),
      }];
    }
    return null;
  }
}

/**
 * Sink process
 */
class Sink extends Writable {
  private results: any[] = [];

  constructor() {
    super({ objectMode: true });
  }

  _write(
    chunk: any,
    encoding: string,
    callback: Function
  ): void {
    const ip = chunk as IP<any>;
    if (ip.type === 'data') {
      this.results.push(ip.data);
      console.log('Received:', ip.data);
    }
    callback();
  }

  getResults(): any[] {
    return this.results;
  }
}

/**
 * Build FBP network
 */
function buildFBPNetwork() {
  const generator = new Generator(10);
  const filter = new FilterProcess(x => x % 2 === 0);
  const mapper = new MapperProcess(x => x * 2);
  const sink = new Sink();

  // Wire network
  generator
    .pipe(filter)
    .pipe(mapper)
    .pipe(sink);

  return { generator, sink };
}

/**
 * SDF Actor interface
 */
interface SDFActor<TIn, TOut> {
  name: string;
  consumptionRates: number[];
  productionRates: number[];
  fire(inputs: TIn[][]): TOut[][];
}

/**
 * Example: SDF Downsampler
 */
class Downsampler implements SDFActor<number, number> {
  name = 'Downsampler';

  constructor(private factor: number) {}

  get consumptionRates(): number[] {
    return [this.factor]; // Consume 'factor' tokens
  }

  get productionRates(): number[] {
    return [1]; // Produce 1 token
  }

  fire(inputs: number[][]): number[][] {
    // Take first sample from each group
    const value = inputs[0][0];
    return [[value]];
  }
}

/**
 * SDF Graph
 */
class SDFGraph {
  private actors: SDFActor<any, any>[] = [];
  private topology: [number, number][] = []; // (src, dst) edges
  private buffers: any[][] = [];

  addActor(actor: SDFActor<any, any>): number {
    this.actors.push(actor);
    return this.actors.length - 1;
  }

  connect(src: number, dst: number): void {
    this.topology.push([src, dst]);
    this.buffers.push([]);
  }

  executeIteration(): void {
    // Simplified: fire each actor in order
    for (let actorIdx = 0; actorIdx < this.actors.length; actorIdx++) {
      const actor = this.actors[actorIdx];

      // Gather inputs
      const inputs: any[][] = [];
      for (let edgeIdx = 0; edgeIdx < this.topology.length; edgeIdx++) {
        const [src, dst] = this.topology[edgeIdx];
        if (dst === actorIdx) {
          const required = actor.consumptionRates[0];
          if (this.buffers[edgeIdx].length >= required) {
            inputs.push(this.buffers[edgeIdx].splice(0, required));
          }
        }
      }

      if (inputs.length > 0) {
        // Fire actor
        const outputs = actor.fire(inputs);

        // Distribute outputs
        for (let edgeIdx = 0; edgeIdx < this.topology.length; edgeIdx++) {
          const [src, dst] = this.topology[edgeIdx];
          if (src === actorIdx) {
            this.buffers[edgeIdx].push(...outputs[0]);
          }
        }
      }
    }
  }
}
```

### PHP: Dataflow with Laravel Queues

```php
<?php

namespace Dataflow;

use Illuminate\Support\Facades\Queue;
use Illuminate\Support\Facades\Redis;

/**
 * KPN Process abstraction using Laravel queues
 */
abstract class KPNProcess
{
    protected string $processId;
    protected array $inputQueues = [];
    protected array $outputQueues = [];

    public function __construct(string $id)
    {
        $this->processId = $id;
    }

    abstract protected function process(array $inputs): array;

    public function run(): void
    {
        while (true) {
            // Blocking read from input queues (KPN semantics)
            $inputs = $this->readInputs();

            if (empty($inputs)) {
                sleep(1); // Wait for data
                continue;
            }

            // Process data
            $outputs = $this->process($inputs);

            // Non-blocking write to output queues
            $this->writeOutputs($outputs);
        }
    }

    protected function readInputs(): array
    {
        $inputs = [];
        foreach ($this->inputQueues as $queueName) {
            $value = Queue::pop($queueName);
            if ($value !== null) {
                $inputs[$queueName] = $value;
            }
        }
        return $inputs;
    }

    protected function writeOutputs(array $outputs): void
    {
        foreach ($outputs as $queueName => $value) {
            if (in_array($queueName, $this->outputQueues)) {
                Queue::push($queueName, $value);
            }
        }
    }
}

/**
 * Generator process
 */
class Generator extends KPNProcess
{
    private int $count = 0;
    private int $max;

    public function __construct(string $id, int $max)
    {
        parent::__construct($id);
        $this->max = $max;
        $this->outputQueues = ['generator_out'];
    }

    public function run(): void
    {
        while ($this->count < $this->max) {
            $this->writeOutputs([
                'generator_out' => $this->count++
            ]);
            usleep(100000); // 100ms
        }
    }

    protected function process(array $inputs): array
    {
        return [];
    }
}

/**
 * Filter process
 */
class FilterProcess extends KPNProcess
{
    private $predicate;

    public function __construct(string $id, callable $predicate)
    {
        parent::__construct($id);
        $this->predicate = $predicate;
        $this->inputQueues = ['filter_in'];
        $this->outputQueues = ['filter_out'];
    }

    protected function process(array $inputs): array
    {
        $value = $inputs['filter_in'] ?? null;

        if ($value !== null && ($this->predicate)($value)) {
            return ['filter_out' => $value];
        }

        return [];
    }
}

/**
 * Mapper process
 */
class MapperProcess extends KPNProcess
{
    private $transform;

    public function __construct(string $id, callable $transform)
    {
        parent::__construct($id);
        $this->transform = $transform;
        $this->inputQueues = ['mapper_in'];
        $this->outputQueues = ['mapper_out'];
    }

    protected function process(array $inputs): array
    {
        $value = $inputs['mapper_in'] ?? null;

        if ($value !== null) {
            $result = ($this->transform)($value);
            return ['mapper_out' => $result];
        }

        return [];
    }
}

/**
 * SDF Actor interface
 */
interface SDFActor
{
    public function getName(): string;
    public function getConsumptionRates(): array;
    public function getProductionRates(): array;
    public function fire(array $inputs): array;
}

/**
 * Example: SDF FIR Filter
 */
class FIRFilter implements SDFActor
{
    private array $coefficients;
    private array $buffer = [];

    public function __construct(array $coefficients)
    {
        $this->coefficients = $coefficients;
    }

    public function getName(): string
    {
        return 'FIRFilter';
    }

    public function getConsumptionRates(): array
    {
        return [1]; // Consume 1 sample per firing
    }

    public function getProductionRates(): array
    {
        return [1]; // Produce 1 sample per firing
    }

    public function fire(array $inputs): array
    {
        $sample = $inputs[0][0];

        // Add to buffer
        array_unshift($this->buffer, $sample);

        // Keep only needed samples
        $this->buffer = array_slice(
            $this->buffer,
            0,
            count($this->coefficients)
        );

        // Compute filter output
        $output = 0;
        foreach ($this->coefficients as $i => $coef) {
            if (isset($this->buffer[$i])) {
                $output += $coef * $this->buffer[$i];
            }
        }

        return [[$output]];
    }
}

/**
 * SDF Graph executor
 */
class SDFGraph
{
    private array $actors = [];
    private array $topology = []; // [[src, dst]]
    private array $buffers = [];

    public function addActor(SDFActor $actor): int
    {
        $this->actors[] = $actor;
        return count($this->actors) - 1;
    }

    public function connect(int $src, int $dst): void
    {
        $this->topology[] = [$src, $dst];
        $this->buffers[] = [];
    }

    public function executeIteration(): void
    {
        foreach ($this->actors as $actorIdx => $actor) {
            // Gather inputs
            $inputs = [];

            foreach ($this->topology as $edgeIdx => [$src, $dst]) {
                if ($dst === $actorIdx) {
                    $required = $actor->getConsumptionRates()[0];

                    if (count($this->buffers[$edgeIdx]) >= $required) {
                        $consumed = array_splice(
                            $this->buffers[$edgeIdx],
                            0,
                            $required
                        );
                        $inputs[] = $consumed;
                    }
                }
            }

            if (!empty($inputs)) {
                // Fire actor
                $outputs = $actor->fire($inputs);

                // Distribute outputs
                foreach ($this->topology as $edgeIdx => [$src, $dst]) {
                    if ($src === $actorIdx) {
                        array_push(
                            $this->buffers[$edgeIdx],
                            ...$outputs[0]
                        );
                    }
                }
            }
        }
    }

    public function getBuffers(): array
    {
        return $this->buffers;
    }
}

/**
 * Example: Audio processing pipeline
 */
function createAudioPipeline(): SDFGraph
{
    $graph = new SDFGraph();

    // Create actors
    $firFilter = new FIRFilter([0.2, 0.5, 0.3]); // Lowpass
    $filterId = $graph->addActor($firFilter);

    // Add more actors and connections as needed

    return $graph;
}
```

---

## Integration Points

### With Actor Model
- **Dataflow nodes can be actors**: Each actor has input/output ports
- **Message passing = data tokens**: Actors communicate via dataflow edges
- **Supervision**: Restart failed dataflow nodes

**Example**: Akka Streams implements dataflow on top of Akka actors.

### With FRP
- **FRP networks are dataflow graphs**: Signals flow through operators
- **Synchronous dataflow**: FRP with discrete time steps
- **Behaviors and events**: Continuous (behavior) vs. discrete (event) dataflow

**Example**: FRP switch operator is dataflow demultiplexer.

### With Process Calculi
- **KPN is process calculus**: Processes + FIFO channels = π-calculus variant
- **SDF has process algebra**: Composition operators with rate annotations
- **Session types**: Specify dataflow channel protocols

**Example**: Prove KPN determinism using process calculus semantics.

### With Streaming Systems
- **Streaming frameworks use dataflow**: Kafka Streams, Apache Flink are dataflow engines
- **Backpressure**: Flow control in bounded dataflow channels
- **Windowing**: Time-based grouping in dataflow

**Example**: Flink's DataStream API is dataflow with windows.

---

## Further Reading

### Foundational Papers
- Kahn (1974) - "The Semantics of a Simple Language for Parallel Programming"
- Lee & Messerschmitt (1987) - "Synchronous Data Flow"
- Dennis (1974) - "First Version of a Data Flow Procedure Language"
- Morrison (1978) - "Flow-Based Programming"

### Books
- Lee & Seshia - "Introduction to Embedded Systems: A Cyber-Physical Systems Approach"
- Johnston, Hanna, Millar - "Advances in Dataflow Programming Languages"
- Morrison - "Flow-Based Programming: A New Approach to Application Development"

### Tools & Frameworks
- **Apache Flink** - Stream processing with dataflow model
- **TensorFlow** - ML framework using dataflow graphs
- **LabVIEW** - Visual dataflow programming for test/measurement
- **Node-RED** - Flow-based IoT programming
- **Ptolemy II** - Heterogeneous modeling framework (includes SDF)

---

**End of Dataflow Models Appendix**
