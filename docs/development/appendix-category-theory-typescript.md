# Category Theory in TypeScript

Practical implementations of category theory concepts in TypeScript.

## Table of Contents

1. [Functors and Monads](#functors-and-monads)
2. [Comonads](#comonads)
3. [Monad Transformers](#monad-transformers)
4. [Optics (Lenses and Prisms)](#optics-lenses-and-prisms)
5. [Free Monads and Interpreters](#free-monads-and-interpreters)

---

## Functors and Monads

### Functor Interface

```typescript
/**
 * Functor interface
 */
interface Functor<A> {
  map<B>(f: (a: A) => B): Functor<B>;
}

/**
 * Option functor
 */
class Option<A> implements Functor<A> {
  private constructor(private value: A | null) {}

  static some<A>(value: A): Option<A> {
    return new Option(value);
  }

  static none<A>(): Option<A> {
    return new Option<A>(null);
  }

  map<B>(f: (a: A) => B): Option<B> {
    if (this.value === null) {
      return Option.none();
    }
    return Option.some(f(this.value));
  }

  flatMap<B>(f: (a: A) => Option<B>): Option<B> {
    if (this.value === null) {
      return Option.none();
    }
    return f(this.value);
  }

  getOrElse(defaultValue: A): A {
    return this.value ?? defaultValue;
  }

  isNone(): boolean {
    return this.value === null;
  }

  isSome(): boolean {
    return this.value !== null;
  }
}

/**
 * Monad interface
 */
interface Monad<A> extends Functor<A> {
  flatMap<B>(f: (a: A) => Monad<B>): Monad<B>;
}

/**
 * Either monad (for error handling)
 */
type Either<E, A> =
  | { type: 'left'; value: E }
  | { type: 'right'; value: A };

class EitherMonad<E, A> implements Monad<A> {
  constructor(private value: Either<E, A>) {}

  static left<E, A>(error: E): EitherMonad<E, A> {
    return new EitherMonad({ type: 'left', value: error });
  }

  static right<E, A>(value: A): EitherMonad<E, A> {
    return new EitherMonad({ type: 'right', value });
  }

  map<B>(f: (a: A) => B): EitherMonad<E, B> {
    if (this.value.type === 'left') {
      return EitherMonad.left(this.value.value);
    }
    return EitherMonad.right(f(this.value.value));
  }

  flatMap<B>(f: (a: A) => EitherMonad<E, B>): EitherMonad<E, B> {
    if (this.value.type === 'left') {
      return EitherMonad.left(this.value.value);
    }
    return f(this.value.value);
  }

  fold<B>(onLeft: (e: E) => B, onRight: (a: A) => B): B {
    return this.value.type === 'left'
      ? onLeft(this.value.value)
      : onRight(this.value.value);
  }

  isLeft(): boolean {
    return this.value.type === 'left';
  }

  isRight(): boolean {
    return this.value.type === 'right';
  }
}

/**
 * Example: Monadic error handling
 */
function divide(x: number, y: number): EitherMonad<string, number> {
  if (y === 0) {
    return EitherMonad.left("Division by zero");
  }
  return EitherMonad.right(x / y);
}

function compute(): EitherMonad<string, number> {
  return divide(10, 2)
    .flatMap(x => divide(x, 0))  // Error here
    .flatMap(y => divide(y, 2));  // Not executed
}

const result = compute().fold(
  error => `Error: ${error}`,
  value => `Result: ${value}`
);
// "Error: Division by zero"
```

---

## Comonads

### Comonad Interface

```typescript
/**
 * Comonad interface
 */
interface Comonad<A> {
  extract(): A;
  extend<B>(f: (wa: Comonad<A>) => B): Comonad<B>;
}
```

### Stream Comonad

```typescript
/**
 * Stream Comonad - infinite list with focus
 */
class Stream<A> implements Comonad<A> {
  constructor(
    private head: A,
    private tailFn: () => Stream<A>
  ) {}

  extract(): A {
    return this.head;
  }

  tail(): Stream<A> {
    return this.tailFn();
  }

  duplicate(): Stream<Stream<A>> {
    return new Stream(
      this,
      () => this.tail().duplicate()
    );
  }

  extend<B>(f: (s: Stream<A>) => B): Stream<B> {
    return new Stream(
      f(this),
      () => this.tail().extend(f)
    );
  }

  take(n: number): A[] {
    const result: A[] = [this.head];
    let current = this as Stream<A>;
    for (let i = 1; i < n; i++) {
      current = current.tail();
      result.push(current.head);
    }
    return result;
  }

  static repeat<A>(value: A): Stream<A> {
    return new Stream(value, () => Stream.repeat(value));
  }

  static iterate<A>(initial: A, f: (a: A) => A): Stream<A> {
    return new Stream(initial, () => Stream.iterate(f(initial), f));
  }

  static from<A>(array: A[], defaultValue: A): Stream<A> {
    const helper = (index: number): Stream<A> => {
      const value = array[index] ?? defaultValue;
      return new Stream(value, () => helper(index + 1));
    };
    return helper(0);
  }
}

/**
 * Example: Moving average with Stream comonad
 */
function movingAverage3(stream: Stream<number>): number {
  const values = stream.take(3);
  return values.reduce((a, b) => a + b, 0) / values.length;
}

function streamExample() {
  // Infinite stream of naturals
  const naturals = Stream.iterate(1, n => n + 1);
  console.log('First 10:', naturals.take(10));
  // [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

  // Moving average
  const data = Stream.iterate(10, n => n + 10);
  const smoothed = data.extend(movingAverage3);
  console.log('Moving averages:', smoothed.take(5));

  // Time series analysis
  const prices = Stream.from([100, 102, 98, 105, 110], 0);
  const avgPrices = prices.extend(movingAverage3);
  console.log('Average prices:', avgPrices.take(3));
}
```

### Store Comonad

```typescript
/**
 * Store Comonad - position in space
 */
class Store<S, A> implements Comonad<A> {
  constructor(
    private getter: (s: S) => A,
    private position: S
  ) {}

  extract(): A {
    return this.getter(this.position);
  }

  peek(pos: S): A {
    return this.getter(pos);
  }

  seek(pos: S): Store<S, A> {
    return new Store(this.getter, pos);
  }

  duplicate(): Store<S, Store<S, A>> {
    return new Store(
      (s: S) => new Store(this.getter, s),
      this.position
    );
  }

  extend<B>(f: (store: Store<S, A>) => B): Store<S, B> {
    return new Store(
      (s: S) => f(new Store(this.getter, s)),
      this.position
    );
  }

  experiment<F>(functor: (pos: S) => F[]): F[] {
    return functor(this.position).map(p => this.peek(p as unknown as S));
  }
}

/**
 * Example: Game of Life with Store comonad
 */
type Pos = readonly [number, number];

function neighbors([x, y]: Pos): Pos[] {
  const result: Pos[] = [];
  for (let dx = -1; dx <= 1; dx++) {
    for (let dy = -1; dy <= 1; dy++) {
      if (dx !== 0 || dy !== 0) {
        result.push([x + dx, y + dy]);
      }
    }
  }
  return result;
}

function gameOfLifeStep(grid: Store<Pos, boolean>): boolean {
  const aliveNeighbors = neighbors(grid['position'])
    .filter(pos => grid.peek(pos))
    .length;

  const current = grid.extract();
  if (current && (aliveNeighbors === 2 || aliveNeighbors === 3)) {
    return true;
  }
  if (!current && aliveNeighbors === 3) {
    return true;
  }
  return false;
}

function storeExample() {
  // Glider pattern
  const initialState = ([x, y]: Pos): boolean => {
    const alive: Pos[] = [[1, 0], [2, 1], [0, 2], [1, 2], [2, 2]];
    return alive.some(([ax, ay]) => ax === x && ay === y);
  };

  const grid = new Store(initialState, [1, 1] as Pos);
  console.log('Current cell:', grid.extract());

  // Evolve one step
  const nextGen = grid.extend(gameOfLifeStep);
  console.log('Next generation:', nextGen.extract());

  // Evolve multiple generations
  let current = grid;
  for (let i = 0; i < 5; i++) {
    current = current.extend(gameOfLifeStep);
    console.log(`Generation ${i + 1} at (1,1):`, current.extract());
  }
}
```

### Env Comonad

```typescript
/**
 * Env Comonad - value with environment
 */
class Env<E, A> implements Comonad<A> {
  constructor(
    private env: E,
    private value: A
  ) {}

  extract(): A {
    return this.value;
  }

  ask(): E {
    return this.env;
  }

  duplicate(): Env<E, Env<E, A>> {
    return new Env(this.env, this);
  }

  extend<B>(f: (env: Env<E, A>) => B): Env<E, B> {
    return new Env(this.env, f(this));
  }

  local<B>(f: (e: E) => E, computation: (env: Env<E, A>) => Env<E, B>): Env<E, B> {
    const newEnv = new Env(f(this.env), this.value);
    return computation(newEnv);
  }
}

/**
 * Example: Theming with Env comonad
 */
type Theme = 'light' | 'dark';

interface ThemeConfig {
  primaryColor: string;
  backgroundColor: string;
  textColor: string;
}

const themes: Record<Theme, ThemeConfig> = {
  light: {
    primaryColor: '#007bff',
    backgroundColor: '#ffffff',
    textColor: '#000000',
  },
  dark: {
    primaryColor: '#0d6efd',
    backgroundColor: '#1a1a1a',
    textColor: '#ffffff',
  },
};

interface Widget {
  content: string;
  type: 'button' | 'text' | 'input';
}

function renderWithTheme(themed: Env<Theme, Widget>): string {
  const theme = themed.ask();
  const widget = themed.extract();
  const config = themes[theme];

  return `<${widget.type}
    style="
      background: ${config.backgroundColor};
      color: ${config.textColor};
      border-color: ${config.primaryColor};
    ">
    ${widget.content}
  </${widget.type}>`;
}

function envExample() {
  const widget: Widget = { content: 'Click Me', type: 'button' };
  const themed = new Env<Theme, Widget>('dark', widget);
  const rendered = themed.extend(renderWithTheme);
  console.log('Rendered:', rendered.extract());
  // HTML with dark theme styles
}
```

---

## Monad Transformers

### OptionT - Add Option to Promise

```typescript
/**
 * OptionT - adds Option semantics to Promise
 */
class OptionT<A> {
  constructor(private inner: Promise<Option<A>>) {}

  static some<A>(value: A): OptionT<A> {
    return new OptionT(Promise.resolve(Option.some(value)));
  }

  static none<A>(): OptionT<A> {
    return new OptionT(Promise.resolve(Option.none<A>()));
  }

  static fromPromise<A>(promise: Promise<A>): OptionT<A> {
    return new OptionT(promise.then(v => Option.some(v)));
  }

  async unwrap(): Promise<Option<A>> {
    return this.inner;
  }

  map<B>(f: (a: A) => B): OptionT<B> {
    return new OptionT(
      this.inner.then(opt => opt.map(f))
    );
  }

  flatMap<B>(f: (a: A) => OptionT<B>): OptionT<B> {
    return new OptionT(
      this.inner.then(async opt => {
        if (opt.isNone()) {
          return Option.none<B>();
        }
        const nextOpt = await f(opt.getOrElse(null as any)).unwrap();
        return nextOpt;
      })
    );
  }

  async getOrElse(defaultValue: A): Promise<A> {
    const opt = await this.inner;
    return opt.getOrElse(defaultValue);
  }
}

/**
 * Example: Database operations with OptionT
 */
interface User {
  id: number;
  name: string;
  email: string;
}

async function findUser(id: number): Promise<OptionT<User>> {
  // Simulate database query
  return new Promise(resolve => {
    setTimeout(() => {
      if (id === 1) {
        resolve(OptionT.some({ id: 1, name: 'Alice', email: 'alice@example.com' }));
      } else {
        resolve(OptionT.none());
      }
    }, 100);
  });
}

async function getUserEmail(id: number): Promise<OptionT<string>> {
  const userOpt = await findUser(id);
  return userOpt.map(user => user.email);
}

async function optionTExample() {
  const email = await getUserEmail(1);
  const result = await email.getOrElse('no-email@example.com');
  console.log('Email:', result);
}
```

### StateT - Add State to Promise

```typescript
/**
 * StateT - adds State semantics to Promise
 */
class StateT<S, A> {
  constructor(
    private run: (state: S) => Promise<readonly [A, S]>
  ) {}

  static pure<S, A>(value: A): StateT<S, A> {
    return new StateT(state => Promise.resolve([value, state] as const));
  }

  static get<S>(): StateT<S, S> {
    return new StateT(state => Promise.resolve([state, state] as const));
  }

  static put<S>(newState: S): StateT<S, void> {
    return new StateT(_ => Promise.resolve([undefined, newState] as const));
  }

  static modify<S>(f: (s: S) => S): StateT<S, void> {
    return new StateT(state => Promise.resolve([undefined, f(state)] as const));
  }

  async runState(initialState: S): Promise<readonly [A, S]> {
    return this.run(initialState);
  }

  async evalState(initialState: S): Promise<A> {
    const [value, _] = await this.run(initialState);
    return value;
  }

  async execState(initialState: S): Promise<S> {
    const [_, state] = await this.run(initialState);
    return state;
  }

  map<B>(f: (a: A) => B): StateT<S, B> {
    return new StateT(async state => {
      const [value, newState] = await this.run(state);
      return [f(value), newState] as const;
    });
  }

  flatMap<B>(f: (a: A) => StateT<S, B>): StateT<S, B> {
    return new StateT(async state => {
      const [value, newState] = await this.run(state);
      return f(value).run(newState);
    });
  }
}

/**
 * Example: Counter with async operations
 */
interface CounterState {
  count: number;
  history: number[];
}

function increment(): StateT<CounterState, void> {
  return StateT.modify(state => ({
    count: state.count + 1,
    history: [...state.history, state.count + 1],
  }));
}

function incrementAsync(): StateT<CounterState, void> {
  return new StateT(async state => {
    await new Promise(resolve => setTimeout(resolve, 100));
    return [
      undefined,
      {
        count: state.count + 1,
        history: [...state.history, state.count + 1],
      },
    ] as const;
  });
}

function getCount(): StateT<CounterState, number> {
  return StateT.get<CounterState>().map(state => state.count);
}

async function stateTExample() {
  const program = increment()
    .flatMap(() => incrementAsync())
    .flatMap(() => increment())
    .flatMap(() => getCount());

  const initialState: CounterState = { count: 0, history: [] };
  const [finalCount, finalState] = await program.runState(initialState);

  console.log('Final count:', finalCount);  // 3
  console.log('History:', finalState.history);  // [1, 2, 3]
}
```

### ReaderT - Add Reader to Promise

```typescript
/**
 * ReaderT - adds Reader (environment) semantics to Promise
 */
class ReaderT<R, A> {
  constructor(private run: (env: R) => Promise<A>) {}

  static pure<R, A>(value: A): ReaderT<R, A> {
    return new ReaderT(_ => Promise.resolve(value));
  }

  static ask<R>(): ReaderT<R, R> {
    return new ReaderT(env => Promise.resolve(env));
  }

  static asks<R, A>(f: (env: R) => A): ReaderT<R, A> {
    return new ReaderT(env => Promise.resolve(f(env)));
  }

  async runReader(env: R): Promise<A> {
    return this.run(env);
  }

  map<B>(f: (a: A) => B): ReaderT<R, B> {
    return new ReaderT(async env => {
      const value = await this.run(env);
      return f(value);
    });
  }

  flatMap<B>(f: (a: A) => ReaderT<R, B>): ReaderT<R, B> {
    return new ReaderT(async env => {
      const value = await this.run(env);
      return f(value).run(env);
    });
  }

  local<B>(f: (r: R) => R): ReaderT<R, A> {
    return new ReaderT(env => this.run(f(env)));
  }
}

/**
 * Example: Application with configuration
 */
interface AppConfig {
  apiUrl: string;
  apiKey: string;
  timeout: number;
}

function fetchUser(id: number): ReaderT<AppConfig, User> {
  return ReaderT.asks<AppConfig, User>(config => {
    // Simulate API call
    console.log(`Fetching from ${config.apiUrl}/users/${id}`);
    return { id, name: `User${id}`, email: `user${id}@example.com` };
  });
}

function fetchUserWithRetry(id: number): ReaderT<AppConfig, User> {
  return ReaderT.ask<AppConfig>().flatMap(config => {
    return new ReaderT(async env => {
      console.log(`Retry timeout: ${env.timeout}ms`);
      // Simulate retry logic
      await new Promise(resolve => setTimeout(resolve, env.timeout));
      return { id, name: `User${id}`, email: `user${id}@example.com` };
    });
  });
}

async function readerTExample() {
  const config: AppConfig = {
    apiUrl: 'https://api.example.com',
    apiKey: 'secret-key',
    timeout: 1000,
  };

  const user = await fetchUser(1).runReader(config);
  console.log('User:', user);

  const userWithRetry = await fetchUserWithRetry(2).runReader(config);
  console.log('User with retry:', userWithRetry);
}
```

---

## Optics (Lenses and Prisms)

### Lens - Focus on Product Types

```typescript
/**
 * Lens for accessing nested fields
 */
class Lens<S, A> {
  constructor(
    private getter: (s: S) => A,
    private setter: (s: S, a: A) => S
  ) {}

  static fromProp<S, K extends keyof S>(prop: K): Lens<S, S[K]> {
    return new Lens(
      s => s[prop],
      (s, a) => ({ ...s, [prop]: a })
    );
  }

  view(source: S): A {
    return this.getter(source);
  }

  set(source: S, value: A): S {
    return this.setter(source, value);
  }

  over(source: S, f: (a: A) => A): S {
    return this.setter(source, f(this.getter(source)));
  }

  compose<B>(other: Lens<A, B>): Lens<S, B> {
    return new Lens(
      s => other.view(this.view(s)),
      (s, b) => this.set(s, other.set(this.view(s), b))
    );
  }
}

/**
 * Example: Person with nested Address
 */
interface Address {
  street: string;
  city: string;
  zipCode: string;
}

interface Person {
  name: string;
  age: number;
  address: Address;
}

const nameLens = Lens.fromProp<Person, 'name'>('name');
const ageLens = Lens.fromProp<Person, 'age'>('age');
const addressLens = Lens.fromProp<Person, 'address'>('address');
const cityLens = Lens.fromProp<Address, 'city'>('city');

// Compose lenses for nested access
const personCityLens = addressLens.compose(cityLens);

function lensExample() {
  const person: Person = {
    name: 'Alice',
    age: 30,
    address: {
      street: '123 Main St',
      city: 'Boston',
      zipCode: '02101',
    },
  };

  // View through lens
  const name = nameLens.view(person);
  console.log('Name:', name);  // "Alice"

  // Set through lens
  const person2 = ageLens.set(person, 31);
  console.log('New age:', person2.age);  // 31

  // Modify through lens
  const person3 = nameLens.over(person, n => n.toUpperCase());
  console.log('Uppercase name:', person3.name);  // "ALICE"

  // Composed lens
  const city = personCityLens.view(person);
  console.log('City:', city);  // "Boston"

  const person4 = personCityLens.set(person, 'New York');
  console.log('New city:', person4.address.city);  // "New York"
}
```

### Prism - Focus on Sum Types

```typescript
/**
 * Prism for accessing sum type variants
 */
class Prism<S, A> {
  constructor(
    private matcher: (s: S) => A | null,
    private builder: (a: A) => S
  ) {}

  preview(source: S): A | null {
    return this.matcher(source);
  }

  review(value: A): S {
    return this.builder(value);
  }

  modify(source: S, f: (a: A) => A): S | null {
    const matched = this.matcher(source);
    if (matched === null) return null;
    return this.builder(f(matched));
  }

  compose<B>(other: Prism<A, B>): Prism<S, B> {
    return new Prism(
      s => {
        const a = this.matcher(s);
        return a === null ? null : other.matcher(a);
      },
      b => this.builder(other.builder(b))
    );
  }
}

/**
 * Example: Result type prisms
 */
type Result<T, E> =
  | { tag: 'success'; value: T }
  | { tag: 'error'; error: E };

function success<T, E>(value: T): Result<T, E> {
  return { tag: 'success', value };
}

function error<T, E>(err: E): Result<T, E> {
  return { tag: 'error', error: err };
}

const successPrism = <T, E>(): Prism<Result<T, E>, T> =>
  new Prism(
    r => (r.tag === 'success' ? r.value : null),
    v => success(v)
  );

const errorPrism = <T, E>(): Prism<Result<T, E>, E> =>
  new Prism(
    r => (r.tag === 'error' ? r.error : null),
    e => error(e)
  );

function prismExample() {
  const result1: Result<number, string> = success(42);
  const result2: Result<number, string> = error('Not found');

  // Preview through prism
  const value = successPrism<number, string>().preview(result1);
  console.log('Success value:', value);  // 42

  const errValue = successPrism<number, string>().preview(result2);
  console.log('Error through success prism:', errValue);  // null

  // Review (construct) through prism
  const newSuccess = successPrism<number, string>().review(100);
  console.log('Constructed:', newSuccess);  // { tag: 'success', value: 100 }

  // Modify through prism
  const modified = successPrism<number, string>().modify(result1, n => n * 2);
  console.log('Modified:', modified);  // { tag: 'success', value: 84 }
}
```

---

## Free Monads and Interpreters

### Free Monad DSL

```typescript
/**
 * Free monad for building DSLs
 */
type Free<F, A> =
  | { type: 'pure'; value: A }
  | { type: 'free'; fa: F };

function pure<F, A>(value: A): Free<F, A> {
  return { type: 'pure', value };
}

function free<F, A>(fa: F): Free<F, A> {
  return { type: 'free', fa };
}

/**
 * File operation DSL
 */
type FileOp<A> =
  | { tag: 'read'; path: string; next: (content: string) => A }
  | { tag: 'write'; path: string; content: string; next: A }
  | { tag: 'delete'; path: string; next: A };

type FileM<A> = Free<FileOp<FileM<A>>, A>;

/**
 * DSL constructors
 */
function readFile(path: string): FileM<string> {
  return free({
    tag: 'read',
    path,
    next: (content: string) => pure(content),
  });
}

function writeFile(path: string, content: string): FileM<void> {
  return free({
    tag: 'write',
    path,
    content,
    next: pure(undefined),
  });
}

function deleteFile(path: string): FileM<void> {
  return free({
    tag: 'delete',
    path,
    next: pure(undefined),
  });
}

/**
 * Bind operation for Free monad
 */
function bind<F, A, B>(fa: Free<F, A>, f: (a: A) => Free<F, B>): Free<F, B> {
  if (fa.type === 'pure') {
    return f(fa.value);
  }
  // For full implementation, need to map over F
  return fa as any;
}

/**
 * Example program
 */
function copyFileProgram(src: string, dest: string): FileM<void> {
  return free({
    tag: 'read',
    path: src,
    next: (content: string) =>
      free({
        tag: 'write',
        path: dest,
        content,
        next: pure(undefined),
      }),
  });
}

/**
 * Interpreter - execute in console
 */
function interpretConsole<A>(program: FileM<A>): A {
  if (program.type === 'pure') {
    return program.value;
  }

  const op = program.fa;
  switch (op.tag) {
    case 'read':
      console.log(`Reading from: ${op.path}`);
      const content = `Content of ${op.path}`;
      return interpretConsole(op.next(content));

    case 'write':
      console.log(`Writing to: ${op.path}`);
      console.log(`Content: ${op.content}`);
      return interpretConsole(op.next);

    case 'delete':
      console.log(`Deleting: ${op.path}`);
      return interpretConsole(op.next);
  }
}

/**
 * Async interpreter
 */
async function interpretAsync<A>(program: FileM<A>): Promise<A> {
  if (program.type === 'pure') {
    return program.value;
  }

  const op = program.fa;
  switch (op.tag) {
    case 'read':
      // Simulate async file read
      await new Promise(resolve => setTimeout(resolve, 100));
      console.log(`[Async] Reading from: ${op.path}`);
      const content = `Content of ${op.path}`;
      return interpretAsync(op.next(content));

    case 'write':
      await new Promise(resolve => setTimeout(resolve, 100));
      console.log(`[Async] Writing to: ${op.path}`);
      return interpretAsync(op.next);

    case 'delete':
      await new Promise(resolve => setTimeout(resolve, 100));
      console.log(`[Async] Deleting: ${op.path}`);
      return interpretAsync(op.next);
  }
}

function freeMon adExample() {
  const program = copyFileProgram('input.txt', 'output.txt');
  interpretConsole(program);
}

async function freeMonadAsyncExample() {
  const program = copyFileProgram('async-input.txt', 'async-output.txt');
  await interpretAsync(program);
}
```

---

**End of TypeScript Category Theory Implementations**
