# Category Theory in Rust

Practical implementations of category theory concepts in Rust.

## Table of Contents

1. [Functors and Monads](#functors-and-monads)
2. [Comonads](#comonads)
3. [Monad Transformers](#monad-transformers)
4. [Optics (Lenses and Prisms)](#optics-lenses-and-prisms)
5. [Free Monads and Interpreters](#free-monads-and-interpreters)

---

## Functors and Monads

### Functor Trait

```rust
/// Functor trait (manual, Rust has no HKT)
trait Functor<A> {
    type Mapped<B>;
    fn map<B, F>(self, f: F) -> Self::Mapped<B>
    where
        F: FnOnce(A) -> B;
}

/// Option functor
impl<A> Functor<A> for Option<A> {
    type Mapped<B> = Option<B>;

    fn map<B, F>(self, f: F) -> Option<B>
    where
        F: FnOnce(A) -> B,
    {
        match self {
            Some(a) => Some(f(a)),
            None => None,
        }
    }
}

/// Result functor
impl<A, E> Functor<A> for Result<A, E> {
    type Mapped<B> = Result<B, E>;

    fn map<B, F>(self, f: F) -> Result<B, E>
    where
        F: FnOnce(A) -> B,
    {
        match self {
            Ok(a) => Ok(f(a)),
            Err(e) => Err(e),
        }
    }
}

/// Monad operations (built-in via ? and combinators)
fn example_monad() -> Result<i32, String> {
    // Monadic composition with ?
    let x = parse("5")?;
    let y = parse("10")?;
    Ok(x + y)
}

fn parse(s: &str) -> Result<i32, String> {
    s.parse().map_err(|_| format!("Parse error: {}", s))
}

/// Natural transformation
trait NaturalTransformation<F, G> {
    fn transform<A>(fa: F) -> G;
}

/// List to Option
struct ListToOption;

impl NaturalTransformation<Vec<i32>, Option<i32>> for ListToOption {
    fn transform(fa: Vec<i32>) -> Option<i32> {
        fa.into_iter().next()
    }
}
```

---

## Comonads

### Comonad Trait

```rust
/// Comonad trait
trait Comonad {
    type Value;
    type Context;

    fn extract(&self) -> Self::Value;
    fn duplicate(&self) -> Self::Context;
    fn extend<B, F>(&self, f: F) -> Self::Context
    where
        F: Fn(&Self) -> B;
}
```

### Stream Comonad

```rust
/// Stream Comonad - infinite list with focus
#[derive(Clone)]
struct Stream<T: Clone> {
    head: T,
    tail: Box<dyn Fn() -> Stream<T>>,
}

impl<T: Clone> Stream<T> {
    fn new(head: T, tail: Box<dyn Fn() -> Stream<T>>) -> Self {
        Stream { head, tail }
    }

    /// Get current element
    fn extract(&self) -> T {
        self.head.clone()
    }

    /// Get all suffixes
    fn duplicate(&self) -> Stream<Stream<T>> {
        let current = self.clone();
        let next_streams = Box::new({
            let tail = self.tail.clone();
            move || (*tail)().duplicate()
        });
        Stream::new(current, next_streams)
    }

    /// Extend function over stream
    fn extend<U: Clone, F>(&self, f: F) -> Stream<U>
    where
        F: Fn(&Stream<T>) -> U + Clone + 'static,
    {
        let result = f(self);
        let f_clone = f.clone();
        let next = Box::new({
            let tail = self.tail.clone();
            move || (*tail)().extend(f_clone.clone())
        });
        Stream::new(result, next)
    }

    /// Take first n elements
    fn take(&self, n: usize) -> Vec<T> {
        let mut result = vec![self.head.clone()];
        let mut current = self.clone();
        for _ in 1..n {
            current = (*current.tail)();
            result.push(current.head.clone());
        }
        result
    }
}

/// Example: Moving average with Stream comonad
fn moving_average_3(stream: &Stream<f64>) -> f64 {
    let values = stream.take(3);
    values.iter().sum::<f64>() / values.len() as f64
}

fn stream_example() {
    // Create stream of integers
    fn integers_from(n: i32) -> Stream<i32> {
        Stream::new(n, Box::new(move || integers_from(n + 1)))
    }

    let numbers = integers_from(1);
    let first_10 = numbers.take(10);
    println!("First 10: {:?}", first_10);  // [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

    // Moving average using extend
    let data = Stream::new(
        10.0,
        Box::new(|| Stream::new(
            20.0,
            Box::new(|| Stream::new(
                30.0,
                Box::new(|| Stream::new(40.0, Box::new(|| Stream::new(50.0, Box::new(|| panic!())))))
            ))
        ))
    );

    let smoothed = data.extend(moving_average_3);
    let result = smoothed.take(3);
    println!("Smoothed: {:?}", result);  // Moving averages
}
```

### Store Comonad

```rust
/// Store Comonad - position in space
#[derive(Clone)]
struct Store<S: Clone, A> {
    getter: fn(S) -> A,
    position: S,
}

impl<S: Clone, A: Clone> Store<S, A> {
    fn new(getter: fn(S) -> A, position: S) -> Self {
        Store { getter, position }
    }

    /// Extract value at current position
    fn extract(&self) -> A {
        (self.getter)(self.position.clone())
    }

    /// Duplicate into Store of Stores
    fn duplicate(&self) -> Store<S, Store<S, A>> {
        let getter = self.getter;
        Store {
            getter: move |s| Store::new(getter, s),
            position: self.position.clone(),
        }
    }

    /// Extend function over Store
    fn extend<B, F>(&self, f: F) -> Store<S, B>
    where
        F: Fn(&Store<S, A>) -> B,
    {
        let getter = self.getter;
        let f_copy = move |s: S| {
            let store = Store::new(getter, s);
            f(&store)
        };
        Store {
            getter: f_copy,
            position: self.position.clone(),
        }
    }

    /// Peek at different position
    fn peek(&self, pos: S) -> A {
        (self.getter)(pos)
    }

    /// Move to new position
    fn seek(self, pos: S) -> Self {
        Store {
            getter: self.getter,
            position: pos,
        }
    }
}

/// Example: Conway's Game of Life with Store comonad
type Pos = (i32, i32);

fn neighbors((x, y): Pos) -> Vec<Pos> {
    let mut result = Vec::new();
    for dx in -1..=1 {
        for dy in -1..=1 {
            if dx != 0 || dy != 0 {
                result.push((x + dx, y + dy));
            }
        }
    }
    result
}

fn game_of_life_step(grid: &Store<Pos, bool>) -> bool {
    let alive_neighbors = neighbors(grid.position)
        .iter()
        .filter(|&&pos| grid.peek(pos))
        .count();

    let current = grid.extract();
    match (current, alive_neighbors) {
        (true, 2) | (true, 3) => true,
        (false, 3) => true,
        _ => false,
    }
}

fn store_example() {
    // Simple grid - glider pattern
    let initial_state = |pos: Pos| match pos {
        (1, 0) | (2, 1) | (0, 2) | (1, 2) | (2, 2) => true,
        _ => false,
    };

    let grid = Store::new(initial_state, (1, 1));
    println!("Current cell alive: {}", grid.extract());

    // Evolve one step
    let next_gen = grid.extend(game_of_life_step);
    println!("Next gen at (1,1): {}", next_gen.extract());
}
```

### Env Comonad

```rust
/// Env Comonad - value with environment
#[derive(Clone)]
struct Env<E: Clone, A> {
    env: E,
    value: A,
}

impl<E: Clone, A: Clone> Env<E, A> {
    fn new(env: E, value: A) -> Self {
        Env { env, value }
    }

    /// Extract value
    fn extract(&self) -> A {
        self.value.clone()
    }

    /// Get environment
    fn ask(&self) -> E {
        self.env.clone()
    }

    /// Duplicate with environment
    fn duplicate(&self) -> Env<E, Env<E, A>> {
        Env {
            env: self.env.clone(),
            value: self.clone(),
        }
    }

    /// Extend function over Env
    fn extend<B, F>(&self, f: F) -> Env<E, B>
    where
        F: Fn(&Env<E, A>) -> B,
    {
        Env {
            env: self.env.clone(),
            value: f(self),
        }
    }
}

#[derive(Clone, Debug)]
enum Theme {
    Light,
    Dark,
}

#[derive(Clone)]
struct Widget {
    content: String,
}

fn render_with_theme(themed: &Env<Theme, Widget>) -> String {
    let theme = themed.ask();
    let widget = themed.extract();
    match theme {
        Theme::Light => format!("[Light] {}", widget.content),
        Theme::Dark => format!("[Dark] {}", widget.content),
    }
}

fn env_example() {
    let widget = Widget {
        content: "Button".to_string(),
    };

    let themed = Env::new(Theme::Dark, widget);
    let rendered = themed.extend(render_with_theme);
    println!("Rendered: {}", rendered.extract());
}
```

---

## Monad Transformers

Rust doesn't have higher-kinded types, so monad transformers require workarounds using associated types or type aliases.

### OptionT - Add Option to any Result

```rust
use std::error::Error;
use std::fmt;

/// OptionT - adds Option semantics to Result
struct OptionT<T, E> {
    inner: Result<Option<T>, E>,
}

impl<T, E> OptionT<T, E> {
    fn new(inner: Result<Option<T>, E>) -> Self {
        OptionT { inner }
    }

    fn some(value: T) -> Self {
        OptionT {
            inner: Ok(Some(value)),
        }
    }

    fn none() -> Self {
        OptionT { inner: Ok(None) }
    }

    fn from_result(result: Result<T, E>) -> Self {
        OptionT {
            inner: result.map(Some),
        }
    }

    fn map<U, F>(self, f: F) -> OptionT<U, E>
    where
        F: FnOnce(T) -> U,
    {
        OptionT {
            inner: self.inner.map(|opt| opt.map(f)),
        }
    }

    fn and_then<U, F>(self, f: F) -> OptionT<U, E>
    where
        F: FnOnce(T) -> OptionT<U, E>,
    {
        OptionT {
            inner: match self.inner {
                Ok(Some(val)) => f(val).inner,
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            },
        }
    }

    fn unwrap(self) -> Result<Option<T>, E> {
        self.inner
    }
}

#[derive(Debug)]
struct DbError(String);

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Database error: {}", self.0)
    }
}

impl Error for DbError {}

/// Example: Database operations with OptionT
fn find_user(id: i32) -> OptionT<String, DbError> {
    if id < 0 {
        OptionT::from_result(Err(DbError("Invalid ID".to_string())))
    } else if id == 0 {
        OptionT::none()  // User not found
    } else {
        OptionT::some(format!("User{}", id))
    }
}

fn get_email(username: String) -> OptionT<String, DbError> {
    if username == "User1" {
        OptionT::some("user1@example.com".to_string())
    } else {
        OptionT::none()
    }
}

fn optiont_example() -> OptionT<String, DbError> {
    find_user(1)
        .and_then(|username| {
            println!("Found user: {}", username);
            get_email(username)
        })
        .map(|email| {
            println!("Email: {}", email);
            email
        })
}
```

### StateT - Add State to Result

```rust
/// StateT - adds State semantics to Result
struct StateT<S, T, E> {
    run: Box<dyn FnOnce(S) -> Result<(T, S), E>>,
}

impl<S: 'static, T: 'static, E: 'static> StateT<S, T, E> {
    fn new<F>(run: F) -> Self
    where
        F: FnOnce(S) -> Result<(T, S), E> + 'static,
    {
        StateT { run: Box::new(run) }
    }

    fn pure(value: T) -> Self {
        StateT::new(|s| Ok((value, s)))
    }

    fn get() -> StateT<S, S, E>
    where
        S: Clone,
    {
        StateT::new(|s| Ok((s.clone(), s)))
    }

    fn put(new_state: S) -> StateT<S, (), E> {
        StateT::new(|_| Ok(((), new_state)))
    }

    fn map<U, F>(self, f: F) -> StateT<S, U, E>
    where
        F: FnOnce(T) -> U + 'static,
    {
        StateT::new(move |s| {
            (self.run)(s).map(|(a, s2)| (f(a), s2))
        })
    }

    fn and_then<U, F>(self, f: F) -> StateT<S, U, E>
    where
        F: FnOnce(T) -> StateT<S, U, E> + 'static,
    {
        StateT::new(move |s| {
            match (self.run)(s) {
                Ok((a, s2)) => (f(a).run)(s2),
                Err(e) => Err(e),
            }
        })
    }

    fn run_state(self, initial: S) -> Result<(T, S), E> {
        (self.run)(initial)
    }
}

/// Example: Stateful counter with error handling
fn increment() -> StateT<i32, (), String> {
    StateT::new(|count| {
        if count >= 100 {
            Err("Counter overflow".to_string())
        } else {
            Ok(((), count + 1))
        }
    })
}

fn get_count() -> StateT<i32, i32, String> {
    StateT::get()
}

fn statet_example() -> StateT<i32, i32, String> {
    increment()
        .and_then(|_| increment())
        .and_then(|_| increment())
        .and_then(|_| get_count())
}

fn run_statet() {
    let result = statet_example().run_state(0);
    match result {
        Ok((final_count, state)) => {
            println!("Final count: {}, State: {}", final_count, state);
        }
        Err(e) => println!("Error: {}", e),
    }
}
```

---

## Optics (Lenses and Prisms)

### Lens - Focus on Product Types

```rust
use std::marker::PhantomData;

/// Lens for accessing nested fields
struct Lens<S, T, A, B> {
    getter: Box<dyn Fn(&S) -> A>,
    setter: Box<dyn Fn(S, B) -> T>,
    _phantom: PhantomData<(A, B)>,
}

impl<S, T, A, B> Lens<S, T, A, B> {
    fn new<G, SE>(getter: G, setter: SE) -> Self
    where
        G: Fn(&S) -> A + 'static,
        SE: Fn(S, B) -> T + 'static,
    {
        Lens {
            getter: Box::new(getter),
            setter: Box::new(setter),
            _phantom: PhantomData,
        }
    }

    /// View through lens
    fn view(&self, source: &S) -> A {
        (self.getter)(source)
    }

    /// Set through lens
    fn set(&self, source: S, value: B) -> T {
        (self.setter)(source, value)
    }

    /// Modify through lens
    fn over<F>(&self, source: S, f: F) -> T
    where
        F: FnOnce(A) -> B,
        S: Clone,
    {
        let old_value = self.view(&source);
        let new_value = f(old_value);
        self.set(source, new_value)
    }
}

// Simple lens type for same input/output types
type Lens_<S, A> = Lens<S, S, A, A>;

/// Example: Person with Address
#[derive(Clone, Debug)]
struct Address {
    street: String,
    city: String,
}

#[derive(Clone, Debug)]
struct Person {
    name: String,
    age: i32,
    address: Address,
}

impl Person {
    /// Lens for name field
    fn name_lens() -> Lens_<Person, String> {
        Lens::new(
            |p| p.name.clone(),
            |mut p, n| {
                p.name = n;
                p
            },
        )
    }

    /// Lens for age field
    fn age_lens() -> Lens_<Person, i32> {
        Lens::new(
            |p| p.age,
            |mut p, a| {
                p.age = a;
                p
            },
        )
    }

    /// Lens for address field
    fn address_lens() -> Lens_<Person, Address> {
        Lens::new(
            |p| p.address.clone(),
            |mut p, a| {
                p.address = a;
                p
            },
        )
    }
}

impl Address {
    /// Lens for city field
    fn city_lens() -> Lens_<Address, String> {
        Lens::new(
            |a| a.city.clone(),
            |mut a, c| {
                a.city = c;
                a
            },
        )
    }
}

fn lens_example() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        address: Address {
            street: "123 Main St".to_string(),
            city: "Boston".to_string(),
        },
    };

    // View through lens
    let name = Person::name_lens().view(&person);
    println!("Name: {}", name);  // "Alice"

    // Set through lens
    let person2 = Person::age_lens().set(person.clone(), 31);
    println!("New age: {}", person2.age);  // 31

    // Modify through lens
    let person3 = Person::name_lens().over(person.clone(), |n| n.to_uppercase());
    println!("Uppercase name: {}", person3.name);  // "ALICE"

    // Compose lenses for nested access
    let city = Person::address_lens().view(&person).city;
    println!("City: {}", city);  // "Boston"
}
```

### Prism - Focus on Sum Types

```rust
/// Prism for accessing sum type variants
enum Result_<T, E> {
    Ok_(T),
    Err_(E),
}

struct Prism<S, T, A, B> {
    matcher: Box<dyn Fn(S) -> Result_<A, S>>,
    builder: Box<dyn Fn(B) -> T>,
}

impl<S, T, A, B> Prism<S, T, A, B> {
    fn new<M, Bu>(matcher: M, builder: Bu) -> Self
    where
        M: Fn(S) -> Result_<A, S> + 'static,
        Bu: Fn(B) -> T + 'static,
    {
        Prism {
            matcher: Box::new(matcher),
            builder: Box::new(builder),
        }
    }

    /// Try to match and extract value
    fn preview(&self, source: S) -> Option<A> {
        match (self.matcher)(source) {
            Result_::Ok_(a) => Some(a),
            Result_::Err_(_) => None,
        }
    }

    /// Build value from prism
    fn review(&self, value: B) -> T {
        (self.builder)(value)
    }

    /// Modify if match succeeds
    fn modify<F>(&self, source: S, f: F) -> T
    where
        F: FnOnce(A) -> B,
        S: Clone,
    {
        match (self.matcher)(source.clone()) {
            Result_::Ok_(a) => self.review(f(a)),
            Result_::Err_(_) => panic!("Prism match failed"),
        }
    }
}

type Prism_<S, A> = Prism<S, S, A, A>;

/// Example: Result prisms
#[derive(Clone, Debug)]
enum ApiResult<T> {
    Success(T),
    Error(String),
}

impl<T: Clone> ApiResult<T> {
    /// Prism for Success variant
    fn success_prism() -> Prism_<ApiResult<T>, T> {
        Prism::new(
            |r| match r {
                ApiResult::Success(t) => Result_::Ok_(t),
                ApiResult::Error(_) => Result_::Err_(r),
            },
            |t| ApiResult::Success(t),
        )
    }

    /// Prism for Error variant
    fn error_prism() -> Prism_<ApiResult<T>, String> {
        Prism::new(
            |r| match r {
                ApiResult::Error(e) => Result_::Ok_(e),
                ApiResult::Success(_) => Result_::Err_(r),
            },
            |e| ApiResult::Error(e),
        )
    }
}

fn prism_example() {
    let success: ApiResult<i32> = ApiResult::Success(42);
    let error: ApiResult<i32> = ApiResult::Error("Not found".to_string());

    // Preview through prism
    let value = ApiResult::success_prism().preview(success.clone());
    println!("Success value: {:?}", value);  // Some(42)

    let err_value = ApiResult::success_prism().preview(error.clone());
    println!("Error through success prism: {:?}", err_value);  // None

    // Review (construct) through prism
    let new_success = ApiResult::success_prism().review(100);
    println!("Constructed: {:?}", new_success);  // Success(100)
}
```

---

## Free Monads and Interpreters

### Free Monad DSL

```rust
/// Free monad for building DSLs
enum Free<F, A> {
    Pure(A),
    Free(Box<F>),
}

/// File operation DSL
enum FileOp<Next> {
    Read {
        path: String,
        continue_with: Box<dyn FnOnce(String) -> Next>,
    },
    Write {
        path: String,
        content: String,
        continue_with: Box<dyn FnOnce(()) -> Next>,
    },
}

type FileM<A> = Free<FileOp<FileM<A>>, A>;

/// DSL constructors
fn read_file(path: String) -> FileM<String> {
    Free::Free(Box::new(FileOp::Read {
        path,
        continue_with: Box::new(|content| Free::Pure(content)),
    }))
}

fn write_file(path: String, content: String) -> FileM<()> {
    Free::Free(Box::new(FileOp::Write {
        path,
        content,
        continue_with: Box::new(|_| Free::Pure(())),
    }))
}

/// Bind operation for Free monad
impl<F, A> Free<F, A> {
    fn and_then<B, G>(self, f: G) -> Free<F, B>
    where
        G: FnOnce(A) -> Free<F, B>,
    {
        match self {
            Free::Pure(a) => f(a),
            Free::Free(_) => {
                // Simplified - full implementation requires functor constraint on F
                panic!("Not implemented for this example")
            }
        }
    }
}

/// Example program
fn copy_file_program(src: String, dest: String) -> FileM<()> {
    // Read from source
    Free::Free(Box::new(FileOp::Read {
        path: src,
        continue_with: Box::new(move |content| {
            // Write to destination
            Free::Free(Box::new(FileOp::Write {
                path: dest,
                content,
                continue_with: Box::new(|_| Free::Pure(())),
            }))
        }),
    }))
}

/// Interpreter - execute in IO
fn interpret_io(program: FileM<()>) {
    match program {
        Free::Pure(_) => println!("Program completed"),
        Free::Free(boxed_op) => match *boxed_op {
            FileOp::Read { path, continue_with } => {
                // Simulate file read
                println!("Reading from: {}", path);
                let content = format!("Content of {}", path);
                let next = continue_with(content);
                interpret_io(next);
            }
            FileOp::Write {
                path,
                content,
                continue_with,
            } => {
                // Simulate file write
                println!("Writing to: {}", path);
                println!("Content: {}", content);
                let next = continue_with(());
                interpret_io(next);
            }
        },
    }
}

fn free_monad_example() {
    let program = copy_file_program("input.txt".to_string(), "output.txt".to_string());
    interpret_io(program);
}
```

---

## Additional Examples

### Bifunctor for Result

```rust
trait Bifunctor {
    type Left;
    type Right;

    fn bimap<L2, R2, F, G>(self, f: F, g: G) -> Result<R2, L2>
    where
        F: FnOnce(Self::Left) -> L2,
        G: FnOnce(Self::Right) -> R2;
}

impl<L, R> Bifunctor for Result<R, L> {
    type Left = L;
    type Right = R;

    fn bimap<L2, R2, F, G>(self, f: F, g: G) -> Result<R2, L2>
    where
        F: FnOnce(L) -> L2,
        G: FnOnce(R) -> R2,
    {
        match self {
            Ok(r) => Ok(g(r)),
            Err(l) => Err(f(l)),
        }
    }
}

fn bifunctor_example() {
    let success: Result<i32, String> = Ok(42);
    let mapped = success.bimap(
        |e| format!("Error: {}", e),
        |v| v * 2,
    );
    println!("{:?}", mapped);  // Ok(84)
}
```

---

**End of Rust Category Theory Implementations**
