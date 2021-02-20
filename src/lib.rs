use std::ptr::NonNull;
use std::alloc;

pub struct MyVec<T> {
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
}

impl <T> MyVec<T> {
    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            capacity: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        if std::mem::size_of::<T>() == 0 {
            panic!("No zero sized types");
        }

        if self.capacity == 0 {
            let layout = alloc::Layout::array::<T>(4).expect("could not allocate");

            // SAFETY: The layout is hardcoded to be 4 * size_of<T> and
            // size_of<T> is > 0

            let ptr = unsafe {alloc::alloc(layout) } as *mut T;
            let ptr = NonNull::new(ptr).expect("could not allocate memory");

            // SAFETY: ptr is non-null and we have just allocated enough
            // for this item (plus 3 more).
            unsafe { ptr.as_ptr().write(item) };

            self.ptr = ptr;
            self.capacity = 4;
            self.len = 1;
        } else if self.len < self.capacity {
            let offset = self
                .len.checked_mul(std::mem::size_of::<T>()).expect("Cannot reach memory location");
            assert!(offset < isize::MAX as usize, "wrapped isize");

            // SAFETY: Offset cannot wrap around and pointer is pointing to valid memory
            // and writing to an offset at self.len is valid.
            unsafe {
                self.ptr.as_ptr().add(self.len).write(item);
            }
            self.len += 1;
        } else {
            debug_assert!(self.len == self.capacity);
            let new_capacity = self.capacity.checked_mul(2).expect("capacity exceeded");

            let layout = alloc::Layout::from_size_align(
                std::mem::size_of::<T>() * self.capacity,
                std::mem::align_of::<T>()).expect("could not get layout");

            let ptr = unsafe {
                let new_size = std::mem::size_of::<T>() * new_capacity;
                let ptr = alloc::realloc(self.ptr.as_ptr() as *mut u8, layout, new_size);
                let ptr = NonNull::new(ptr as *mut T).expect("could not reallocate");

                ptr.as_ptr().add(self.len).write(item);
                ptr
            };

            self.ptr = ptr;
            self.len += 1;
            self.capacity = new_capacity;
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len (&self) -> usize {
        self.len
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }

        Some(unsafe {
            &*self.ptr.as_ptr().add(index)
        })
    }
}

impl <T> Drop for MyVec<T> {
    fn drop(&mut self) {
        unsafe {
            // std::ptr::drop_in_place(self.ptr.as_ptr())
            std::ptr::drop_in_place(std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len()));
            let layout = alloc::Layout::from_size_align(
                std::mem::size_of::<T>() * self.capacity,
                std::mem::align_of::<T>()
            ).expect("could not get layout");
            alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout)
        }
    }
}

#[derive(PartialEq, Debug)]
struct Dropping(usize);

impl Drop for Dropping {
    fn drop(&mut self) {
        println!("Droppin'")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut vec: MyVec<usize> = MyVec::new();
        vec.push(1usize);
        vec.push(2);
        vec.push(3);
        vec.push(4);
        vec.push(5);

        assert_eq!(vec.capacity(), 8);
        assert_eq!(vec.len(), 5);

        assert_eq!(vec.get(6), None);
        assert_eq!(vec.get(2), Some(&3))
    }

    #[test]
    fn test_dropping() {
        let mut vec: MyVec<Dropping> = MyVec::new();

        vec.push(Dropping(1));
        vec.push(Dropping(2));

        println!("going to get");
        let expected = Some(&Dropping(1));
        assert_eq!(vec.get(0), expected);
        println!("just got");

        assert_eq!(vec.capacity(), 4);
        assert_eq!(vec.len(), 2);
    }
}
