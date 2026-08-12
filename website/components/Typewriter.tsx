"use client";

import { useEffect, useState } from "react";

/**
 * Types out `command` one character at a time, then shows `children`
 * (the command's output). Skips the animation for reduced-motion users.
 */
export default function Typewriter({
  command,
  children,
  speed = 55,
}: {
  command: string;
  children?: React.ReactNode;
  speed?: number;
}) {
  const [typed, setTyped] = useState("");
  const [done, setDone] = useState(false);

  useEffect(() => {
    const prefersReduced =
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (prefersReduced) {
      setTyped(command);
      setDone(true);
      return;
    }
    let i = 0;
    const timer = setInterval(() => {
      i += 1;
      setTyped(command.slice(0, i));
      if (i >= command.length) {
        clearInterval(timer);
        setDone(true);
      }
    }, speed);
    return () => clearInterval(timer);
  }, [command, speed]);

  return (
    <>
      <div className="line">
        <span className="prompt">
          <span className="user">user</span>
          <span className="host">@gitx</span>
          <span className="path">:~$</span>
        </span>{" "}
        <span className="cmd">{typed}</span>
        {!done && <span className="cursor" aria-hidden="true" />}
      </div>
      {done && <div className="mt">{children}</div>}
    </>
  );
}
