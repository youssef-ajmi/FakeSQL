import { useEffect, useRef } from "react";
import { CheckCircle2, ChevronRight, Info, Terminal, XCircle } from "lucide-react";

import type { ConsoleEntry } from "../../lib/types";

interface ConsolePanelProps {
  entries: ConsoleEntry[];
}

function EntryIcon({ kind }: { kind: ConsoleEntry["kind"] }) {
  switch (kind) {
    case "command":
      return <ChevronRight size={13} className="console-icon command" />;
    case "success":
      return <CheckCircle2 size={13} className="console-icon success" />;
    case "error":
      return <XCircle size={13} className="console-icon error" />;
    default:
      return <Info size={13} className="console-icon info" />;
  }
}

export function ConsolePanel({ entries }: ConsolePanelProps) {
  const scrollRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const element = scrollRef.current;
    if (element) {
      element.scrollTop = element.scrollHeight;
    }
  }, [entries]);

  if (entries.length === 0) {
    return (
      <div className="panel-empty">
        <Terminal size={24} />
        <p>La console affichera les messages d'exécution.</p>
      </div>
    );
  }

  return (
    <div className="console-scroll" ref={scrollRef}>
      {entries.map((entry) => (
        <div className={`console-line ${entry.kind}`} key={entry.id}>
          <span className="console-time">{entry.time}</span>
          <EntryIcon kind={entry.kind} />
          <span className="console-text">{entry.text}</span>
        </div>
      ))}
    </div>
  );
}
