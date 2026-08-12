"use client";

import { useState } from "react";

const ISSUES_URL = "https://github.com/abuzarkhan1/gitx/issues/new";

export default function IssueForm() {
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [sent, setSent] = useState<string | null>(null);

  function submit(event: React.FormEvent) {
    event.preventDefault();
    const params = new URLSearchParams();
    if (title.trim()) params.set("title", title.trim());
    if (body.trim()) {
      params.set(
        "body",
        `${body.trim()}\n\n---\n_sent from the gitx website_`,
      );
    }
    const url = `${ISSUES_URL}?${params.toString()}`;
    setSent(url);
    window.open(url, "_blank", "noopener,noreferrer");
  }

  return (
    <form className="term-form" onSubmit={submit}>
      <label htmlFor="issue-title">$ title</label>
      <input
        id="issue-title"
        type="text"
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        placeholder="bug: gitx crashes on empty repo"
      />
      <label htmlFor="issue-body">$ message</label>
      <textarea
        id="issue-body"
        rows={6}
        value={body}
        onChange={(e) => setBody(e.target.value)}
        placeholder={"steps to reproduce:\n\n$ gitx stats\n# what happened"}
      />
      <p className="hint">
        submits as a pre-filled GitHub issue — no account on this site, no data
        leaves your browser
      </p>
      <button className="btn" type="submit">
        $ send — open issue
      </button>
      {sent && (
        <div className="mt">
          <div className="line">
            <span className="prompt">
              <span className="user">user</span>
              <span className="host">@gitx</span>
              <span className="path">:~$</span>
            </span>{" "}
            <span className="cmd">gitx issue --open</span>
          </div>
          <div className="line out">
            <span>
              → opened{" "}
              <a href={sent} target="_blank" rel="noopener noreferrer">
                github.com/abuzarkhan1/gitx/issues/new
              </a>{" "}
              with your message pre-filled
            </span>
          </div>
        </div>
      )}
    </form>
  );
}
