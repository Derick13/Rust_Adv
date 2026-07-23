# RawVec

### Instantiation

A Vec has three parts: a pointer to the allocation, the size of the allocation and the number of elements
that have been initialized.

We can implement all of the above requirements in stable Rust. To do this, instead of using Unique<T> we will 
use NonNull<T>, another wrapper around a raw pointer, which gives us two of the above properties, namely it is 
covariant over T and is declared to never be null.

"Covariant over T" means: if you have a type Foo<T>, and you can substitute T with a more specific type, then Foo<U> can 
be substituted where Foo<T> is expected (where U is a subtype of T).

In other words: the "direction" of subtyping is preserved.

- If you have a Box<Cat> and Cat is a subtype of Animal
- Covariant means Box<Cat> is a subtype of Box<Animal>
- You can use a Box<Cat> wherever a Box<Animal> is expected

```rust
pub struct Vec<T> {
    ptr: NonNull<T>, // NonNull is Send/Sync
    cap: isize,
    len: isize,
}
```

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

Vec<T> needs unsafe impl Send/Sync because of other invariants that Rust can't automatically verify are safe to send 
between threads. The compiler can't prove that the internal representation is thread-safe, so we have to promise it 
ourselves with unsafe.

### Space Allocation

```rust
impl<T> Vec<T> {
    pub fn new() -> Self {
        assert_ne!(mem::size_of::<T>(), 0, "Cannot handle zero-sized types");
        Vec {
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
        }
    }
}
```

```bash
if cap == 0:
  allocate()
  cap = 1
else:
  reallocate()
  cap *= 2
```


We need to figure out what to actually do when we do want space. For that, we use the 
global allocation functions `alloc`, `realloc` and `dealloc` which are available in 
stable Rust in `std::alloc`.

Case 1: Index into arrays with unsigned integers

Issue:
C and Rust use signed integers (int, isize) for array indexing
LLVM GEP (GetElementPtr) expects unsigned (like i64 without sign)
Signed vs unsigned have different overflow behaviors
    - Unsigned indexing: You can only walk forward (0, 1, 2, 3...)
    - Signed indexing: You can walk forward OR backward (-1, 0, 1, 2...)

Solution: To prevent GEP going in the wrong direction we need to limit allocation.
To prevent negative indices from wrapping around to huge values, we limit allocations to `isize::MAX` so that even 
if a negative index is misinterpreted, it won't overflow.

Case 2: Empty Allocations

Issue:
Two kinds of empty allocations we need to worry about: 
- cap = 0 for all T
- cap > 0 for zero-sized types.

Case 3: Add Elements (push)

First we need to check if the array is full so we need to grow, write to the next one in the index and increment the
length.

We cannot just write to the index since that would override or call drop on the old value.

Hence we use ptr::write to overwrite the target address with the add method which guarantees that 
the resulting pointer is pointing to an allocation.

Case 4: Remove Elements (pop)

ptr::read will copy out the bits from the target address and leave the memory logically uninitialized but
the memory `cap` stays allocated.
If the old len is 1, we want to read out the 0 index. So we should offset with the new len.

Case 5: Reading elements (Index)

Read from the Vec by .add(index) to offset the pointer by `index * size_of::<T>()` bytes. Without it
you always read from index 0.

