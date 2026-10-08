use super::super::builder::Embedded;
use super::super::serialization::validate_namespace;
use super::super::types::MemorySearchRequest;

impl Embedded {
    /// Match a search-only namespace pattern against the known namespace set.
    ///
    /// Returns `None` when `ns` is NOT a pattern (the caller falls back to
    /// [`validate_namespace`]); `Some(None)` for `"*"` (match all);
    /// `Some(Some(prefix))` for `"prefix/*"` where `prefix` keeps its
    /// trailing slash so `"kb/"` matches `"kb/docs"` but not `"kbx"`.
    /// The prefix must be non-empty and use only the `validate_namespace`
    /// charset — anything else containing `*` (e.g. `"a*b"`, `"**"`, `"/*"`)
    /// is NOT a pattern and keeps the previous silent-skip behavior.
    fn wildcard_prefix(ns: &str) -> Option<Option<&str>> {
        if !ns.contains('*') {
            return None;
        }
        if ns == "*" {
            return Some(None);
        }
        if let Some(prefix) = ns.strip_suffix("/*") {
            if !prefix.is_empty()
                && prefix
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'/' | b'-'))
            {
                // Keep the trailing slash: prefix match on "kb/" is exact.
                return Some(Some(&ns[..ns.len() - 1]));
            }
        }
        None
    }

    /// Search across **multiple namespaces** with a single request.
    ///
    /// The `namespace` field on `request` is ignored; instead, every namespace
    /// listed in `namespaces` is searched independently and the results are
    /// merged into a single list sorted by descending score, capped at
    /// `request.top_k` globally.
    ///
    /// Entries may be search-only wildcard patterns: `"*"` matches every
    /// known namespace and `"prefix/*"` matches every namespace starting
    /// with `"prefix/"` (e.g. `"kb/*"`). Patterns expand against a single
    /// [`Self::list_namespaces`] snapshot; overlapping patterns are
    /// de-duplicated (order-preserving) so no namespace is searched twice.
    /// A pattern with zero matches contributes nothing.
    ///
    /// Namespaces that produce no results or fail validation are silently
    /// skipped.  An empty `namespaces` slice returns an empty `Vec`.
    ///
    /// WIRE-08 pagination is **not available** here: the per-namespace search
    /// runs through [`search`](Self::search), which drops the cursor. Use
    /// [`search_page`](Self::search_page) on a single namespace to paginate.
    ///
    /// # Errors
    /// Returns the first fatal engine error encountered (e.g. storage I/O
    /// failure).  Invalid namespace strings are silently skipped rather than
    /// propagated.
    pub fn search_multi(
        &self,
        namespaces: &[&str],
        request: MemorySearchRequest,
    ) -> crate::Result<Vec<crate::sdk::types::MemorySearchHit>> {
        if namespaces.is_empty() || request.top_k == 0 {
            return Ok(Vec::new());
        }

        // EGO-03: expand search-only wildcard patterns against one namespace
        // snapshot. Literals pass through untouched; malformed patterns fall
        // through to `validate_namespace` below (silent-skip, as before).
        let mut targets: Vec<String> = Vec::with_capacity(namespaces.len());
        if namespaces.iter().any(|ns| ns.contains('*')) {
            let known = self.list_namespaces()?;
            for &ns in namespaces {
                match Self::wildcard_prefix(ns) {
                    None => targets.push(ns.to_string()),
                    Some(None) => targets.extend(known.iter().cloned()),
                    Some(Some(prefix)) => {
                        targets.extend(known.iter().filter(|k| k.starts_with(prefix)).cloned())
                    }
                }
            }
            // Order-preserving dedup: overlapping patterns must not search
            // the same namespace twice (duplicate hits in the merge).
            let mut seen = std::collections::HashSet::with_capacity(targets.len());
            targets.retain(|ns| seen.insert(ns.clone()));
        } else {
            targets.extend(namespaces.iter().map(|s| s.to_string()));
        }

        let mut all_hits: Vec<crate::sdk::types::MemorySearchHit> = Vec::new();

        for ns in &targets {
            // Build a per-namespace request by cloning the prototype and
            // overwriting the namespace field.
            let ns_req = MemorySearchRequest {
                namespace: ns.clone(),
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
        request: MemorySearchRequest,
    ) -> crate::Result<Vec<crate::sdk::types::MemorySearchHit>> {
        let namespaces = self.list_namespaces()?;
        if namespaces.is_empty() {
            return Ok(Vec::new());
        }

        // Convert owned Strings to &str slices for search_multi.
        let ns_refs: Vec<&str> = namespaces.iter().map(String::as_str).collect();
        self.search_multi(&ns_refs, request)
    }
}

#[cfg(test)]
mod tests {
    use super::Embedded;
    use crate::sdk::connect::connect;
    use crate::sdk::types::{MemoryInput, MemoryMetadata, MemorySearchRequest};

    fn setup() -> Embedded {
        connect(":memory:").expect("in-memory db open")
    }

    fn put(db: &Embedded, namespace: &str, key: &str, payload: &str) {
        db.put(MemoryInput {
            namespace: namespace.into(),
            key: key.into(),
            payload: payload.into(),
            metadata: MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
            ..Default::default()
        })
        .expect("put should succeed");
    }

    fn text_req(text_query: &str, top_k: usize) -> MemorySearchRequest {
        MemorySearchRequest {
            namespace: String::new(),
            text_query: Some(text_query.into()),
            top_k,
            ..Default::default()
        }
    }

    fn seed(db: &Embedded) {
        put(db, "kb/docs", "a", "rust memory engine");
        put(db, "kb/facts", "b", "rust borrow checker");
        put(db, "other", "c", "rust unrelated namespace");
    }

    fn hit_namespaces(db: &Embedded, namespaces: &[&str]) -> Vec<String> {
        let hits = db
            .search_multi(namespaces, text_req("rust", 10))
            .expect("search_multi should succeed");
        let mut ns: Vec<String> = hits.iter().map(|h| h.record.namespace.clone()).collect();
        ns.sort();
        ns.dedup();
        ns
    }

    #[test]
    fn search_multi_expands_prefix_pattern_ego03() {
        let db = setup();
        seed(&db);
        // "kb/" prefix matches kb/docs + kb/facts, not "other".
        assert_eq!(hit_namespaces(&db, &["kb/*"]), vec!["kb/docs", "kb/facts"]);
    }

    #[test]
    fn search_multi_star_matches_all_ego03() {
        let db = setup();
        seed(&db);
        let via_star = hit_namespaces(&db, &["*"]);
        let hits = db.search_all(text_req("rust", 10)).expect("search_all");
        let mut via_all: Vec<String> = hits.iter().map(|h| h.record.namespace.clone()).collect();
        via_all.sort();
        via_all.dedup();
        assert_eq!(via_star, via_all);
        assert_eq!(via_star, vec!["kb/docs", "kb/facts", "other"]);
    }

    #[test]
    fn search_multi_malformed_pattern_silently_skipped_ego03() {
        let db = setup();
        seed(&db);
        // Mid-string stars and empty patterns are not patterns: same silent
        // skip as before (no hits, no error).
        assert!(hit_namespaces(&db, &["k*b"]).is_empty());
        assert!(hit_namespaces(&db, &["**"]).is_empty());
        assert!(hit_namespaces(&db, &["/*"]).is_empty());
        assert!(hit_namespaces(&db, &["nomatch/*"]).is_empty());
    }

    #[test]
    fn search_multi_literals_and_overlap_unchanged_ego03() {
        let db = setup();
        seed(&db);
        // Literals behave exactly as before.
        assert_eq!(hit_namespaces(&db, &["kb/docs"]), vec!["kb/docs"]);
        // Overlapping patterns dedup: each record appears once.
        let hits = db
            .search_multi(&["kb/*", "*"], text_req("rust", 10))
            .expect("search_multi should succeed");
        let mut keys: Vec<String> = hits.iter().map(|h| h.record.key.clone()).collect();
        keys.sort();
        assert_eq!(keys, vec!["a", "b", "c"]);
    }
}
