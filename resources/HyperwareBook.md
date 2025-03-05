# Documentation


## Overview

Hyperware processes are the building blocks for peer-to-peer applications.
The Hyperware runtime (e.g. Hyperdrive) handles message-passing between processes, plus the startup and teardown of said processes.
This section describes the message design as it relates to processes.

Each process instance has a globally unique identifier, or `Address`, composed of four elements.
- the publisher's node (containing a-z, 0-9, `-`, and `.`)
- the package name (containing a-z, 0-9, and `-`)
- the process name  (containing a-z, 0-9, and `-`).
  This may be a developer-selected string or a randomly-generated number as string.
- the node the process is running on (often your node: `our` for short).

A package is a set of one or more processes and optionally GUIs: a package is synonymous with an App and packages are distributed via the built-in App Store.

The way these elements compose is the following:

[`PackageId`s](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/hyperware/process/standard/struct.PackageId.html) look like:
```
[package-name]:[publisher-node]
my-cool-software:publisher-node.os
```

[`ProcessId`s](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/hyperware/process/standard/struct.ProcessId.html) look like:
```
[process-name]:[package-name]:[publisher-node]
process-one:my-cool-software:publisher-node.os
8513024814:my-cool-software:publisher-node.os
```

Finally, [`Address`es](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/hyperware/process/standard/struct.Address.html) look like:

```
[node]@[process-name]:[package-name]:[publisher-node]
some-user.os@process-one:my-cool-software:publisher-node.os
```

--------

Processes are compiled to Wasm.
They can be started once and complete immediately, or they can run forever.
They can spawn other processes, and coordinate in arbitrarily complex ways by passing messages to one another.

## Process State

Hyperware processes can be stateless or stateful.
In this case, state refers to data that is persisted between process instantiations.
Nodes get turned off, intentionally or otherwise.
The kernel handles rebooting processes that were running previously, but their state is not persisted by default.

Instead, processes elect to persist data, and what data to persist, when desired.
Data might be persisted after every message ingested, after every X minutes, after a certain specific event, or never.
When data is persisted, the kernel saves it to our abstracted filesystem, which not only persists data on disk, but also across arbitrarily many encrypted remote backups as configured at the user-system-level.

This design allows for ephemeral state that lives in-memory, or truly permanent state, encrypted across many remote backups, synchronized and safe.

Processes have access to multiple methods for persisting state:

- Saving a state object with the system calls available to every process.
- Using the virtual filesystem to read and write from disk, useful for persisting state that needs to be shared between processes.
- Using the SQLite or KV APIs to persist state in a database.

## Requests and Responses

Processes communicate by passing messages, of which there are two kinds: `Request`s and `Response`s.

#### Addressing

When a `Request` or `Response` is received, it has an attached `Address`, which consists of: the source of the message, including the ID of the process that produced the `Request`, as well as the ID of the originating node.

The integrity of a source `Address` differs between local and remote messages.
If a message is local, the validity of its source is ensured by the local kernel, which can be trusted to label the `ProcessId` and node ID correctly.
If a message is remote, only the node ID can be validated (via networking keys associated with each node ID).
The `ProcessId` comes from the remote kernel, which could claim any `ProcessId`.
This is fine — merely consider remote `ProcessId`s a *claim* about the initiating process rather than an infallible ID like in the local case.

#### Please Respond

`Request`s can be issued at any time by a running process.
A `Request` can optionally expect a `Response`.
If it does, the `Request` will be retained by the kernel, along with an optional `context` object created by the `Request`s issuer.
A `Request` will be considered outstanding until the kernel receives a matching `Response`, at which point that `Response` will be delivered to the requester alongside the optional `context`.
`context`s allow `Response`s to be disambiguated when handled asynchronously, for example, when some information about the `Request` must be used in handling the `Response`.
`Response`s can also be handled in an async-await style, discussed [below](#awaiting-a-response).

`Request`s that expect a `Response` set a timeout value, after which, if no `Response` is received, the initial `Request` is returned to the process that issued it as an error.
[Send errors](#errors) are handled in processes alongside other incoming messages.

##### Inheriting a `Response`

If a process receives a `Request`, that doesn't mean it must directly issue a `Response`.
The process can instead issue `Request`(s) that "inherit" from the incipient `Request`, continuing its lineage.
If a `Request` does not expect a `Response` and also "inherits" from another `Request`, `Response`s to the child `Request` will be returned to the parent `Request`s issuer.
This allows for arbitrarily complex `Request`-`Response` chains, particularly useful for "middleware" processes.

There is one other use of inheritance, discussed below: [passing data in `Request` chains cheaply](#inheriting-a-lazy_load_blob).

##### Awaiting a Response

When sending a `Request`, a process can await a `Response` to that specific `Request`, queueing other messages in the meantime.
Awaiting a `Response` leads to easier-to-read code:
* The `Response` is handled in the next line of code, rather than in a separate iteration of the message-handling loop
* Therefore, the `context` need not be set.

The downside of awaiting a `Response` is that all other messages to a process will be queued until that `Response` is received and handled.
As such, certain applications lend themselves to blocking with an await, and others don't.
A rule of thumb is: await `Response`s (because simpler code) except when a process needs to performantly handle other messages in the meantime.

For example, if a `file-transfer` process can only transfer one file at a time, `Request`s can simply await `Response`s, since the only possible next message will be a `Response` to the `Request` just sent.
In contrast, if a `file-transfer` process can transfer more than one file at a time, `Request`s that await `Response`s will block others in the meantime; for performance it may make sense to write the process fully asynchronously, i.e. without ever awaiting.
The constraint on awaiting is a primary reason why it is desirable to [spawn child processes](#spawning-child-processes).
Continuing the `file-transfer` example, by spawning one child "worker" process per file to be transferred, each worker can use the await mechanic to simplify the code, while not limiting performance.



#### Message Structure

Messages, both `Request`s and `Response`s, can contain arbitrary data, which must be interpreted by the process that receives it.
The structure of a message contains hints about how best to do this:

First, messages contain a field labeled `body`, which holds the actual contents of the message.
In order to cross the [Wasm boundary](https://component-model.bytecodealliance.org/design/why-component-model.html) and be language-agnostic, the `body` field is simply a byte vector.
To achieve composability between processes, a process should be very clear, in code and documentation, about what it expects in the `body` field and how it gets parsed, usually into a language-level struct or object.

A message also contains a `lazy_load_blob`, another byte vector, used for opaque, arbitrary, or large data.
`lazy_load_blob`s, along with being suitable location for miscellaneous message data, are an optimization for shuttling messages across the Wasm boundary.
Unlike other message fields, the `lazy_load_blob` is only moved into a process if explicitly called with (`get_blob()`).
Processes can thus choose whether to ingest a `lazy_load_blob` based on the `body`/`metadata`/`source`/`context` of a given message.
`lazy_load_blob`s hold bytes alongside a `mime` field for explicit process-and-language-agnostic format declaration, if desired.
See [inheriting a `lazy_load_blob`](#inheriting-a-lazy_load_blob) for a discussion of why lazy loading is useful.

Lastly, messages contain an optional `metadata` field, expressed as a JSON-string, to enable middleware processes and other such things to manipulate the message without altering the `body` itself.

##### Inheriting a `lazy_load_blob`

The reason `lazy_load_blob`s are not automatically loaded into a process is that an intermediate process may not need to access the blob.
If process A sends a message with a blob to process B, process B can send a message that inherits to process C.
If process B does not attach a new `lazy_load_blob` to that inheriting message, the original blob from process A will be attached and accessible to C.

For example, consider again the file-transfer process discussed [above](#awaiting-a-response).
Say one node, `send.os`, is transferring a file to another node, `recv.os`.
The process of sending a file chunk will look something like:
1. `recv.os` sends a `Request` for chunk N
2. `send.os` receives the `Request` and itself makes a `Request` to the filesystem for the piece of the file
3. `send.os` receives a `Response` from the filesystem with the piece of the file in the `lazy_load_blob`;
   `send.os` sends a `Response` that inherits the blob back to `recv.os` without itself having to load the blob, saving the compute and IO required to move the blob across the Wasm boundary.

This is the second functionality of inheritance; the first is discussed above: [eliminating the need for bucket-brigading of `Response`s](#inheriting-a-response).

#### Errors

Messages that result in networking failures, like `Request`s that timeout, are returned to the process that created them as an error.
There are only two kinds of send errors: `Offline` and `Timeout`.
Offline means a message's remote target definitively cannot be reached.
Timeout is multi-purpose: for remote nodes, it may indicate compromised networking; for both remote and local nodes, it may indicate that a process is simply failing to respond in the required time.

A send error will return to the originating process the initial message, along with any optional `context`, so that the process can re-send the message, crash, or otherwise handle the failure as the developer desires.
If the error results from a `Response`, the process may optionally try to re-send a `Response`: it will be directed towards the original outstanding `Request`.

### Capabilities

Processes must acquire capabilities from the kernel in order to perform certain operations.
Processes themselves can also produce capabilities in order to give them to other processes.
For more information about the general capabilities-based security paradigm, see the paper "Capability Myths Demolished".

The kernel gives out capabilities that allow a process to message another *local* process.
It also gives a capability allowing processes to send and receive messages over the network.
A process can optionally mark itself as `public`, meaning that it can be messaged by any *local* process regardless of capabilities.

[See the capabilities chapter for more details.](./capabilities.md)

### Spawning child processes

A process can spawn "child" processes — in which case the spawner is known as the "parent".
As discussed [above](#awaiting-a-response), one of the primary reasons to write an application with multiple processes is to enable both simple code and high performance.

Child processes can be used to:
1. Run code that may crash without risking crashing the parent
2. Run compute-heavy code without blocking the parent
3. Run IO-heavy code without blocking the parent
4. Break out code that is more easily written with awaits to avoid blocking the parent

There is more discussion of child processes in the documentation, and examples of them in action in the file-transfer cookbook.

### Conclusion

This is a high-level overview of process semantics.
In practice, processes are combined and shared in **packages**, which are generally synonymous with **apps**.

#### Wasm and Hyperware

It's briefly discussed here that processes are compiled to Wasm.
The details of this are not covered in the Hyperware Book, but can be found in the documentation for [Hyperdrive](https://github.com/hyperware-ai/hyperdrive), which uses [Wasmtime](https://wasmtime.dev/), a WebAssembly runtime, to load, execute, and provide an interface for the subset of Wasm components that are valid Hyperware processes.

Wasm runs modules by default, or components, as described [here](https://component-model.bytecodealliance.org/design/why-component-model.html): components are just modules that follow some specific format.
Hyperware processes are Wasm components that have certain imports and exports so they can be run by Hyperware.
Pragmatically, processes can be compiled using the [`kit`](https://github.com/hyperware-ai/kit) developer toolkit.

The long term goal of Hyperware is, using [WASI](https://wasi.dev/), to provide a secure, sandboxed environment for Wasm components to make use of the kernel features described in this document.
Further, Hyperware has a Virtual File System which processes can interact with to access files on a user's machine, and in the future WASI could also expose access to the filesystem for Wasm components directly.


# Capability-Based Security

# Capability-Based Security

Capabilities are a security paradigm in which an ability that is usually handled as a *permission* (i.e. certain processes are allowed to perform an action if they are saved on an "access control list") are instead handled as a *token* (i.e. the process that possesses token can perform a certain action).
These unforgeable tokens (as enforced by the kernel) can be passed to other owners, held by a given process, and checked for.

Each Hyperware process has an associated set of capabilities, which are each represented internally as an arbitrary JSON object with a source process:

```rust
pub struct Capability {
    pub issuer: Address,
    pub params: String, // JSON-string
}
```
The kernel abstracts away the process of ensuring that a capability is not forged.
As a process developer, if a capability comes in on a message or is granted to you by the kernel, you are guaranteed that it is legitimate.

Runtime processes, including the kernel itself, the filesystem, and the HTTP client, issue capabilities to processes.
Then, when a request is made by a process, the responder verifies the process's capability.
If the process does not have the capability to make such a request, it will be denied.

To give a concrete example: the filesystem can read/write, and it has the capabilities for doing so.
The FS may issue capabilities to processes to read/write to certain drives.
A process can request to read/write somewhere, and then the FS checks if that process has the required capability.
If it does, the FS does the read/write; if not, the request will be denied.

[System level capabilities](#startup-capabilities-with-manifestjson) like the above can only be given when a process is first installed.


## Startup Capabilities with `manifest.json`

When developing an application, `manifest.json` will be your first encounter with capabilties. With this file, capabilities are directly granted to a process on startup.
Upon install, the package manager (also referred to as "app store") surfaces these requested capabilities to the user, who can then choose to grant them or not.
Here is a `manifest.json` example for the `chess` app:
```json
[
    {
        "process_name": "chess",
        "process_wasm_path": "/chess.wasm",
        "on_exit": "Restart",
        "request_networking": true,
        "request_capabilities": [
            "net:distro:sys"
        ],
        "grant_capabilities": [
            "http-server:distro:sys"
        ],
        "public": true
    }
]
```
By setting `request_networking: true`, the kernel will give it the `"networking"` capability. In the `request_capabilities` field, `chess` is asking for the capability to message `net:distro:sys`.
Finally, in the `grant_capabilities` field, it is giving `http-server:distro:sys` the ability to message `chess`.

When booting the `chess` app, all of these capabilities will be granted throughout your node.
If you were to print out `chess`' capabilities using `hyperware_process_lib::our_capabilities() -> Vec<Capability>`, you would see something like this:

```rust
[
    // obtained because of `request_networking: true`
    Capability { issuer: "our-node.os@kernel:distro:sys", params: "\"network\"" },
    // obtained because we asked for it in `request_capabilities`
    Capability { issuer: "our-node.os@net:distro:sys", params: "\"messaging\"" }
]
```
Note that [userspace capabilities](#userspace-capabilities), those *created by other processes*, can also be requested in a package manifest, though it's not guaranteed that the user will have installed the process that can grant the capability.
Therefore, when a userspace process uses the capabilities system, it should have a way to grant capabilities through its `body` protocol, as described below.

## Userspace Capabilities

While the manifest fields are useful for getting a process started, it is not sufficient for creating and giving custom capabilities to other processes.
To create your own capabilities, simply declare a new one and attach it to a `Request` or `Response` like so:

```rust
let my_new_cap = hyperware_process_lib::Capability::new(our, "\"my-new-capability\"");

Request::new()
    .to(a_different_process)
    .capabilities(vec![my_new_cap])
    .send();
```

On the other end, if a process wants to save and reuse that capability, they can do something like this:

```rust
hyperware_process_lib::save_capabilities(req.capabilities);
```
This call will automatically save the caps for later use.
Next time you attach this capability to a message, whether that is for authentication with the `issuer`, or to share it with another process, it will reach the other side just fine, and they can check it using the exact same flow.

For a code example of creating and using capabilities in userspace, see the cookbook recipes.


# Startup, Spindown, and Crashes

# Startup, Spindown, and Crashes

Along with learning how processes communicate, understanding the lifecycle paradigm of Hyperware processes is essential to developing useful p2p applications.
Recall that a 'package' is a userspace construction that contains one or more processes.
The Hyperware kernel is only aware of processes.
When a process is first initialized, its compiled Wasm code is loaded into memory and, if the code is valid, the process is added to the kernel's process table.
Then, the kernel starts the process by calling the `init()` function (which is common to all processes).

This scenario is identical to when a process is re-initialized after being shut down. From the perspective of both the kernel and the process code, there is no difference.

## Defining Exit Behavior

In the [capabilities chapter](./capabilities.md), we saw `manifest.json` used to request and grant capabilities to a process. Another field in the manifest is `on_exit`, which defines the behavior of the process when it exits.

There are three possible behaviors for a process when it exits:

1. `OnExit::None` - The process is not restarted and nothing happens.

2. `OnExit::Restart` - The process is restarted.

3. `OnExit::Requests` - The process is not restarted, and a list of requests set by the process are fired off. These requests have the `source` and `capabilities` of the exiting process.

Once a process has been initialized it can exit in 4 ways:

1. Process code executes to completion -- `init()` returns.
2. Process code panics for any reason.
3. The kernel shuts it down via `KillProcess` call.
4. The runtime shuts it down via graceful exit or crash.

In the event of a runtime exit the process often is best suited by restarting on the next boot. But this should be optional. This is the impetus for `OnExit::Restart` and `OnExit::None`. However, `OnExit::Requests` is also useful in this case, as the process can notify the appropriate services (which restarted, most likely) that it has exited.

If a process is killed by the kernel, it doesn't make sense to honor `OnExit::Restart`. This would reduce the strength of KillProcess and forces a full package uninstall to get it to stop running. Therefore, `OnExit::Restart` is treated as `OnExit::None` in this case only.

*NOTE: If a process crashes for a 'structural' reason, i.e. the process code leads directly to a panic, and uses `OnExit::Restart`, it will crash continuously until it is uninstalled or killed manually.
Be careful of this!
The kernel waits an exponentially-increasing time between process restarts to avoid DOSing iteself with a crash-looping process.*

If a process executes to completion, its exit behavior is always honored.

Thus we can rewrite the three possible OnExit behaviors with their full accurate logic:

1. `OnExit::None` - The process is not restarted and nothing happens -- no matter what.

2. `OnExit::Restart` - The process is restarted, unless it was killed by the kernel, in which case it is treated as `OnExit::None`.

3. `OnExit::Requests` - The process is not restarted, and a list of `Request`s set by the process are fired off. These `Request`s have the `source` and `capabilities` of the exiting process. If the target process of a given `Request` in the list is no longer running, the `Request` will be dropped.


### Implications

Here are some good practices for working with these behaviors:

1. When a process has `OnExit::Restart` as its behavior, it should be written in such a way that it can restart at any time. This means that the `init()` function should start by picking up where the process may have left off, for example, reading from a local database that the process uses, or re-establishing an ETH RPC subscription (and making sure to `get_logs` for any events that may have been missed!).

2. Processes that produce 'child' processes should handle the exit behavior of those children. A parent process should usually use `OnExit::Restart` as its behavior unless it intends to hand off the child processes to another process via some established API. A child process can use `None`, `Restart`, or `Requests`, depending on its needs.

3. If a child process uses `OnExit::None`, the parent must be aware that the child could exit at any time and not notify the parent. This can be fine and easy to deal with if the parent has outstanding `Request`s to the child and can assume failure on timeout, or if the work-product of the child is irrelevant to the continued operations of the parent.

4. If a child process uses `OnExit::Restart`, the parent must be aware that the child will persist itself indefinitely. This is a natural fit for long-lived child processes which engage in cross-network activity and are themselves presenting a useful API. However, like `OnExit::None`, the parent will not be notified if the child process *is manually killed*. Again, the parent should be programmed to consider this.

5. If a child process uses `OnExit::Requests`, it has the ability to notify the parent process when it exits. This is quite useful for child processes that create a work-product to return to the parent or if it is important that the parent do some action immediately upon the child's exit. Note that the programmer must *create* the `Request`s in the child process. They can target any process, as long as the child process has the capability to message that target. The target will often simply be the parent process.

6. Requests made in `OnExit::Requests` must also comport to the capabilities requirements that applied to the process when it was alive.

7. If your processes does not have any places that it can panic, you don't have to worry about crash behavior, Rust is good for this :)

8. Parent processes "kill" child processes by building in a `Request` type that the child will respond to by exiting, which the parent can then send. The kernel does not actually have any conception of hierarchical process relationships. The actual kernel `KillProcess` command requires root capabilities to use, and it is unlikely that your app will acquire those.

## Persisting State With Processes

Given that nodes can, comporting with the realities of the physical world, be turned off, a well-written process must withstand being shut down and re-initialized at any time.
This raises the question: how does a process persist information between initializations?
There are two ways: either the process can use the built-in `set_state` and `get_state` functions, or it can send data to a process that does this for them.

The first option is a maximally-simple way to write some bytes to disk (where they'll be backed up, if the node owner has configured that behavior).
The second option is vastly more general, because runtime modules, which can be messaged directly from custom userspace processes, offer any number of APIs.
So far, there are three modules built into Hyperdrive that are designed for persisted data: a filesystem, a key-value store, and a SQLite database.

Each of these modules offer APIs accessed via message-passing and write data to disk.
Between initializations of a process, this data remains saved, even backed up.
The process can then retrieve this data when it is re-initialized.


# WIT APIs

# WIT APIs

This document describes how Hyperware processes use WIT to export or import APIs at a conceptual level.
If you are interested in usage examples, see the Package APIs documentation.

## High-level Overview

Hyperware runs processes that are [WebAssembly components](https://component-model.bytecodealliance.org/design/components.html), as discussed [elsewhere](processes.md#wasm-and-hyperware).
Two key advantages of WebAssembly components are

1. The declaration of types and functions using the cross-language Wasm Interface Type (WIT) language
2. The composibility of components.
See discussion [here](https://component-model.bytecodealliance.org/design/why-component-model.html).

Hyperware processes make use of these two advantages.
Processes within a package — a group of processes, also referred to as an app — may define an API in WIT format.
Each process defines a [WIT `interface`](https://component-model.bytecodealliance.org/design/wit.html#interfaces); the package defines a [WIT `world`](https://component-model.bytecodealliance.org/design/wit.html#interfaces).
The API is published alongside the package.
Other packages may then import and depend upon that API, and thus communicate with the processes in that package.
The publication of the API also allows for easy inspection by developers or by machines, e.g., LLM agents.

More than types can be published.
Because components are composable, packages may publish, along with the types in their API, library functions that may be of use in interacting with that package.
When set as as a dependency, these functions will be composed into new packages.
Libraries unassociated with packages can also be published and composed.

## WIT for Hyperware

The following is a brief discussion of the WIT language for use in writing Hyperware package APIs.
A more full discussion of the WIT language is [here](https://component-model.bytecodealliance.org/design/wit.html).

### Conventions

WIT uses `kebab-case` for multi-word variable names.
WIT uses `// C-style comments`.

Hyperware package APIs must be placed in the top-level `api/` directory.
They have a name matching the `PackageId` and appended with a version number, e.g.,
```
$ tree chat
chat
├── api
│   └── chat:template.os-v0.wit
...
```

### What WIT compiles into

WIT compiles into types of your preferred language.
Hyperware currently recommends Rust, but also supports Python and Javascript, with plans to support C/C++ and JVM languages like Java, Scala, and Clojure.
You can see the code generated by your WIT file using the [`wit-bindgen` CLI](https://github.com/bytecodealliance/wit-bindgen).
For example, to generate the Rust code for the `app-store` API in [Hyperdrive](https://github.com/hyperware-ai/hyperdrive/tree/main/hyperware/packages/app-store), use, e.g.,
```
kit b app-store
wit-bindgen rust -w app-store-sys-v1 --generate-unused-types --additional_derive_attribute serde::Deserialize app-store/app-store/target/wit
```

In the case of Rust, `kebab-case` WIT variable names become `UpperCamelCase`.

Rust `derive` macros can be applied to the WIT types in the `wit_bindgen::generate!` macro that appears in each process.
A typical macro invocation looks like
```rust
wit_bindgen::generate!({
    path: "target/wit",
    world: "chat-template-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});
```
where the field of interest here is the `additional_derives`.

### Types

[The built-in types of WIT](https://component-model.bytecodealliance.org/design/wit.html#built-in-types) closely mirror Rust's types, with the exception of sets and maps.
Users can define `struct`-like types called [`record`](https://component-model.bytecodealliance.org/design/wit.html#records)s, and `enum`-like types called [`variant`](https://component-model.bytecodealliance.org/design/wit.html#variants)s.
Users can also define [`func`](https://component-model.bytecodealliance.org/design/wit.html#functions)s with function signatures.

### Interfaces

[`interface`s](https://component-model.bytecodealliance.org/design/wit.html#interfaces) define a set of types and functions and, in Hyperware, are how a process signals its API.

### Worlds

[`world`s](https://component-model.bytecodealliance.org/design/wit.html#worlds) define a set of `import`s and `export`s and, in Hyperware, correspond to a package's API.
They can also `include` other `world`s, copying that `world`s `import`s and `export`s.
An `export` is an `interface` that a package defines and makes available, while an `import` is an `interface` that must be made available to the package.
If an `interface` contains only types, the presence of the WIT file is enough to provide that interface: the types can be generated from the WIT file.
However, if an `import`ed `interface` contains `func`s as well, a Wasm component is required that `export`s those functions.
For example, consider the `chat` template's `test/` package:

```
kit n chat
cat chat/test/chat_test/api/*
cat chat/api/*
```

Here, `chat-template-dot-os-v0` is the `test/` package `world`.
It `import`s types from `interface`s defined in two other WIT files: the top-level `chat` as well as [`tester`](https://github.com/hyperware-ai/hyperdrive/blob/main/hyperware/packages/tester/api/tester%3Asys-v0.wit).


# boot-fake-node

# `kit boot-fake-node`

short: `kit f`

`kit boot-fake-node` starts a "fake" node connected to a "fake" chain (i.e. not connected to the live network), e.g.,

```
kit boot-fake-node
```

By default, `boot-fake-node` fetches a prebuilt binary and launches the node using it.
Alternatively, `boot-fake-node` can build a local Hyperdrive repo and use the resulting binary.

It also boots a fake chain with [`anvil`](https://book.getfoundry.sh/anvil/) in the background (see [`kit chain`](../kit/chain.md)).
The fake chain comes preseeded with two contracts: HNS, which nodes use to index networking info of other nodes; and `app-store`, which nodes use to index published packages.

## Example Usage

You can start a network of fake nodes that can communicate with each other (but not the live network).
You'll need to start a new terminal for each fake node.
For example, to start two fake nodes, `fake.dev` and `fake2.dev`:

```bash
kit boot-fake-node

# In a new terminal
kit boot-fake-node -f fake2.dev -p 8081 -o /tmp/hyperware-fake-node-2

# Send a message from fake2.dev to fake.dev
# In the terminal of fake2.dev:
hi fake.dev hello!

# You should see "hello!" in the first node's terminal
```

## Discussion

Fake nodes make development easier.
A fake node is not connected to the network, but otherwise behaves the same as a live node.
Fake nodes are connected to each other on your local machine through a network router that passes messages between them.
Fake nodes also clean up after themselves, so you don't have to worry about state from a previous iterations messing up the current one.
If you wish to persist the state of a fake node between boots, you can do so with `--persist`.
Thus, fake nodes are an excellent testing ground during development for fast iteration.

There are some cases where fake nodes are not appropriate.
The weakness of fake nodes is also their strength: they are not connected to the live Hyperware network.
Though this lack of connectivity makes them easy to spin up and throw away, the downside is no access to services on the network which live nodes may provide.

## Arguments

```
$ kit boot-fake-node --help
Boot a fake node for development

Usage: kit boot-fake-node [OPTIONS]

Options:
  -r, --runtime-path <PATH>
          Path to Hyperdrive repo (overrides --version)
  -v, --version <VERSION>
          Version of Hyperdrive binary to use (overridden by --runtime-path) [default: latest] [possible values: latest]
  -p, --port <NODE_PORT>
          The port to run the fake node on [default: 8080]
  -o, --home <HOME>
          Path to home directory for fake node [default: /tmp/hyperdrive-fake-node]
  -f, --fake-node-name <NODE_NAME>
          Name for fake node [default: fake.dev]
  -c, --fakechain-port <FAKECHAIN_PORT>
          The port to run the fakechain on (or to connect to) [default: 8545]
      --rpc <RPC_ENDPOINT>
          Ethereum Optimism mainnet RPC endpoint (wss://)
      --persist
          If set, do not delete node home after exit
      --password <PASSWORD>
          Password to login [default: secret]
      --release
          If set and given --runtime-path, compile release build [default: debug build]
      --verbosity <VERBOSITY>
          Verbosity of node: higher is more verbose [default: 0]
  -h, --help
          Print help
```

# new

# `kit new`

short: `kit n`

`kit new` creates a Hyperware package template at the specified path, e.g.,

```
kit new foo
```

creates the default template (a Rust chat app with no UI) in the `foo/` directory.

The package name must be "Hypermap-safe": contain only a-z, 0-9, and `-`.

## Example Usage

```bash
# Create the default template: rust chat with no UI
kit new my-rust-chat

# Create rust chat with UI
kit new my-rust-chat-with-ui --ui
```

## Discussion

You can create a variety of templates using `kit new`.
Currently, one language is supported: `rust`.
Ask us in the [Discord](https://discord.com/invite/KaPXX7SFTD) about `python`, and `javascript` templates.
Four templates are currently supported, as described in the [following section](./new.html#existshas-ui-enabled-version).
In addition, some subset of these templates also have a UI-enabled version.

### Exists/Has UI-enabled Version

The following table specifies whether a template "Exists/Has UI-enabled version" for each language/template combination:

Language     | `chat`  | `echo` | `fibonacci` | `file-transfer`
------------ | ------- | ------ | ----------- | ---------------
`rust`       | yes/yes | yes/no | yes/no      | yes/no

Brief description of each template:

- `chat`: A simple chat app.
- `echo`: Echos back any message it receives.
- `fibonacci`: Computes the n-th Fibonacci number.
- `file-transfer`: Allows for file transfers between nodes.

## Arguments

```
$ kit new --help
Create a Hyperware template package

Usage: kit new [OPTIONS] <DIR>

Arguments:
  <DIR>  Path to create template directory at (must contain only a-z, 0-9, `-`)

Options:
  -a, --package <PACKAGE>      Name of the package (must contain only a-z, 0-9, `-`) [default: DIR]
  -u, --publisher <PUBLISHER>  Name of the publisher (must contain only a-z, 0-9, `-`, `.`) [default: template.os]
  -l, --language <LANGUAGE>    Programming language of the template [default: rust] [possible values: rust]
  -t, --template <TEMPLATE>    Template to create [default: chat] [possible values: blank, chat, echo, fibonacci, file-transfer]
      --ui                     If set, use the template with UI
  -h, --help                   Print help
```


# build

# `kit build`

short: `kit b`

`kit build` builds the indicated package directory, or the current working directory if none supplied, e.g.,

```
kit build foo
```

or

```
kit build
```

`kit build` builds each process in the package and places the `.wasm` binaries into the `pkg/` directory for installation with [`kit start-package`](./start-package.md).
It automatically detects what language each process is, and builds it appropriately (from amongst the supported `rust`, `python`, and `javascript`).

## Discussion

`kit build` builds a Hyperware package directory.
Specifically, it iterates through all directories within the given package directory and looks for `src/lib.??`, where the `??` is the file extension.
Currently, `rs` is supported, corresponding to processes written in `rust`.
Note that a package may have more than one process and those processes need not be written in the same language.

After compiling each process, it places the output `.wasm` binaries within the `pkg/` directory at the top-level of the given package directory.
Here is an example of what a package directory will look like after using `kit build`:

```
my-rust-chat
├── Cargo.lock
├── Cargo.toml
├── metadata.json
├── pkg
│   ├── manifest.json
│   ├── my-rust-chat.wasm
│   ├── scripts.json
│   └── send.wasm
├── my-rust-chat
│   └── ...
└── send
    └── ...
```

The `pkg/` directory is then zipped and can be injected into the node with [`kit start-package`](./start-package.md).

`kit build` also builds the UI if it is found in `pkg/ui/`.
There must exist a `ui/package.json` file with a `scripts` object containing the following arguments:
```json
"scripts": {
  "build": "tsc && vite build",
  "copy": "mkdir -p ../pkg/ui && rm -rf ../pkg/ui/* && cp -r dist/* ../pkg/ui/",
  "build:copy": "npm run build && npm run copy",
}
```

Additional UI dev info can be found in the documentation.
To both `build` and `start-package` in one command, use `kit build-start-package`.

## Arguments

```
$ kit build --help
Build a Hyperware package

Usage: kit build [OPTIONS] [DIR]

Arguments:
  [DIR]  The package directory to build [default: /home/nick]

Options:
      --no-ui
          If set, do NOT build the web UI for the process; no-op if passed with UI_ONLY
      --ui-only
          If set, build ONLY the web UI for the process; no-op if passed with NO_UI
  -i, --include <INCLUDE>
          Build only these processes/UIs (can specify multiple times) [default: build all]
  -e, --exclude <EXCLUDE>
          Build all but these processes/UIs (can specify multiple times) [default: build all]
  -s, --skip-deps-check
          If set, do not check for dependencies
      --features <FEATURES>
          Pass these comma-delimited feature flags to Rust cargo builds
  -p, --port <NODE_PORT>
          localhost node port; for remote see https://book.hyperware.ai/hosted-nodes.html#using-kit-with-your-hosted-node [default: 8080]
  -d, --download-from <NODE>
          Download API from this node if not found
  -w, --world <WORLD>
          Fallback WIT world name
  -l, --local-dependency <DEPENDENCY_PACKAGE_PATH>
          Path to local dependency package (can specify multiple times)
  -a, --add-to-api <PATH>
          Path to file to add to api.zip (can specify multiple times)
      --rewrite
          Rewrite the package (disables `Spawn!()`) [default: don't rewrite]
  -r, --reproducible
          Make a reproducible build using Docker
  -f, --force
          Force a rebuild
  -v, --verbose
          If set, output stdout and stderr
  -h, --help
          Print help

```


# start-package

# `kit start-package`

short: `kit s`

`kit start-package` installs and starts the indicated package directory (or current working directory) on the given Hyperware node (at `localhost:8080` by default), e.g.,

```
kit start-package foo
```

or

```
kit start-package
```

## Discussion

`kit start-package` injects a built package into the given node and starts it.
`start-package` is designed to be used after a package has been built with [`kit build`](./build.md).
The `pkg/` directory contains metadata about the package for the node as well as the `.wasm` binaries for each process.
The final step in the `build` process is to zip the `pkg/` directory.
`kit start-package` looks for the zipped `pkg/` and then injects a message to the node to start the package.

To both `build` and `start-package` in one command, use `kit build-start-package`.

## Arguments

```
$ kit start-package --help
Start a built Hyprware package

Usage: kit start-package [OPTIONS] [DIR]

Arguments:
  [DIR]  The package directory to start [default: /home/nick]

Options:
  -p, --port <NODE_PORT>  localhost node port; for remote see https://book.hyperware.ai/hosted-nodes.html#using-kit-with-your-hosted-node [default: 8080]
  -h, --help              Print help
```


# publish

# `kit publish`

short: `kit p`

`kit publish` creates entries in the Hypermap, publishing the given package according to the `app-store`s protocol.
It can also be used to update or unpublish previously-published packages.
`kit publish` writes directly to the Hypermap: it does not interact with a Hyperware node.

## Example Usage

```bash
# Publish a package on the real network (Optimism mainnet).
kit publish --metadata-uri https://raw.githubusercontent.com/path/to/metadata.json --keystore-path ~/.foundry/keystores/dev --rpc wss://opt-mainnet.g.alchemy.com/v2/<ALCHEMY_API_KEY> --real

# Unublish a package.
kit publish --metadata-uri https://raw.githubusercontent.com/path/to/metadata.json --keystore-path ~/.foundry/keystores/dev --rpc wss://opt-mainnet.g.alchemy.com/v2/<ALCHEMY_API_KEY> --real --unpublish
```

See [Sharing with the World](../my_first_app/chapter_5.md) for a tutorial on how to use `kit publish`.

## Arguments

```
$ kit publish --help
Publish or update a package

Usage: kit publish [OPTIONS] --metadata-uri <URI> --rpc <RPC_URI> [DIR]

Arguments:
  [DIR]  The package directory to publish [default: /home/nick]

Options:
  -k, --keystore-path <PATH>
          Path to private key keystore (choose 1 of `k`, `l`, `t`)
  -l, --ledger
          Use Ledger private key (choose 1 of `k`, `l`, `t`)
  -t, --trezor
          Use Trezor private key (choose 1 of `k`, `l`, `t`)
  -u, --metadata-uri <URI>
          URI where metadata lives
  -r, --rpc <RPC_URI>
          Ethereum Optimism mainnet RPC endpoint (wss://)
  -e, --real
          If set, deploy to real network [default: fake node]
      --unpublish
          If set, unpublish existing published package [default: publish a package]
  -g, --gas-limit <GAS_LIMIT>
          The ETH transaction gas limit [default: 1_000_000]
  -p, --priority-fee <MAX_PRIORITY_FEE_PER_GAS>
          The ETH transaction max priority fee per gas [default: estimated from network conditions]
  -f, --fee-per-gas <MAX_FEE_PER_GAS>
          The ETH transaction max fee per gas [default: estimated from network conditions]
  -m, --mock
          If set, don't actually publish: just dry-run
  -h, --help
          Print help
```


# build-start-package

# `kit build-start-package`

short: `kit bs`

`kit build-start-package` builds, installs and starts the indicated package directory, or the current working directory if none supplied, e.g.,

```
kit build-start-package foo
```

or

```
kit build-start-package
```

## Discussion

`kit build-start-package` runs [`kit build`](./build.md) followed by [`kit start-package`](./start-package.md).

## Arguments

```
$ kit build-start-package --help
Build and start a Hyperware package

Usage: kit build-start-package [OPTIONS] [DIR]

Arguments:
  [DIR]  The package directory to build [default: /home/nick/git/kit]

Options:
  -p, --port <NODE_PORT>
          localhost node port; for remote see https://book.hyperware.ai/hosted-nodes.html#using-kit-with-your-hosted-node [default: 8080]
  -d, --download-from <NODE>
          Download API from this node if not found
  -w, --world <WORLD>
          Fallback WIT world name
  -l, --local-dependency <DEPENDENCY_PACKAGE_PATH>
          Path to local dependency package (can specify multiple times)
  -a, --add-to-api <PATH>
          Path to file to add to api.zip (can specify multiple times)
      --no-ui
          If set, do NOT build the web UI for the process; no-op if passed with UI_ONLY
      --ui-only
          If set, build ONLY the web UI for the process
  -i, --include <INCLUDE>
          Build only these processes/UIs (can specify multiple times) (default: build all)
  -e, --exclude <EXCLUDE>
          Build all but these processes/UIs (can specify multiple times) (default: build all)
  -s, --skip-deps-check
          If set, do not check for dependencies
      --features <FEATURES>
          Pass these comma-delimited feature flags to Rust cargo builds
  -r, --reproducible
          Make a reproducible build using Docker
  -f, --force
          Force a rebuild
  -v, --verbose
          If set, output stdout and stderr
  -h, --help
          Print help
```


# remove-package

# `kit remove-package`

short: `kit r`

`kit remove-package` removes an installed package from the given node (defaults to `localhost:8080`).

For example,
```
kit remove-package foo
```

or

```
kit remove-package -package foo --publisher template.os
```

## Discussion

If passed an optional positional argument `DIR` (the path to a package directory), the `metadata.json` therein is parsed to get the `package_id` and that package is removed from the node.
If no arguments are provided, the same process happens for the current working directory.
Alternatively, a `--package` and `--publisher` can be provided as arguments, and that package will be removed.

## Arguments

```
$ kit remove-package --help
Remove a running package from a node

Usage: kit remove-package [OPTIONS] [DIR]

Arguments:
  [DIR]  The package directory to remove (Overridden by PACKAGE/PUBLISHER) [default: CWD]

Options:
  -a, --package <PACKAGE>      Name of the package (Overrides DIR)
  -u, --publisher <PUBLISHER>  Name of the publisher (Overrides DIR)
  -p, --port <NODE_PORT>       localhost node port; for remote see https://book.hyperware.ai/hosted-nodes.html#using-kit-with-your-hosted-node [default: 8080]
  -h, --help                   Print help
```


# chain

# kit chain

short: `kit c`

`kit chain` starts a local fakechain with foundry's [anvil](https://github.com/foundry-rs/foundry/tree/master/crates/anvil), e.g.,

```
kit chain
```

The default port is `8545` and the chain ID is `31337`.

## Discussion

`kit chain` starts an anvil node with the arguments `--load-state kinostate.json`.
This json file includes the HNS (Hyperware Name System) & `app-store` contracts, and is included in the `kit` binary.

The [`kinostate.json`](https://github.com/hyperware-ai/kit/blob/master/src/chain/kinostate) files can be found written into `/tmp/hyperdrive-kit-cache/kinostate-{hash}.json` upon running the command.

Note that while the `hns-indexer` and `app-store` apps in fake nodes use this chain to index events, any events loaded from a json statefile, aren't replayed upon restarting anvil.

## Arguments

```
$ kit chain --help
Start a local chain for development

Usage: kit chain [OPTIONS]

Options:
  -p, --port <PORT>        Port to run the chain on [default: 8545]
  -v, --version <VERSION>  Version of Hyperdrive to run chain for [default: latest] [possible values: latest, v1.1.0]
  -v, --verbose            If set, output stdout and stderr
  -h, --help               Print help
```


# dev-ui

# `kit dev-ui`

short: `kit d`

`kit dev-ui` starts a web development server with hot reloading for the indicated UI-enabled package (or the current working directory), e.g.,

```
kit dev-ui foo
```

or

```
kit dev-ui
```

## Arguments

```
$ kit dev-ui --help
Start the web UI development server with hot reloading (same as `cd ui && npm i && npm run dev`)

Usage: kit dev-ui [OPTIONS] [DIR]

Arguments:
  [DIR]  The package directory to build (must contain a `ui` directory) [default: CWD]

Options:
  -p, --port <NODE_PORT>  localhost node port; for remote see https://book.hyperware.ai/hosted-nodes.html#using-kit-with-your-hosted-node [default: 8080]
      --release           If set, create a production build
  -s, --skip-deps-check   If set, do not check for dependencies
  -h, --help              Print help
```



# inject-message

# `kit inject-message`

short: `kit i`

`kit inject-message` injects the given message to the node running at given port/URL, e.g.,

```bash
kit inject-message foo:foo:template.os '{"Send": {"target": "fake2.os", "message": "hello world"}}'
```

## Discussion

`kit inject-message` injects the given message into the given node.
It is useful for:
1. Testing processes from the outside world during development
2. Injecting data into the node
3. Combining the above with `bash` or other scripting.
For example, using the [`--blob`](#--blob) flag you can directly inject the contents of a file.
You can script in the outside world, dump the result to a file, and inject it with `inject-message`.

By default, `inject-message` expects a Response from the target process.
To instead "fire and forget" a message and exit immediately, use the [`--non-block`](#--non-block) flag.

## Arguments

```
$ kit inject-message --help
Inject a message to a running node

Usage: kit inject-message [OPTIONS] <PROCESS> <BODY_JSON>

Arguments:
  <PROCESS>    PROCESS to send message to
  <BODY_JSON>  Body in JSON format

Options:
  -p, --port <NODE_PORT>  localhost node port; for remote see https://book.hyperware.ai/hosted-nodes.html#using-kit-with-your-hosted-node [default: 8080]
  -n, --node <NODE_NAME>  Node ID (default: our)
  -b, --blob <PATH>       Send file at Unix path as bytes blob
  -l, --non-block         If set, don't block on the full node response
  -h, --help              Print help
```


# run-tests

# `kit run-tests`

short: `kit t`

`kit run-tests` runs the tests specified by the given `.toml` file, or `tests.toml`, e.g.,

```
kit run-tests my_tests.toml
```

or

```
kit run-tests
```

to run the current working directory's `tests.toml` or the current package's `test/`.

## Discussion

`kit run-tests` runs a series of tests specified  by [a `.toml` file](#teststoml).
Each test is run in a fresh environment of one or more fake nodes.
A test can setup one or more packages before running a series of test packages.
Each test package is [a single-process package that accepts and responds with certain messages](#test-package-interface).

Tests are orchestrated from the outside of the node by `kit run-tests` and run on the inside of the node by the [`tester`](https://github.com/hyperware-ai/hyperdrive/tree/main/hyperware/packages/tester) core package.
For a given test, the `tester` package runs the specified test packages in order.
Each test package must respond to the `tester` package with a `Pass` or `Fail`.
The `tester` package stops on the first `Fail`, or responds with a `Pass` if all tests `Pass`.
If a given test `Pass`es, the next test in the series is run.

Examples of tests are the [Hyperware Book's code examples](https://github.com/hyperware-ai/hyperware-book/tree/main/code) and [`kit`s templates](https://github.com/hyperware-ai/kit/tree/master/src/new/templates/rust).

## Arguments

```
$ kit run-tests --help
Run Hyperware tests

Usage: kit run-tests [PATH]

Arguments:
  [PATH]  Path to tests configuration file [default: tests.toml]

Options:
  -h, --help  Print help
```


# connect

# `kit connect`

`kit connect` is a thin wrapper over `ssh` to make creating SSH tunnels to remote nodes easy.

## Example Usage

Without any configuration, get your SSH Address from Valet, as discussed [here](../hosted-nodes.md#accessing-your-nodes-terminal).
Then use
```
kit connect --host <SSH Address>
```
and paste in the node's SSH password when prompted.
You will be prompted for your password twice.
This is to first determine the port to create the SSH tunnel to, and then to create the tunnel.
You can also provide the port (Valet displays it as Local HTTP Port) and only be prompted for password once:
```
kit connect --host <SSH Address> --port <Valet Local HTTP Port>
```

It is recommended to [set up your SSH configuration on your local machine and the remote host](../hosted-nodes.md#using-ssh-keys).
Then `kit connect` usage looks like:
```
kit connect --host <Host>
```
where `<Host>` here is defined in your `~/.ssh/config` file.

To disconnect an SSH tunnel, use the `--disconnect` flag and the local port bound, by default, `9090`:
```
kit connect 9090 --disconnect
```

## Discussion

See discussion of why SSH tunnels are useful for development with `kit` [here](../hosted-nodes.md#using-kit-with-your-hosted-node).
Briefly, creating an SSH tunnel allows you to use `kit` with a remote hosted node in the same way you do with a local one.
Setting up your SSH configuration will make `kit connect` work better.
You can find instructions for doing so [here](../hosted-nodes.md#using-ssh-keys).

## Arguments

```
$ kit connect --help
Connect (or disconnect) a ssh tunnel to a remote server

Usage: kit connect [OPTIONS] [LOCAL_PORT]

Arguments:
  [LOCAL_PORT]  Local port to bind [default: 9090]

Options:
  -d, --disconnect        If set, disconnect an existing tunnel [default: connect a new tunnel]
  -o, --host <HOST>       Host URL/IP node is running on (not required for disconnect)
  -p, --port <HOST_PORT>  Remote (host) port node is running on
  -h, --help              Print help
```


# boot-real-node

# `kit boot-real-node`

short: `kit e`

`kit boot-real-node` starts a Hyperware node connected to the live network, e.g.,

```
kit boot-real-node
```

By default, `boot-real-node` fetches a prebuilt binary and launches the node using it.
Alternatively, `boot-real-node` can build a local Hyperdrive repo and use the resulting binary.

## Example Usage

You can create a new node, creating a home directory at, e.g., `~/<my-new-node-name>.os`, using

```
kit boot-real-node --home ~/<my-new-node-name>.os
```

or you can boot an existing node with home directory at, e.g., `~/<my-old-node-name>.os`, using

```
kit boot-real-node --home ~/<my-old-node-name>.os
```

## Discussion

`kit boot-real-node` makes it easier to run a node by reducing the number of steps to download the Hyperdrive binary and launch a node.
Be cautious using `boot-real-node` before Hyperdrive `1.0.0` launch without specifying the `--version` flag: the default `--version latest` may use a new major version of Hyperdrive!

## Arguments

```
$ kit boot-real-node --help
Boot a real node

Usage: kit boot-real-node [OPTIONS] --home <HOME>

Options:
  -r, --runtime-path <PATH>    Path to Hyperdrive repo (overrides --version)
  -v, --version <VERSION>      Version of Hyperdrive to use (overridden by --runtime-path) [default: latest] [possible values: latest, v0.8.7, v0.8.6, v0.8.5]
  -p, --port <NODE_PORT>       The port to run the real node on [default: 8080]
  -o, --home <HOME>            Path to home directory for real node
      --rpc <RPC_ENDPOINT>     Ethereum Optimism mainnet RPC endpoint (wss://)
      --release                If set and given --runtime-path, compile release build [default: debug build]
      --verbosity <VERBOSITY>  Verbosity of node: higher is more verbose [default: 0]
  -h, --help                   Print help
```


# Environment Setup

# Environment Setup

In this section, you'll walk through setting up a Hyperware development environment.
By the end, you will have created a Hyperware application, or package, composed of one or more processes that run on a live Hyperware.
The application will be a simple chat interface: `my-chat-app`.

The following assumes a Unix environment — macOS or Linux.
If on Windows, [get WSL](https://learn.microsoft.com/en-us/windows/wsl/install) first.
In general, Hyperware does not support development on Windows.

## Acquiring Rust and the Hyperware Development Tools (`kit`)

Install Rust and the Hyperware Development Tools, or `kit`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install --git https://github.com/hyperware-ai/kit --locked
```

You can find a video guide that walks through setting up `kit` [here](https://www.youtube.com/watch?v=N8B_s_cm61k).

## Creating a New Hyperware Package Template

The `kit` toolkit has a variety of features.
One of those tools is `new`, which creates a template for a Hyperware package.
The `new` tool takes two arguments: a path to create the template directory and a name for the package:

```
$ kit new --help
Create a Hyperware template package

Usage: kit new [OPTIONS] <DIR>

Arguments:
  <DIR>  Path to create template directory at (must contain only a-z, 0-9, `-`)

Options:
  -a, --package <PACKAGE>      Name of the package (must contain only a-z, 0-9, `-`) [default: DIR]
  -u, --publisher <PUBLISHER>  Name of the publisher (must contain only a-z, 0-9, `-`, `.`) [default: template.os]
  -l, --language <LANGUAGE>    Programming language of the template [default: rust] [possible values: rust]
  -t, --template <TEMPLATE>    Template to create [default: chat] [possible values: blank, chat, echo, fibonacci, file-transfer]
      --ui                     If set, use the template with UI
  -h, --help                   Print help
```

Create a package `my-chat-app` (you can name it anything "Hypermap-safe", i.e. containing only a-z, 0-9, `-`; but we'll assume you're working with `my-chat-app` in this document):

```bash
kit new my-chat-app
```

## Exploring the Package

Hyperware packages are sets of one or more Hyperware [processes](../system/process/processes.md).
A Hyperware package is represented in Unix as a directory that has a `pkg/` directory within.
Each process within the package is its own directory.
By default, the `kit new` command creates a simple, one-process package, a chat app.
Other templates, including a Python template and a UI-enabled template can be used by passing [different flags to `kit new`](../kit/new.html#discussion).
The default template looks like:

```
$ tree my-chat-app
my-chat-app
├── api
│   └── my-chat-app:template.os-v0.wit
├── Cargo.toml
├── metadata.json
├── my-chat-app
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
├── pkg
│   ├── manifest.json
│   └── scripts.json
├── send
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
└── test
    ├── my-chat-app-test
    │   ├── api
    │   │   └── my-chat-app-test:template.os-v0.wit
    │   ├── Cargo.toml
    │   ├── metadata.json
    │   ├── my-chat-app-test
    │   │   ├── Cargo.toml
    │   │   └── src
    │   │       ├── lib.rs
    │   │       └── tester_lib.rs
    │   └── pkg
    │       └── manifest.json
    └── tests.toml
```

The `my-chat-app/` package here contains two processes, each represented by a directory:
- `my-chat-app/` — containing the main application code, and
- `send/` — containing a script.

Rust process directories, like the ones here, contain:
- `src/` — source files where the code for the process lives, and
- `Cargo.toml` — the standard Rust file specifying dependencies, etc., for that process.

Another standard Rust `Cargo.toml` file, a [virtual manifest](https://doc.rust-lang.org/cargo/reference/workspaces.html#virtual-workspace) is also included in `my-chat-app/` root.

Also within the package directory is a `pkg/` directory.
The `pkg/` dirctory contains two files:
- `manifest.json` — required: specifes information Hyperware needs to run the package, and
- `scripts.json` — optional: specifies details needed to run scripts.

The `pkg/` directory is also where `.wasm` binaries (and, optionally, built UI files) will be deposited by [`kit build`](#building-the-package).
The files in the `pkg/` directory are injected into the Hyperware node with [`kit start-package`](#starting-the-package).

The `metadata.json` is a required file that contains app metadata which is used in the Hyperware [App Store](./chapter_5.html).

The `api/` directory contains the [WIT API](../system/process/wit_apis.md) for the `my-chat-app` package, see more discussion [below](#api).

Lastly, the `test/` directory contains tests for the `my-chat-app` package.
The `tests.toml` file specifies the configuration of the tests.
The `my-chat-app-test/` directory is itself a package: the test for `my-chat-app`.
For more discussion of tests see [`kit run-tests`](../kit/run-tests.md), or see usage, [below](#testing-the-package).

Though not included in this template, packages with a frontend have a `ui/` directory as well.
For an example, look at the result of:
```bash
kit new my-chat-app-with-ui --ui
tree my-chat-app-with-ui
```
Note that not all templates have a UI-enabled version.
More details about templates can be found [here](../kit/new.html#existshas-ui-enabled-version).

### `pkg/manifest.json`

The `manifest.json` file contains information the node needs in order to run the package:

```bash
$ cat my-chat-app/pkg/manifest.json
[
    {
        "process_name": "my-chat-app",
        "process_wasm_path": "/my-chat-app.wasm",
        "on_exit": "Restart",
        "request_networking": true,
        "request_capabilities": [
            "http-server:distro:sys",
            "vfs:distro:sys"
        ],
        "grant_capabilities": [],
        "public": true
    }
]
```

This is a JSON array of JSON objects.
Each object represents one process that will be started when the package is installed.
A package with multiple processes need not start them all at install time.
A package may start more than one of the same process, as long as they each have a unique `process_name`.

Each object requires the following fields:

Key                      | Value Type                                                                                     | Description
------------------------ | ---------------------------------------------------------------------------------------------- | -----------
`"process_name"`         | String                                                                                         | The name of the process
`"process_wasm_path"`    | String                                                                                         | The path to the process
`"on_exit"`              | String (`"None"` or `"Restart"`) or Object (covered [elsewhere](./chapter_2.md#aside-on_exit)) | What to do in case the process exits
`"request_networking"`   | Boolean                                                                                        | Whether to ask for networking capabilities from kernel
`"request_capabilities"` | Array of Strings or Objects                                                                    | Strings are `ProcessId`s to request messaging capabilties from; Objects have a `"process"` field (`ProcessId` to request from) and a `"params"` field (capability to request)
`"grant_capabilities"`   | Array of Strings or Objects                                                                    | Strings are `ProcessId`s to grant messaging capabilties to; Objects have a `"process"` field (`ProcessId` to grant to) and a `"params"` field (capability to grant)
`"public"`               | Boolean                                                                                        | Whether to allow any process to message us

### `metadata.json`

The `metadata.json` file contains ERC721 compatible metadata about the package.
The only required fields are `package_name`, `current_version`, and `publisher`, which are filled in with default values:

```bash
$ cat my-chat-app/metadata.json
{
    "name": "my-chat-app",
    "description": "",
    "image": "",
    "properties": {
        "package_name": "my-chat-app",
        "current_version": "0.1.0",
        "publisher": "template.os",
        "mirrors": [],
        "code_hashes": {
            "0.1.0": ""
        },
        "wit_version": 1,
        "dependencies": []
    },
    "external_url": "",
    "animation_url": ""
}
```
Here, the `publisher` is the default value (`"template.os"`), but for a real package, this field should contain the HNS ID of the publishing node.
The `publisher` can also be set with a `kit new --publisher` flag.
The `wit_version` is an optional field:

`wit_version` value | Resulting `hyperware.wit` version
------------------- | ------------------------------
`1`                 | [`hyperware.wit` `1.0.0`](https://github.com/hyperware-ai/hyperware-wit/blob/v1.0.0/hyperware.wit)

The `dependencies` field is also optional; see discussion in [WIT APIs](../system/process/wit_apis.md).
The rest of these fields are not required for development, but become important when publishing a package with the [`app-store`](https://github.com/hyperware-ai/hyperdrive/tree/main/hyperware/packages/app-store).

As an aside: each process has a unique `ProcessId`, used to address messages to that process, that looks like

```
<process-name>:<package-name>:<publisher-node>
```

Each field separated by `:`s must be "Hypermap safe", i.e. can only contain a-z, 0-9, `-` (and, for publisher node, `.`).

You can read more about `ProcessId`s [here](../system/process/processes.md#overview).

### `api/`

The `api/` directory is an optional directory where packages can declare their public API.
Other packages can then mark a package as a dependency in their `metadata.json` to include those types and functions defined therein.
The API is useful for composability and for LLM agents as definitions of "tools" for programatic access.

For further reading, see discussion in [WIT APIs](../system/process/wit_apis.md).

## Building the Package

To build the package, use the [`kit build`](../kit/build.md#) tool.

This tool accepts an optional directory path as the first argument, or, if none is provided, attempts to build the current working directory.
As such, either of the following will work:

```bash
kit build my-chat-app
```

or

```bash
cd my-chat-app
kit build
```

## Booting a Fake Node

Often, it is optimal to develop on a fake node.
Fake nodes are simple to set up, easy to restart if broken, and mocked networking makes development testing very straightforward.
To boot a fake node for development purposes, use the [`kit boot-fake-node` tool](../kit/boot-fake-node.md).

`kit boot-fake-node` downloads the OS- and architecture-appropriate Hyperdrive binary and runs it without connecting to the live network.
Instead, it connects to a mocked local network, allowing different fake nodes on the same machine to communicate with each other.
`kit boot-fake-node` has many optional configuration flags, but the defaults should work fine:

```bash
kit boot-fake-node
```

The fake node, just like a real node, will accept inputs from the terminal.
To exit from the fake node, press `Ctrl + C`.

By default, the fake node will bind to port `8080`.
Note the port number in the output for [later](#starting-the-package); it will look something like:

```
Serving Hyperdrive at http://localhost:8080
```

`kit boot-fake-node` also accepts a `--runtime-path` argument.
When supplied, if it is a path to the Hyperdrive repo, it will compile and use that binary to start the node.
Or, if it is a path to a Hyperdrive, it will use that binary to start the node.
For example:

```bash
kit boot-fake-node --runtime-path ~/path/to/hyperdrive
```

where `~/path/to/hyperdrive` must be replaced with a path to the Hyperdrive repo.

Note that your node will be named `fake.dev`, as opposed to `fake.os`.
The `.dev` suffix is used for development nodes.

## Optional: Starting a Real Node

Alternatively, development sometimes calls for a real node, which has access to the actual Hyperware network and its providers.

To develop on a real Node, connect to the network and follow the instructions to setup a node.

## Starting the Package

Now it is time to load and initiate the `my-chat-app` package. For this, you will use the [`kit start-package`](../kit/start-package.md) tool.
Like [`kit build`](#building-the-package), the `kit start-package` tool takes an optional directory argument — the package — defaulting to the current working directory.
It also accepts a URL: the address of the node on which to initiate the package.
The node's URL can be input in one of two ways:

1. If running on localhost, the port can be supplied with `-p` or `--port`,
2. More generally, the node's entire URL can be supplied with a `-u` or `--url` flag.

If neither the `--port` nor the `--url` is given, `kit start-package` defaults to `http://localhost:8080`.

You can start the package from either within or outside `my-chat-app` directory.
After completing the previous step, you should be one directory above the `my-chat-app` directory and can use the following:

```bash
kit start-package my-chat-app -p 8080
```

or, if you are already in the correct package directory:

```bash
kit start-package -p 8080
```

where here the port provided following `-p` must match the port bound by the node or fake node (see discussion [above](#booting-a-fake-node)).

The node's terminal should display something like

```
Thu 22:51 app-store:sys: successfully installed my-chat-app:template.os
```

Congratulations: you've now built and installed your first application on Hyperware!

## Using the Package

To test out the functionality of `my-chat-app`, spin up another fake node to chat with in a new terminal:

```bash
kit boot-fake-node -o /tmp/hyperware-fake-node-2 -p 8081 -f fake2.dev
```

The fake nodes communicate over a mocked local network.

To start the same `my-chat-app` on the second fake node, again note the port, and supply it with a `start-package`:

```bash
kit start-package my-chat-app -p 8081
```

or, if already in the `my-chat-app/` package directory:

```bash
kit start-package -p 8081
```

To send a chat message from the first node, run the following in its terminal:

```
m our@my-chat-app:my-chat-app:template.os '{"Send": {"target": "fake2.dev", "message": "hello world"}}'
```

and replying, from the other terminal:

```
m our@my-chat-app:my-chat-app:template.os '{"Send": {"target": "fake.dev", "message": "wow, it works!"}}'
```

Messages can also be injected from the outside.
From a bash terminal, use `kit inject-message`, like so:

```bash
kit inject-message my-chat-app:my-chat-app:template.os '{"Send": {"target": "fake2.dev", "message": "hello from the outside world"}}'
kit inject-message my-chat-app:my-chat-app:template.os '{"Send": {"target": "fake.dev", "message": "replying from fake2.dev using first method..."}}' --node fake2.dev
kit inject-message my-chat-app:my-chat-app:template.os '{"Send": {"target": "fake.dev", "message": "and second!"}}' -p 8081
```

## Testing the Package

To run the `my-chat-app/` tests, *first close all fake nodes* and then run

```bash
kit run-tests my-chat-app
```

or, if already in the `my-chat-app/` package directory:

```bash
kit run-tests
```

For more details, see [`kit run-tests`](../kit/run-tests.md).


# Sending and Responding to a Message

# Sending and Responding to a Message

In this section you will learn how to use different parts of a process, how `Request`-`Response` handling works, and other implementation details with regards to messaging.
The process you will build will be simple — it messages itself and responds to itself, printing whenever it gets messages.

Note — the app you will build in Sections 2 through 5 is *not* `my-chat-app`; it is simply a series of examples designed to demonstrate how to use the system's features.

## Requirements

This section assumes you've completed the steps outlined in [Environment Setup](./chapter_1.md) to setup your development environment or otherwise have a basic Hyperware app open in your code editor of choice.
You should also be actively running a Hyperware node (live or [fake](./chapter_1.md#booting-a-fake-hyperware-node)) such that you can quickly compile and test your code!
Tight feedback loops when building: very important.

## Starting from Scratch

If you want to hit the ground running by yourself, you can take the template code or the chess tutorial and start hacking away.
Here, you'll start from scratch and learn about every line of boilerplate.
Open `src/lib.rs`, clear its contents so it's empty, and code along!

The last section explained packages, the package manifest, and metadata.
Every package contains one or more processes, which are the actual Wasm programs that will run on a node.

The [Generating WIT Bindings](#generating-wit-bindings) and [`init()` Function](#init-function) subsections explain the boilerplate code in detail, so if you just want to run some code, you can skip to [Running First Bits of Code](#running-first-bits-of-code).

### Generating WIT Bindings

For the purposes of this tutorial, crucial information from this [Wasm documentation](https://component-model.bytecodealliance.org/design/why-component-model.html) has been abridged in this small subsection.

A [Wasm component](https://component-model.bytecodealliance.org/design/components.html) is a wrapper around a core module that specifies its imports and exports.
E.g. a Go component can communicate directly and safely with a C or Rust component.
It need not even know what language another component was written in — it needs only the component interface, expressed in WIT.

The external interface of a component — its imports and exports — is described by a [`world`](https://component-model.bytecodealliance.org/design/wit.html#worlds).
Exports are provided by the component, and define what consumers of the component may call; imports are things the component may call.
The component, however, internally defines how that `world` is implemented.
This interface is defined via [WIT](https://component-model.bytecodealliance.org/design/wit.html).

WIT bindings are the glue code that is necessary for the interaction between Wasm modules and their host environment.
They may be written in any Wasm-compatible language — Hyperware offers the most support for Rust with `kit` and `process_lib`.
The `world`, types, imports, and exports are all declared in a [WIT file](https://github.com/hyperware-ai/hyperdrive-wit/blob/v0.8/hyperware.wit), and using that file, [`wit_bindgen`](https://github.com/bytecodealliance/wit-bindgen) generates the code for the bindings.

So, to bring it all together...

Every process must generate WIT bindings based on a WIT file for either the default `process-v1` world or a package-specific `world` in order to interface with the Hyperware kernel:

```rust
wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v1",
});
```

### `init()` Function

The entrypoint of a process is the `init()` function.
You can use the `call_init!()` macro as shown below.
Skip to the code to see how to do so.
The following prose is an explanation of what is happening under-the-hood in the `call_init()` macro.

After generating the bindings, every process must define a `Component` struct which implements the `Guest` trait (i.e. a wrapper around the process which defines the export interface, as discussed [above](#generating-wit-bindings)).
The `Guest` trait should define a single function — `init()`.
This is the entry point for the process, and the `init()` function is the first function called by the Hyperware runtime (such as Hyperdrive) when the process is started.

The definition of the `Component` struct can be done manually, but it's easier to import the `hyperware_process_lib` crate (a sort of standard library for Hyperware processes written in Rust) and use the `call_init!` macro.

```rust
use hyperware_process_lib::{await_message, call_init, println, Address, Request, Response};

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v1",
});

call_init!(init);
fn init(our: Address) {
...
```

Every Hyperware process written in Rust will need code that does the same thing as the code above (i.e. use the `wit_bindgen::generate!()` and `call_init!()` macros).
See `hyperware.wit` for more details on what is imported by the WIT bindgen macro.
These imports are the necessary "system calls" for talking to other processes and runtime components on Hyperware.
Note that there are a variety of imports from the `process_lib` including a `println!` macro that replaces the standard Rust one.

The [`our` parameter](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/hyperware/process/standard/struct.Address.html) tells the process what its globally-unique name is.

The `init()` function can either do one task and then return, or it can `loop`, waiting for a new message and then acting based on the nature of the message.
The first pattern is more usual for scripts that do one task and then exit.
The second pattern is more usual for long-lived state machine processes that, e.g., serve data over the network or over HTTP.

## Sending a Message

The [`Request`](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/struct.Request.html) type from the `process_lib` provides all the necessary functionality to send a Message.

`Request` is a builder struct that abstracts over the raw interface presented in the WIT bindings.
It's very simple to use:
```rust
    Request::to(my_target_address)
        .body(my_body_bytes)
        .send();
```

Because this process might not have capabilities to message any other (local or remote) processes, for the purposes of this tutorial, just send the message to itself.

```rust
    Request::to(&our)
        .body(b"hello world")
        .send();
```

Note that `send()` returns a Result.
If you know that a `target` and `body` was set, you can safely unwrap this: send will only fail if one of those two fields are missing.

You can modify your `Request` to expect a `Response`, and your message-handling to send one back, as well as parse the received `Request` into a string.

```rust
    Request::to(&our)
        .body(b"hello world")
        .expects_response(5)
        .send()
        .unwrap();
```

The `expects_response` method takes a timeout in seconds.
If the timeout is reached, the `Request` will be returned to the process that sent it as an error.
If you add that to the code above, you'll see the error after 5 seconds in your node's terminal.

## Responding to a Message

Now, consider how to handle the `Request`.
The `await_message()` function returns a `Result` that looks like this:
```rust
Result<Message, SendError>
```

The `SendError` is returned when a `Request` times out or, if the `Request` passes over the network, in case of a networking issue.
Use a `match` statement to check whether the incoming value is a message or an error, then branch on whether the message is a `Request` or a `Response`.
To send a `Response` back, import the `Response` type from `process_lib` and send one from the `Request` branch.

```rust
    loop {
        match await_message() {
            Err(send_error) => println!("got SendError: {send_error}"),
            Ok(message) => {
                let body = String::from_utf8_lossy(message.body());
                if message.is_request() {
                    println!("got a request: {body}");
                    Response::new()
                        .body(b"hello world to you too!")
                        .send()
                        .unwrap();
                } else {
                    println!("got a response: {body}");
                }
            }
        }
    }
```

Putting it all together:

```rust
use hyperware_process_lib::{await_message, call_init, println, Address, Request, Response};

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v1",
});

call_init!(init);
fn init(our: Address) {
    println!("begin");

    Request::to(&our)
        .body(b"hello world")
        .expects_response(5)
        .send()
        .unwrap();

    loop {
        match await_message() {
            Err(send_error) => println!("got SendError: {send_error}"),
            Ok(message) => {
                let body = String::from_utf8_lossy(message.body());
                if message.is_request() {
                    println!("got a request: {body}");
                    Response::new()
                        .body(b"hello world to you too!")
                        .send()
                        .unwrap();
                } else {
                    println!("got a response: {body}");
                }
            }
        }
    }
}
```

Run
```bash
kit build your_pkg_directory
kit start-package your_pkg_directory -p 8080
```
to see the messages being sent by your process.

You can find the full code [here](https://github.com/hyperware-ai/hyperware-book/tree/main/code/mfa-message-demo).

The basic structure of this process — an infinite loop of `await_message()` and then handling logic — can be found in the majority of Hyperware processes.
The other common structure is a script-like process, that handles and sends a fixed series of messages and then exits.

In the next section, you will learn how to turn this very basic `Request-`Response` pattern into something that can be extensible and composable.

# Messaging with More Complex Data Types

# Messaging with More Complex Data Types

In this section, you will upgrade your app so that it can handle messages with more elaborate data types such as `enum`s and `struct`s.
Additionally, you will learn how to handle processes completing or crashing.

## (De)Serialization With Serde

In the last section, you created a simple request-response pattern that uses strings as a `body` field type.
This is fine for certain limited cases, but in practice, most Hyperware processes written in Rust use a `body` type that is serialized and deserialized to bytes using [Serde](https://serde.rs/).
There are a multitude of libraries that implement Serde's `Serialize` and `Deserialize` traits, and the process developer is responsible for selecting a strategy that is appropriate for their use case.

Some popular options are [`bincode`](https://docs.rs/bincode/latest/bincode/), [`rmp_serde`](https://docs.rs/rmp-serde/latest/rmp_serde/) ([MessagePack](https://msgpack.org/index.html)), and [`serde_json`](https://docs.rs/serde_json/latest/serde_json/).
In this section, you will use `serde_json` to serialize your Rust structs to a byte vector of JSON.

### Defining the `body` Type

Our old request looked like this:
```rust
    Request::to(&our)
        .body(b"hello world")
        .expects_response(5)
        .send()
        .unwrap();
```

What if you want to have two kinds of messages, which your process can handle differently?
You need a type that implements the `serde::Serialize` and `serde::Deserialize` traits, and use that as your `body` type.
You can define your types in Rust, but then:
1. Processes in other languages will then have to rewrite your types.
2. Importing types is haphazard and on a per-package basis.
3. Every package might place the types in a different place.

Instead, use the WIT language to define your API, discussed further [here](../system/process/wit_apis.md).
Briefly, WIT is a language-independent way to define types and functions for [Wasm components](https://component-model.bytecodealliance.org/design/why-component-model.html) like Hyperware processes.
Hyperware packages can define their API using a WIT file.
That WIT file is used to generate code in the given language during compile-time.
Hyperware also defines a conventional place for these WIT APIs and provides infrastructure for viewing and importing the APIs of other packages.

```wit
interface mfa-data-demo {
    variant request {
        hello(string),
        goodbye,
    }

    variant response {
        hello(string),
        goodbye,
    }
}

world mfa-data-demo-template-dot-os-v0 {
    import mfa-data-demo;
    include process-v1;
}
```

The `wit_bindgen::generate!()` macro changes slightly, since the `world` is now as defined in the API:
```rust
wit_bindgen::generate!({
    path: "target/wit",
    world: "mfa-data-demo-template-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});
```
which generates the types defined in the WIT API:
```rust
use crate::hyperware::process::mfa_data_demo::{Request as MfaRequest, Response as MfaResponse};
```
It further adds the derives for `serde` so that these types can be used smoothly.

Now, when you form Requests and Responses, instead of putting a bytes-string in the `body` field, you can use the `MfaRequest`/`MfaResponse` type.
This comes with a number of benefits:

- You can now use the `body` field to send arbitrary data, not just strings.
- Other programmers can look at your code and see what kinds of messages this process might send to their code.
- Other programmers can see what kinds of messages you expect to receive.
- By using an `enum` ([WIT `variant`s become Rust `enum`s](https://component-model.bytecodealliance.org/design/wit.html#variants)), you can exhaustively handle all possible message types, and handle unexpected messages with a default case or an error.

Defining `body` types is just one step towards writing interoperable code.
It's also critical to document the overall structure of the program along with message `blob`s and `metadata` used, if any.
Writing interoperable code is necessary for enabling permissionless composability, and Hyperware aims to make this the default kind of program, unlike the centralized web.

### Handling Messages

In this example, you will learn how to handle a Request.
So, create a request that uses the new `body` type:

```rust
    Request::to(&our)
        .body(MfaRequest::Hello("hello world".to_string()))
        .expects_response(5)
        .send()
        .unwrap();
```

Next, change the way you handle a message in your process to use your new `body` type.
Break out the logic to handle a message into its own function, `handle_message()`.
`handle_message()` should branch on whether the message is a Request or Response.
Then, attempt to parse every message into the `MfaRequest`/`MfaResponse`, `enum` as appropriate, handle the two cases, and handle any message that doesn't comport to the type.
```rust
fn handle_message(message: &Message) -> anyhow::Result<bool> {
    if message.is_request() {
        match message.body().try_into()? {
            MfaRequest::Hello(text) => {
                println!("got a Hello: {text}");
                Response::new()
                    .body(MfaResponse::Hello("hello to you too!".to_string()))
                    .send()?
            }
            MfaRequest::Goodbye => {
                println!("goodbye!");
                Response::new().body(MfaResponse::Goodbye).send()?;
                return Ok(true);
            }
        }
    } else {
        match message.body().try_into()? {
            MfaResponse::Hello(text) => println!("got a Hello response: {text}"),
            MfaResponse::Goodbye => println!("got a Goodbye response"),
        }
    }
    Ok(false)
}

```

### Granting Capabilities

Finally, edit your `pkg/manifest.json` to grant the terminal process permission to send messages to this process.
That way, you can use the terminal to send `Hello` and `Goodbye` messages.
Go into the manifest, and under the process name, edit (or add) the `grant_capabilities` field like so:

```json
...
        "grant_capabilities": [
            "terminal:terminal:sys",
            "tester:tester:sys"
        ],
...
```

### Build and Run the Code!

After all this, your code should look like:
```rust
use crate::hyperware::process::mfa_data_demo::{Request as MfaRequest, Response as MfaResponse};
use hyperware_process_lib::{await_message, call_init, println, Address, Message, Request, Response};

wit_bindgen::generate!({
    path: "target/wit",
    world: "mfa-data-demo-template-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

fn handle_message(message: &Message) -> anyhow::Result<bool> {
    if message.is_request() {
        match message.body().try_into()? {
            MfaRequest::Hello(text) => {
                println!("got a Hello: {text}");
                Response::new()
                    .body(MfaResponse::Hello("hello to you too!".to_string()))
                    .send()?
            }
            MfaRequest::Goodbye => {
                println!("goodbye!");
                Response::new().body(MfaResponse::Goodbye).send()?;
                return Ok(true);
            }
        }
    } else {
        match message.body().try_into()? {
            MfaResponse::Hello(text) => println!("got a Hello response: {text}"),
            MfaResponse::Goodbye => println!("got a Goodbye response"),
        }
    }
    Ok(false)
}

call_init!(init);
fn init(our: Address) {
    println!("begin");

    Request::to(&our)
        .body(MfaRequest::Hello("hello world".to_string()))
        .expects_response(5)
        .send()
        .unwrap();

    loop {
        match await_message() {
            Err(send_error) => println!("got SendError: {send_error}"),
            Ok(ref message) => match handle_message(message) {
                Err(e) => println!("got error while handling message: {e:?}"),
                Ok(should_exit) => {
                    if should_exit {
                        return;
                    }
                }
            },
        }
    }
}
```
You should be able to build and start your package, then see that initial `Hello` message.
At this point, you can use the terminal to test your message types!

First, try sending a `Hello` using the `m` terminal script.
Get the address of your process by looking at the "started" printout that came from it in the terminal.
As a reminder, these values (`<your_process>`, `<your_package>`, `<your_publisher>`) can be found in the `metadata.json` and `manifest.json` package files.

```bash
m our@<your-process>:<your-package>:<your-publisher> '{"Hello": "hey there"}'
```

You should see the message text printed.
To grab and print the Response, append a `-a 5` to the terminal command:
```bash
m our@<your-process>:<your-package>:<your-publisher> '{"Hello": "hey there"}' -a 5
```
Next, try a goodbye.
This will cause the process to exit.

```bash
m our@<your-process>:<your-package>:<your-publisher> '"Goodbye"'
```

If you try to send another `Hello` now, nothing will happen, because the process has exited [(assuming you have set `on_exit: "None"`; with `on_exit: "Restart"` it will immediately start up again)](#aside-on_exit).
Nice!
You can use `kit start-package` to try again.

## Aside: `on_exit`

As mentioned in the [previous section](./chapter_1.md#pkgmanifestjson), one of the fields in the `manifest.json` is `on_exit`.
When the process exits, it does one of:

`on_exit` Setting | Behavior When Process Exits
----------------- | ---------------------------
`"None"`          | Do nothing
`"Restart"`       | Restart the process
JSON object       | Send the requests described by the JSON object

A process intended to do something once and exit should have `"None"` or a JSON object `on_exit`.
If it has `"Restart"`, it will repeat in an infinite loop.

A process intended to run over a period of time and serve requests and responses will often have `"Restart"` `on_exit` so that, in case of crash, it will start again.
Alternatively, a JSON object `on_exit` can be used to inform another process of its untimely demise.
In this way, Hyperware processes become quite similar to Erlang processes in that crashing can be [designed into your process to increase reliability](https://ferd.ca/the-zen-of-erlang.html).

You can find the full code [here](https://github.com/hyperware-ai/hyperware-book/tree/main/code/mfa-data-demo).


# Frontend Time

# Frontend Time

After the last section, you should have a simple process that responds to two commands from the terminal.
In this section, you'll add some basic HTTP logic to serve a frontend and accept an HTTP PUT request that contains a command.



## Adding HTTP request handling

Using the built-in HTTP server will require handling a new type of Request in our main loop, and serving a Response to it.
The `process_lib` contains types and functions for doing so.

At the top of your process, import [`get_blob`](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/hyperware/process/standard/fn.get_blob.html), [`homepage`](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/homepage/index.html), and [`http`](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/http/index.html) from `hyperware_process_lib` along with the rest of the imports.
You'll use `get_blob()` to grab the `body` bytes of an incoming HTTP request.

Keep the custom WIT-defined `MfaRequest` the same, and keep using that for terminal input.

At the beginning of the `init()` function, in order to receive HTTP requests, use the `hyperware_process_lib::http` library to bind a new path.
Binding a path will cause the process to receive all HTTP requests that match that path.
You can also bind static content to a path using another function in the library.

```rust
...
fn init(our: Address) {
    println!("begin");

    let server_config = http::server::HttpBindingConfig::default().authenticated(false);
    let mut server = http::server::HttpServer::new(5);
    server.bind_http_path("/", server_config.authenticated(false)).unwrap();
...
```

[`http::HttpServer::bind_http_path("/", server_config)`](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/http/server/struct.HttpServer.html#method.bind_http_path) arguments mean the following:
1. The first argument is the path to bind.
   Note that requests will be namespaced under the process name, so this will be accessible at e.g. `/my-process-name/`.
2. The second argument [configures the binding](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/http/server/struct.HttpBindingConfig.html).
   A default setting here serves the page only to the owner of the node, suitable for private app access.
   Here, setting `authenticated(false)` serves the page to anyone with the URL.

To handle different kinds of Requests (or Responses), wrap them in a meta `Req` or `Res`:
```rust
#[derive(Debug, serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto)]
#[serde(untagged)] // untagged as a meta-type for all incoming responses
enum Req {
    MfaRequest(MfaRequest),
    HttpRequest(http::server::HttpServerRequest),
}
```
and `match` on it in the top-level `handle_message()`:
```rust
fn handle_message(our: &Address, message: &Message) -> Result<bool> {
    if message.is_request() {
        match message.body().try_into()? {
            Req::MfaRequest(ref mfa_request) => {
                return Ok(handle_mfa_request(mfa_request)?);
            }
            Req::HttpRequest(http_request) => {
                handle_http_request(our, http_request)?;
            }
        }
    } else {
        handle_mfa_response(message.body().try_into()?)?;
    }
    Ok(false)
}
```

Here, the [logic that was previously](./chapter_3.md#handling-messages) in `handle_message()` is now factored out into `handle_mfa_request()` and `handle_mfa_response()`:

```rust
fn handle_mfa_request(request: &MfaRequest) -> Result<bool> {
    match request {
        MfaRequest::Hello(text) => {
            println!("got a Hello: {text}");
            Response::new()
                .body(MfaResponse::Hello("hello to you too!".to_string()))
                .send()?
        }
        MfaRequest::Goodbye => {
            println!("goodbye!");
            Response::new().body(MfaResponse::Goodbye).send()?;
            return Ok(true);
        }
    }
    Ok(false)
}

...

fn handle_mfa_response(response: MfaResponse) -> Result<()> {
    match response {
        MfaResponse::Hello(text) => println!("got a Hello response: {text}"),
        MfaResponse::Goodbye => println!("got a Goodbye response"),
    }
    Ok(())
}
```

As a side-note, different apps will want to discriminate between incoming messages differently.
For example, to restrict what senders are accepted (say to your own node or to some set of allowed nodes), your process can branch on the [`source().node`](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/enum.Message.html#method.source).

### Handling an HTTP Message

Finally, define `handle_http_message()`.
```rust
fn handle_http_request(our: &Address, request: http::server::HttpServerRequest) -> Result<()> {
    let Some(http_request) = request.request() else {
        return Err(anyhow!("received a WebSocket message, skipping"));
    };
    if http_request.method().unwrap() != http::Method::PUT {
        return Err(anyhow!("received a non-PUT HTTP request, skipping"));
    }
    let Some(body) = get_blob() else {
        return Err(anyhow!(
            "received a PUT HTTP request with no body, skipping"
        ));
    };
    http::server::send_response(http::StatusCode::OK, None, vec![]);
    Request::to(our).body(body.bytes).send().unwrap();
    Ok(())
}
```

Walking through the code, first, you must parse out the HTTP request from the `HttpServerRequest`.
This is necessary because the `HttpServerRequest` enum contains both HTTP protocol requests and requests related to WebSockets.
If your application only needs to handle one type of request (e.g., only HTTP requests), you could simplify the code by directly handling that type without having to check for a specific request type from the `HttpServerRequest` enum each time.
This example is overly thorough for demonstration purposes.

```rust
    let Some(http_request) = request.request() else {
        return Err(anyhow!("received a WebSocket message, skipping"));
    };
```

Next, check the HTTP method in order to only handle PUT requests:
```rust
    if http_request.method().unwrap() != http::Method::PUT {
        return Err(anyhow!("received a non-PUT HTTP request, skipping"));
    }
```

Finally, grab the `blob` from the request, send a `200 OK` response to the client, and handle the `blob` by sending a `Request` to ourselves with the `blob` as the `body`.
```rust
    let Some(body) = get_blob() else {
        return Err(anyhow!(
            "received a PUT HTTP request with no body, skipping"
        ));
    };
    http::server::send_response(http::StatusCode::OK, None, vec![]);
    Request::to(our).body(body.bytes).send().unwrap();
```
This could be done in a different way, but this simple pattern is useful for letting HTTP requests masquerade as in-Hyperware requests.

Putting it all together, you get a process that you can build and start, then use cURL to send `Hello` and `Goodbye` requests via HTTP PUTs!

### Requesting Capabilities

Also, remember to request the capability to message `http-server` in `manifest.json`:
```json
...
"request_capabilities": [
    "http-server:distro:sys"
],
...
```

### The Full Code

```rust
use anyhow::{anyhow, Result};

use crate::hyperware::process::mfa_data_demo::{Request as MfaRequest, Response as MfaResponse};
use hyperware_process_lib::{
    await_message, call_init, get_blob, homepage, http, println, Address, Message, Request,
    Response,
};

wit_bindgen::generate!({
    path: "target/wit",
    world: "mfa-data-demo-template-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

// base64-encoded bytes prepended with image type like `data:image/png;base64,`, e.g.
// echo "data:image/png;base64,$(base64 < gosling.png)" | tr -d '\n' > icon
const ICON: &str = include_str!("./icon");

// you can embed an external URL
// const WIDGET: &str = "<iframe src='https://example.com'></iframe>";
// or you can embed your own HTML
const WIDGET: &str = "<html><body><h1>Hello, Hyperware!</h1></body></html>";

#[derive(Debug, serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto)]
#[serde(untagged)] // untagged as a meta-type for all incoming responses
enum Req {
    MfaRequest(MfaRequest),
    HttpRequest(http::server::HttpServerRequest),
}

fn handle_mfa_request(request: &MfaRequest) -> Result<bool> {
    match request {
        MfaRequest::Hello(text) => {
            println!("got a Hello: {text}");
            Response::new()
                .body(MfaResponse::Hello("hello to you too!".to_string()))
                .send()?
        }
        MfaRequest::Goodbye => {
            println!("goodbye!");
            Response::new().body(MfaResponse::Goodbye).send()?;
            return Ok(true);
        }
    }
    Ok(false)
}

fn handle_http_request(our: &Address, request: http::server::HttpServerRequest) -> Result<()> {
    let Some(http_request) = request.request() else {
        return Err(anyhow!("received a WebSocket message, skipping"));
    };
    if http_request.method().unwrap() != http::Method::PUT {
        return Err(anyhow!("received a non-PUT HTTP request, skipping"));
    }
    let Some(body) = get_blob() else {
        return Err(anyhow!(
            "received a PUT HTTP request with no body, skipping"
        ));
    };
    http::server::send_response(http::StatusCode::OK, None, vec![]);
    Request::to(our).body(body.bytes).send().unwrap();
    Ok(())
}

fn handle_mfa_response(response: MfaResponse) -> Result<()> {
    match response {
        MfaResponse::Hello(text) => println!("got a Hello response: {text}"),
        MfaResponse::Goodbye => println!("got a Goodbye response"),
    }
    Ok(())
}

fn handle_message(our: &Address, message: &Message) -> Result<bool> {
    if message.is_request() {
        match message.body().try_into()? {
            Req::MfaRequest(ref mfa_request) => {
                return Ok(handle_mfa_request(mfa_request)?);
            }
            Req::HttpRequest(http_request) => {
                handle_http_request(our, http_request)?;
            }
        }
    } else {
        handle_mfa_response(message.body().try_into()?)?;
    }
    Ok(false)
}

call_init!(init);
fn init(our: Address) {
    println!("begin");

    let server_config = http::server::HttpBindingConfig::default().authenticated(false);
    let mut server = http::server::HttpServer::new(5);
    server.bind_http_path("/api", server_config).unwrap();
    server
        .serve_file(
            "ui/index.html",
            vec!["/"],
            http::server::HttpBindingConfig::default(),
        )
        .unwrap();
    homepage::add_to_homepage("My First App", Some(ICON), Some("/"), Some(WIDGET));

    Request::to(&our)
        .body(MfaRequest::Hello("hello world".to_string()))
        .expects_response(5)
        .send()
        .unwrap();

    loop {
        match await_message() {
            Err(send_error) => println!("got SendError: {send_error}"),
            Ok(ref message) => match handle_message(&our, message) {
                Err(e) => println!("got error while handling message: {e:?}"),
                Ok(should_exit) => {
                    if should_exit {
                        return;
                    }
                }
            },
        }
    }
}
```

Use the following cURL command to send a `Hello` Request
Make sure to replace the URL with your node's local port and the correct process name.
Note: if you had set `authenticated` to true in `bind_http_path()`, you would need to add an `Authorization` header to this request with the [JWT](https://jwt.io/) cookie of your node.
This is saved in your browser automatically on login.

```bash
curl -X PUT -d '{"Hello": "greetings"}' http://localhost:8080/mfa_fe_demo:mfa_fe_demo:template.os/api
```

You can find the full code [here](https://github.com/hyperware-ai/hyperware-book/tree/main/code/mfa-fe-demo).

There are a few lines we haven't covered yet: learn more about [serving a static frontend](#serving-a-static-frontend) and [adding a homepage icon and widget](#adding-a-homepage-icon-and-widget) below.

## Serving a static frontend

If you just want to serve an API, you've seen enough now to handle PUTs and GETs to your heart's content.
But the classic personal node app also serves a webpage that provides a user interface for your program.

You *could* add handling to root `/` path to dynamically serve some HTML on every GET.
But for maximum ease and efficiency, use the static bind command on `/` and move the PUT handling to `/api`.
To do this, edit the bind commands in `my_init_fn` to look like this:

```rust
    let mut server = http::server::HttpServer::new(5);
    server.bind_http_path("/api", server_config).unwrap();
    server
        .serve_file(
            "ui/index.html",
            vec!["/"],
            http::server::HttpBindingConfig::default(),
        )
        .unwrap();
    homepage::add_to_homepage("My First App", Some(ICON), Some("/"), Some(WIDGET));
```

Here you are setting `authenticated` to `false` in the [`bind_http_path()`](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/http/server/struct.HttpServer.html#method.bind_http_path) call, but to `true` in the [`serve_file`](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/http/server/struct.HttpServer.html#method.serve_file) call.
This means the API is public; if instead you want the webpage to be served exclusively by the browser, change `authenticated` to `true` in `bind_http_path()` as well.

You must also add a static `index.html` file to the package.
UI files are stored in the `ui/` directory and built into the application by `kit build` automatically.
Create a `ui/` directory in the package root, and then a new file in `ui/index.html` with the following contents.
**Make sure to replace the fetch URL with your process ID!**

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
  </head>
  <body>
    <main>
        <h1>This is a website!</h1>
        <p>Enter a message to send to the process:</p>
        <form id="hello-form" class="col">
        <input id="hello" required="" name="hello" placeholder="hello world" value="">
        <button> PUT </button>
      </form>
    </main>
    <script>
        async function say_hello(text) {
          const result = await fetch("/mfa-fe-demo:mfa-fe-demo:template.os/api", {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ "Hello": text }),
          });
          console.log(result);
        }


        document.addEventListener("DOMContentLoaded", () => {
          const form = document.getElementById("hello-form");
          form.addEventListener("submit", (e) => {
            e.preventDefault();
            e.stopPropagation();
            const text = document.getElementById("hello").value;
            say_hello(text);
          });
        });
    </script>
  </body>
</html>

```

This is a super barebones `index.html` that provides a form to make requests to the `/api` endpoint.
Additional UI dev info can be found in the documentation.

Next, add two more entries to `manifest.json`: messaging capabilities to the VFS which is required to store and access the UI `index.html`, and the `homepage` capability which is required to add our app to the user's homepage (next section):
```json
...
        "request_capabilities": [
            "homepage:homepage:sys",
            "http-server:distro:sys",
            "vfs:distro:sys"
        ],
...
```

After saving `ui/index.html`, rebuilding the program, and starting the package again with `kit bs`, you should be able to navigate to your `http://localhost:8080/mfa_fe_demo:mfa_fe_demo:template.os` and see the form page.
Because you now set `authenticated` to `true` in the `/api` binding, the webpage will still work, but cURL will not.

The user will navigate to `/` to see the webpage, and when they make a PUT request, it will automatically happen on `/api` to send a message to the process.

This frontend is now fully packaged with the process — there are no more steps!
Of course, this can be made arbitrarily complex with various frontend frameworks that produce a static build.

In the next and final section, learn about the package metadata and how to share this app across the Hyperware network.


## Adding a Homepage Icon and Widget

In this section, you will learn how to customize your app icon with a clickable link to your frontend, and how to create a widget to display on the homepage.

### Adding the App to the Home Page

#### Encoding an Icon

Choosing an emblem is a difficult task.
You may elect to use your own, or use this one:

![gosling](../assets/gosling.png)

On the command line, encode your image as base64, and prepend `data:image/png;base64,`:

```bash
echo "data:image/png;base64,$(base64 < gosling.png)" | tr -d '\n' > icon
```

Then, move `icon` next to `lib.rs` in your app's `src/` directory.
Finally, include the icon data in your `lib.rs` file just after the imports:

```rust
const ICON: &str = include_str!("./icon");
```

#### Clicking the Button

The Hyperware process lib exposes an [`add_to_homepage()`](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/homepage/fn.add_to_homepage.html) function that you can use to add your app to the homepage.

In your `init()`, add the following line:
This line in the `init()` function adds your process, with the given icon, to the homepage:

```rust
    server.bind_http_path("/api", server_config).unwrap();
```

### Writing a Widget

A widget is an HTML iframe.
Hyperware apps can send widgets to the `homepage` process, which will display them on the user's homepage.
They are quite simple to configure.
In `add_to_homepage()`, the final field optionally sets the widget:

```rust
    server.bind_http_path("/api", server_config).unwrap();
```
which uses the `WIDGET` constant, here:
```rust
// you can embed an external URL
// const WIDGET: &str = "<iframe src='https://example.com'></iframe>";
// or you can embed your own HTML
const WIDGET: &str = "<html><body><h1>Hello, Hyperware!</h1></body></html>";
```

After another `kit bs`, you should be able to reload your homepage and see your app icon under "All Apps", as well as your new widget.
To dock your app, click the heart icon on it.
Click the icon itself to go to the UI served by your app.

For an example of a more complex widget, see the source code of our [app store widget](#widget-case-study-app-store), below.

#### Widget Case Study: App Store

The app store's [widget](https://github.com/hyperware-ai/hyperdrive/blob/3719ab38e19143a7bcd501fd245c7a10b2239ee7/hyperware/packages/app-store/app-store/src/http_api.rs#L59C1-L133C2) makes a single request to the node, to determine the apps that are listed in the app store.
It then creates some HTML to display the apps in a nice little list.

```html
<html>
{{#webinclude https://raw.githubusercontent.com/hyperware-ai/hyperdrive/3719ab38e19143a7bcd501fd245c7a10b2239ee7/hyperware/packages/app-store/app-store/src/http_api.rs 62:130}}
</html>
```


# Sharing with the World

# Sharing with the World

So, you've made a new process.
You've tested your code and are ready to share with friends, or perhaps just install across multiple nodes in order to do more testing.

First, it's a good idea to publish the code to a public repository.
This can be added to your package `metadata.json` like so:
```json
...
"website": "https://github.com/your_package_repo",
...
```
At a minimum you will need to publish the `metadata.json`.
An easy option is to publish it on GitHub.
If you use GitHub, make sure to use the static link to the specific commit, not a branch-specific URL (e.g. `main`) that wil change with new commits.
For example, `https://raw.githubusercontent.com/nick1udwig/chat/master/metadata.json` is not the correct link to use, because it will change when new commits are added.
You want to use a link like `https://raw.githubusercontent.com/nick1udwig/chat/191dce595ad00a956de04b9728f479dee04863c7/metadata.json` which will not change when new commits are added.

You'll need to populate the `code_hashes` field of `metadata.json`.
The hash can be found by running `kit build` on your package: it will be output after a successful build.

Next, review all the data in [`pkg/manifest.json`](./chapter_1.md#pkgmanifestjson) and [`metadata.json`](./chapter_1.md#pkgmetadatajson).
The `package_name` field in `metadata.json` determines the name of the package.
The `publisher` field determines the name of the publisher (you!).

Once you're ready to share, it's quite easy.

If you are developing on a fake node, you'll have to boot a real one, then install this package locally in order to publish on the network, e.g.
```
kit s my-package
```

## Using the App Store GUI

Navigate to the App Store and follow the `Publish` flow, which will guide you through publishing your application.

## Using [`kit publish`](../kit/publish.md)

Alternatively, you can publish your application from the command-line using [`kit publish`](../kit/publish.md).
To do so, you'll either need to
1. Create a keystore.
2. Use a Ledger.
3. Use a Trezor.

The keystore is an encrypted wallet private key: the key that owns your publishing node.
[See below](#making-a-keystore) for discussion of how to create the keystore.
To use a hardware wallet, simply input the appropriate flag to `kit publish` (`-l` for Ledger or `-t` for Trezor).

In addition, you'll need an ETH RPC endpoint.
See the `OPTIONAL: Acquiring an RPC API Key` section for a walkthrough of how to get an Alchemy API key.

### Making a Keystore

Keystores, also known as [Web3 Secret Storage](https://ethereum.org/en/developers/docs/data-structures-and-encoding/web3-secret-storage/), can be created in many ways; here, use [`foundry`](https://getfoundry.sh/)s `cast`.
First, [get `foundry`](https://getfoundry.sh/), and then run:
```
cast wallet import -i my-wallet
```
following the prompts to create your keystore named `my-wallet`.

### Running [`kit publish`](../kit/publish.md)

To publish your package, run:
```
kit publish --metadata-uri https://raw.githubusercontent.com/path/to/metadata.json --keystore-path ~/.foundry/keystores/my-wallet --rpc wss://opt-mainnet.g.alchemy.com/v2/<ALCHEMY_API_KEY> --real
```
and enter the password you created when making the keystore, here `my-wallet`.

Congratulations, your app is now live on the network!


# Hosted Nodes User Guide

# Hosted Nodes User Guide

Sybil Technologies runs a Hyperware hosting service for users who do not want to run a node themselves.
These hosted nodes are useful for both end users and developers.
This guide is largely targeted at developers who want to develop Hyperware applications using their hosted Hyperware node.
End users may also find the [Managing Your Node](#managing-your-node) section useful.

Here, `ssh` is used extensively.
This guide is specifically tailored to `ssh`s use for the Hyperware hosting service.
A more expansive guide can be found [here](https://www.digitalocean.com/community/tutorials/ssh-essentials-working-with-ssh-servers-clients-and-keys).

## Managing Your Node

[Valet](https://valet.uncentered.systems) is the interface for managing your hosted node.
We plan to open-source the hosting code so there will be other hosting options in the future.
Once logged in, `Your Nodes` will be displayed: clicking on the name of a node will navigate to the homepage for that node.
Clicking on the gear icon will open a modal with some information about the node.
Click `Show advanced details` to reveal information for accessing your nodes terminal.

## Accessing Your Nodes Terminal

As discussed in [Managing Your Node](#managing-your-node), navigate to:
1. [https://valet.uncentered.systems](https://valet.uncentered.systems)
2. `Your Nodes`
3. Gear icon
4. `Show advanced details`

In the advanced details, note the `SSH Address` and `SSH Password`.

To access your node remote instance, open a terminal and
```bash
ssh <SSH Address>
```
where `<SSH Address>` should be replaced with the one from your Valet advanced details.
You will be prompted for a password: copy-paste the `SSH Password`.

You should now have a different terminal prompt, indicating you have `ssh`d into the remote instance hosting your node.

### Using SSH keys

Rather than typing in a password to create a SSH connection, you can use a keypair.

#### Generating Keypair

[How to generate a keypair](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/generating-a-new-ssh-key-and-adding-it-to-the-ssh-agent#generating-a-new-ssh-key)

#### `ssh-agent`

[How to use `ssh-agent` to store a keypair](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/generating-a-new-ssh-key-and-adding-it-to-the-ssh-agent#adding-your-ssh-key-to-the-ssh-agent)

#### SSH Config

[How to use `~/.ssh/config` to make SSH easier to use](https://www.digitalocean.com/community/tutorials/ssh-essentials-working-with-ssh-servers-clients-and-keys#defining-server-specific-connection-information)

#### Adding Public Key to Remote Node

[How to add the public key to a remote node to allow login with it](https://www.digitalocean.com/community/tutorials/ssh-essentials-working-with-ssh-servers-clients-and-keys#copying-your-public-ssh-key-to-a-server-with-ssh-copy-id)

## Using `kit` With Your Hosted Node

`kit` interacts with a node through the nodes HTTP RPC.
However, Hyperware limits HTTP RPC access to localhost — remote requests are rejected.
The local limit is a security measure, since the HTTP RPC allows injection of arbitrary messages with "root" capabilities.

To use `kit` with a hosted node, you need to create a SSH tunnel, which maps a port on your local machine to a port on the nodes remote host.
HTTP requests routed to that local port will then appear to the remote host as originating from its localhost.

It is recommended to use [`kit connect`](./kit/connect.md) to create and destroy a SSH tunnel.
Otherwise, you can also follow the instructions below to do it yourself.

Create a SSH tunnel like so (again, replacing [assumed values with those in your `advanced details`](#accessing-your-nodes-terminal)):
```bash
ssh -L 9090:localhost:<HTTP port> <SSH address>
```
e.g.,
``` bash
ssh -L 9090:localhost:8099 kexampleuser@template.hosting.hyperware.ai
```

or, if you've added your host to your [`~/.ssh/config`](#ssh-config),
```bash
ssh -L 9090:localhost:<HTTP port> <Host>
```
You should see a `ssh` session open.
While this session is open, `kit` requests sent to `9090` will be routed to the remote node, e.g.,
```
kit s foo -p 9090
```
will function the same as for a locally-hosted node.

Closing the `ssh` connections with `Ctrl+D` or typing `exit` will close the tunnel.


# Glossary

# Glossary

Hyperware uses a variety of technical terms.
The glossary defines those terms.

## Address

[Processes](#process) have a globally-unique [address](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/hyperware/process/standard/struct.Address.html) to and from which [messages](#message) can be routed.

## App Store

The Hyperware App Store is the place where users download Hyperware apps and where devs distribute [Hyperware packages](#package).


## Blob

See [LazyLoadBlob](#LazyLoadBlob).

## Capability

Hyperware uses [capabilities](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/hyperware/process/standard/struct.Capability.html) to restrict what [processes](#process) can do.
A capability is issued by a process (the "issuer") and signed by the [kernel](#kernel).
The holder of a capability can then attach that capability to a message.
The kernel will confirm it is valid by checking the signature.
If valid, it will be passed to the recipient of the message.
Only trust capabilties from [local](#local) holders!
A [remote](#remote) [node](#node) need not follow the rules above (i.e. it may run a modified [runtime](#runtime)).

There are system-level and userspace-level capabilities.

System-level capabilities are of two types:
- `"messaging"`, which allows the holder to send messages to the issuer.
- `"net"`, which allows the holder to send/receive messages over the network to/from remote nodes.

System-level capabilities need not be attached explicitly to messages.
They are requested and granted at process start-time in the [manifest](#manifest).

Userspace-level capabilities are defined within a process.
They are issued by that process.
Holders must explictly attach these capabilities to their messages to the issuing process.
The issuer must define the logic that determines what happens if a sender has or does not have a capability.
E.g., the system Contacts app defines capabilities [here](https://github.com/hyperware-ai/hyperdrive/blob/main/hyperware/packages/contacts/api/contacts%3Asys-v0.wit#L2-L7) and the logic that allows/disallows access given a sender's capabilities [here](https://github.com/hyperware-ai/hyperdrive/blob/main/hyperware/packages/contacts/contacts/src/lib.rs#L291-L314).

## Inherit

## Kernel

The Hyperware microkernel is responsible for:
1. Starting and stopping [processes](#process).
2. Routing [messages](#message).
3. Enforcing [capabilities](#capability).

## Hyperdrive

The reference Hyperware [runtime](#runtime).

It is written in Rust, uses [wasmtime](https://github.com/bytecodealliance/wasmtime) to run [processes](#process), and lives [here](https://github.com/hyperware-ai/hyperdrive).

## Hypermap

Hypermap is the onchain component of Hyperware.
Hypermap is a path-value map.
Protocols can be defined on Hypermap.

Examples:

The HNS protocol stores contact information for all [nodes](#node) in Hypermap entries.
That contact information looks like:
1. A public key.
2. Either an IP address or a list of other nodes that will route messages to that node.
The `hns-indexer` Hyperware [process](#process) reads the Hypermap, looking for these specific path/entries, and then uses that information to contact other nodes offchain.

The Hyperware [App Store](#app-store) protocol stores the app metadata URI and hash of that metadata in Hypermap entries.
The `app-store` Hyperware process reads the Hypermap, looking for these specific path/entries, and then uses that information to coordinate:
1. Fetching app information.
2. Finding mirrors to download from (over the Hyperware network or HTTP).
3. Confirming those mirrors gave the expected files.
4. Fetching and installing updates, if desired, when made available.

Read more in the Hypermap documentation.

## Hypermap-safe

A String containing only a-z, 0-9, `-`, and, for a publisher [node](#node), `.`.

## LazyLoadBlob

An optional part of a [message](#message) that is "loaded lazily".
The purpose of he [LazyLoadBlob](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/hyperware/process/standard/struct.LazyLoadBlob.html) is to avoid the cost of repeatedly bringing large data across the Wasm boundary in a [message-chain](#message-chain) when only the ends of the chain need to access the data.

## Local

Of or relating to our [node](#node).
Contrasts with [remote](#remote).

E.g., a local [process](#process) is one that is running on our node.
[Messages](#message) sent to a local process need not traverse the Hyperware network.

Capabilities attached to messages received from a local process can be trusted since the [kernel](#kernel) can be trusted.

## Manifest

## Message

Hyperware [processes](#process) communicate with each other by passing messages.
Messages are [addressed](#address) to a local or remote [process](#process), contain some content, and have a variety of associated metadata.
Messages can be [requests](#request) or [responses](#response).
Messages can set off [message-chains](#message-chain) of requests and responses.
A process that sends a request must specify the address of the recipient.
In contrast, a response will be routed automatically to the sender of the most recently-received request in the message-chain that expected a response.

## Message-chain

## Module

A module, or runtime module, is similar to a [process](#process).
[Messages](#message) are [addressed](#address) to and received from a module just like a process.
The difference is that processes are [Wasm components](#wasm-component), which restricts them in a number of ways, e.g., to be single-threaded.
Runtime modules do not have these same restrictions.
As such they provide some useful features for processes, such as access to the Hyperware network, a virtual file system, databases, the Ethereum blockchain, an HTTP server and client, etc.

## Node

A node (sometimes referred to as a Hyperware node) is a server running the Hyperware [runtime](#runtime).
It communicates with other nodes over the Hyperware network using [message](#message) passing.
It has a variety of [runtime modules](#module) and also runs userspace [processes](#process) which are [Wasm components](#component).

## Package

An "app".
A set of one-or-more [processes](#process) along with one-or-more UIs.
Packages can be distributed using the Hyperware [App Store](#app-store).

Packages have a unique [identity](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/hyperware/process/standard/struct.PackageId.html).

## PackageId

## Process

Hyperware processes are the code bundles that make up userspace.
Hyperware processes are [Wasm components](#wasm-component) that use either the [Hyperware process WIT file](https://github.com/hyperware-ai/hyperdrive-wit/blob/v1.0.0/hyperware.wit) or that define their own [WIT](#wit) file that wraps the Hyperware process WIT file.

Processes have a unique [identity](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/hyperware/process/standard/struct.ProcessId.html) and a globally unique [address](#address).

## ProcessId

## Remote

Of or relating to someone else's [node](#node).
Contrasts with [local](#local).

E.g., a remote [process](#process) is one that is running elsewhere.
[Messages](#message) sent to a remote process must traverse the Hyperware network.

Capabilities attached to messages received from a remote process cannot be trusted since the [kernel](#kernel) run by that remote node might be modified.
E.g., the hypothetical modified kernel might take all capabilities issued to any process it runs and give it to all processes it runs.

## Request

A [message](#message) that requires the [address](#address) of the recipient.
A [request](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/struct.Request.html) can start off a messsage-chain if the sender sets metadata that indicates it expects a [response](#response).

## Response

A [message](#message) that is automatically routed to the sender of the most recently-received [request](#request) in the [message-chain](#message-chain) that expected a [response](https://docs.rs/hyperware_process_lib/latest/hyperware_process_lib/struct.Response.html).

## Runtime

The part of the Hyperware stack the runs the microkernel and other [runtime modules](#module).

The reference implementation is called [Hyperdrive](#hyperdrive).

## Wasm Component

[The WebAssembly Component Model](https://component-model.bytecodealliance.org/) is a standard that builds on top of WebAssembly and WASI.
Wasm components define their interfaces using [WIT](#wit).
Hyperware [processes](#process) are Wasm components.

## WIT

WIT is the [Wasm Interface Type](https://component-model.bytecodealliance.org/design/wit.html).
WIT is used to define the interface for a [Wasm component](#wasm-component).
Hyperware [processes](#process) must use either the [Hyperware process WIT file](https://github.com/hyperware-ai/hyperdrive-wit/blob/v1.0.0/hyperware.wit) or define their own WIT file that wraps the Hyperware process WIT file


