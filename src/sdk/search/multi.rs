use super::super::builder::VantaEmbedded;
use super::super::serialization::validate_namespace;
use super::super::types::VantaMemorySearchRequest;

impl VantaEmbedded {
    /// Search across **multiple namespaces** with a single request.
    ///
    /// The `namespace` field on `request` is ignored; instead, every namespace
    /// listed in `namespaces` is searched independently and the results are
    /// merged into a single list sorted by descending score, capped at
    /// `request.top_k` globally.
    ///
    /// Namespaces that produce no results or fail validation are silently
    /// skipped.  An empty `namespaces` slice returns an empty `Vec`.
    ///
    /// # Errors
    /// Returns the first fatal engine error encountered (e.g. storage I/O
    /// failure).  Invalid namespace strings are silently skipped rather than
    /// propagated.
    pub fn search_multi(
        &self,
        namespaces: &[&str],
        request: VantaMemorySearchRequest,
    ) -> crate::Result<Vec<crate::sdk::types::VantaMemorySearchHit>> {
        if namespaces.is_empty() || request.top_k == 0 {
            return Ok(Vec::new());
        }

        let mut all_hits: Vec<crate::sdk::types::VantaMemorySearchHit> = Vec::new();

        for &ns in namespaces {
            // Build a per-namespace request by cloning the prototype and
            // overwriting the namespace field.
            let ns_req = VantaMemorySearchRequest {
                namespace: ns.to_string(),
                ..request.clone()
            };

            // Skip namespaces that fail validation (e.g. empty string) rather
            // than short-circuiting the whole call.
            if validate_namespace(ns).is_err() {
                continue;
            }

            // Storage / engine errors propagate (only namespace validation
            // above short-circuits per-namespace via continue).
            let hits = self.search(ns_req)?;
            all_hits.extend(hits);
        }

        // Merge: sort by score descending, stable (preserve per-namespace order
        // for ties), then truncate to the global top_k.
        all_hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all_hits.truncate(request.top_k);

        Ok(all_hits)
    }

    /// Search across **all known namespaces**.
    ///
    /// Discovers the full namespace set via [`Self::list_namespaces`] and
    /// delegates to [`Self::search_multi`].  Results are merged and sorted
    /// by descending score, capped at `request.top_k`.
    ///
    /// This is a convenience wrapper that performs a complete namespace scan
    /// before searching; prefer [`Self::search_multi`] when the target
    /// namespaces are known ahead of time.
    ///
    /// # Errors
    /// Propagates any engine error from `list_namespaces` or `search_multi`.
    pub fn search_all(
        &self,
        request: VantaMemorySearchRequest,
    ) -> crate::Result<Vec<crate::sdk::types::VantaMemorySearchHit>> {
        let namespaces = self.list_namespaces()?;
        if namespaces.is_empty() {
            return Ok(Vec::new());
        }

        // Convert owned Strings to &str slices for search_multi.
        let ns_refs: Vec<&str> = namespaces.iter().map(String::as_str).collect();
        self.search_multi(&ns_refs, request)
    }
}
