# Sources & References

**Purpose**: Comprehensive bibliography for all theoretical frameworks, patterns, and concepts covered in this codex. Organized by topic area for easy research and further study.

**Last Updated**: 2026-01-07

---

## Table of Contents

1. [Category Theory & Type Theory](#category-theory--type-theory)
2. [Functional Programming](#functional-programming)
3. [Concurrency & Parallelism](#concurrency--parallelism)
4. [Software Architecture & Design Patterns](#software-architecture--design-patterns)
5. [Domain-Driven Design](#domain-driven-design)
6. [Advanced Mathematics for Programming](#advanced-mathematics-for-programming)
7. [Complex Systems & Emergence](#complex-systems--emergence)
8. [Evolutionary Computation & Genetic Algorithms](#evolutionary-computation--genetic-algorithms)
9. [Programming Language Theory](#programming-language-theory)
10. [Distributed Systems](#distributed-systems)
11. [Online Resources & Documentation](#online-resources--documentation)
12. [Quick References](#quick-references)

---

## Category Theory & Type Theory

### Foundational Books

**Category Theory for Programmers** by Bartosz Milewski
- URL: https://bartoszmilewski.com/2014/10/28/category-theory-for-programmers-the-preface/
- Free online book with video lectures
- Best introduction to category theory for software engineers
- Covers: functors, natural transformations, monads, Yoneda lemma, Kan extensions

**Categories for the Working Mathematician** by Saunders Mac Lane
- ISBN: 978-0387984032
- Springer Graduate Texts in Mathematics
- The definitive graduate-level textbook on category theory
- Comprehensive coverage of limits, adjunctions, monoidal categories

**Basic Category Theory** by Tom Leinster
- arXiv: https://arxiv.org/abs/1612.09375
- Cambridge Studies in Advanced Mathematics
- Accessible introduction with programming-relevant examples

**Conceptual Mathematics: A First Introduction to Categories** by F. William Lawvere, Stephen H. Schanuel
- ISBN: 978-0521719162
- Cambridge University Press
- Gentle introduction emphasizing intuition over formalism

### Academic Papers

**Notions of Computation and Monads** by Eugenio Moggi (1991)
- DOI: 10.1016/0890-5401(91)90052-4
- Information and Computation, Vol 93, Issue 1
- Foundational paper on monads for programming language semantics

**Applicative Programming with Effects** by Conor McBride, Ross Paterson (2008)
- Journal of Functional Programming, Vol 18, Issue 1
- DOI: 10.1017/S0956796807006326
- Introduces applicative functors

**Composing Monads** by Mark P. Jones, Luc Duponcheel (1993)
- Research Report YALEU/DCS/RR-1004
- Yale University Department of Computer Science
- Foundational work on monad transformers

**Comonads for User Interfaces** by Phil Freeman (2016)
- Blog: https://blog.functorial.com/posts/2016-08-07-Comonads-As-Spaces.html
- Practical application of comonads to UI programming

**Profunctor Optics: Modular Data Accessors** by Matthew Pickering, Jeremy Gibbons, Nicolas Wu (2017)
- arXiv: https://arxiv.org/abs/1703.10857
- Unified framework for lenses, prisms, traversals

### Type Theory

**Types and Programming Languages** by Benjamin C. Pierce
- ISBN: 978-0262162098
- MIT Press
- Comprehensive introduction to type systems

**Practical Foundations for Programming Languages** by Robert Harper
- ISBN: 978-1107150300
- Cambridge University Press
- Graduate-level programming language theory

**Homotopy Type Theory: Univalent Foundations of Mathematics** (2013)
- URL: https://homotopytypetheory.org/book/
- Free online book from the Univalent Foundations Program
- Basis for "Homotopy-Theoretic Refactoring" pattern

---

## Functional Programming

### Books

**Functional Programming in Scala** by Paul Chiusano, Rúnar Bjarnason
- ISBN: 978-1617290657
- Manning Publications
- Comprehensive FP principles

**Purely Functional Data Structures** by Chris Okasaki
- ISBN: 978-0521663502
- Cambridge University Press
- Immutable data structures

**Learn You a Haskell for Great Good!** by Miran Lipovača
- URL: http://learnyouahaskell.com/
- Free online book

**Real World Haskell** by Bryan O'Sullivan, Don Stewart, John Goerzen
- URL: http://book.realworldhaskell.org/
- Free online book

---

## Concurrency & Parallelism

### Books

**The Art of Multiprocessor Programming** by Maurice Herlihy, Nir Shavit
- ISBN: 978-0124159501
- Morgan Kaufmann, 2nd Edition (2020)
- Comprehensive coverage of concurrent algorithms

**Programming Rust** by Jim Blandy, Jason Orendorff, Leonora F. S. Tindall
- ISBN: 978-1492052593
- O'Reilly Media, 2nd Edition (2021)
- Chapter on concurrency

**Rust Atomics and Locks** by Mara Bos
- ISBN: 978-1098119447
- O'Reilly Media (2023)
- **PRIMARY REFERENCE for rust-concurrency appendix**
- Memory ordering, atomic operations

**Java Concurrency in Practice** by Brian Goetz et al.
- ISBN: 978-0321349606
- Addison-Wesley (2006)
- Classic text on concurrent programming

### Papers

**C11 Standard: Memory Model and Atomic Operations**
- ISO/IEC 9899:2011
- Defines memory ordering semantics

**Threads Cannot Be Implemented as a Library** by Hans-J. Boehm (2005)
- PLDI '05
- DOI: 10.1145/1065010.1065042

**Wait-Free Synchronization** by Maurice Herlihy (1991)
- ACM TOPLAS
- DOI: 10.1145/114005.102808

**Linearizability: A Correctness Condition for Concurrent Objects** by Maurice Herlihy, Jeannette Wing (1990)
- ACM TOPLAS
- DOI: 10.1145/78969.78972

### Rust Documentation

**Rustonomicon: The Dark Arts of Unsafe Rust**
- URL: https://doc.rust-lang.org/nomicon/
- Memory model, atomics

**Rust Atomic Ordering Documentation**
- URL: https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html
- Memory ordering semantics

---

## Software Architecture & Design Patterns

### Classic Texts

**Design Patterns: Elements of Reusable Object-Oriented Software** by Gang of Four
- ISBN: 978-0201633610
- Addison-Wesley (1994)
- The foundational design patterns book

**Patterns of Enterprise Application Architecture** by Martin Fowler
- ISBN: 978-0321127420
- Addison-Wesley (2002)

**Enterprise Integration Patterns** by Gregor Hohpe, Bobby Woolf
- ISBN: 978-0321200686
- Addison-Wesley (2003)
- Essential for event-driven architectures

**Clean Architecture** by Robert C. Martin
- ISBN: 978-0134494166
- Prentice Hall (2017)
- Hexagonal architecture, ports and adapters

### Hexagonal Architecture

**Hexagonal Architecture** by Alistair Cockburn (2005)
- URL: https://alistair.cockburn.us/hexagonal-architecture/
- Original article introducing the pattern

**Growing Object-Oriented Software, Guided by Tests** by Steve Freeman, Nat Pryce
- ISBN: 978-0321503627
- Addison-Wesley (2009)

### Reactive & Event-Driven

**Reactive Design Patterns** by Roland Kuhn, Brian Hanafee, Jamie Allen
- ISBN: 978-1617291807
- Manning Publications (2017)

**Designing Data-Intensive Applications** by Martin Kleppmann
- ISBN: 978-1449373320
- O'Reilly Media (2017)
- **Essential reading** for modern architectures

---

## Domain-Driven Design

### Books

**Domain-Driven Design: Tackling Complexity in the Heart of Software** by Eric Evans
- ISBN: 978-0321125215
- Addison-Wesley (2003)
- The original DDD book

**Implementing Domain-Driven Design** by Vaughn Vernon
- ISBN: 978-0321834577
- Addison-Wesley (2013)
- Practical implementation guide

**Domain Modeling Made Functional** by Scott Wlaschin
- ISBN: 978-1680502541
- Pragmatic Bookshelf (2018)
- DDD with functional programming

---

## Advanced Mathematics for Programming

### Category Theory (Mathematical)

**Category Theory** by Steve Awodey
- ISBN: 978-0199237180
- Oxford Logic Guides

**Category Theory in Context** by Emily Riehl
- ISBN: 978-0486820804
- Dover Publications

### Algebraic Topology

**Algebraic Topology** by Allen Hatcher
- ISBN: 978-0521795401
- Cambridge University Press
- Free PDF: https://pi.math.cornell.edu/~hatcher/AT/ATpage.html
- Basis for "Homological Debugging" pattern

**Topology** by James Munkres
- ISBN: 978-0131816299
- Pearson, 2nd Edition

### Differential Geometry

**Introduction to Smooth Manifolds** by John Lee
- ISBN: 978-1441999818
- Springer Graduate Texts in Mathematics
- Foundation for "Differential Code Evolution"

### Sheaf Theory

**Sheaves in Geometry and Logic** by Saunders Mac Lane, Ieke Moerdijk
- ISBN: 978-0387977102
- Springer
- Basis for "Sheaf-Theoretic Distributed Systems"

**An Invitation to Applied Category Theory** by Brendan Fong, David Spivak
- ISBN: 978-1108711821
- Cambridge University Press
- Free PDF: https://arxiv.org/abs/1803.05316

### Operad Theory

**Operads in Algebra, Topology and Physics** by Martin Markl, Steve Shnider, Jim Stasheff
- ISBN: 978-0821843628
- American Mathematical Society
- Foundation for "Operadic UI Composition"

### Quantum Probability

**Quantum Theory: Concepts and Methods** by Asher Peres
- ISBN: 978-0792336327
- Springer

**Quantum Computation and Quantum Information** by Michael A. Nielsen, Isaac L. Chuang
- ISBN: 978-1107002173
- Cambridge University Press
- Standard quantum computing textbook

---

## Complex Systems & Emergence

### Books

**The Nature of Code** by Daniel Shiffman
- URL: https://natureofcode.com/
- Free online book with Processing examples
- **PRIMARY INSPIRATION for software-as-life.md**
- Vectors, forces, autonomous agents, cellular automata, genetic algorithms

**Complexity: A Guided Tour** by Melanie Mitchell
- ISBN: 978-0199798100
- Oxford University Press

**Emergence: From Chaos to Order** by John H. Holland
- ISBN: 978-0738201429
- Basic Books

**Gödel, Escher, Bach: An Eternal Golden Braid** by Douglas Hofstadter
- ISBN: 978-0465026562
- Basic Books
- Pulitzer Prize winner

**A New Kind of Science** by Stephen Wolfram
- ISBN: 978-1579550080
- Wolfram Media

**The Algorithmic Beauty of Plants** by Przemyslaw Prusinkiewicz, Aristid Lindenmayer
- ISBN: 978-0387946764
- Springer
- Free PDF: http://algorithmicbotany.org/papers/#abop

### Papers

**More Is Different** by P.W. Anderson (1972)
- Science, Vol 177, Issue 4047
- DOI: 10.1126/science.177.4047.393
- Foundational paper on emergence

**The Architecture of Complexity** by Herbert A. Simon (1962)
- Proceedings of the American Philosophical Society

---

## Evolutionary Computation & Genetic Algorithms

### Books

**An Introduction to Genetic Algorithms** by Melanie Mitchell
- ISBN: 978-0262631853
- MIT Press

**Genetic Programming** by John R. Koza
- ISBN: 978-0262111706
- MIT Press

**Adaptation in Natural and Artificial Systems** by John H. Holland
- ISBN: 978-0262082136
- MIT Press (1992 edition)
- Original work on genetic algorithms

---

## Programming Language Theory

### Books

**Programming Language Pragmatics** by Michael L. Scott
- ISBN: 978-0124104099
- Morgan Kaufmann, 4th Edition

**Essentials of Programming Languages** by Daniel P. Friedman, Mitchell Wand
- ISBN: 978-0262062794
- MIT Press, 3rd Edition

**Structure and Interpretation of Computer Programs** by Harold Abelson, Gerald Jay Sussman
- ISBN: 978-0262510871
- MIT Press, 2nd Edition
- Free online: https://mitpress.mit.edu/sites/default/files/sicp/index.html

---

## Distributed Systems

### Books

**Designing Data-Intensive Applications** by Martin Kleppmann
- ISBN: 978-1449373320
- O'Reilly Media (2017)
- **Essential reading**

**Distributed Systems** by Maarten van Steen, Andrew S. Tanenbaum
- ISBN: 978-1543057386
- 3rd Edition (2017)

**Database Internals** by Alex Petrov
- ISBN: 978-1492040347
- O'Reilly Media (2019)

### Consensus Papers

**Paxos Made Simple** by Leslie Lamport (2001)
- ACM SIGACT News

**In Search of an Understandable Consensus Algorithm (Raft)** by Diego Ongaro, John Ousterhout (2014)
- USENIX ATC '14
- URL: https://raft.github.io/

**CAP Twelve Years Later** by Eric Brewer (2012)
- IEEE Computer, Vol 45, Issue 2

---

## Online Resources & Documentation

### Interactive Learning

**3Blue1Brown** by Grant Sanderson
- YouTube: https://www.youtube.com/c/3blue1brown
- Visual explanations of mathematics

**Category Theory for Programmers Video Series** by Bartosz Milewski
- YouTube playlist of video lectures

### Documentation

**Rust Language Documentation**
- URL: https://doc.rust-lang.org/

**TypeScript Handbook**
- URL: https://www.typescriptlang.org/docs/handbook/

**PHP Documentation**
- URL: https://www.php.net/manual/en/

### Blogs & Communities

**Lambda the Ultimate**
- URL: http://lambda-the-ultimate.org/
- Programming language theory blog

**InfoQ**
- URL: https://www.infoq.com/
- Software architecture, distributed systems

### Academic Resources

**arXiv Computer Science**
- URL: https://arxiv.org/archive/cs
- Preprints of computer science papers

**ACM Digital Library**
- URL: https://dl.acm.org/

**IEEE Xplore**
- URL: https://ieeexplore.ieee.org/

---

## Quick References

### Existing References (from original file)

- [Mealy Machine](https://en.wikipedia.org/wiki/Mealy_machine)
- [Single Responsibility Principle](https://en.wikipedia.org/wiki/Single-responsibility_principle)

### Wikipedia References

- [Category Theory](https://en.wikipedia.org/wiki/Category_theory)
- [Monad (functional programming)](https://en.wikipedia.org/wiki/Monad_(functional_programming))
- [Concurrent computing](https://en.wikipedia.org/wiki/Concurrent_computing)
- [Memory model (programming)](https://en.wikipedia.org/wiki/Memory_model_(programming))
- [Design Patterns](https://en.wikipedia.org/wiki/Design_Patterns)
- [Domain-driven design](https://en.wikipedia.org/wiki/Domain-driven_design)
- [Cellular automaton](https://en.wikipedia.org/wiki/Cellular_automaton)
- [Genetic algorithm](https://en.wikipedia.org/wiki/Genetic_algorithm)
- [CAP theorem](https://en.wikipedia.org/wiki/CAP_theorem)

---

## Direct Inspirations for This Codex

**The Nature of Code** by Daniel Shiffman
- **PRIMARY INSPIRATION for software-as-life.md**
- Chapters referenced:
  - Vectors & Forces → Physics section
  - Oscillation → Cause and Effect
  - Particle Systems → Emergence
  - Autonomous Agents → Boids flocking
  - Cellular Automata → Conway's Life, emergence
  - Genetic Algorithms → Evolution section

**Category Theory for Programmers** by Bartosz Milewski
- **PRIMARY INSPIRATION for category theory appendices**
- Bridge between mathematics and programming

**Homotopy Type Theory Book**
- **INSPIRATION for Homotopy-Theoretic Refactoring**
- Programs as spaces, refactorings as paths

**Rust Atomics and Locks** by Mara Bos
- **PRIMARY REFERENCE for rust-concurrency appendix**

---

## Citation Examples

### Book
> Milewski, B. (2019). *Category Theory for Programmers*. Available online: https://bartoszmilewski.com/

### Paper
> Moggi, E. (1991). "Notions of Computation and Monads." *Information and Computation*, Vol 93, Issue 1, pp. 55-92.

### Online
> Shiffman, D. (2024). *The Nature of Code*. https://natureofcode.com/. Accessed: 2026-01-07

---

## Version History

**v1.0** (2026-01-07):
- Initial compilation
- 100+ sources across all codex topics
- Organized by subject area
- Preserved existing references

---

**End of Sources Document**

*These sources represent decades of accumulated knowledge in computer science, mathematics, and software engineering.*