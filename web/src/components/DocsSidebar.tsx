import { useEffect, useRef, useState } from "react";

interface DocsSidebarItem {
  id: string;
  num: string;
  title: string;
}

interface DocsSidebarProps {
  items: DocsSidebarItem[];
}

export function DocsSidebar({ items }: DocsSidebarProps) {
  const [activeId, setActiveId] = useState(items[0]?.id ?? "");
  const [search, setSearch] = useState("");
  const observerRef = useRef<IntersectionObserver | null>(null);

  useEffect(() => {
    observerRef.current = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setActiveId(entry.target.id);
          }
        }
      },
      { rootMargin: "-80px 0px -60% 0px", threshold: 0 },
    );

    const currentObserver = observerRef.current;
    for (const item of items) {
      const el = document.getElementById(item.id);
      if (el) currentObserver.observe(el);
    }

    return () => currentObserver.disconnect();
  }, [items]);

  const filtered = search.trim()
    ? items.filter(
        (item) =>
          item.title.toLowerCase().includes(search.toLowerCase()) ||
          item.id.toLowerCase().includes(search.toLowerCase()),
      )
    : items;

  const handleClick = (id: string) => {
    setActiveId(id);
    const el = document.getElementById(id);
    if (el) {
      el.scrollIntoView({ behavior: "smooth", block: "start" });
    }
  };

  return (
    <aside className="docs-sidebar">
      <div className="docs-sidebar-header">
        <span className="docs-sidebar-label">On this page</span>
        <input
          type="text"
          className="docs-sidebar-search"
          placeholder="Filter sections..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          aria-label="Filter documentation sections"
        />
      </div>

      <nav className="docs-sidebar-nav" aria-label="Documentation sections">
        {filtered.map((item) => (
          <button
            key={item.id}
            className={`docs-nav-link${activeId === item.id ? " active" : ""}`}
            onClick={() => handleClick(item.id)}
            type="button"
          >
            [{item.num}] {item.title}
          </button>
        ))}
      </nav>
    </aside>
  );
}
