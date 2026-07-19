#![cfg(miri)]

use std::ptr;

struct SendPtr(*const f32);
unsafe impl Send for SendPtr {}
unsafe impl Sync for SendPtr {}

#[test]
fn miri_raw_ptr_slice_roundtrip() {
    let data: [f32; 4] = [1.0, 2.0, 3.0, 4.0];
    let ptr = &data as *const f32;
    let len: usize = 4;

    let slice: &[f32] = unsafe { std::slice::from_raw_parts(ptr, len) };
    assert_eq!(slice.len(), 4);
    assert_eq!(slice[0], 1.0);
    assert_eq!(slice[3], 4.0);
}

#[test]
fn miri_raw_ptr_slice_empty() {
    let data: [f32; 0] = [];
    let ptr = &data as *const f32;

    let slice: &[f32] = unsafe { std::slice::from_raw_parts(ptr, 0) };
    assert_eq!(slice.len(), 0);
}

#[test]
fn miri_raw_ptr_mut_slice_roundtrip() {
    let mut data: [f32; 3] = [10.0, 20.0, 30.0];
    let ptr = &mut data as *mut f32;

    let slice: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(ptr, 3) };
    slice[1] = 99.0;

    assert_eq!(data[1], 99.0);
}

#[test]
fn miri_send_ptr_across_threads() {
    let data: Box<[f32; 2]> = Box::new([1.5, 2.5]);
    let ptr = SendPtr(&*data as *const f32);

    let raw = ptr.0;
    let handle = std::thread::spawn(move || {
        let slice: &[f32] = unsafe { std::slice::from_raw_parts(raw, 2) };
        assert_eq!(slice[0], 1.5);
    });

    handle.join().unwrap();
}

#[test]
fn miri_raw_ptr_alignment() {
    #[repr(align(64))]
    struct Aligned {
        data: [f32; 8],
    }

    let aligned = Aligned {
        data: [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
    };

    let ptr = &aligned.data as *const f32;
    let slice: &[f32] = unsafe { std::slice::from_raw_parts(ptr, 8) };
    assert_eq!(slice[7], 7.0);
}

#[test]
fn miri_raw_ptr_slice_from_nonnull() {
    let data: [f32; 5] = [0.0, -1.0, -2.0, -3.0, -4.0];
    let ptr = ptr::NonNull::from(&data).as_ptr();

    let slice: &[f32] = unsafe { std::slice::from_raw_parts(ptr as *const f32, 5) };
    assert_eq!(slice[4], -4.0);
}

#[test]
fn miri_raw_ptr_cast_roundtrip() {
    let val: u64 = 0xDEAD_BEEF;
    let ptr = &val as *const u64;

    let bytes: &[u8] = unsafe { std::slice::from_raw_parts(ptr as *const u8, 8) };
    assert_eq!(bytes.len(), 8);

    let restored: &u64 = unsafe { &*(bytes.as_ptr() as *const u64) };
    assert_eq!(*restored, 0xDEAD_BEEF);
}

#[test]
fn miri_sync_ptr_concurrent_access() {
    use std::sync::Arc;

    let data: [f32; 3] = [100.0, 200.0, 300.0];
    let ptr = SendPtr(&data as *const f32);
    let arc_ptr = Arc::new(ptr);

    let mut handles = vec![];
    for _ in 0..4 {
        let p = Arc::clone(&arc_ptr);
        handles.push(std::thread::spawn(move || {
            let slice: &[f32] = unsafe { std::slice::from_raw_parts(p.0, 3) };
            assert!(slice[0] > 0.0);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn miri_raw_ptr_subslice() {
    let data: [f32; 10] = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let base_ptr = &data as *const f32;

    let ptr_3 = unsafe { base_ptr.add(3) };
    let slice: &[f32] = unsafe { std::slice::from_raw_parts(ptr_3, 4) };
    assert_eq!(slice[0], 3.0);
    assert_eq!(slice[3], 6.0);
}
