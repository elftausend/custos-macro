# custos-macro

Adds a `Stack` operation based on a `CPU` operation.

# Example

Expands a `CPU` implementation to a `Stack` and `CPU` implementation.

```rust
#[impl_stack]
impl<T, D, S> ElementWise<T, D, S> for CPU
where
    T: Number,
    D: MainMemory,
    S: Shape
{
    fn add(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, CPU, S> {
        let mut out = self.retrieve(lhs.len, (lhs, rhs));
        cpu_element_wise(lhs, rhs, &mut out, |o, a, b| *o = a + b);
        out
    }

    fn mul(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, CPU, S> {
        let mut out = self.retrieve(lhs.len, (lhs, rhs));
        cpu_element_wise(lhs, rhs, &mut out, |o, a, b| *o = a * b);
        out
    }
}

'#[impl_stack]' expands the implementation above to the following 'Stack' implementation:

impl<T, D, S> ElementWise<T, D, S> for Stack
where
    T: Number,
    D: MainMemory,
    S: Shape
{
    fn add(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, Stack, S> {
        let mut out = self.retrieve(lhs.len, (lhs, rhs));
        cpu_element_wise(lhs, rhs, &mut out, |o, a, b| *o = a + b);
        out
    }

    fn mul(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, Stack, S> {
        let mut out = self.retrieve(lhs.len, (lhs, rhs));
        cpu_element_wise(lhs, rhs, &mut out, |o, a, b| *o = a * b);
        out
    }
}

// Now is it possible to execute this operations with a CPU and Stack device.
```