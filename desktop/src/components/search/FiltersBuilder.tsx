// FiltersBuilder (VS-07): UI del query builder visual AND/OR sobre metadata
// tipada (string/int/float/bool/datetime) usando react-querybuilder (v8).
//
// Componente pesado (~200 kB) → lazy-load desde WorkspaceShell (mismo patrón
// del Inspector en VS-03). La lógica pura (inferir tipos, serializar a
// `VantaMemoryFilter`, evaluar el árbol) vive en `./filters-core` y el shell
// la importa directamente — este archivo solo pinta el builder.
import { useMemo } from "react";
import { QueryBuilder, type Field, type Operator, type RuleGroupType } from "react-querybuilder";
import "react-querybuilder/dist/query-builder.css";
import type { MetaField, MetaFieldType } from "./filters-core";

const CMP_OPS: Operator[] = [
  { name: "=", label: "=", value: "=" },
  { name: "!=", label: "!=", value: "!=" },
  { name: "<", label: "<", value: "<" },
  { name: ">", label: ">", value: ">" },
  { name: "<=", label: "<=", value: "<=" },
  { name: ">=", label: ">=", value: ">=" },
];
/** Bool solo admite igualdad/desigualdad (no tiene orden). */
const EQ_OPS: Operator[] = [
  { name: "=", label: "=", value: "=" },
  { name: "!=", label: "!=", value: "!=" },
];

interface FiltersBuilderProps {
  /** Esquema tipado de metadata (inferMetaFields) — alimenta editores/ops. */
  fields: MetaField[];
  query: RuleGroupType;
  onChange: (q: RuleGroupType) => void;
}

export default function FiltersBuilder({ fields, query, onChange }: FiltersBuilderProps) {
  const typeByName = useMemo(() => new Map(fields.map((f) => [f.name, f.type])), [fields]);
  const qbFields: Field[] = useMemo(
    () =>
      fields.map((f) => ({
        name: f.name,
        label: f.type === "datetime" ? `${f.name} (RFC3339)` : f.name,
        type: f.type,
      })),
    [fields],
  );
  const fieldType = (name: string): MetaFieldType => typeByName.get(name) ?? "string";

  return (
    <div className="qb-manga">
      <QueryBuilder
        fields={qbFields}
        query={query}
        onQueryChange={onChange}
        operators={CMP_OPS}
        getOperators={(field) => (fieldType(field) === "bool" ? EQ_OPS : CMP_OPS)}
        getValueEditorType={(field) => (fieldType(field) === "bool" ? "checkbox" : "text")}
        getInputType={(field) =>
          fieldType(field) === "int" || fieldType(field) === "float" ? "number" : "text"
        }
        parseNumbers="enhanced"
        resetOnFieldChange={false}
        maxLevels={4}
      />
      {/* Estética manga/linocut (VS-01 tokens) sobre el CSS default del builder. */}
      <style>{`
        .qb-manga .queryBuilder {
          border: 2px solid #000;
          background: var(--color-paper, #F2EDE2);
          border-radius: 0;
          padding: 8px;
          box-shadow: 4px 4px 0 0 #000;
        }
        .qb-manga .queryBuilder .ruleGroup {
          border: 2px solid #000;
          background: rgba(0,0,0,0.04);
          padding: 8px;
        }
        .qb-manga .queryBuilder .ruleGroup .ruleGroup { background: rgba(255,85,0,0.07); }
        .qb-manga .queryBuilder .rule { margin: 4px 0; }
        .qb-manga .queryBuilder select,
        .qb-manga .queryBuilder input[type="text"],
        .qb-manga .queryBuilder input[type="number"] {
          border: 2px solid #000;
          background: var(--color-cream, #FBF9F5);
          color: var(--color-foreground, #000);
          border-radius: 0;
          padding: 2px 6px;
          font-family: var(--font-mono, "Space Mono", monospace);
          font-size: 11px;
        }
        .qb-manga .queryBuilder input[type="checkbox"] { accent-color: #FF5500; }
        .qb-manga .queryBuilder button {
          border: 2px solid #000;
          background: var(--color-cream, #FBF9F5);
          color: #000;
          border-radius: 0;
          padding: 1px 6px;
          font-weight: 600;
          cursor: pointer;
        }
        .qb-manga .queryBuilder button:hover { background: #FF5500; }
        .qb-manga .queryBuilder .queryBuilder-remove,
        .qb-manga .queryBuilder .ruleGroup-remove { background: #000; color: #FBF9F5; }
      `}</style>
    </div>
  );
}