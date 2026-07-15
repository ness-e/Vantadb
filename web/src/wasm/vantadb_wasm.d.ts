/* tslint:disable */
/* eslint-disable */

export class VantaDB {
    free(): void;
    [Symbol.dispose](): void;
    add_edge(source_id: bigint, target_id: bigint, label: string, weight?: number | null): void;
    audit_text_index(namespace?: string | null): any;
    audit_text_index_deep(namespace?: string | null): any;
    capabilities(): any;
    close(): void;
    compact_layout(): bigint;
    compact_wal(): void;
    static connect_persistent(path: string): Promise<VantaDB>;
    delete(namespace: string, key: string): boolean;
    delete_node(id: bigint, reason: string): void;
    explain_memory_search(request: any): any;
    export_all(path: string): any;
    export_namespace(path: string, namespace: string): any;
    flush(): void;
    generate_snippet(payload: string, text_query: string, with_highlighting: boolean): string | undefined;
    get(namespace: string, key: string): any;
    get_node(id: bigint): any;
    graph_bfs(roots: BigUint64Array, max_depth: number): any;
    graph_dfs(roots: BigUint64Array, max_depth: number): any;
    graph_is_dag(roots: BigUint64Array): boolean;
    graph_topological_sort(roots: BigUint64Array): any;
    import_file(path: string): any;
    import_records(records: any): any;
    insert_node(id: bigint, content: string | null | undefined, vector: Float32Array | null | undefined, fields: any): void;
    list(namespace: string, options: any): any;
    list_namespaces(): any;
    load(): Promise<void>;
    constructor(config_val?: any | null);
    static open(path: string): VantaDB;
    operational_metrics(): any;
    purge_expired(): bigint;
    put(input: any): any;
    put_batch(inputs: any): any;
    query(query: string): any;
    rebuild_index(): any;
    repair_text_index(): any;
    save(): Promise<void>;
    search(request: any): any;
    search_vector(vector: Float32Array, top_k: number): any;
}
