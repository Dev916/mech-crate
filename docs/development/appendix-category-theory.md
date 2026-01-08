# Category Theory: Mathematical Composition and Abstraction

**Purpose**: Mathematical framework for studying composition, providing abstract patterns that unify diverse concepts across mathematics and computer science.

**Core Insight**: **Category theory is the mathematics of composition**. Focus on relationships (morphisms) rather than internal structure, enabling powerful abstractions.

---

## Table of Contents

1. [Foundational Concepts](#foundational-concepts)
2. [Functors](#functors)
3. [Natural Transformations](#natural-transformations)
4. [Yoneda Lemma](#yoneda-lemma)
5. [Monads](#monads)
6. [Comonads](#comonads)
7. [Monad Transformers](#monad-transformers)
8. [Applicative Functors](#applicative-functors)
9. [Bifunctors and Profunctors](#bifunctors-and-profunctors)
10. [Adjunctions](#adjunctions)
11. [F-Algebras and Catamorphisms](#f-algebras-and-catamorphisms)
12. [Recursion Schemes](#recursion-schemes)
13. [Monoidal Categories](#monoidal-categories)
14. [Cartesian Closed Categories](#cartesian-closed-categories)
15. [Limits and Colimits](#limits-and-colimits)
16. [Optics: Lenses, Prisms, and Traversals](#optics-lenses-prisms-and-traversals)
17. [Kan Extensions](#kan-extensions)
18. [Practical Programming Applications](#practical-programming-applications)
19. [Stack-Specific Implementations](#stack-specific-implementations)
20. [Integration Points](#integration-points)

---

## Foundational Concepts

### What is Category Theory?

**Category**: Collection of objects and morphisms (arrows) between them.

**Definition**: Category C consists of:
1. **Objects**: Ob(C) = set of objects
2. **Morphisms**: For objects A, B, set Hom(A, B) of arrows A → B
3. **Composition**: f: A → B, g: B → C ⇒ g ∘ f: A → C
4. **Identity**: For each A, identity 1_A: A → A

**Laws**:
- **Associativity**: h ∘ (g ∘ f) = (h ∘ g) ∘ f
- **Identity**: f ∘ 1_A = f = 1_B ∘ f

### Example Categories

**Set**: Objects = sets, morphisms = functions
- Composition = function composition
- Identity = identity function

**Poset**: Objects = elements of partial order, morphisms = a ≤ b
- Composition = transitivity
- Identity = reflexivity

**Mon**: Objects = monoids, morphisms = monoid homomorphisms
- Preserves unit and multiplication

**Hask** (Haskell): Objects = Haskell types, morphisms = functions
- Composition = (.)
- Identity = id

### Isomorphism

**Isomorphism**: Morphism f: A → B with inverse g: B → A such that:
- g ∘ f = 1_A
- f ∘ g = 1_B

**Meaning**: A and B are "the same" from category's perspective.

**Example**: List and tree representations of sequences (isomorphic as sequences, different as data structures).

---

## Functors

**Functor**: Structure-preserving map between categories.

### Definition

**Functor F: C → D** consists of:
1. **Object mapping**: For each A ∈ Ob(C), F(A) ∈ Ob(D)
2. **Morphism mapping**: For f: A → B in C, F(f): F(A) → F(B) in D

**Laws**:
- **Identity**: F(1_A) = 1_{F(A)}
- **Composition**: F(g ∘ f) = F(g) ∘ F(f)

### Functors in Programming

**Functor in Haskell**:
```haskell
class Functor f where
  fmap :: (a -> b) -> f a -> f b

-- Laws:
-- fmap id = id
-- fmap (g . h) = fmap g . fmap h
```

**Examples**:

**List Functor**:
```haskell
instance Functor [] where
  fmap = map

-- fmap (+1) [1,2,3] = [2,3,4]
```

**Maybe Functor**:
```haskell
instance Functor Maybe where
  fmap f Nothing = Nothing
  fmap f (Just x) = Just (f x)

-- fmap (*2) (Just 5) = Just 10
-- fmap (*2) Nothing = Nothing
```

**Function Functor** (Reader):
```haskell
instance Functor ((->) r) where
  fmap = (.)

-- fmap (+1) (*2) $ 5 = 11  -- (5*2)+1
```

### Contravariant Functors

**Contravariant Functor**: Reverses arrows.

```haskell
class Contravariant f where
  contramap :: (b -> a) -> f a -> f b

-- Example: Predicate
newtype Predicate a = Predicate { getPredicate :: a -> Bool }

instance Contravariant Predicate where
  contramap f (Predicate p) = Predicate (p . f)
```

**Use Case**: Input-focused types (e.g., parsers, predicates).

---

## Natural Transformations

**Natural Transformation**: Morphism between functors.

### Definition

**Natural Transformation η: F ⇒ G** (for functors F, G: C → D):
- For each A ∈ Ob(C), η_A: F(A) → G(A)

**Naturality Condition**:
For every f: A → B in C, this square commutes:
```
F(A) --η_A--> G(A)
 |              |
F(f)          G(f)
 |              |
 v              v
F(B) --η_B--> G(B)
```

**Meaning**: G(f) ∘ η_A = η_B ∘ F(f)

### Examples in Programming

**List to Maybe**:
```haskell
listToMaybe :: [a] -> Maybe a
listToMaybe [] = Nothing
listToMaybe (x:_) = Just x

-- Natural transformation: [] ⇒ Maybe
-- Naturality: fmap f . listToMaybe = listToMaybe . fmap f
```

**Reverse**:
```haskell
reverse :: [a] -> [a]
-- Natural transformation: [] ⇒ []
-- Naturality: fmap f . reverse = reverse . fmap f
```

---

## Yoneda Lemma

**The Yoneda Lemma**: One of the most important results in category theory.

### Statement

**Yoneda Lemma**: For any functor F: C → Set and object A in C:

```
Nat(Hom(A, -), F) ≅ F(A)
```

**Meaning**: Natural transformations from Hom(A, -) to F are in bijection with elements of F(A).

**Intuition**: An object is completely determined by its relationships to all other objects.

### The Bijection

**Forward** (η → η_A(id_A)):
Given natural transformation η: Hom(A, -) → F, take η_A(id_A) ∈ F(A).

**Backward** (x → λf. F(f)(x)):
Given x ∈ F(A), construct η: for each f: A → B, define η_B(f) = F(f)(x).

**Naturality**: For g: B → C and f: A → B:
```
F(g)(η_B(f)) = F(g)(F(f)(x))
             = F(g ∘ f)(x)
             = η_C(g ∘ f)
```

### Yoneda Embedding

**Yoneda Embedding**: Functor Y: C → [C^op, Set]

```
Y(A) = Hom(-, A)
```

**Fully Faithful**: Y preserves and reflects all structure.

**Consequence**: Every category embeds into presheaf category.

### Practical Implications

**Representation**: If F ≅ Hom(A, -), then A "represents" F.

**Example: Reader Monad**:
```haskell
-- Reader r a ≅ (r → a) ≅ Hom(r, a)
newtype Reader r a = Reader { runReader :: r -> a }

-- Yoneda: Nat(Reader r, f) ≅ f r
-- Natural transformations from Reader r correspond to values of f r
```

### Coyoneda Trick

**Problem**: Need Functor instance but don't have one.

**Solution**: Coyoneda lemma - every functor is naturally isomorphic to its Coyoneda transform.

```haskell
data Coyoneda f a where
  Coyoneda :: (b -> a) -> f b -> Coyoneda f a

instance Functor (Coyoneda f) where
  fmap f (Coyoneda g fb) = Coyoneda (f . g) fb

-- No Functor constraint on f!

lowerCoyoneda :: Functor f => Coyoneda f a -> f a
lowerCoyoneda (Coyoneda f fa) = fmap f fa
```

**Use Case**: Build functor without Functor instance, convert later.

---

## Monads

**Monad**: Functor with additional structure for sequencing computations.

### Definition

**Monad M** consists of:
1. **Functor**: M is a functor
2. **Unit**: η: A → M(A) (also called `return` or `pure`)
3. **Join**: μ: M(M(A)) → M(A) (flatten)

**Or equivalently**:
1. **Functor**: M is a functor
2. **Unit**: return :: a -> m a
3. **Bind**: (>>=) :: m a -> (a -> m b) -> m b

**Monad Laws**:
```haskell
-- Left identity
return a >>= f  =  f a

-- Right identity
m >>= return  =  m

-- Associativity
(m >>= f) >>= g  =  m >>= (\x -> f x >>= g)
```

### Example Monads

**Maybe Monad** (partiality):
```haskell
instance Monad Maybe where
  return = Just

  Nothing >>= f = Nothing
  Just x >>= f = f x

-- Example: Safe division
safeDivide :: Double -> Double -> Maybe Double
safeDivide _ 0 = Nothing
safeDivide x y = Just (x / y)

compute :: Maybe Double
compute = do
  a <- safeDivide 10 2    -- Just 5
  b <- safeDivide a 0     -- Nothing
  c <- safeDivide b 2     -- Skipped
  return c                -- Nothing
```

**List Monad** (nondeterminism):
```haskell
instance Monad [] where
  return x = [x]
  xs >>= f = concat (map f xs)  -- flatMap

-- Example: Generate pairs
pairs :: [(Int, Int)]
pairs = do
  x <- [1, 2, 3]
  y <- [4, 5]
  return (x, y)

-- Result: [(1,4),(1,5),(2,4),(2,5),(3,4),(3,5)]
```

**State Monad**:
```haskell
newtype State s a = State { runState :: s -> (a, s) }

instance Monad (State s) where
  return a = State $ \s -> (a, s)

  m >>= f = State $ \s ->
    let (a, s') = runState m s
        m' = f a
    in runState m' s'

-- Example: Stateful computation
tick :: State Int ()
tick = State $ \s -> ((), s + 1)

compute :: State Int Int
compute = do
  tick
  tick
  get

-- runState compute 0 = (2, 2)
```

### Kleisli Category

**Kleisli Category for Monad M**:
- Objects: Same as base category
- Morphisms: f: A → M(B) (Kleisli arrows)
- Composition: g <=< f = \x -> f x >>= g
- Identity: return

**Why Useful**: Monads form a category where morphisms are effectful computations.

---

## Comonads

**Comonad**: Dual to monad - represents context-dependent computation.

### Definition

**Comonad W** consists of:
1. **Functor**: W is a functor
2. **Extract**: ε: W(A) → A (dual to return)
3. **Duplicate**: δ: W(A) → W(W(A)) (dual to join)

**Or equivalently**:
1. **Functor**: W is a functor
2. **Extract**: extract :: w a -> a
3. **Extend**: (=>=) :: (w a -> b) -> w a -> w b

**Comonad Laws**:
```haskell
-- Left identity
extract . duplicate  =  id

-- Right identity
fmap extract . duplicate  =  id

-- Associativity
fmap duplicate . duplicate  =  duplicate . duplicate
```

### Example Comonads

**Stream Comonad** (infinite lists):
```haskell
data Stream a = Cons a (Stream a)

instance Functor Stream where
  fmap f (Cons x xs) = Cons (f x) (fmap f xs)

instance Comonad Stream where
  extract (Cons x _) = x  -- Get current element

  duplicate stream@(Cons _ xs) =
    Cons stream (duplicate xs)  -- All suffixes

  extend f stream@(Cons _ xs) =
    Cons (f stream) (extend f xs)

-- Example: Moving average
movingAverage :: Stream Int -> Int
movingAverage (Cons a (Cons b (Cons c _))) =
  (a + b + c) `div` 3

smoothed :: Stream Int -> Stream Int
smoothed = extend movingAverage
```

**Store Comonad** (position in space):
```haskell
data Store s a = Store (s -> a) s  -- function and position

instance Functor (Store s) where
  fmap f (Store g s) = Store (f . g) s

instance Comonad (Store s) where
  extract (Store f s) = f s  -- Value at current position

  duplicate (Store f s) = Store (\s' -> Store f s') s
    -- For each position, a store focused there

-- Example: Game of Life
type Pos = (Int, Int)
type Grid = Store Pos Bool

neighbors :: Pos -> [Pos]
neighbors (x, y) = [(x+dx, y+dy) | dx <- [-1..1], dy <- [-1..1], (dx,dy) /= (0,0)]

step :: Grid -> Bool
step (Store f pos) =
  let aliveNeighbors = length $ filter f $ neighbors pos
      current = f pos
  in case (current, aliveNeighbors) of
    (True, 2) -> True
    (True, 3) -> True
    (False, 3) -> True
    _ -> False

evolve :: Grid -> Grid
evolve = extend step
```

**Env Comonad** (Reader + value):
```haskell
data Env e a = Env e a

instance Functor (Env e) where
  fmap f (Env e a) = Env e (f a)

instance Comonad (Env e) where
  extract (Env _ a) = a

  duplicate (Env e a) = Env e (Env e a)

-- Access environment
ask :: Env e a -> e
ask (Env e _) = e

-- Example: Theming
data Theme = Light | Dark

type Themed a = Env Theme a

render :: Themed Widget -> String
render = extend $ \env ->
  let theme = ask env
      widget = extract env
  in renderWith theme widget
```

### Comonads vs. Monads

| Aspect | Monad | Comonad |
|--------|-------|---------|
| Core Operation | return: a → m a | extract: w a → a |
| Sequencing | join: m (m a) → m a | duplicate: w a → w (w a) |
| Direction | Produces effects | Consumes context |
| Composition | Kleisli (a → m b) | Cokleisli (w a → b) |

**Intuition**:
- **Monad**: Produce values in context (Maybe, List, IO)
- **Comonad**: Extract values from context (Stream position, Store location)

---

## Monad Transformers

**Problem**: Monads don't compose. M(N(a)) is not automatically a monad.

**Solution**: Monad transformers - add one monad's effects to another.

### Definition

**Monad Transformer T**: Type constructor T such that:
- For any monad M, T M is a monad
- Has lift: M a → T M a (embed base monad)

```haskell
class MonadTrans t where
  lift :: Monad m => m a -> t m a
```

### Common Transformers

**MaybeT** (add failure to any monad):
```haskell
newtype MaybeT m a = MaybeT { runMaybeT :: m (Maybe a) }

instance Monad m => Functor (MaybeT m) where
  fmap f (MaybeT ma) = MaybeT $ do
    maybeA <- ma
    return $ fmap f maybeA

instance Monad m => Monad (MaybeT m) where
  return = MaybeT . return . Just

  MaybeT mma >>= f = MaybeT $ do
    maybeA <- mma
    case maybeA of
      Nothing -> return Nothing
      Just a -> runMaybeT (f a)

instance MonadTrans MaybeT where
  lift = MaybeT . fmap Just

-- Example: Database query with error handling
type DB = MaybeT IO

query :: String -> DB String
query sql = do
  lift $ putStrLn $ "Executing: " ++ sql
  result <- lift $ performQuery sql  -- IO String
  if null result
    then MaybeT $ return Nothing     -- Query failed
    else return result                -- Query succeeded
```

**StateT** (add state to any monad):
```haskell
newtype StateT s m a = StateT { runStateT :: s -> m (a, s) }

instance Monad m => Monad (StateT s m) where
  return a = StateT $ \s -> return (a, s)

  StateT sma >>= f = StateT $ \s -> do
    (a, s') <- sma s
    runStateT (f a) s'

instance MonadTrans (StateT s) where
  lift ma = StateT $ \s -> do
    a <- ma
    return (a, s)

-- Example: Stateful IO
type StatefulIO s = StateT s IO

tick :: StatefulIO Int ()
tick = do
  count <- get
  put (count + 1)
  lift $ putStrLn $ "Count: " ++ show (count + 1)
```

**ReaderT** (add environment to any monad):
```haskell
newtype ReaderT r m a = ReaderT { runReaderT :: r -> m a }

instance Monad m => Monad (ReaderT r m) where
  return a = ReaderT $ \_ -> return a

  ReaderT rma >>= f = ReaderT $ \r -> do
    a <- rma r
    runReaderT (f a) r

instance MonadTrans (ReaderT r) where
  lift ma = ReaderT $ \_ -> ma

-- Example: Configuration + IO
data Config = Config { dbUrl :: String, apiKey :: String }

type App = ReaderT Config IO

loadUser :: UserId -> App User
loadUser userId = do
  config <- ask
  lift $ fetchFromDB (dbUrl config) userId
```

### Transformer Stack

**Stack multiple transformers**:
```haskell
-- Combine State + Maybe + IO
type App s = StateT s (MaybeT IO)

runApp :: App s a -> s -> IO (Maybe (a, s))
runApp app initialState =
  runMaybeT $ runStateT app initialState

-- Example usage
appLogic :: App Int String
appLogic = do
  count <- get            -- StateT
  lift $ lift $ putStrLn $ "Count: " ++ show count  -- IO
  if count > 10
    then lift $ MaybeT $ return Nothing   -- Fail
    else do
      put (count + 1)     -- Update state
      return "Success"
```

### MTL-Style Type Classes

**Problem**: Deep lifting (lift . lift . lift) is ugly.

**Solution**: Type classes for each effect:

```haskell
class Monad m => MonadState s m | m -> s where
  get :: m s
  put :: s -> m ()

class Monad m => MonadReader r m | m -> r where
  ask :: m r
  local :: (r -> r) -> m a -> m a

class Monad m => MonadError e m | m -> e where
  throwError :: e -> m a
  catchError :: m a -> (e -> m a) -> m a

-- Example: Clean code without explicit lifting
appLogic :: (MonadState Int m, MonadReader Config m, MonadIO m) => m ()
appLogic = do
  count <- get              -- Auto-lifted
  config <- ask             -- Auto-lifted
  liftIO $ putStrLn "Hello" -- Explicit IO lift
  put (count + 1)           -- Auto-lifted
```

---

## Applicative Functors

**Applicative**: Functor with additional structure for applying wrapped functions.

### Definition

**Applicative F** consists of:
1. **Functor**: F is a functor
2. **Pure**: pure :: a -> f a
3. **Apply**: (<*>) :: f (a -> b) -> f a -> f b

**Laws**:
```haskell
-- Identity
pure id <*> v  =  v

-- Composition
pure (.) <*> u <*> v <*> w  =  u <*> (v <*> w)

-- Homomorphism
pure f <*> pure x  =  pure (f x)

-- Interchange
u <*> pure y  =  pure ($ y) <*> u
```

### Examples

**Maybe Applicative**:
```haskell
instance Applicative Maybe where
  pure = Just

  Nothing <*> _ = Nothing
  _ <*> Nothing = Nothing
  Just f <*> Just x = Just (f x)

-- Example: Apply function to multiple Maybe values
add3 :: Int -> Int -> Int -> Int
add3 x y z = x + y + z

result = add3 <$> Just 1 <*> Just 2 <*> Just 3  -- Just 6
result' = add3 <$> Just 1 <*> Nothing <*> Just 3  -- Nothing
```

**ZipList Applicative**:
```haskell
newtype ZipList a = ZipList { getZipList :: [a] }

instance Applicative ZipList where
  pure x = ZipList (repeat x)
  ZipList fs <*> ZipList xs = ZipList (zipWith ($) fs xs)

-- Example: Parallel application
result = getZipList $ (+) <$> ZipList [1,2,3] <*> ZipList [10,20,30]
-- [11, 22, 33]
```

### Applicative vs. Monad

**Applicative**: Fixed structure, independent computations
```haskell
(+) <$> Just 1 <*> Just 2  -- Structure known upfront
```

**Monad**: Dynamic structure, dependent computations
```haskell
Just 1 >>= \x -> if x > 0 then Just (x+1) else Nothing
-- Structure depends on value
```

**Hierarchy**: Monad ⊂ Applicative ⊂ Functor

---

## Bifunctors and Profunctors

### Bifunctors

**Bifunctor**: Functor in two arguments.

**Definition**: F: C × D → E

```haskell
class Bifunctor f where
  bimap :: (a -> c) -> (b -> d) -> f a b -> f c d

  -- Derived
  first :: (a -> c) -> f a b -> f c b
  first f = bimap f id

  second :: (b -> d) -> f a b -> f a d
  second = bimap id
```

**Examples**:

**Pair**:
```haskell
instance Bifunctor (,) where
  bimap f g (a, b) = (f a, g b)
```

**Either**:
```haskell
instance Bifunctor Either where
  bimap f _ (Left a) = Left (f a)
  bimap _ g (Right b) = Right (g b)
```

**Const**:
```haskell
newtype Const a b = Const a

instance Bifunctor Const where
  bimap f _ (Const a) = Const (f a)
```

### Profunctors

**Profunctor**: Contravariant in first argument, covariant in second.

**Definition**: P: C^op × D → E

```haskell
class Profunctor p where
  dimap :: (a' -> a) -> (b -> b') -> p a b -> p a' b'

  -- Derived
  lmap :: (a' -> a) -> p a b -> p a' b  -- Contravariant
  lmap f = dimap f id

  rmap :: (b -> b') -> p a b -> p a b'  -- Covariant
  rmap = dimap id
```

**Examples**:

**Function** (→):
```haskell
instance Profunctor (->) where
  dimap f g h = g . h . f
  -- Pre-compose with f (contravariant)
  -- Post-compose with g (covariant)
```

**Kleisli**:
```haskell
newtype Kleisli m a b = Kleisli { runKleisli :: a -> m b }

instance Monad m => Profunctor (Kleisli m) where
  dimap f g (Kleisli amb) = Kleisli $ \a' ->
    fmap g (amb (f a'))
```

**Use Case**: **Optics** (lenses, prisms) built on profunctors.

---

## Adjunctions

**Adjunction**: Deep relationship between two functors.

### Definition

**Adjunction F ⊣ G** (F left adjoint, G right adjoint):

Functors F: C → D, G: D → C such that:
```
Hom_D(F(A), B) ≅ Hom_C(A, G(B))
```

**Meaning**: Morphisms from F(A) to B correspond to morphisms from A to G(B).

**Unit**: η: A → G(F(A))
**Counit**: ε: F(G(B)) → B

**Triangle Identities**:
```
G(ε_B) ∘ η_{G(B)} = 1_{G(B)}
ε_{F(A)} ∘ F(η_A) = 1_{F(A)}
```

### Example: Free-Forgetful Adjunction

**Free Monoid ⊣ Forgetful**:
- F: Set → Mon (list construction)
- G: Mon → Set (forget monoid structure)

```haskell
-- Free monoid: lists
free :: Set a -> Mon [a]
free = map (:[])

-- Forgetful: underlying set
forget :: Mon m -> Set m
forget = id  -- Just forget monoid operations

-- Adjunction: Hom(F(A), M) ≅ Hom(A, G(M))
-- Functions [a] -> M correspond to functions a -> M
```

### Monads from Adjunctions

**Every adjunction gives rise to a monad**: M = G ∘ F

- **Unit**: η: A → G(F(A))
- **Join**: μ = G(ε_F): G(F(G(F(A)))) → G(F(A))

**Example**: List monad from free-forgetful adjunction.

---

## F-Algebras and Catamorphisms

**F-Algebra**: Generalization of algebraic structures using functors.

### Definition

**F-Algebra** for functor F: C → C:
- **Carrier**: Object A
- **Structure map**: α: F(A) → A

**Example**: Natural numbers as algebra
```haskell
data NatF a = Zero | Succ a

type Algebra f a = f a -> a

natAlg :: Algebra NatF Int
natAlg Zero = 0
natAlg (Succ n) = n + 1
```

### Initial Algebra

**Initial F-Algebra**: (μF, in: F(μF) → μF) such that:
- For any F-algebra (A, α), unique morphism (catamorphism) cata α: μF → A
- Makes diagram commute:

```
F(μF) --F(cata α)--> F(A)
  |                    |
 in|                   |α
  |                    |
  v                    v
 μF -----cata α-----> A
```

**Lambek's Lemma**: in: F(μF) → μF is an isomorphism.

**Consequence**: μF ≅ F(μF) (fixed point)

### Catamorphism (Fold)

**Catamorphism**: Unique morphism from initial algebra.

```haskell
cata :: Functor f => (f a -> a) -> Fix f -> a
cata alg = alg . fmap (cata alg) . unFix

newtype Fix f = Fix { unFix :: f (Fix f) }
```

**Example**: List catamorphism

```haskell
data ListF a r = Nil | Cons a r deriving Functor

type List a = Fix (ListF a)

-- foldr as catamorphism
foldr' :: (a -> b -> b) -> b -> List a -> b
foldr' f z = cata alg
  where
    alg Nil = z
    alg (Cons a b) = f a b

-- sum as catamorphism
sum :: List Int -> Int
sum = cata alg
  where
    alg Nil = 0
    alg (Cons n acc) = n + acc
```

### Anamorphism (Unfold)

**Anamorphism**: Dual of catamorphism - build from seed.

```haskell
ana :: Functor f => (a -> f a) -> a -> Fix f
ana coalg = Fix . fmap (ana coalg) . coalg
```

**Example**: Generate infinite list

```haskell
-- Generate naturals
naturals :: Fix (ListF Int)
naturals = ana coalg 0
  where
    coalg n = Cons n (n + 1)
```

### Hylomorphism

**Hylomorphism**: Composition of anamorphism and catamorphism.

```haskell
hylo :: Functor f => (f b -> b) -> (a -> f a) -> a -> b
hylo alg coalg = cata alg . ana coalg

-- More efficient:
hylo alg coalg = alg . fmap (hylo alg coalg) . coalg
```

**Example**: Factorial

```haskell
factorial :: Int -> Int
factorial = hylo alg coalg
  where
    coalg 0 = Nil
    coalg n = Cons n (n - 1)

    alg Nil = 1
    alg (Cons n acc) = n * acc
```

---

## Recursion Schemes

**Recursion Schemes**: Structured recursion patterns derived from category theory.

### Common Schemes

**Catamorphism** (fold): F(μF) → μF → A
- Tear down structure bottom-up
- Example: `foldr`, `sum`, `product`

**Anamorphism** (unfold): A → F(A) → μF
- Build structure top-down
- Example: `iterate`, `unfoldr`

**Hylomorphism**: A → F(A) → F(B) → B
- Build then tear down
- Example: `factorial`, `quicksort`

**Paramorphism**: Access original structure while folding
```haskell
para :: Functor f => (f (Fix f, a) -> a) -> Fix f -> a
para alg = alg . fmap (id &&& para alg) . unFix
```

**Apomorphism**: Early termination while unfolding
```haskell
apo :: Functor f => (a -> f (Either (Fix f) a)) -> a -> Fix f
apo coalg = Fix . fmap (either id (apo coalg)) . coalg
```

**Histomorphism**: Access all past results
```haskell
histo :: Functor f => (f (Cofree f a) -> a) -> Fix f -> a
```

**Futumorphism**: Generate multiple layers at once
```haskell
futu :: Functor f => (a -> f (Free f a)) -> a -> Fix f
```

### Example: Paramorphism

```haskell
-- Sliding window operation
slidingWindow :: Int -> List a -> List (List a)
slidingWindow n = para alg
  where
    alg Nil = nil
    alg (Cons a (original, rest)) =
      cons (take n (cons a original)) rest
```

### Example: Complete Binary Tree

```haskell
data TreeF a r = Leaf a | Branch r r deriving Functor

type Tree a = Fix (TreeF a)

-- Catamorphism: tree depth
depth :: Tree a -> Int
depth = cata alg
  where
    alg (Leaf _) = 0
    alg (Branch l r) = 1 + max l r

-- Anamorphism: build complete tree
completeTree :: Int -> a -> Tree a
completeTree n x = ana coalg n
  where
    coalg 0 = Leaf x
    coalg d = Branch (d-1) (d-1)

-- Hylomorphism: rebuild tree with transformation
mapTree :: (a -> b) -> Tree a -> Tree b
mapTree f = hylo alg coalg
  where
    coalg = unFix
    alg (Leaf a) = Fix (Leaf (f a))
    alg (Branch l r) = Fix (Branch l r)
```

---

## Monoidal Categories

**Monoidal Category**: Category with tensor product and unit object.

### Definition

**Monoidal Category** (C, ⊗, I) consists of:
- Category C
- Bifunctor ⊗: C × C → C (tensor)
- Object I (unit)
- Natural isomorphisms:
  - Associator: α: (A ⊗ B) ⊗ C → A ⊗ (B ⊗ C)
  - Left unitor: λ: I ⊗ A → A
  - Right unitor: ρ: A ⊗ I → A

**Coherence conditions**: Pentagon and triangle diagrams commute.

### Examples

**Cartesian Monoidal Category**:
- Tensor = product (×)
- Unit = terminal object (1)
- Example: (Set, ×, {∗})

**Cocartesian (Monoidal)**:
- Tensor = coproduct (+)
- Unit = initial object (0)
- Example: (Set, +, ∅)

**Endofunctors**:
- (End(C), ∘, Id)
- Composition as tensor
- Identity functor as unit

### Monoids in Monoidal Categories

**Monoid Object** in (C, ⊗, I):
- Object M
- Multiplication: μ: M ⊗ M → M
- Unit: η: I → M

**Laws**:
- Associativity: μ ∘ (μ ⊗ id) = μ ∘ (id ⊗ μ)
- Identity: μ ∘ (η ⊗ id) = id = μ ∘ (id ⊗ η)

**Example**: Monad is monoid in endofunctor category
- (End(C), ∘, Id)
- M: Monad (endofunctor)
- μ: M ∘ M → M (join)
- η: Id → M (return)

### Braided and Symmetric

**Braided Monoidal**: Has braiding σ: A ⊗ B → B ⊗ A

**Symmetric Monoidal**: σ ∘ σ = id

---

## Cartesian Closed Categories

**Cartesian Closed Category** (CCC): Category with products and exponentials.

### Definition

**CCC** has:
1. **Terminal object**: 1
2. **Binary products**: A × B
3. **Exponentials**: B^A (function space)

**Universal property of exponentials**:
```
Hom(Z, B^A) ≅ Hom(Z × A, B)
```

**Curry/Uncurry**:
- curry: (Z × A → B) → (Z → B^A)
- uncurry: (Z → B^A) → (Z × A → B)

### Lambda Calculus

**CCC models typed lambda calculus**:
- Objects = types
- Morphisms = terms
- Products = product types
- Exponentials = function types

**Example**:
```
f: A × B → C  corresponds to  curry(f): A → (B → C)
```

### Internal Language

**CCC has internal logic**:
- ⊤: Terminal object (truth)
- A × B: Conjunction (and)
- B^A: Implication (A ⇒ B)

**Intuitionistic logic**: CCC ≅ Simply typed lambda calculus ≅ Intuitionistic propositional logic

---

## Limits and Colimits

**Limits/Colimits**: Universal constructions generalizing products, coproducts, pullbacks, etc.

### Products

**Product** of A and B:
- Object A × B
- Projections π₁: A × B → A, π₂: A × B → B

**Universal Property**:
For any X with f: X → A, g: X → B, unique h: X → A × B such that:
```
π₁ ∘ h = f
π₂ ∘ h = g
```

**Programming**:
```haskell
data (a, b) = (a, b)  -- Product type

fst :: (a, b) -> a
snd :: (a, b) -> b

-- Universal property: pairing
pair :: (x -> a) -> (x -> b) -> (x -> (a, b))
pair f g = \x -> (f x, g x)
```

### Coproducts (Sums)

**Coproduct** of A and B:
- Object A + B
- Injections i₁: A → A + B, i₂: B → A + B

**Universal Property**:
For any X with f: A → X, g: B → X, unique h: A + B → X such that:
```
h ∘ i₁ = f
h ∘ i₂ = g
```

**Programming**:
```haskell
data Either a b = Left a | Right b

-- Universal property: case analysis
either :: (a -> x) -> (b -> x) -> (Either a b -> x)
either f g (Left a) = f a
either f g (Right b) = g b
```

### Initial and Terminal Objects

**Initial Object** 0:
- For any A, unique morphism 0 → A

**Terminal Object** 1:
- For any A, unique morphism A → 1

**Programming**:
```haskell
-- Initial: empty type (Void)
data Void

absurd :: Void -> a  -- Unique morphism from Void

-- Terminal: unit type
data () = ()

unit :: a -> ()
unit _ = ()  -- Unique morphism to ()
```

---

## Optics: Lenses, Prisms, and Traversals

**Optics**: Composable accessors for nested data structures, built on profunctors.

### Lenses

**Lens**: Getter + Setter for product types.

**Van Laarhoven Encoding**:
```haskell
type Lens s t a b = forall f. Functor f => (a -> f b) -> s -> f t
type Lens' s a = Lens s s a a
```

**Laws**:
```
view l (set l v s) = v           -- Get-Put
set l (view l s) s = s           -- Put-Get
set l v' (set l v s) = set l v' s -- Put-Put
```

**Example**:
```haskell
data Person = Person { name :: String, age :: Int }

_name :: Lens' Person String
_name f (Person n a) = fmap (\n' -> Person n' a) (f n)

_age :: Lens' Person Int
_age f (Person n a) = fmap (\a' -> Person n a') (f a)

-- Usage
view _name (Person "Alice" 30)  -- "Alice"
set _name "Bob" (Person "Alice" 30)  -- Person "Bob" 30
over _name (++ "!") (Person "Alice" 30)  -- Person "Alice!" 30
```

### Prisms

**Prism**: Getter + Constructor for sum types.

```haskell
type Prism s t a b = forall p f. (Choice p, Applicative f) =>
                     p a (f b) -> p s (f t)
type Prism' s a = Prism s s a a
```

**Example**:
```haskell
data Result a = Success a | Failure String

_Success :: Prism' (Result a) a
_Success = prism Success $ \case
  Success a -> Right a
  Failure e -> Left (Failure e)

_Failure :: Prism' (Result a) String
_Failure = prism Failure $ \case
  Failure e -> Right e
  Success a -> Left (Success a)

-- Usage
preview _Success (Success 42)  -- Just 42
preview _Success (Failure "err")  -- Nothing
review _Success 42  -- Success 42
```

### Traversals

**Traversal**: Access multiple elements.

```haskell
type Traversal s t a b = forall f. Applicative f =>
                         (a -> f b) -> s -> f t
type Traversal' s a = Traversal s s a a
```

**Example**:
```haskell
-- Traverse list elements
_each :: Traversal' [a] a
_each = traverse

-- Usage
toListOf _each [1,2,3]  -- [1,2,3]
over _each (*2) [1,2,3]  -- [2,4,6]
```

### Optic Composition

**Optics compose with (.)** - most general to most specific:
```
Lens . Lens = Lens
Lens . Prism = (neither)
Prism . Lens = (neither)
Prism . Prism = Prism
Traversal . Lens = Traversal
Lens . Traversal = Traversal
```

**Example**:
```haskell
data Company = Company { employees :: [Person] }

_employees :: Lens' Company [Person]

-- Compose: focus on all employee names
allNames :: Traversal' Company String
allNames = _employees . traverse . _name

-- Usage
toListOf allNames company  -- ["Alice", "Bob", ...]
over allNames (++ "!") company  -- Add ! to all names
```

---

## Kan Extensions

**Kan Extensions**: Universal way to extend functors along other functors.

### Left Kan Extension

**Left Kan Extension** Lan_K F: For functors K: C → D, F: C → E:

```
Lan_K F: D → E
```

**Universal Property**:
```
Nat(Lan_K F, G) ≅ Nat(F, G ∘ K)
```

**Formula** (when C small):
```
(Lan_K F)(d) = ∫^c F(c) × Hom_D(K(c), d)
```

**Intuition**: Best approximation of F extended along K.

### Right Kan Extension

**Right Kan Extension** Ran_K F:

**Universal Property**:
```
Nat(G, Ran_K F) ≅ Nat(G ∘ K, F)
```

**Formula**:
```
(Ran_K F)(d) = ∫_c F(c)^{Hom_D(d, K(c))}
```

### Example: Yoneda as Kan Extension

**Yoneda Lemma** can be expressed as:
```
Ran_{Id} F ≅ F
```

**Meaning**: Right Kan extension along identity is the functor itself.

### Codensity Monad

**Codensity Monad**: Ran_M M for any functor M.

```haskell
newtype Codensity m a = Codensity {
  runCodensity :: forall b. (a -> m b) -> m b
}

instance Monad (Codensity m) where
  return a = Codensity $ \k -> k a
  m >>= f = Codensity $ \k ->
    runCodensity m $ \a ->
      runCodensity (f a) k
```

**Use Case**: Improve performance of free monads (asymptotic improvement).

**Example**:
```haskell
-- Slow: left-associated (>>=)
slow :: Free f a
slow = do
  x1 <- step
  x2 <- step
  ...
  xn <- step
  return result

-- Fast: Church encoding via Codensity
fast :: Codensity (Free f) a
fast = do
  x1 <- lift step
  x2 <- lift step
  ...
  xn <- lift step
  return result

-- Convert back
improved :: Free f a
improved = lowerCodensity fast
```

---

## Practical Programming Applications

### Functor-Based Abstractions

**Functor**: Map pure functions over effects
```rust
// Rust: Option functor
let x = Some(5);
let y = x.map(|v| v + 1);  // Some(6)
```

**Use Cases**:
- Error handling (Option, Result)
- Collections (Vec, HashMap)
- Async computations (Future)

### Monadic Composition

**Monad**: Sequence effectful computations

**Example**: Error handling with ?
```rust
fn process() -> Result<i32, Error> {
    let x = step1()?;  // Early return if Err
    let y = step2(x)?;
    let z = step3(y)?;
    Ok(z)
}
```

**Equivalent to**:
```rust
step1().and_then(|x|
  step2(x).and_then(|y|
    step3(y)
  )
)
```

### Free Structures

**Free Monad**: Build interpreters, DSLs

```haskell
data Free f a
  = Pure a
  | Free (f (Free f a))

instance Functor f => Monad (Free f) where
  return = Pure
  Pure a >>= f = f a
  Free fa >>= f = Free (fmap (>>= f) fa)

-- Example: DSL for file operations
data FileOp next
  = Read FilePath (String -> next)
  | Write FilePath String next

type FileM = Free FileOp

readFile' :: FilePath -> FileM String
readFile' path = Free (Read path Pure)

writeFile' :: FilePath -> String -> FileM ()
writeFile' path content = Free (Write path content (Pure ()))
```

**Interpreter**:
```haskell
interpret :: FileM a -> IO a
interpret (Pure a) = return a
interpret (Free (Read path k)) = do
  content <- readFile path
  interpret (k content)
interpret (Free (Write path content next)) = do
  writeFile path content
  interpret next
```

---

## Stack-Specific Implementations

**Note**: Comprehensive implementations for Rust, TypeScript, and PHP have been moved to dedicated files for better organization:

### Language-Specific Files

- **[Rust Implementations](./appendix-category-theory-rust.md)**: Functors, Monads, Comonads, Monad Transformers, Optics, and Free Monads
- **[TypeScript Implementations](./appendix-category-theory-typescript.md)**: Full category theory patterns with TypeScript type system
- **[PHP Implementations](./appendix-category-theory-php.md)**: Modern PHP 8+ implementations of categorical abstractions

Each file contains:
1. **Functors and Monads**: Option, Either, Result patterns
2. **Comonads**: Stream, Store, and Env comonads with practical examples
3. **Monad Transformers**: OptionT, StateT, ReaderT for composing effects
4. **Optics**: Lenses and Prisms for nested data access
5. **Free Monads**: Building DSLs and interpreters

### Quick Examples

**Rust**:
```rust
// Option monad with error propagation
fn process() -> Result<i32, Error> {
    let x = step1()?;
    let y = step2(x)?;
    Ok(y)
}

// Comonad for spatial computations
let grid = Store::new(cell_getter, position);
let next_gen = grid.extend(game_of_life_step);
```

**TypeScript**:
```typescript
// Either monad for error handling
function divide(x: number, y: number): EitherMonad<string, number> {
  if (y === 0) return EitherMonad.left("Division by zero");
  return EitherMonad.right(x / y);
}

// Stream comonad for time series
const prices = Stream.iterate(100, n => n + Math.random() * 10);
const smoothed = prices.extend(movingAverage3);
```

**PHP**:
```php
// Option monad
function safeDivide(float $x, float $y): Option {
    return $y == 0 ? new None() : new Some($x / $y);
}

// Lens for nested access
$cityLens = $addressLens->compose($cityLens);
$newPerson = $cityLens->set($person, 'New York');
```

For detailed implementations with comonads, monad transformers, and optics, see the language-specific files linked above.

---

## Integration Points

### With Type Theory
- **Curry-Howard**: Categories model types and programs
- **Dependent types**: Internal language of categories

### With Functional Programming
- **Haskell**: Category theory as design pattern language
- **Monads**: Fundamental abstraction for effects

### With Formal Methods
- **Categorical logic**: Foundation for proof theory
- **Topos theory**: Generalized set theory for verification

---

## Further Reading

### Papers
- Mac Lane (1971) - "Categories for the Working Mathematician"
- Moggi (1991) - "Notions of Computation and Monads"
- Wadler (1992) - "Comprehending Monads"

### Books
- Mac Lane - "Categories for the Working Mathematician"
- Awodey - "Category Theory" (2nd ed.)
- Milewski - "Category Theory for Programmers"
- Bird & de Moor - "Algebra of Programming"

### Online
- [Milewski's Category Theory Blog](https://bartoszmilewski.com/2014/10/28/category-theory-for-programmers-the-preface/)
- [nLab](https://ncatlab.org/) - Wiki for category theory

---

**End of Category Theory Appendix**
