# Asynchronous

### ABI (application binary interface)
Specifies a set of rules that programmer has to adhere to for their programs to work correctly on that platform.
A very important part of the ABI that operating systems must specify is its **calling convention**.

CPU architecture

### ISA (instruction set architecture)
Describes an abstract model of a CPU that defines how the CPU is constrolled by the software it runs.
It defines what instructions the CPU can execute, what registers programmers can use, how the hardware manages memory...
Ex: X86-64k x86, ARM ISA

CPUs implement and instruction set. 

## Futures

Fibers/green threads are an example of stackful corountines.
The other way of modeling asynchronous program flow is by susing what we call stackless corountines, and combining Rust's futures with async/await

### What is a future?

A future is a representaion of some operation that will be asynchronous task will have three phase. 

1. **The poll phase**
A future is polled, which results in the task progressing until a point where it can no longer make progress.
We often refer to the part of the runtime that polls a future as an executor


2. **The wait phase**
An event source, most often referred to as a reactor, registers that a future is waiting for an event to happen and makes sure that it will wake the future when that event is ready.


3. **The wake phase**
The event happens and the future is woken up. It's now up to the executor that polled the future in step 1 to schedule the future to be polled again and make further progress until it completes or reaches a new point where it can't make further progress and the cycle repeats.

A fully working async system in Rust can be divided into three parts:

- Reactor (responsible for notifying about I/O events)
- Executor (scheduler)
- Future (a task that can stop and resume at specific points)


What Rust language and standard library take care of

- A common interface that represents an operation, which will be completed in the future through the Future trait
- An ergonomic way of creating tasks (stackless coroutines to be precise) that can be suspended and resumed through the async and await keywords
- A defined interface to wake up a suspended task through the Waker type

## Runtime, Wakers, and the Reactor-Executor Pattern

Most runtimes such as Tokio, Smol  will divide their runtime into two parts.

**Reactor or Driver**
The part that tracks events we're waiting on and makes 

**Executor**
Schedule tasks and polls them to completion

[Waker](https://doc.rust-lang.org/std/task/struct.Waker.html)
[Future](https://doc.rust-lang.org/std/future/trait.Future.html)


## Map Reduce

### Abstract
- for processing and generating large data sets
- parallelized and executed on a large cluster of commodity machines

### Programming model
2 main functions

1. Map
	written by the user, takes an input pair and produces a set of intermediate key/value pairs

2. Reduce
	also written by the user, accepts an intermediate key I and a set of values for that key. It merges togetherthese values to form a possibly smaller set of values. 

```
map(String key, String value): 
 // key: document name
 // value: document contents
 for each word w in value:
  EmitIntermediate(w, "1");

reduce(String key, Iterator values):
 // key: a word
 // values: a list of counts
 int result = 0;
 for each v in values:
  result += ParseInt(v);
 Emit(AsString(result));
```

### Implementation


### Github
- https://github.com/PacktPublishing/Asynchronous-Programming-in-Rust/tree/main
- https://github.com/rust-lang/libc
- https://github.com/conradludgate/what-the-async/tree/main


### REFERENCES

#### Async
- [How epoll works efficiently](https://www.sobyte.net/post/2022-04/epoll-efficiently/)
- [Let’s write async rust from the ground up!](https://www.youtube.com/watch?v=7pU3gOVAeVQ&t=6s)
- https://www.youtube.com/watch?v=tAMruxkwVf0
- [Understanding Rust futures by going way too deep ](https://fasterthanli.me/articles/understanding-rust-futures-by-going-way-too-deep)
- [Let's talk about this async](https://conradludgate.com/posts/async)
- [Learning Async Rust With Entirely Too Many Web Servers](https://ibraheem.ca/posts/too-many-web-servers/)

#### Distributed Systems
- [MapReduce: Simplified Data Processing on Large Clusters](http://nil.csail.mit.edu/6.824/2020/papers/mapreduce.pdf)
- [6.824 Lab 1: MapReduce](http://nil.csail.mit.edu/6.824/2020/labs/lab-mr.html)
- [MapReduce - Computerphile](https://www.youtube.com/watch?v=cQP8WApzIQQ)
- [Lecture 1: Introduction](https://www.youtube.com/watch?v=cQP8WApzIQQ)
