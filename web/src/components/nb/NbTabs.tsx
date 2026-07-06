import { useState } from "react";

interface Tab {
  id: string;
  label: string;
  content: React.ReactNode;
}

interface NbTabsProps {
  tabs: Tab[];
  defaultTab?: string;
}

export function NbTabs({ tabs, defaultTab }: NbTabsProps) {
  const [active, setActive] = useState(defaultTab ?? tabs[0]?.id);

  return (
    <div className="nb-tabs">
      <div className="nb-tabs-header" role="tablist">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            role="tab"
            aria-selected={active === tab.id}
            className={`nb-tabs-btn${active === tab.id ? " nb-tabs-btn--active" : ""}`}
            onClick={() => setActive(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </div>
      {tabs.map((tab) => (
        <div
          key={tab.id}
          role="tabpanel"
          className={`nb-tabs-panel${active === tab.id ? " nb-tabs-panel--active" : ""}`}
        >
          {tab.content}
        </div>
      ))}
    </div>
  );
}
