use std::alloc::Layout;
use std::ptr::NonNull;
use std::{alloc, mem, ptr};
use std::ops::Index;

#[derive(Debug)]
pub struct Vec<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
}

unsafe impl<T: Send> Send for Vec<T> {}

impl<T> Vec<T> {
    pub fn new() -> Self {
        assert_ne!(mem::size_of::<T>(), 0, "Cannot handle zero-sized types");
        Vec {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }

    pub fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1))
        } else {
            // This cannot overflow since self.cap <= isize::MAX
            let new_cap = 2 * self.cap;
            (new_cap, Layout::array::<T>(new_cap))
        };

        // Layout::array uses isize::MAX under the hood through Rust compiler, LLVM and hardware
        // architecture
        let new_layout = new_layout.expect("Allocation exceeded.");

        // layout - Describes how much memory and alignment
        // alloc - Reserves the address space in memory

        // raw pointer to a byte: a memory address (usize-sized) pointing to where an u8 lives
        let new_ptr: *mut u8 = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout)}
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size())}
        };
        // alloc() finds free address space
        // Returns: *mut u8 at start of that space
        // ptr is a virtual address (e.g., 0x7f8a1c000000)
        // This doesn't correspond to physical RAM directly
        // The OS maps virtual → physical addresses through page tables

        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout)
        };

        self.cap = new_cap;
    }

    pub fn push(&mut self, elem: T) {
        // Grow if 0 or full
        if self.len == self.cap {
            self.grow();
        }

        // Note: If the len of the array is 0 we want to write to the old len
        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), elem);
        }

        // Adjust the new len
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(ptr::read(self.ptr.as_ptr().add(self.len)))
            }
        }
    }

    pub fn last(&mut self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            unsafe { Some(&*self.ptr.as_ptr().add(self.len - 1)) }
        }
    }
}

impl<T> Index<i32> for Vec<T> {
    type Output = T;

    fn index(&self, index: i32) -> &T {
        assert!(index < self.len as i32, "Index out of bounds");
        unsafe { &*self.ptr.as_ptr().add(index as usize) }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_new() {
        let v: Vec<i32> = Vec {
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
        };

        assert_eq!(v.len, 0);
        assert_eq!(v.cap, 0);
    }

    #[test]
    pub fn test_init() {
        let v: Vec<i32> = Vec::new();
        assert_eq!(v.len, 0);
        assert_eq!(v.cap, 0);
    }

    #[test]
    pub fn test_vec_grow() {
        let mut v: Vec<i32> = Vec::new();
        v.grow();
        v.grow();
        v.grow();
        assert_eq!(v.len, 0);
        assert_eq!(v.cap, 4);
        println!("Vec struct: {:?}", v)
    }

    #[test]
    pub fn test_vec_push() {
        let mut v: Vec<i32> = Vec::new();
        v.push(3);
        assert_eq!(v.len, 1);
        assert_eq!(v.cap, 1);
        assert_eq!(v[0], 3);
        println!("Vec struct: {:?}", v);
    }

    #[test]
    pub fn test_vec_pop() {
        let mut v: Vec<i32> = Vec::new();
        v.push(3);
        v.push(4);
        println!("Vec struct: {:?}", v);
        v.pop();
        println!("Vec struct: {:?}", v);
        assert_eq!(v.len, 1);
        assert_eq!(v.cap, 2);
        assert_eq!(v[0], 3);
        assert_eq!(v.last(), Some(3).as_ref());
    }
}