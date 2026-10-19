//! Per-thread pool of `NeighborVec` lists for reuse across `search_layer`
//! calls — reduces SmallVec heap allocations when neighbor lists exceed 32.

use crate::index::graph::NeighborVec;

/// Cap máximo de buffers de vecinos cacheados por hilo para evitar memory leaks (FIND-213).
pub(super) const MAX_POOL_CAPACITY: usize = 8;

thread_local! {
    static NL_POOL: std::cell::RefCell<Vec<NeighborVec>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Extrae un `NeighborVec` reciclado del pool del hilo actual o aloca uno nuevo si está vacío.
#[inline]
#[allow(dead_code)] // Reason: utility companion to give_nl for recycling buffers and unit tests
pub(super) fn take_nl() -> NeighborVec {
    NL_POOL
        .with(|pool| pool.borrow_mut().pop())
        .unwrap_or_default()
}

/// Devuelve un `NeighborVec` al pool del hilo para reutilización. Limpia los contenidos primero.
/// Si el pool ya alcanzó `MAX_POOL_CAPACITY`, el buffer se descarta para ser liberado por el asignador.
#[inline]
pub(super) fn give_nl(mut v: NeighborVec) {
    v.clear();
    NL_POOL.with(|pool| {
        let mut p = pool.borrow_mut();
        if p.len() < MAX_POOL_CAPACITY {
            p.push(v);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    #[test]
    fn test_nl_pool_bounded_capacity() {
        // Drenar el pool por si tests previos dejaron buffers
        NL_POOL.with(|p| p.borrow_mut().clear());

        // Insertar más de MAX_POOL_CAPACITY buffers
        for i in 0..20 {
            let v: NeighborVec = smallvec![i as u128; 40]; // Excede inline capacity (32)
            give_nl(v);
        }

        // Verificar que el pool retiene como máximo MAX_POOL_CAPACITY
        let pooled_count = NL_POOL.with(|p| p.borrow().len());
        assert_eq!(
            pooled_count, MAX_POOL_CAPACITY,
            "NL_POOL no debe exceder MAX_POOL_CAPACITY={MAX_POOL_CAPACITY}"
        );

        // Extraer elementos y verificar que se reutilizan limpios
        for _ in 0..MAX_POOL_CAPACITY {
            let v = take_nl();
            assert!(v.is_empty(), "El buffer reciclado debe estar limpio");
        }

        // Una vez vacío, take_nl debe devolver un vector nuevo vacío
        let fresh = take_nl();
        assert!(fresh.is_empty());
    }
}
