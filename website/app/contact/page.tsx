import type { Metadata } from "next";
import TerminalWindow from "@/components/TerminalWindow";
import IssueForm from "@/components/IssueForm";

export const metadata: Metadata = { title: "contact" };

const CONTACT_LINES = [
  ["github", "https://github.com/abuzarkhan1/gitx"],
  ["issues", "https://github.com/abuzarkhan1/gitx/issues"],
  ["docs", "https://github.com/abuzarkhan1/gitx/tree/main/docs"],
  ["license", "MIT"],
];

export default function ContactPage() {
  return (
    <>
      <div className="line">
        <span className="faint">$ </span>
        <span className="cmd">gitx contact</span>
      </div>
      <h1 className="block-head">contact</h1>

      <div className="grid grid-2">
        <TerminalWindow title="gitx contact" right="open channel">
          <div className="line">
            <span className="prompt">
              <span className="user">user</span>
              <span className="host">@gitx</span>
              <span className="path">:~$</span>
            </span>{" "}
            <span className="cmd">gitx contact</span>
          </div>
          <ul className="kvlist mt">
            {CONTACT_LINES.map(([key, value]) => (
              <li key={key}>
                <span className="k">{key}</span>
                <span className="a">→</span>
                {key === "license" ? (
                  <span className="out">{value}</span>
                ) : (
                  <a href={value} target="_blank" rel="noopener noreferrer">
                    {value}
                  </a>
                )}
              </li>
            ))}
          </ul>
          <div className="line mt dim">
            <span>
              best channel: open an issue. bugs, ideas, archaeology stories —
              all welcome.
            </span>
          </div>
        </TerminalWindow>

        <section className="block" aria-labelledby="form-head">
          <h2 className="block-head" id="form-head">
            send a message
          </h2>
          <TerminalWindow title="gitx issue --new" right="pre-filled">
            <IssueForm />
          </TerminalWindow>
        </section>
      </div>

      <section className="block" aria-labelledby="faq-head">
        <h2 className="block-head" id="faq-head">
          answers
        </h2>
        <TerminalWindow title="gitx faq" right="3 lines">
          <ul className="features">
            <li>
              <b>Is it really offline?</b>{" "}
              <span className="out">
                yes — everything runs against your local repository, nothing is
                uploaded.
              </span>
            </li>
            <li>
              <b>Does it need AI?</b>{" "}
              <span className="out">
                no — every score is a deterministic formula over raw git
                signals, so you can audit it.
              </span>
            </li>
            <li>
              <b>Is it free?</b>{" "}
              <span className="out">
                yes — MIT licensed, installable in one line.
              </span>
            </li>
          </ul>
        </TerminalWindow>
      </section>
    </>
  );
}
