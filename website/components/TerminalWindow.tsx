import type { ReactNode } from "react";

export default function TerminalWindow({
  title,
  right,
  children,
}: {
  title: string;
  right?: string;
  children: ReactNode;
}) {
  return (
    <div className="win">
      <div className="win-bar">
        <span className="dots" aria-hidden="true">
          <span className="dot dot-green" />
          <span className="dot dot-amber" />
          <span className="dot dot-off" />
        </span>
        <span className="win-title">{title}</span>
        <span className="faint">{right ?? ""}</span>
      </div>
      <div className="win-body">{children}</div>
    </div>
  );
}
