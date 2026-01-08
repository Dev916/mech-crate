# Category Theory in PHP

Practical implementations of category theory concepts in PHP 8+.

## Table of Contents

1. [Functors and Monads](#functors-and-monads)
2. [Comonads](#comonads)
3. [Monad Transformers](#monad-transformers)
4. [Optics (Lenses)](#optics-lenses)
5. [Free Monads and Interpreters](#free-monads-and-interpreters)

---

## Functors and Monads

### Functor and Monad Traits

```php
<?php

namespace Category;

/**
 * Option monad
 */
abstract class Option
{
    abstract public function map(callable $f): Option;
    abstract public function flatMap(callable $f): Option;
    abstract public function getOrElse($default);
    abstract public function isNone(): bool;
    abstract public function isSome(): bool;
}

class Some extends Option
{
    private $value;

    public function __construct($value)
    {
        $this->value = $value;
    }

    public function map(callable $f): Option
    {
        return new Some($f($this->value));
    }

    public function flatMap(callable $f): Option
    {
        return $f($this->value);
    }

    public function getOrElse($default)
    {
        return $this->value;
    }

    public function isNone(): bool
    {
        return false;
    }

    public function isSome(): bool
    {
        return true;
    }

    public function get()
    {
        return $this->value;
    }
}

class None extends Option
{
    public function map(callable $f): Option
    {
        return $this;
    }

    public function flatMap(callable $f): Option
    {
        return $this;
    }

    public function getOrElse($default)
    {
        return $default;
    }

    public function isNone(): bool
    {
        return true;
    }

    public function isSome(): bool
    {
        return false;
    }
}

/**
 * Either monad
 */
abstract class Either
{
    abstract public function map(callable $f): Either;
    abstract public function flatMap(callable $f): Either;
    abstract public function isLeft(): bool;
    abstract public function isRight(): bool;
}

class Left extends Either
{
    private $value;

    public function __construct($value)
    {
        $this->value = $value;
    }

    public function map(callable $f): Either
    {
        return $this;
    }

    public function flatMap(callable $f): Either
    {
        return $this;
    }

    public function getValue()
    {
        return $this->value;
    }

    public function isLeft(): bool
    {
        return true;
    }

    public function isRight(): bool
    {
        return false;
    }
}

class Right extends Either
{
    private $value;

    public function __construct($value)
    {
        $this->value = $value;
    }

    public function map(callable $f): Either
    {
        return new Right($f($this->value));
    }

    public function flatMap(callable $f): Either
    {
        return $f($this->value);
    }

    public function getValue()
    {
        return $this->value;
    }

    public function isLeft(): bool
    {
        return false;
    }

    public function isRight(): bool
    {
        return true;
    }
}

/**
 * Example: Safe operations with Option
 */
function safeDivide(float $x, float $y): Option
{
    if ($y == 0) {
        return new None();
    }
    return new Some($x / $y);
}

function compute(): Option
{
    return safeDivide(10, 2)
        ->flatMap(fn($x) => safeDivide($x, 5))
        ->map(fn($x) => $x * 2);
}

$result = compute()->getOrElse(0);  // 2.0

/**
 * Example: Error handling with Either
 */
function divide(float $x, float $y): Either
{
    if ($y == 0) {
        return new Left("Division by zero");
    }
    return new Right($x / $y);
}

function pipeline(): Either
{
    return divide(10, 2)
        ->flatMap(fn($x) => divide($x, 0))
        ->flatMap(fn($y) => divide($y, 2));
}

$result = pipeline();
if ($result instanceof Left) {
    echo "Error: " . $result->getValue();  // "Error: Division by zero"
} else {
    echo "Result: " . $result->getValue();
}
```

---

## Comonads

### Stream Comonad

```php
<?php

namespace Category\Comonad;

/**
 * Stream Comonad - infinite list with focus
 */
class Stream
{
    private $head;
    private $tailFn;

    public function __construct($head, callable $tailFn)
    {
        $this->head = $head;
        $this->tailFn = $tailFn;
    }

    /** Extract current element */
    public function extract()
    {
        return $this->head;
    }

    /** Get tail */
    public function tail(): Stream
    {
        return ($this->tailFn)();
    }

    /** Duplicate into Stream of Streams */
    public function duplicate(): Stream
    {
        return new Stream(
            $this,
            fn() => $this->tail()->duplicate()
        );
    }

    /** Extend function over stream */
    public function extend(callable $f): Stream
    {
        return new Stream(
            $f($this),
            fn() => $this->tail()->extend($f)
        );
    }

    /** Take first n elements */
    public function take(int $n): array
    {
        $result = [$this->head];
        $current = $this;
        for ($i = 1; $i < $n; $i++) {
            $current = $current->tail();
            $result[] = $current->head;
        }
        return $result;
    }

    /** Create infinite stream from iterator */
    public static function iterate($initial, callable $f): Stream
    {
        return new Stream(
            $initial,
            fn() => self::iterate($f($initial), $f)
        );
    }

    /** Create stream from array with default */
    public static function from(array $array, $default): Stream
    {
        $index = 0;
        $generator = function() use (&$index, $array, $default, &$generator) {
            $index++;
            $value = $array[$index] ?? $default;
            return new Stream($value, $generator);
        };

        return new Stream($array[0] ?? $default, $generator);
    }
}

/**
 * Example: Moving average with Stream
 */
function movingAverage3(Stream $stream): float
{
    $values = $stream->take(3);
    return array_sum($values) / count($values);
}

function streamExample(): void
{
    // Infinite stream of naturals
    $naturals = Stream::iterate(1, fn($n) => $n + 1);
    print_r($naturals->take(10));
    // [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

    // Moving average
    $data = Stream::iterate(10.0, fn($n) => $n + 10);
    $smoothed = $data->extend(fn($s) => movingAverage3($s));
    print_r($smoothed->take(5));

    // Time series from array
    $prices = Stream::from([100, 102, 98, 105, 110], 0);
    $avgPrices = $prices->extend(fn($s) => movingAverage3($s));
    print_r($avgPrices->take(3));
}
```

### Store Comonad

```php
<?php

namespace Category\Comonad;

/**
 * Store Comonad - position in space
 */
class Store
{
    private $getter;
    private $position;

    public function __construct(callable $getter, $position)
    {
        $this->getter = $getter;
        $this->position = $position;
    }

    /** Extract value at current position */
    public function extract()
    {
        return ($this->getter)($this->position);
    }

    /** Peek at different position */
    public function peek($pos)
    {
        return ($this->getter)($pos);
    }

    /** Move to new position */
    public function seek($pos): Store
    {
        return new Store($this->getter, $pos);
    }

    /** Duplicate into Store of Stores */
    public function duplicate(): Store
    {
        $getter = $this->getter;
        return new Store(
            fn($s) => new Store($getter, $s),
            $this->position
        );
    }

    /** Extend function over Store */
    public function extend(callable $f): Store
    {
        $getter = $this->getter;
        return new Store(
            fn($s) => $f(new Store($getter, $s)),
            $this->position
        );
    }

    /** Get current position */
    public function getPosition()
    {
        return $this->position;
    }
}

/**
 * Example: Game of Life with Store
 */
function neighbors(array $pos): array
{
    [$x, $y] = $pos;
    $result = [];
    for ($dx = -1; $dx <= 1; $dx++) {
        for ($dy = -1; $dy <= 1; $dy++) {
            if ($dx !== 0 || $dy !== 0) {
                $result[] = [$x + $dx, $y + $dy];
            }
        }
    }
    return $result;
}

function gameOfLifeStep(Store $grid): bool
{
    $aliveNeighbors = count(array_filter(
        neighbors($grid->getPosition()),
        fn($pos) => $grid->peek($pos)
    ));

    $current = $grid->extract();
    if ($current && ($aliveNeighbors === 2 || $aliveNeighbors === 3)) {
        return true;
    }
    if (!$current && $aliveNeighbors === 3) {
        return true;
    }
    return false;
}

function storeExample(): void
{
    // Glider pattern
    $alive = [[1, 0], [2, 1], [0, 2], [1, 2], [2, 2]];
    $initialState = function($pos) use ($alive) {
        foreach ($alive as [$x, $y]) {
            if ($pos[0] === $x && $pos[1] === $y) {
                return true;
            }
        }
        return false;
    };

    $grid = new Store($initialState, [1, 1]);
    echo "Current cell: " . ($grid->extract() ? "alive" : "dead") . "\n";

    // Evolve one step
    $nextGen = $grid->extend(fn($g) => gameOfLifeStep($g));
    echo "Next generation: " . ($nextGen->extract() ? "alive" : "dead") . "\n";

    // Evolve multiple generations
    $current = $grid;
    for ($i = 0; $i < 5; $i++) {
        $current = $current->extend(fn($g) => gameOfLifeStep($g));
        echo "Generation " . ($i + 1) . ": " .
             ($current->extract() ? "alive" : "dead") . "\n";
    }
}
```

### Env Comonad

```php
<?php

namespace Category\Comonad;

/**
 * Env Comonad - value with environment
 */
class Env
{
    private $env;
    private $value;

    public function __construct($env, $value)
    {
        $this->env = $env;
        $this->value = $value;
    }

    /** Extract value */
    public function extract()
    {
        return $this->value;
    }

    /** Get environment */
    public function ask()
    {
        return $this->env;
    }

    /** Duplicate with environment */
    public function duplicate(): Env
    {
        return new Env($this->env, $this);
    }

    /** Extend function over Env */
    public function extend(callable $f): Env
    {
        return new Env($this->env, $f($this));
    }
}

/**
 * Example: Theming with Env
 */
const THEMES = [
    'light' => [
        'primaryColor' => '#007bff',
        'backgroundColor' => '#ffffff',
        'textColor' => '#000000',
    ],
    'dark' => [
        'primaryColor' => '#0d6efd',
        'backgroundColor' => '#1a1a1a',
        'textColor' => '#ffffff',
    ],
];

function renderWithTheme(Env $themed): string
{
    $theme = $themed->ask();
    $widget = $themed->extract();
    $config = THEMES[$theme];

    return sprintf(
        '<%s style="background: %s; color: %s; border-color: %s;">%s</%s>',
        $widget['type'],
        $config['backgroundColor'],
        $config['textColor'],
        $config['primaryColor'],
        $widget['content'],
        $widget['type']
    );
}

function envExample(): void
{
    $widget = ['content' => 'Click Me', 'type' => 'button'];
    $themed = new Env('dark', $widget);
    $rendered = $themed->extend(fn($env) => renderWithTheme($env));
    echo "Rendered: " . $rendered->extract() . "\n";
}
```

---

## Monad Transformers

### OptionT - Add Option to Async Operations

```php
<?php

namespace Category\Transformer;

use Category\Option;
use Category\Some;
use Category\None;

/**
 * OptionT - adds Option semantics to Promise/Generator
 */
class OptionT
{
    private $inner;

    public function __construct(callable $inner)
    {
        $this->inner = $inner;
    }

    public static function some($value): OptionT
    {
        return new OptionT(fn() => new Some($value));
    }

    public static function none(): OptionT
    {
        return new OptionT(fn() => new None());
    }

    public function unwrap(): Option
    {
        return ($this->inner)();
    }

    public function map(callable $f): OptionT
    {
        return new OptionT(function() use ($f) {
            $opt = $this->unwrap();
            return $opt->map($f);
        });
    }

    public function flatMap(callable $f): OptionT
    {
        return new OptionT(function() use ($f) {
            $opt = $this->unwrap();
            if ($opt->isNone()) {
                return new None();
            }
            return $f($opt->get())->unwrap();
        });
    }

    public function getOrElse($default)
    {
        return $this->unwrap()->getOrElse($default);
    }
}

/**
 * Example: Database operations with OptionT
 */
function findUser(int $id): OptionT
{
    // Simulate database query
    if ($id === 1) {
        return OptionT::some([
            'id' => 1,
            'name' => 'Alice',
            'email' => 'alice@example.com'
        ]);
    }
    return OptionT::none();
}

function getUserEmail(int $id): OptionT
{
    return findUser($id)->map(fn($user) => $user['email']);
}

function optionTExample(): void
{
    $email = getUserEmail(1);
    $result = $email->getOrElse('no-email@example.com');
    echo "Email: $result\n";
}
```

### StateT - Add State to Operations

```php
<?php

namespace Category\Transformer;

/**
 * StateT - adds State semantics to operations
 */
class StateT
{
    private $run;

    public function __construct(callable $run)
    {
        $this->run = $run;
    }

    public static function pure($value): StateT
    {
        return new StateT(fn($state) => [$value, $state]);
    }

    public static function get(): StateT
    {
        return new StateT(fn($state) => [$state, $state]);
    }

    public static function put($newState): StateT
    {
        return new StateT(fn($_) => [null, $newState]);
    }

    public static function modify(callable $f): StateT
    {
        return new StateT(fn($state) => [null, $f($state)]);
    }

    public function runState($initialState): array
    {
        return ($this->run)($initialState);
    }

    public function evalState($initialState)
    {
        [$value, $_] = $this->runState($initialState);
        return $value;
    }

    public function execState($initialState)
    {
        [$_, $state] = $this->runState($initialState);
        return $state;
    }

    public function map(callable $f): StateT
    {
        return new StateT(function($state) use ($f) {
            [$value, $newState] = $this->runState($state);
            return [$f($value), $newState];
        });
    }

    public function flatMap(callable $f): StateT
    {
        return new StateT(function($state) use ($f) {
            [$value, $newState] = $this->runState($state);
            return $f($value)->runState($newState);
        });
    }
}

/**
 * Example: Counter with state
 */
function increment(): StateT
{
    return StateT::modify(function($state) {
        return [
            'count' => $state['count'] + 1,
            'history' => array_merge($state['history'], [$state['count'] + 1]),
        ];
    });
}

function getCount(): StateT
{
    return StateT::get()->map(fn($state) => $state['count']);
}

function stateTExample(): void
{
    $program = increment()
        ->flatMap(fn($_) => increment())
        ->flatMap(fn($_) => increment())
        ->flatMap(fn($_) => getCount());

    $initialState = ['count' => 0, 'history' => []];
    [$finalCount, $finalState] = $program->runState($initialState);

    echo "Final count: $finalCount\n";  // 3
    echo "History: " . implode(', ', $finalState['history']) . "\n";  // 1, 2, 3
}
```

---

## Optics (Lenses)

### Lens - Focus on Product Types

```php
<?php

namespace Category\Optics;

/**
 * Lens for accessing nested fields
 */
class Lens
{
    private $getter;
    private $setter;

    public function __construct(callable $getter, callable $setter)
    {
        $this->getter = $getter;
        $this->setter = $setter;
    }

    public static function fromKey(string $key): Lens
    {
        return new Lens(
            fn($source) => $source[$key],
            fn($source, $value) => array_merge($source, [$key => $value])
        );
    }

    public function view($source)
    {
        return ($this->getter)($source);
    }

    public function set($source, $value)
    {
        return ($this->setter)($source, $value);
    }

    public function over($source, callable $f)
    {
        $oldValue = $this->view($source);
        $newValue = $f($oldValue);
        return $this->set($source, $newValue);
    }

    public function compose(Lens $other): Lens
    {
        return new Lens(
            fn($s) => $other->view($this->view($s)),
            fn($s, $b) => $this->set($s, $other->set($this->view($s), $b))
        );
    }
}

/**
 * Example: Person with nested Address
 */
$nameLens = Lens::fromKey('name');
$ageLens = Lens::fromKey('age');
$addressLens = Lens::fromKey('address');
$cityLens = Lens::fromKey('city');

// Compose lenses for nested access
$personCityLens = $addressLens->compose($cityLens);

function lensExample(): void
{
    global $nameLens, $ageLens, $personCityLens;

    $person = [
        'name' => 'Alice',
        'age' => 30,
        'address' => [
            'street' => '123 Main St',
            'city' => 'Boston',
            'zipCode' => '02101',
        ],
    ];

    // View through lens
    $name = $nameLens->view($person);
    echo "Name: $name\n";  // "Alice"

    // Set through lens
    $person2 = $ageLens->set($person, 31);
    echo "New age: " . $person2['age'] . "\n";  // 31

    // Modify through lens
    $person3 = $nameLens->over($person, fn($n) => strtoupper($n));
    echo "Uppercase name: " . $person3['name'] . "\n";  // "ALICE"

    // Composed lens
    $city = $personCityLens->view($person);
    echo "City: $city\n";  // "Boston"

    $person4 = $personCityLens->set($person, 'New York');
    echo "New city: " . $person4['address']['city'] . "\n";  // "New York"
}
```

### Prism - Focus on Sum Types

```php
<?php

namespace Category\Optics;

/**
 * Prism for accessing sum type variants
 */
class Prism
{
    private $matcher;
    private $builder;

    public function __construct(callable $matcher, callable $builder)
    {
        $this->matcher = $matcher;
        $this->builder = $builder;
    }

    public function preview($source)
    {
        return ($this->matcher)($source);
    }

    public function review($value)
    {
        return ($this->builder)($value);
    }

    public function modify($source, callable $f)
    {
        $matched = $this->preview($source);
        if ($matched === null) {
            return null;
        }
        return $this->review($f($matched));
    }

    public function compose(Prism $other): Prism
    {
        return new Prism(
            function($s) use ($other) {
                $a = $this->preview($s);
                return $a === null ? null : $other->preview($a);
            },
            fn($b) => $this->review($other->review($b))
        );
    }
}

/**
 * Example: Result type prisms
 */
function success($value): array
{
    return ['tag' => 'success', 'value' => $value];
}

function error($err): array
{
    return ['tag' => 'error', 'error' => $err];
}

$successPrism = new Prism(
    fn($r) => $r['tag'] === 'success' ? $r['value'] : null,
    fn($v) => success($v)
);

$errorPrism = new Prism(
    fn($r) => $r['tag'] === 'error' ? $r['error'] : null,
    fn($e) => error($e)
);

function prismExample(): void
{
    global $successPrism, $errorPrism;

    $result1 = success(42);
    $result2 = error('Not found');

    // Preview through prism
    $value = $successPrism->preview($result1);
    echo "Success value: $value\n";  // 42

    $errValue = $successPrism->preview($result2);
    echo "Error through success prism: " . var_export($errValue, true) . "\n";  // null

    // Review (construct) through prism
    $newSuccess = $successPrism->review(100);
    print_r($newSuccess);  // ['tag' => 'success', 'value' => 100]

    // Modify through prism
    $modified = $successPrism->modify($result1, fn($n) => $n * 2);
    print_r($modified);  // ['tag' => 'success', 'value' => 84]
}
```

---

## Free Monads and Interpreters

### Free Monad DSL

```php
<?php

namespace Category\Free;

/**
 * Free monad for building DSLs
 */
abstract class Free
{
    abstract public function bind(callable $f): Free;
}

class Pure extends Free
{
    private $value;

    public function __construct($value)
    {
        $this->value = $value;
    }

    public function getValue()
    {
        return $this->value;
    }

    public function bind(callable $f): Free
    {
        return $f($this->value);
    }
}

class FreeF extends Free
{
    private $fa;

    public function __construct($fa)
    {
        $this->fa = $fa;
    }

    public function getFA()
    {
        return $this->fa;
    }

    public function bind(callable $f): Free
    {
        // Simplified implementation
        return $this;
    }
}

/**
 * File operation DSL
 */
class FileOp
{
    public string $tag;
    public string $path;
    public $content;
    public $next;

    public static function read(string $path, callable $next): array
    {
        return [
            'tag' => 'read',
            'path' => $path,
            'next' => $next,
        ];
    }

    public static function write(string $path, string $content, $next): array
    {
        return [
            'tag' => 'write',
            'path' => $path,
            'content' => $content,
            'next' => $next,
        ];
    }

    public static function delete(string $path, $next): array
    {
        return [
            'tag' => 'delete',
            'path' => $path,
            'next' => $next,
        ];
    }
}

/**
 * DSL constructors
 */
function readFile(string $path): Free
{
    return new FreeF(FileOp::read($path, fn($content) => new Pure($content)));
}

function writeFile(string $path, string $content): Free
{
    return new FreeF(FileOp::write($path, $content, new Pure(null)));
}

function deleteFile(string $path): Free
{
    return new FreeF(FileOp::delete($path, new Pure(null)));
}

/**
 * Example program
 */
function copyFileProgram(string $src, string $dest): Free
{
    return new FreeF(
        FileOp::read($src, function($content) use ($dest) {
            return new FreeF(
                FileOp::write($dest, $content, new Pure(null))
            );
        })
    );
}

/**
 * Interpreter - execute in console
 */
function interpretConsole(Free $program)
{
    if ($program instanceof Pure) {
        return $program->getValue();
    }

    if ($program instanceof FreeF) {
        $op = $program->getFA();

        switch ($op['tag']) {
            case 'read':
                echo "Reading from: {$op['path']}\n";
                $content = "Content of {$op['path']}";
                return interpretConsole($op['next']($content));

            case 'write':
                echo "Writing to: {$op['path']}\n";
                echo "Content: {$op['content']}\n";
                return interpretConsole($op['next']);

            case 'delete':
                echo "Deleting: {$op['path']}\n";
                return interpretConsole($op['next']);
        }
    }
}

function freeMonadExample(): void
{
    $program = copyFileProgram('input.txt', 'output.txt');
    interpretConsole($program);
}
```

---

## Additional Examples

### Bifunctor for Arrays

```php
<?php

namespace Category;

/**
 * Bifunctor operations on arrays (key, value)
 */
class ArrayBifunctor
{
    public static function bimap(array $arr, callable $fKey, callable $fValue): array
    {
        $result = [];
        foreach ($arr as $key => $value) {
            $newKey = $fKey($key);
            $newValue = $fValue($value);
            $result[$newKey] = $newValue;
        }
        return $result;
    }

    public static function first(array $arr, callable $f): array
    {
        return self::bimap($arr, $f, fn($v) => $v);
    }

    public static function second(array $arr, callable $f): array
    {
        return self::bimap($arr, fn($k) => $k, $f);
    }
}

function bifunctorExample(): void
{
    $data = ['a' => 1, 'b' => 2, 'c' => 3];

    // Map over both keys and values
    $mapped = ArrayBifunctor::bimap(
        $data,
        fn($k) => strtoupper($k),
        fn($v) => $v * 2
    );
    print_r($mapped);  // ['A' => 2, 'B' => 4, 'C' => 6]

    // Map only values
    $values = ArrayBifunctor::second($data, fn($v) => $v + 10);
    print_r($values);  // ['a' => 11, 'b' => 12, 'c' => 13]
}
```

---

**End of PHP Category Theory Implementations**
