# 🧱 TOREL LANGUAGE SPEC v0.1: THE SERIOUS BEAST IS BORN

Official branding: **TOREL - Typed Ownership and Resource Effects Language**.

Friendly name: **Torel**.

Technical name: **TOREL**.

Compiler: `torelc`.

Main CLI: `torel`.

File extension: `.torel`.

LLVM backend label: `LLVM.TOREL`.

Manifest: `Torel.toml` and `Torel.lock`.

Torel is a **strict native systems/backend language** with elite math/science capability, designed around:

```txt
maximum performance
maximum safety
explicit memory
checked effects
checked failures
no normal GC
no hidden magic
```

The vibe is:

> \*\*C/C++ power, Rust-level safety ambition, but with its own syntax and laws.\*\*

Not easy. Not soft. Not “Python with a katana.”
This thing wears steel boots to breakfast. 🥾

\---

# 1\. Core Syntax Rules

## 1.1 Files and Units

A source file belongs to a **unit**.

```torel
unit app.server;
```

Imports use `bring`.

```torel
bring std.http.{Server, Request, Response};
bring std.time.Clock;
bring app.auth.AuthService;
```

Public declarations use `export`.

```torel
export proc main() -> Exit {
    return Exit.ok;
}
```

Default visibility is private to the unit.

\---

# 2\. Comments

```torel
// Single-line comment.

/\*
    Block comment.
\*/

/// Documentation comment for public APIs.
```

Documentation comments are used by the official doc generator.

\---

# 3\. Declarations

Torel has strict declaration categories.

|Keyword|Meaning|
|-|-|
|`const`|Compile-time constant|
|`fix`|Immutable local binding|
|`slot`|Mutable local storage|
|`own`|Owned resource/value binding|
|`arena`|Region allocator|
|`type`|Product type / struct-like type|
|`choice`|Sum type / tagged union|
|`proc`|Procedure / function|
|`contract`|Interface / trait-like requirement|
|`implement`|Implementation of a contract|

\---

# 4\. Values and Variables

## 4.1 Immutable Binding: `fix`

```torel
fix port: UInt16 = 8080;
fix name: Text = "Torel";
```

Cannot be reassigned:

```torel
fix port: UInt16 = 8080;
port = 9000; // compile error
```

## 4.2 Mutable Storage: `slot`

```torel
slot attempts: Int32 = 0;
attempts = attempts + 1;
```

Use `slot` only when mutation is needed. Compiler warnings should punish pointless mutability with tiny bureaucratic hammers. 🔨

\---

# 5\. Types

## 5.1 Built-In Primitive Types

Torel uses explicit names.

```torel
Bool
Byte

Int8
Int16
Int32
Int64
Int128

UInt8
UInt16
UInt32
UInt64
UInt128

Float32
Float64

Text
Char
Void
Never
```

No cursed `i32`, no alphabet soup, no “guess the width” goblin.

\---

## 5.2 Numeric Types for Serious Work

For AI, math, backend money systems, and scientific computing:

```torel
BigInt
Decimal128
Fixed<digits: Const UInt32, scale: Const UInt32>
Complex<T>
Rational<T>
Matrix<T, rows: Const UIntSize, cols: Const UIntSize>
Tensor<T, shape: Const Shape>
```

Example:

```torel
fix z: Complex<Float64> = Complex.new(3.0, 4.0);
fix mag: Float64 = z.abs();
```

\---

# 6\. Type Inference

Torel allows **local-only inference**.

Inside procedure bodies:

```torel
fix count = 10;          // inferred Int32 by default rules
fix name = "Jade";       // inferred Text
```

Public APIs must be explicit:

```torel
export proc load\_user(id: UserId) -> User
    does \[db]
    fails \[UserNotFound, DbError]
{
    ...
}
```

Not allowed:

```torel
export proc load\_user(id) {
    ...
}
// error: exported procedures require explicit parameter and return types
```

\---

# 7\. No Null in Normal Code

Torel has **no null** in safe code.

Optional values use `Maybe<T>`.

```torel
choice Maybe<T> {
    some(T);
    none;
}
```

Example:

```torel
proc find\_user(id: UserId) -> Maybe<User>
    does \[db]
{
    ...
}
```

Raw null exists only in unsafe or FFI code:

```torel
unsafe {
    fix ptr: Raw<User> = raw\_null;
}
```

Normal code never touches raw null. The swamp stays fenced. 🐊

\---

# 8\. Procedures

## 8.1 Basic Procedure

```torel
proc add(a: Int32, b: Int32) -> Int32 {
    return a + b;
}
```

Final expression return is allowed:

```torel
proc square(x: Int32) -> Int32 {
    x \* x
}
```

Semicolon means “this is a statement, not the returned value.”

```torel
proc bad\_square(x: Int32) -> Int32 {
    x \* x;
}
// error: missing return value
```

\---

## 8.2 Procedure Effects

Torel uses checked effects with `does`.

```torel
proc read\_config(path: Path) -> Config
    does \[fs, alloc]
    fails \[IoError, ParseError]
{
    fix text: Text = try fs.read\_text(path);
    parse\_config(text)
}
```

Effects describe what a procedure is allowed to do.

Common effects:

|Effect|Meaning|
|-|-|
|`alloc`|Heap or arena allocation|
|`fs`|File system access|
|`net`|Network access|
|`db`|Database access|
|`clock`|Reads time|
|`random`|Uses randomness|
|`spawn`|Creates concurrent tasks|
|`block`|May block thread|
|`unsafe`|Contains unsafe operation|
|`ffi`|Calls foreign code|

Pure procedure:

```torel
proc checksum(data: view Bytes) -> UInt64
    does \[]
{
    ...
}
```

A procedure cannot call another procedure with effects it has not declared.

```torel
proc pure\_thing() -> Text
    does \[]
{
    return try fs.read\_text("secret.txt");
}
// error: fs effect not allowed here
```

This is strict. This is brutal. This is how the compiler becomes a security guard with a clipboard. 📋

\---

# 9\. Checked Failures

Torel uses checked failures, not normal unchecked exceptions.

## 9.1 Declaring Failures

```torel
proc load\_user(id: UserId) -> User
    does \[db]
    fails \[UserNotFound, DbError]
{
    ...
}
```

If a procedure can fail, it must declare the failure types.

\---

## 9.2 Propagation with `try`

```torel
proc load\_profile(id: UserId) -> Profile
    does \[db]
    fails \[UserNotFound, DbError]
{
    fix user: User = try load\_user(id);
    fix prefs: Preferences = try load\_preferences(id);

    return Profile.from(user, prefs);
}
```

`try` means:

> If this fails, return the failure through this procedure’s failure channel.

No hidden throw. No “oops it exploded somewhere else.” Very courtroom.

\---

## 9.3 Handling Failures

Torel uses `attempt`.

```torel
attempt {
    fix user: User = try load\_user(id);
    return Response.ok(user);
}
catch UserNotFound {
    return Response.not\_found();
}
catch err: DbError {
    log.error(err);
    return Response.server\_error();
}
```

Rules:

* Every checked failure must be handled or declared.
* Catch blocks must be exhaustive unless the failure is re-declared.
* Failure types are normal typed values.

\---

# 10\. Product Types

Torel uses `type` for structured data.

```torel
type User {
    id: UserId;
    name: Text;
    age: UInt8;
}
```

Construction:

```torel
fix user: User = User {
    id: UserId.from(42),
    name: "Jade",
    age: 18,
};
```

Field access:

```torel
print(user.name);
```

Mutable field update requires mutable access:

```torel
proc rename(user: view slot User, name: Text) -> Void {
    user.name = name;
}
```

\---

# 11\. Opaque Types

Useful for IDs, tokens, money, handles, and security boundaries.

```torel
type UserId hides UInt64;
```

Create through declared constructors only:

```torel
implement UserId {
    proc from(raw: UInt64) -> UserId {
        ...
    }

    proc raw(self: view UserId) -> UInt64 {
        ...
    }
}
```

This prevents accidentally passing `OrderId` where `UserId` was expected.

The compiler refuses ID soup. 🍲

\---

# 12\. Choice Types

Torel uses `choice` for tagged unions.

```torel
choice PaymentState {
    pending;
    paid(receipt: ReceiptId);
    failed(reason: Text);
}
```

Example:

```torel
proc describe(state: PaymentState) -> Text {
    match state {
        pending => "Payment pending",
        paid(receipt) => "Paid with receipt {receipt}",
        failed(reason) => "Failed: {reason}",
    }
}
```

Pattern matching must be exhaustive.

```torel
match state {
    pending => "Pending",
}
// error: missing paid and failed cases
```

No forgotten variants. No production gremlins hiding in enums. 🧌

\---

# 13\. Contracts

Torel uses `contract` for shared behavior.

```torel
contract Hashable<T> {
    proc hash(value: view T) -> UInt64
        does \[];
}
```

Implementation:

```torel
implement Hashable<UserId> for UserId {
    proc hash(value: view UserId) -> UInt64
        does \[]
    {
        return value.raw();
    }
}
```

Generic constraint:

```torel
proc put<K, V>(map: view slot Map<K, V>, key: K, value: V) -> Void
    where K meets Hashable<K>
{
    ...
}
```

Torel uses:

```torel
where T meets Contract<T>
```

instead of colon-style trait syntax.

Distinct enough. Serious enough. Not cosplay. 🗿

\---

# 14\. Generics and Compile-Time Parameters

Torel supports full compile-time generics.

```torel
type Array<T, len: Const UIntSize> {
    ...
}
```

Matrix example:

```torel
type Matrix<T, rows: Const UIntSize, cols: Const UIntSize> {
    data: Array<T, rows \* cols>;
}
```

Procedure:

```torel
proc dot<T, n: Const UIntSize>(
    a: view Array<T, n>,
    b: view Array<T, n>
) -> T
    where T meets Numeric<T>
{
    ...
}
```

Shape mismatch becomes compile error:

```torel
fix a: Matrix<Float64, 3, 4> = ...;
fix b: Matrix<Float64, 5, 2> = ...;

fix c = a \* b;
// error: matrix dimensions incompatible
```

The math goblin is contained. 📐

\---

# 15\. Casting and Conversion

No implicit casts.

```torel
fix small: Int32 = 10;
fix big: Int64 = Int64.from(small);
```

Checked conversion:

```torel
fix narrowed: Int8 = try Int8.checked(big);
```

Unsafe conversion:

```torel
unsafe {
    fix narrowed: Int8 = Int8.unchecked(big);
}
```

Not allowed:

```torel
fix big: Int64 = small;
// error: implicit numeric conversion forbidden
```

\---

# 16\. Integer Overflow

Default arithmetic is checked.

```torel
fix z: Int32 = x + y;
```

If overflow is possible and not proven safe, behavior depends on context:

* compile-time constants overflow at compile time
* runtime checked overflow traps or enters failure mode depending on function policy

Explicit operators:

|Operator|Meaning|
|-|-|
|`+`|Checked add|
|`+%`|Wrapping add|
|`+^`|Saturating add|
|`+!`|Trap-on-overflow add|
|`+?`|Returns `Maybe<T>`|

Same pattern:

```torel
-   -%   -^   -!   -?
\*   \*%   \*^   \*!   \*?
```

Example:

```torel
fix a: UInt8 = 250;
fix b: UInt8 = 10;

fix wrap: UInt8 = a +% b;       // wraps
fix sat: UInt8 = a +^ b;        // becomes UInt8.max
fix maybe: Maybe<UInt8> = a +? b;
```

No surprise overflow. No silent number betrayal. 🧮

\---

# 17\. Ownership

Torel has no normal garbage collector.

Memory is controlled by:

```txt
ownership
views
moves
arenas
deterministic drop
```

## 17.1 Owned Values

```torel
own buffer: Buffer = Buffer.alloc(4096);
```

This is shorthand for:

```torel
fix buffer: own Buffer = Buffer.alloc(4096);
```

The binding owns the resource.

\---

## 17.2 Explicit Move

Ownership transfer must be explicit.

```torel
own buffer: Buffer = Buffer.alloc(4096);

send(move buffer);

buffer.clear();
// error: buffer was moved
```

No implicit move. No spooky action at a distance. 👻

\---

## 17.3 Read-Only View

A `view` gives temporary read-only access.

```torel
proc checksum(data: view Buffer) -> UInt64 {
    ...
}
```

Call site:

```torel
fix sum: UInt64 = checksum(view buffer);
```

\---

## 17.4 Mutable View

A `view slot` gives temporary exclusive mutable access.

```torel
proc zero\_fill(data: view slot Buffer) -> Void {
    data.fill(0);
}
```

Call site:

```torel
zero\_fill(view slot buffer);
```

Rules:

* Many read-only views may exist at once.
* Only one mutable view may exist at once.
* A mutable view cannot overlap with read-only views.
* Views cannot outlive the value they view.

\---

## 17.5 Return Views

When a procedure returns a view, it must declare what the return is tied to.

```torel
proc first\_line(text: view Text) -> view Text
    ties \[return: text]
{
    ...
}
```

Meaning:

> The returned view is valid only as long as `text` is valid.

This avoids Rust-style lifetime punctuation while still giving the compiler exact lifetime logic.

Another example:

```torel
proc get\_header(packet: view Packet) -> view Header
    ties \[return: packet]
{
    return packet.header;
}
```

No lifetime hieroglyphics. Still strict. Deliciously severe.

\---

# 18\. Arenas

Arenas are first-class.

## 18.1 Scoped Arena

```torel
arena request\_mem {
    own body: Buffer = request\_mem.make\_buffer(8192);
    fix packet: Packet = try parse\_packet(view body);

    process(packet);
}
// all arena memory released here
```

The arena name is available inside the block.

\---

## 18.2 Arena Variable

```torel
arena session\_mem: Arena = Arena.create();

own cache: Cache = session\_mem.make Cache();

session\_mem.release();
```

Rules:

* Scoped arenas are preferred.
* Arena variables are allowed for advanced lifetime control.
* Values allocated in an arena cannot escape the arena unless copied or moved into a longer-lived owner.
* The compiler tracks arena escape rules.

Invalid:

```torel
proc bad() -> view Buffer {
    arena temp {
        own buf: Buffer = temp.make\_buffer(128);
        return view buf;
    }
}
// error: returned view escapes arena temp
```

\---

# 19\. Destructors

Types may define `drop`.

```torel
type File {
    handle: OsHandle;

    drop {
        os.close(self.handle);
    }
}
```

Runs deterministically at end of ownership lifetime.

```torel
proc read\_log(path: Path) -> Text
    does \[fs, alloc]
    fails \[IoError]
{
    own file: File = try File.open(path);
    fix text: Text = try file.read\_all();

    return text;
}
// file drops here
```

Rules:

* `drop` must not fail.
* `drop` may log only if declared safe by type policy.
* Destructors run in reverse ownership order.
* Cycles are not automatically collected because there is no normal GC.

For cycles, use explicit weak references or arenas.

\---

# 20\. Unsafe

Unsafe is loud at definition and call site.

## 20.1 Unsafe Procedure

```torel
unsafe proc write\_raw(ptr: Raw<Byte>, value: Byte) -> Void {
    ptr.write(value);
}
```

## 20.2 Unsafe Call Site

```torel
unsafe {
    write\_raw(ptr, 255);
}
```

Unsafe blocks may contain:

* raw pointer dereference
* unchecked casts
* FFI calls
* manual layout assumptions
* unchecked memory access
* volatile hardware access

Unsafe does **not** turn off the whole compiler. It only permits specific unsafe operations inside the block.

\---

# 21\. Control Flow

## 21.1 If

```torel
if age >= 18 {
    print("adult");
}
else {
    print("minor");
}
```

Condition must be `Bool`.

Not allowed:

```torel
if count {
    ...
}
// error: Int32 is not Bool
```

\---

## 21.2 While

```torel
while attempts < 3 {
    attempts = attempts + 1;
}
```

\---

## 21.3 Loop

```torel
loop {
    fix packet: Packet = try read\_packet();

    if packet.done {
        break;
    }
}
```

\---

## 21.4 For

```torel
for item in items {
    process(item);
}
```

Mutable iteration:

```torel
for item in view slot items {
    item.normalize();
}
```

Range:

```torel
for i in 0..count {
    print(i);
}
```

Inclusive range:

```torel
for i in 0..=count {
    print(i);
}
```

\---

# 22\. Pattern Matching

```torel
match result {
    ok(value) => print(value),
    fail(err) => log.error(err),
}
```

Block arms:

```torel
match state {
    paid(receipt) => {
        audit(receipt);
        return true;
    }

    failed(reason) => {
        log.warn(reason);
        return false;
    }

    pending => false,
}
```

Rules:

* Must be exhaustive.
* No fallthrough.
* Guards allowed.

```torel
match value {
    n if n > 0 => "positive",
    n if n < 0 => "negative",
    \_ => "zero",
}
```

\---

# 23\. Methods

Methods are procedures attached to a type.

```torel
type Buffer {
    data: Raw<Byte>;
    len: UIntSize;

    proc length(self: view Self) -> UIntSize {
        return self.len;
    }

    proc clear(self: view slot Self) -> Void {
        ...
    }
}
```

Calling:

```torel
fix n: UIntSize = buffer.length();
buffer.clear();
```

The receiver type controls access:

|Receiver|Meaning|
|-|-|
|`self: view Self`|Read-only|
|`self: view slot Self`|Mutable|
|`self: own Self`|Consumes ownership|

Consuming method:

```torel
proc into\_bytes(self: own Self) -> Buffer {
    ...
}
```

Call:

```torel
own bytes: Buffer = file.into\_bytes(move file);
```

Cleaner method-call sugar is allowed:

```torel
own bytes: Buffer = move file.into\_bytes();
```

The compiler lowers it to an ownership-consuming call.

\---

# 24\. Arrays, Slices, and Buffers

Fixed-size array:

```torel
fix nums: Array<Int32, 4> = \[1, 2, 3, 4];
```

Slice view:

```torel
proc sum(nums: view Slice<Int32>) -> Int32 {
    slot total: Int32 = 0;

    for n in nums {
        total = total + n;
    }

    total
}
```

Mutable slice:

```torel
proc sort(nums: view slot Slice<Int32>) -> Void {
    ...
}
```

Buffer:

```torel
own buf: Buffer = Buffer.alloc(4096);
```

Bounds checks are default. Compiler may remove them when proven safe.

Unsafe unchecked access:

```torel
unsafe {
    fix byte: Byte = buf.get\_unchecked(i);
}
```

\---

# 25\. Strings and Text

Torel separates human text from raw bytes.

|Type|Meaning|
|-|-|
|`Text`|Valid Unicode text|
|`Bytes`|Raw byte sequence|
|`Char`|Unicode scalar value|
|`Ascii`|Verified ASCII text|
|`Path`|Platform-safe path type|

Example:

```torel
fix name: Text = "Jade";
fix raw: Bytes = name.encode\_utf8();
```

Invalid UTF-8 must be handled:

```torel
fix text: Text = try Text.from\_utf8(bytes);
```

\---

# 26\. Concurrency

Torel supports strict structured concurrency.

## 26.1 Async Procedures

```torel
async proc fetch\_user(id: UserId) -> User
    does \[net, alloc]
    fails \[Timeout, DecodeError]
{
    fix response: HttpResponse = try await http.get("/users/{id}");
    return try response.decode<User>();
}
```

## 26.2 Task Scope

Tasks cannot outlive their scope unless explicitly detached with strict rules.

```torel
task\_scope {
    own user\_task = spawn fetch\_user(id);
    own orders\_task = spawn fetch\_orders(id);

    fix user: User = try await user\_task;
    fix orders: List<Order> = try await orders\_task;

    return Dashboard.from(user, orders);
}
```

Rules:

* Spawned tasks must be awaited or cancelled before scope exit.
* Moved values into tasks require `move`.
* Shared mutable state is forbidden unless synchronized.

\---

## 26.3 Channels

```torel
own channel: Channel<Message> = Channel.bounded(1024);

spawn worker(move channel.sender());

channel.send(Message.start);
```

Channel messages must be safe to move between tasks.

\---

## 26.4 Actors

For services with isolated state:

```torel
actor Cache {
    slot map: Map<Text, Bytes>;

    receive get(key: Text) -> Maybe<Bytes> {
        return map.get(view key);
    }

    receive put(key: Text, value: Bytes) -> Void {
        map.insert(move key, move value);
    }
}
```

Actor rules:

* Actor state is private.
* Messages are copied or moved.
* No shared mutable state crosses actor boundaries.
* Actor calls are async by default.

\---

# 27\. Backend Syntax

Torel should make backend code first-class without becoming a toy web DSL.

Example:

```torel
unit app.main;

bring std.http.{Server, Request, Response};
bring app.users.UserService;

proc get\_user(req: view Request) -> Response
    does \[db, alloc]
    fails \[BadRequest, UserNotFound, DbError]
{
    fix id: UserId = try req.path\_param<UserId>("id");
    fix user: User = try UserService.load(id);

    return Response.json(user);
}

export proc main() -> Exit
    does \[net, db, alloc, clock]
{
    own server: Server = Server.bind("0.0.0.0", 8080);

    server.get("/users/{id}", get\_user);

    attempt {
        try server.run();
    }
    catch err: NetError {
        log.error(err);
        return Exit.fail;
    }

    return Exit.ok;
}
```

Production backend priorities:

* predictable latency
* explicit allocation
* structured concurrency
* no hidden thread spawning
* typed requests and responses
* checked database and network failures
* tracing hooks
* memory-safe parsing

\---

# 28\. Scientific and Math Syntax

Compile-time dimensions:

```torel
type Vec3 = Matrix<Float64, 3, 1>;
type Mat4 = Matrix<Float64, 4, 4>;
```

Example:

```torel
proc transform(point: Vec3, matrix: Mat4) -> Vec3
    does \[]
{
    ...
}
```

Tensor example:

```torel
fix input: Tensor<Float32, Shape\[1, 3, 224, 224]> = ...;
fix output: Tensor<Float32, Shape\[1, 1000]> = model.forward(input);
```

Numeric kernels may request compiler optimization policy:

```torel
@optimize("vectorize")
proc dot<T, n: Const UIntSize>(
    a: view Array<T, n>,
    b: view Array<T, n>
) -> T
    where T meets Numeric<T>
{
    slot total: T = T.zero();

    for i in 0..n {
        total = total + a\[i] \* b\[i];
    }

    total
}
```

\---

# 29\. FFI

Foreign code is explicit and unsafe by default.

```torel
foreign "C" {
    proc strlen(ptr: Raw<Byte>) -> UIntSize;
}
```

Calling:

```torel
unsafe {
    fix len: UIntSize = strlen(ptr);
}
```

C-compatible layout:

```torel
@repr("C")
type PacketHeader {
    id: UInt32;
    flags: UInt16;
    checksum: UInt16;
}
```

FFI rules:

* Raw pointers only in unsafe zones.
* Foreign functions are assumed unsafe unless annotated.
* ABI layout must be explicit.
* No automatic ownership assumptions across FFI boundaries.

\---

# 30\. Compile-Time Execution

Torel supports controlled compile-time evaluation.

```torel
const MAX\_USERS: UInt32 = 1024;
const PAGE\_SIZE: UIntSize = 4096;
```

Compile-time procedures:

```torel
const proc align\_up(value: UIntSize, align: UIntSize) -> UIntSize {
    ((value + align - 1) / align) \* align
}
```

Restrictions:

* no filesystem unless explicitly allowed by build policy
* no network
* deterministic by default
* no hidden randomness
* no time access unless explicitly declared

Builds must be reproducible. The compiler does not get to become a weather vane. 🌬️

\---

# 31\. Attributes

Attributes use `@`.

```torel
@inline
proc tiny(x: Int32) -> Int32 {
    x + 1
}
```

Common attributes:

```torel
@inline
@no\_inline
@repr("C")
@packed
@align(16)
@deprecated("Use new\_parser instead.")
@target("linux")
@test
@bench
@optimize("speed")
@optimize("size")
```

Example:

```torel
@target("linux")
proc tune\_socket(socket: view slot Socket) -> Void
    does \[sys]
{
    ...
}
```

\---

# 32\. Testing

Built-in test syntax:

```torel
test "parser accepts valid packet" {
    fix packet: Packet = try parse\_packet(valid\_bytes);
    assert(packet.kind == PacketKind.data);
}
```

Failure-aware tests:

```torel
test "parser rejects invalid packet" {
    expect\_fail ParseError {
        try parse\_packet(invalid\_bytes);
    }
}
```

Benchmarks:

```torel
bench "checksum 4kb buffer" {
    own data: Buffer = Buffer.random(4096);

    measure {
        checksum(view data);
    }
}
```

\---

# 33\. Module and Package Logic

Package manifest:

```toml
name = "app\_server"
version = "0.1.0"
edition = "2026"

\[target]
default = "native"

\[dependencies]
std = "1.0"
```

Unit layout:

```txt
src/
  main.torel
  app/
    users.torel
    auth.torel
    db.torel
```

Imports:

```torel
bring app.users.UserService;
bring app.auth.{AuthToken, AuthError};
```

Export:

```torel
export type User;
export proc load\_user(id: UserId) -> User
    does \[db]
    fails \[UserNotFound, DbError];
```

\---

# 34\. Standard Library Shape

Torel standard library should be split into layers.

## 34.1 `core`

Always available, no OS required.

```txt
core.mem
core.num
core.text
core.array
core.option
core.result
core.hash
core.iter
```

## 34.2 `std`

Requires operating system.

```txt
std.fs
std.io
std.net
std.http
std.time
std.crypto
std.process
std.thread
std.task
std.log
std.json
std.database
```

## 34.3 `sys`

Low-level platform layer.

```txt
sys.linux
sys.windows
sys.macos
sys.posix
sys.wasm
```

Most `sys` APIs require unsafe or platform effects.

\---

# 35\. Compiler Logic

The compiler enforces:

## 35.1 Type Safety

* no implicit casts
* no null in safe code
* exhaustive matches
* explicit public API types
* checked numeric behavior

## 35.2 Memory Safety

* moved values cannot be reused
* views cannot outlive owners
* mutable views cannot alias
* arena values cannot escape
* destructors run deterministically
* raw pointers only in unsafe blocks

## 35.3 Effect Safety

* functions cannot perform undeclared effects
* pure functions cannot allocate, use I/O, read clocks, or touch randomness
* unsafe operations require unsafe context
* FFI is visible in signatures

## 35.4 Failure Safety

* checked failures must be handled or declared
* `try` can only propagate declared failures
* public APIs document failure modes automatically

\---

# 36\. Full Example

```torel
unit app.server;

bring std.http.{Server, Request, Response};
bring std.log;
bring app.user.{User, UserId, UserService};

type ApiError {
    message: Text;
    status: UInt16;
}

choice RouteFailure {
    bad\_request(Text);
    not\_found(Text);
    database(Text);
}

proc parse\_user\_id(req: view Request) -> UserId
    does \[alloc]
    fails \[RouteFailure]
{
    fix raw: Text = try req.path\_param("id")
        map\_fail RouteFailure.bad\_request("missing user id");

    return try UserId.parse(raw)
        map\_fail RouteFailure.bad\_request("invalid user id");
}

proc get\_user(req: view Request) -> Response
    does \[db, alloc]
    fails \[RouteFailure]
{
    fix id: UserId = try parse\_user\_id(req);

    fix user: User = try UserService.load(id)
        map\_fail RouteFailure.not\_found("user not found");

    return Response.json(user);
}

proc handle\_failure(err: RouteFailure) -> Response
    does \[]
{
    match err {
        bad\_request(msg) => Response.text(msg, 400),
        not\_found(msg) => Response.text(msg, 404),
        database(msg) => Response.text(msg, 500),
    }
}

export proc main() -> Exit
    does \[net, db, alloc, clock]
{
    own server: Server = Server.bind("0.0.0.0", 8080);

    server.get("/users/{id}", proc(req: view Request) -> Response
        does \[db, alloc]
    {
        attempt {
            return try get\_user(req);
        }
        catch err: RouteFailure {
            return handle\_failure(err);
        }
    });

    attempt {
        try server.run();
    }
    catch err: NetError {
        log.error(err);
        return Exit.fail;
    }

    return Exit.ok;
}
```

\---

# 37\. Current Locked Design Summary 🧾

```txt
Language style:
    strict native systems/backend language

Primary target:
    backend, systems, infrastructure
    with serious AI/math/science support

Performance:
    maximum priority

Safety:
    maximum priority

Difficulty:
    very strict

Memory:
    ownership + arenas
    no normal garbage collector

Ownership:
    explicit move
    view for read access
    view slot for mutable access

Destructors:
    deterministic drop

Unsafe:
    unsafe proc and unsafe block
    unsafe required at call sites

Functions:
    proc

Variables:
    fix for immutable
    slot for mutable

Types:
    explicit names: Int32, UInt64, Float64, Bool, Text

Return:
    return allowed
    final expression allowed

Semicolons:
    required for statements

Type inference:
    local only

Null:
    forbidden in safe code
    raw null only in unsafe/FFI

Generics:
    full compile-time generics

Casting:
    no implicit casts

Overflow:
    explicit modes
```



# 🏗️ TOREL Ecosystem Spec v0.2

## Standard Library, Toolchain, Package Manager, Testing, Web, DB, Math, FFI, IDE, Debugger, Profiler

We are extending the current TOREL v0.1 language spec: strict native systems/backend language, no normal GC, ownership + arenas, checked effects, checked failures, explicit `move`, `view`, `view slot`, `fix`, `slot`, `proc`, and the `core` / `std` / `sys` library split.

Torel is now entering **production ecosystem mode**. The compiler dragon has a factory permit. 🐉🏭

\---

# 1\. Ecosystem Philosophy

Torel tooling must follow the same law as the language:

```txt
no hidden magic
no vague behavior
no unsafe defaults
no dependency soup
no performance guesswork
```

Everything should be:

|Area|Principle|
|-|-|
|Standard library|Small core, strong OS layer, explicit effects|
|Compiler|Strict, optimizing, reproducible|
|Package manager|Locked, audited, deterministic|
|Testing|Built-in, effect-aware, failure-aware|
|Benchmarking|Statistically useful, not toy stopwatch behavior|
|Documentation|Generated from types, effects, failures, examples|
|HTTP|Production server stack, not a toy router|
|Database|Typed queries, migrations, transactions|
|Math/tensors|Compile-time shapes, SIMD/GPU paths|
|FFI|Unsafe by default, generated wrappers|
|IDE|Language-server-first|
|Debugger|Ownership/effects-aware|
|Profiler|CPU, memory, async, allocation, syscalls|

\---

# 2\. Standard Library Architecture

Torel keeps the standard library split into **three official layers**:

```txt
core    // no OS required
std     // OS/runtime required
sys     // low-level platform APIs
```

## 2.1 `core`

Always available. Works in embedded, kernels, bootloaders, WASM, and `no\_std` style builds.

```txt
core.mem
core.num
core.text
core.bytes
core.array
core.slice
core.maybe
core.result
core.choice
core.iter
core.hash
core.compare
core.convert
core.alloc
core.arena
core.sync
core.atomic
core.marker
```

### Key `core` types

```torel
Maybe<T>
Result<T, E>
Array<T, n: Const UIntSize>
Slice<T>
Span<T>
Bytes
TextView
Raw<T>
NonZero<T>
Range<T>
```

### Example

```torel
bring core.{Maybe, Slice};
bring core.num.UInt64;

proc checksum(data: view Slice<Byte>) -> UInt64
    does \[]
{
    slot sum: UInt64 = 0;

    for byte in data {
        sum = sum + UInt64.from(byte);
    }

    return sum;
}
```

`core` must never secretly allocate unless the API declares `does \[alloc]`.

\---

## 2.2 `std`

Requires an operating system.

```txt
std.io
std.fs
std.path
std.net
std.http
std.tls
std.time
std.clock
std.random
std.crypto
std.process
std.env
std.thread
std.task
std.channel
std.actor
std.log
std.trace
std.metrics
std.json
std.toml
std.yaml
std.xml
std.database
std.compress
std.archive
std.testing
std.bench
std.debug
```

Every `std` function must declare effects.

Example:

```torel
proc read\_config(path: Path) -> Config
    does \[fs, alloc]
    fails \[IoError, ParseError]
{
    fix text: Text = try std.fs.read\_text(path);
    return try Config.parse(text);
}
```

\---

## 2.3 `sys`

Low-level platform layer. It is sharp metal.

```txt
sys.posix
sys.linux
sys.windows
sys.macos
sys.wasm
sys.cpu
sys.mem
sys.socket
sys.signal
sys.epoll
sys.kqueue
sys.iocp
sys.cuda
sys.rocm
sys.metal
```

Most `sys` APIs require:

```torel
does \[sys, unsafe]
```

or direct `unsafe`.

Example:

```torel
@target("linux")
unsafe proc set\_nonblocking(fd: RawFd) -> Void
    does \[sys, unsafe]
    fails \[SysError]
{
    ...
}
```

\---

# 3\. Compiler Commands

Official command:

```bash
torel
```

Compiler binary:

```bash
torelc
```

The user normally uses `torel`; `torelc` is for tooling and advanced builds.

\---

## 3.1 Core Commands

```bash
torel new <name>
torel init
torel build
torel run
torel test
torel bench
torel check
torel fmt
torel lint
torel doc
torel clean
torel package
torel publish
torel install
torel update
torel audit
torel tree
torel vendor
torel repl
```

Yes, Torel gets a REPL, but it is mostly for expressions, type exploration, and small tests. Not the main identity.

\---

## 3.2 Build Commands

```bash
torel build
torel build --release
torel build --target linux-x64
torel build --target wasm32
torel build --mode sanitize
torel build --mode deterministic
torel build --features tls,postgres
```

Example:

```bash
torel build --release --target linux-x64 --lto full --pgo use
```

Torel should not require five different tools just to build one app. The CLI is the command throne. 👑

\---

## 3.3 Check-Only Compilation

```bash
torel check
```

Performs:

```txt
parse
name resolution
type checking
effect checking
failure checking
ownership checking
arena escape checking
unsafe validation
```

But does not emit machine code.

This is what IDEs and CI should use constantly.

\---

# 4\. Package Manager

Package manager is built into `torel`.

Manifest file:

```txt
Torel.toml
```

Lock file:

```txt
Torel.lock
```

Package cache:

```txt
\~/.torel/packages
```

\---

## 4.1 Manifest Example

```toml
\[package]
name = "api\_server"
version = "0.1.0"
edition = "2026"
license = "Apache-2.0"
authors = \["Pearl Vale"]

\[target]
default = "native"
min\_torel = "0.2.0"

\[features]
default = \["tls", "json"]
tls = \[]
postgres = \[]
cuda = \[]

\[dependencies]
std = "1.0"
http = "1.0"
postgres = { version = "0.8", features = \["pool"] }

\[dev-dependencies]
testkit = "1.0"
mockdb = "0.3"

\[build]
mode = "debug"
warnings = "deny"
unsafe = "audit"

\[security]
deny\_unsafe\_dependencies = false
require\_lockfile = true
```

\---

## 4.2 Dependency Rules

Torel package management should be **hostile to supply-chain chaos**.

Rules:

|Rule|Behavior|
|-|-|
|Lockfile required for apps|Prevents surprise updates|
|Semantic versioning|Standard version compatibility|
|Package checksums|Every package verified|
|Unsafe manifest flag|Packages must declare unsafe usage|
|Effect manifest|Packages expose declared effects|
|Native dependency manifest|C/C++/system deps must be explicit|
|Reproducible builds|Same lockfile gives same output|

Package metadata includes:

```txt
uses\_unsafe = true/false
uses\_ffi = true/false
declared\_effects = \[fs, net, db, unsafe]
supported\_targets = \[...]
license = ...
```

\---

## 4.3 Package Commands

```bash
torel add postgres
torel remove postgres
torel update
torel update postgres
torel tree
torel audit
torel vendor
torel publish
torel yank <version>
```

Security audit:

```bash
torel audit
```

Checks:

```txt
known vulnerabilities
malicious package reports
license conflicts
unsafe dependency use
abandoned packages
native build scripts
network access during build
```

Build scripts are not allowed to casually do internet goblin behavior. 🧌

\---

# 5\. Build Modes

Torel has official build modes.

|Mode|Command|Purpose|
|-|-|-|
|`debug`|`torel build`|Fast compile, rich checks|
|`release`|`torel build --release`|Optimized production|
|`sanitize`|`torel build --mode sanitize`|Runtime bug detection|
|`profile`|`torel build --mode profile`|Profiling instrumentation|
|`bench`|`torel build --mode bench`|Stable benchmark builds|
|`deterministic`|`torel build --mode deterministic`|Reproducible builds|
|`embedded`|`torel build --mode embedded`|No OS assumptions|
|`hardened`|`torel build --mode hardened`|Security-heavy production|
|`size`|`torel build --mode size`|Small binaries|

\---

## 5.1 Release Mode

```bash
torel build --release
```

Defaults:

```txt
optimizations: speed
debug symbols: split
bounds checks: kept unless proven removable
integer overflow: checked unless explicit operator used
panic strategy: abort or report, configurable
LTO: thin
```

\---

## 5.2 Hardened Mode

```bash
torel build --mode hardened
```

Enables:

```txt
stack protection
control-flow integrity where supported
hardened allocator options
dependency audit gate
unsafe callsite report
no debug secrets
strict TLS defaults
panic scrubbing
```

This is for payments, auth, infra, and “please do not become tomorrow’s breach headline” energy. 🛡️

\---

## 5.3 Deterministic Mode

```bash
torel build --mode deterministic
```

Guarantees:

```txt
stable object ordering
fixed timestamps
locked dependencies
no ambient filesystem reads
no network access
stable codegen flags
```

Required for serious infrastructure and verified builds.

\---

# 6\. Testing Framework

Testing is built into the language and standard library.

```torel
test "parser accepts valid packet" {
    fix packet: Packet = try parse\_packet(valid\_packet);
    assert(packet.kind == PacketKind.data);
}
```

Run:

```bash
torel test
```

\---

## 6.1 Test Types

Torel supports:

```txt
unit tests
integration tests
property tests
snapshot tests
failure tests
effect tests
concurrency tests
fuzz tests
end-to-end tests
```

Commands:

```bash
torel test
torel test --unit
torel test --integration
torel test --fuzz
torel test --filter parser
torel test --coverage
torel test --report junit
```

\---

## 6.2 Effect-Aware Tests

A test must declare effects if needed.

```torel
test "loads config from disk"
    does \[fs, alloc]
{
    fix config: Config = try Config.load("testdata/app.toml");
    assert(config.port == 8080);
}
```

Pure tests are enforced:

```torel
test "checksum is deterministic"
    does \[]
{
    fix sum = checksum(\[1, 2, 3]);
    assert(sum == 6);
}
```

If a pure test touches filesystem, the compiler says no.

\---

## 6.3 Failure Tests

```torel
test "invalid user id fails" {
    expect\_fail ParseError {
        try UserId.parse("not-an-id");
    }
}
```

\---

## 6.4 Property Testing

```torel
property "reverse twice returns original"
    for\_all list: List<Int32>
{
    assert(list.reverse().reverse() == list);
}
```

\---

## 6.5 Fuzz Testing

```torel
fuzz "packet parser never crashes"
    input bytes: Bytes
{
    attempt {
        parse\_packet(bytes);
    }
    catch \_ {
        pass;
    }
}
```

Run:

```bash
torel test --fuzz packet\_parser
```

\---

# 7\. Benchmarking

Benchmarking is official, not a kitchen timer taped to a function.

```torel
bench "checksum 4kb buffer" {
    own data: Buffer = Buffer.random(4096);

    measure {
        checksum(view data);
    }
}
```

Run:

```bash
torel bench
```

\---

## 7.1 Benchmark Features

Torel bench should include:

```txt
warmup
multiple samples
outlier detection
confidence intervals
CPU pinning
allocation counting
cache behavior reports
branch miss reports
comparison against baseline
JSON output for CI
```

Commands:

```bash
torel bench
torel bench --compare main
torel bench --save baseline
torel bench --filter checksum
torel bench --profile cpu
torel bench --profile alloc
```

Output example:

```txt
checksum 4kb buffer
  mean:      89.2 ns
  p95:       91.7 ns
  allocs:    0
  regress:   +1.2% from baseline
```

If a benchmark secretly allocates, Torel should snitch immediately. 🕵️

\---

# 8\. Documentation System

Command:

```bash
torel doc
```

Output:

```txt
HTML docs
Markdown docs
JSON symbol index
LSP documentation database
API effect/failure reports
```

\---

## 8.1 Documentation Comments

```torel
/// Loads a user by ID.
///
/// Effects:
/// - `db`: queries the configured database.
///
/// Failures:
/// - `UserNotFound`
/// - `DbError`
export proc load\_user(id: UserId) -> User
    does \[db]
    fails \[UserNotFound, DbError];
```

But Torel docs should auto-generate effects and failures from signatures, so humans do not duplicate the contract.

\---

## 8.2 Generated API Page Includes

Each procedure doc shows:

```txt
signature
visibility
effects
failures
ownership behavior
unsafe status
examples
complexity notes
allocation behavior
target support
deprecated status
```

Example generated section:

```txt
proc load\_user(id: UserId) -> User
effects: \[db]
fails: \[UserNotFound, DbError]
allocates: no direct allocation
unsafe: no
```

\---

## 8.3 Doc Tests

Examples in docs compile and run.

````torel
/// Example:
/// ```torel
/// fix id = UserId.from(42);
/// fix user = try load\_user(id);
/// ```
````

Run:

```bash
torel test --doc
```

No rotten docs. No fossil examples from 2026 haunting 2030. 🦴

\---

# 9\. HTTP Framework

Official module:

```torel
std.http
```

Higher-level optional package:

```torel
torel.web
```

`std.http` should be serious and low-level enough for frameworks to build on.

\---

## 9.1 HTTP Server Example

```torel
bring std.http.{Server, Request, Response};
bring std.log;

proc health(req: view Request) -> Response
    does \[]
{
    return Response.text("ok", 200);
}

export proc main() -> Exit
    does \[net, alloc, clock]
    fails \[NetError]
{
    own server: Server = try Server.bind("0.0.0.0", 8080);

    server.get("/health", health);

    try server.run();

    return Exit.ok;
}
```

\---

## 9.2 Router

```torel
own router: Router = Router.new();

router.get("/users/{id}", get\_user);
router.post("/users", create\_user);
router.group("/admin", admin\_routes);
```

Typed path params:

```torel
fix id: UserId = try req.path\_param<UserId>("id");
```

Typed query params:

```torel
fix page: UInt32 = try req.query\_param<UInt32>("page");
```

\---

## 9.3 Middleware

Middleware must declare effects.

```torel
proc auth\_middleware(req: view slot Request, next: Handler) -> Response
    does \[db, alloc]
    fails \[AuthError]
{
    fix token: AuthToken = try req.header<AuthToken>("Authorization");
    fix user: User = try Auth.verify(token);

    req.context.insert("user", move user);

    return try next(req);
}
```

\---

## 9.4 HTTP Features

Official support:

```txt
HTTP/1.1
HTTP/2
HTTP/3 optional
TLS
WebSockets
streaming bodies
multipart forms
server-sent events
request limits
timeouts
backpressure
graceful shutdown
structured logging
OpenTelemetry-style tracing
```

The server must be safe against:

```txt
slowloris attacks
unbounded body reads
header bombs
path traversal
request smuggling
accidental blocking in async handlers
```

Torel should make dangerous server defaults impossible or extremely loud.

\---

# 10\. Database Layer

Official module:

```torel
std.database
```

Official drivers can live as packages:

```txt
db.postgres
db.mysql
db.sqlite
db.mssql
db.redis
```

\---

## 10.1 Design Goals

Torel database layer must provide:

```txt
typed queries
checked failures
transaction scopes
connection pooling
migration tooling
prepared statements by default
explicit effects
streaming rows
safe parameter binding
```

No string-concatenated SQL goblin festivals. 🎪

\---

## 10.2 Connection

```torel
bring std.database.{DbPool, Query};

own pool: DbPool = try DbPool.connect(env.database\_url())
    does \[net, db, alloc]
    fails \[DbError];
```

\---

## 10.3 Typed Query

```torel
type UserRow {
    id: UserId;
    name: Text;
    email: Email;
}

const GET\_USER = sql<UserRow>(
    "select id, name, email from users where id = $1"
);

proc load\_user(pool: view DbPool, id: UserId) -> UserRow
    does \[db, alloc]
    fails \[UserNotFound, DbError]
{
    return try pool.fetch\_one(GET\_USER, id);
}
```

The query macro validates:

```txt
parameter count
parameter types
result column names
result column types
nullable columns
SQL syntax where possible
```

\---

## 10.4 Transactions

```torel
proc transfer(
    pool: view DbPool,
    from: AccountId,
    to: AccountId,
    amount: Money
) -> Void
    does \[db]
    fails \[DbError, InsufficientFunds]
{
    transaction tx = try pool.begin();

    try debit(view tx, from, amount);
    try credit(view tx, to, amount);

    try tx.commit();
}
```

If the transaction scope exits without commit:

```txt
rollback automatically
```

but rollback failures are logged through a non-throwing cleanup policy.

\---

## 10.5 Migrations

Commands:

```bash
torel db init
torel db new create\_users
torel db migrate
torel db rollback
torel db status
torel db check
```

Migration file:

```torel
migration "2026\_05\_12\_create\_users" {
    up {
        sql """
        create table users (
            id bigint primary key,
            name text not null,
            email text not null unique
        );
        """;
    }

    down {
        sql "drop table users;";
    }
}
```

\---

# 11\. Math and Tensor Stack

Official modules:

```txt
std.math
std.linalg
std.stats
std.signal
std.optimize
std.tensor
std.autodiff
std.gpu
```

Optional accelerated packages:

```txt
math.blas
math.lapack
tensor.cuda
tensor.rocm
tensor.metal
tensor.onnx
```

\---

## 11.1 Numeric Contracts

```torel
contract Numeric<T> {
    proc zero() -> T does \[];
    proc one() -> T does \[];
    proc add(a: T, b: T) -> T does \[];
    proc mul(a: T, b: T) -> T does \[];
}
```

Specialized contracts:

```txt
Integer<T>
Float<T>
Signed<T>
Unsigned<T>
ComplexNumber<T>
Field<T>
Ring<T>
Ordered<T>
Differentiable<T>
```

\---

## 11.2 Matrix Types

```torel
type Matrix<T, rows: Const UIntSize, cols: Const UIntSize>;
type Vector<T, len: Const UIntSize> = Matrix<T, len, 1>;
```

Example:

```torel
fix a: Matrix<Float64, 3, 4> = ...;
fix b: Matrix<Float64, 4, 2> = ...;

fix c: Matrix<Float64, 3, 2> = a.matmul(b);
```

Invalid dimensions fail at compile time.

```torel
fix bad = a.matmul(Matrix<Float64, 9, 9>.zero());
// compile error: incompatible matrix dimensions
```

\---

## 11.3 Tensor Types

```torel
type Tensor<T, shape: Const Shape>;
```

Example:

```torel
fix input: Tensor<Float32, Shape\[1, 3, 224, 224]> = ...;
fix output: Tensor<Float32, Shape\[1, 1000]> = model.forward(input);
```

\---

## 11.4 Device-Aware Tensors

```torel
type Cpu;
type Cuda<device: Const UInt32>;
type Metal<device: Const UInt32>;

type Tensor<T, shape: Const Shape, device: Device = Cpu>;
```

Example:

```torel
fix x: Tensor<Float32, Shape\[1024, 1024], Cpu> = ...;
fix gpu\_x = try x.to\_device<Cuda<0>>();
```

Transfer declares effects:

```torel
does \[gpu, alloc]
fails \[GpuError]
```

\---

## 11.5 Autodiff

```torel
proc loss(weights: Tensor<Float32, Shape\[784, 10]>) -> Float32
    does \[alloc]
{
    ...
}

fix grad = autodiff.grad(loss, weights);
```

Compiler/libraries should support:

```txt
forward-mode autodiff
reverse-mode autodiff
static graph optimization
dynamic graph execution
GPU kernels
mixed precision
```

\---

# 12\. FFI Generator

Command:

```bash
torel ffi
```

Purpose:

```txt
generate safe Torel bindings from C headers
generate C headers from Torel APIs
generate ABI reports
generate unsafe boundary summaries
```

\---

## 12.1 Importing C

```bash
torel ffi import c \\
  --header ./native/lib.h \\
  --name native.lib \\
  --out src/native/lib.torel
```

Generated raw binding:

```torel
foreign "C" {
    unsafe proc native\_open(path: Raw<Byte>) -> Int32;
}
```

Generated safe wrapper:

```torel
proc open\_file(path: Path) -> File
    does \[ffi, fs, alloc]
    fails \[NativeError]
{
    ...
}
```

\---

## 12.2 Exporting Torel to C

```bash
torel ffi export c \\
  --unit app.plugin \\
  --out dist/plugin.h
```

Torel API:

```torel
@export("C")
proc plugin\_init(config: view PluginConfig) -> Int32
    does \[]
{
    ...
}
```

\---

## 12.3 FFI Safety Report

```bash
torel ffi audit
```

Output:

```txt
unsafe foreign functions: 18
safe wrappers: 16
raw pointer leaks into safe API: 0
ABI unstable types exported: 2
missing ownership annotations: 1
```

FFI is where dragons rent apartments. Torel makes them sign paperwork. 🐉📄

\---

# 13\. IDE Support

Torel gets an official language server:

```bash
torells
```

Works with:

```txt
VS Code
Visual Studio
JetBrains IDEs
Neovim
Emacs
Zed
Helix
```

\---

## 13.1 IDE Features

Must support:

```txt
autocomplete
go to definition
find references
rename
inline type hints
effect hints
failure hints
ownership diagnostics
arena lifetime visualization
unsafe callsite highlighting
formatter
linter
test runner
benchmark runner
debugger integration
profiler integration
dependency graph
```

\---

## 13.2 Example Inline Hints

```torel
fix user = try load\_user(id);
```

IDE shows:

```txt
user: User
load\_user effects: \[db]
load\_user fails: \[UserNotFound, DbError]
```

For ownership:

```torel
send(move buffer);
```

IDE shows:

```txt
buffer moved here
```

Then later:

```torel
buffer.clear();
```

IDE shows:

```txt
error: buffer was moved at line 42
```

\---

## 13.3 Refactoring

Official refactors:

```txt
rename symbol
extract proc
inline proc
move unit
split type
generate contract implementation
convert failure handling to attempt/catch
add missing match arms
add missing effect declarations
remove unused effects
convert copy to view
convert allocation to arena allocation
```

That last one is nasty-good. Very compiler-butler energy. 🫖

\---

# 14\. Debugger

Official debugger:

```bash
toreldb
```

Run:

```bash
torel debug
```

Attach:

```bash
toreldb attach <pid>
```

\---

## 14.1 Debugger Features

```txt
breakpoints
conditional breakpoints
watch expressions
step in/out/over
async task inspection
actor mailbox inspection
ownership state inspection
arena inspection
raw memory view
effect trace
failure trace
thread view
stack traces with inlined frames
core dump analysis
```

\---

## 14.2 Ownership-Aware Debugging

Example debugger output:

```txt
buffer: Buffer
state: moved
moved\_at: src/server.torel:88
moved\_to: send\_packet()
```

Arena view:

```txt
arena request\_mem
  allocated: 64 KiB
  live objects: 18
  peak: 96 KiB
  released\_at: scope exit
```

This is crucial because Torel memory is explicit. The debugger must understand the language model, not just raw addresses.

\---

## 14.3 Failure Trace

For checked failures:

```txt
Failure: DbError.ConnectionLost
origin: db/postgres/connection.torel:214
propagated through:
  UserService.load
  get\_user
  route handler /users/{id}
handled at:
  app.server.handle\_failure
```

No “somewhere deep in stack land” nonsense. The failure leaves footprints. 🐾

\---

# 15\. Profiler

Official profiler:

```bash
torel profile
```

Modes:

```bash
torel profile cpu
torel profile alloc
torel profile memory
torel profile async
torel profile lock
torel profile io
torel profile db
torel profile gpu
```

\---

## 15.1 CPU Profiling

```bash
torel profile cpu -- ./dist/server
```

Reports:

```txt
hot functions
inlining decisions
branch misses
cache misses
SIMD use
system calls
per-route CPU usage for HTTP apps
```

\---

## 15.2 Allocation Profiling

```bash
torel profile alloc -- ./dist/server
```

Reports:

```txt
allocation count
allocation size
allocation site
arena allocation patterns
heap vs arena usage
temporary allocations
zero-allocation path violations
```

Example:

```txt
Route GET /users/{id}
  allocations: 3
  total: 1.2 KiB
  avoidable: 1 allocation in json.encode
```

Torel should let you mark zero-allocation procedures:

```torel
@no\_alloc
proc parse\_header(bytes: view Bytes) -> Header
    does \[]
    fails \[ParseError]
{
    ...
}
```

If it allocates:

```txt
compile error: @no\_alloc procedure performs alloc effect
```

Absolutely delicious. 🍽️

\---

## 15.3 Async Profiling

```bash
torel profile async -- ./dist/server
```

Reports:

```txt
task count
task lifetime
await points
blocked tasks
slow futures
actor mailbox backlog
channel pressure
executor utilization
```

\---

## 15.4 Database Profiling

```bash
torel profile db -- ./dist/server
```

Reports:

```txt
slow queries
query count per request
connection pool wait time
transaction duration
prepared statement cache hits
row decoding cost
```

This should catch the classic backend crime:

```txt
N+1 query goblin detected
```

The profiler should literally call it that in dev mode. Emotionally necessary. 🧌

\---

# 16\. Linter and Formatter

Formatter:

```bash
torel fmt
```

Linter:

```bash
torel lint
```

Strict mode:

```bash
torel lint --deny warnings
```

\---

## 16.1 Lint Categories

```txt
style
performance
safety
security
concurrency
effects
failures
unsafe
ffi
docs
dependencies
```

Examples:

```txt
unused slot
unnecessary allocation
blocking call in async proc
public proc missing docs
unsafe block missing safety comment
too-broad effect declaration
failure declared but never produced
copy of large value where view is enough
SQL query not parameterized
crypto API deprecated
```

\---

# 17\. Deployment Tooling

Command:

```bash
torel package
```

Outputs:

```txt
native binary
static binary where supported
container image
WASM module
systemd unit
SBOM
debug symbols
profile data
```

\---

## 17.1 Container Build

```bash
torel package container --release
```

Produces:

```txt
minimal image
non-root user
CA certificates if needed
healthcheck metadata
SBOM
binary provenance
```

\---

## 17.2 Production Check

```bash
torel deploy check
```

Checks:

```txt
release build
lockfile present
no dev dependencies
no debug secrets
audited dependencies
unsafe report reviewed
TLS configured
structured logging enabled
panic policy configured
```

Torel should refuse clown deployment unless manually overridden. 🤡🚫

\---

# 18\. Final Toolchain Map

```txt
torel       main CLI
torelc      compiler
torells     language server
toreldb     debugger
torelfmt    formatter backend
torellint   linter backend
torelprof   profiler backend
torelpkg    package registry helper
torelffi    FFI generator
toreldoc    documentation generator
```

Users mostly interact with:

```bash
torel build
torel test
torel bench
torel doc
torel profile
torel debug
torel publish
```

The sub-tools exist for IDEs, CI, and advanced use.

\---

# 19\. Standard Library Final Layout

```txt
core
  mem
  num
  text
  bytes
  array
  slice
  maybe
  result
  choice
  iter
  hash
  compare
  convert
  alloc
  arena
  sync
  atomic
  marker

std
  io
  fs
  path
  net
  http
  tls
  time
  clock
  random
  crypto
  process
  env
  thread
  task
  channel
  actor
  log
  trace
  metrics
  json
  toml
  yaml
  xml
  database
  compress
  archive
  testing
  bench
  debug
  math
  linalg
  stats
  signal
  optimize
  tensor
  autodiff
  gpu

sys
  posix
  linux
  windows
  macos
  wasm
  cpu
  mem
  socket
  signal
  epoll
  kqueue
  iocp
  cuda
  rocm
  metal
```

\---

# 20\. The Big Design Choice

Torel’s standard library should **not** be gigantic and magical.

It should be:

```txt
small where correctness matters
complete where production demands it
strict where safety matters
fast where performance matters
explicit everywhere
```

So the official ecosystem is divided like this:

|Layer|Responsibility|
|-|-|
|`core`|Language foundation|
|`std`|Production-safe common APIs|
|`sys`|Low-level power|
|official packages|HTTP extras, DB drivers, GPU backends|
|community packages|frameworks, tools, integrations|

This gives Torel long-term survival instead of “one standard library to rule them all and collapse under its own furniture.” 🪑💥

\---

# 21\. Current Ecosystem Identity

Torel is now:

> \*\*A strict native systems/backend language with explicit memory, checked effects, checked failures, production tooling, typed infrastructure libraries, and serious math/tensor capability.\*\*

Or in less polite compiler goblin terms:

```txt
C power
Rust-ish safety ambition
Go-like deployment simplicity
TypeScript-like tooling expectations
Zig-like explicitness
MLIR/LLVM-grade optimization dreams
but with its own syntax and laws
```

This is no longer “language idea.”
This is now **platform architecture**. ⚙️👑

