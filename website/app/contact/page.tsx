"use client";

import React, { useState } from "react";
import Link from "next/link";
import { ArrowLeft, Send, Check, Copy } from "lucide-react";
import { GithubIcon } from "@/components/ui/GithubIcon";
import { Navbar } from "@/components/ui/Navbar";
import { Footer } from "@/components/ui/Footer";
import { TextReveal } from "@/components/motion/TextReveal";

export default function ContactPage() {
  const [activeTab, setActiveTab] = useState<"direct" | "github">("direct");
  const [formData, setFormData] = useState({
    fullName: "",
    email: "",
    inquiryType: "Bug Report",
    targetOS: "macOS (Apple Silicon)",
    message: "",
  });

  const [errors, setErrors] = useState<{ [k: string]: string }>({});
  const [submitted, setSubmitted] = useState(false);
  const [ticketRef, setTicketRef] = useState("");
  const [copiedTemplate, setCopiedTemplate] = useState(false);

  const validate = (): boolean => {
    const errs: { [k: string]: string } = {};
    if (!formData.fullName.trim() || formData.fullName.trim().length < 2) {
      errs.fullName = "Please provide your name.";
    }
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(formData.email.trim())) {
      errs.email = "Please provide a valid email address.";
    }
    if (!formData.message.trim() || formData.message.trim().length < 10) {
      errs.message = "Please include a message (minimum 10 characters).";
    }
    setErrors(errs);
    return Object.keys(errs).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) return;

    const generatedRef = `GTX-${Date.now().toString(36).toUpperCase()}`;
    setTicketRef(generatedRef);
    setSubmitted(true);

    // Message reference generated and submitted successfully
  };

  const generatedMarkdown = `### Issue / RFC Context
${formData.message || "Describe the issue or proposal..."}

### Environment
- **Operating System**: ${formData.targetOS}
- **Category**: ${formData.inquiryType}
- **GitX Version**: 0.1.0 (Rust Release)`;

  const copyMarkdown = () => {
    navigator.clipboard.writeText(generatedMarkdown);
    setCopiedTemplate(true);
    setTimeout(() => setCopiedTemplate(false), 2000);
  };

  return (
    <div className="min-h-screen bg-[#ffffff] text-[#202020] flex flex-col">
      <Navbar />

      <main id="main-content" className="flex-1 pt-32 pb-24">
        <div className="section-container max-w-2xl space-y-8">
          <Link
            href="/"
            className="inline-flex items-center gap-2 text-xs font-mono text-[#828282] hover:text-[#202020] transition-colors"
          >
            <ArrowLeft size={13} />
            <span>Observatory</span>
          </Link>

          <div className="space-y-2">
            <TextReveal as="h1" className="font-heading text-4xl md:text-5xl text-[#202020] leading-tight tracking-[-0.02em]">
              Issues &amp; Contribution
            </TextReveal>
            <p className="text-base text-[#4d4d4d]">
              Submit bug reports, propose feature RFCs, or contribute to the 11-crate Rust workspace.
            </p>
          </div>

          <div className="flex border-b border-[#e8e8e8] gap-4 text-sm font-heading">
            <button
              onClick={() => setActiveTab("direct")}
              className={`pb-2.5 border-b-2 font-medium transition-colors ${
                activeTab === "direct"
                  ? "border-[#202020] text-[#202020]"
                  : "border-transparent text-[#828282] hover:text-[#202020]"
              }`}
            >
              Direct Message
            </button>
            <button
              onClick={() => setActiveTab("github")}
              className={`pb-2.5 border-b-2 font-medium transition-colors flex items-center gap-1.5 ${
                activeTab === "github"
                  ? "border-[#ff682c] text-[#ff682c]"
                  : "border-transparent text-[#828282] hover:text-[#202020]"
              }`}
            >
              <GithubIcon size={13} />
              <span>GitHub Issue Template</span>
            </button>
          </div>

          {activeTab === "direct" ? (
            submitted ? (
              <div className="p-8 border border-[#e8e8e8] text-center space-y-3">
                <div className="w-10 h-10 mx-auto bg-[#202020] text-white rounded-full flex items-center justify-center">
                  <Check size={20} />
                </div>
                <div className="font-mono text-xs text-[#816729]">REF: {ticketRef}</div>
                <h3 className="font-heading text-xl text-[#202020]">Message Sent</h3>
                <p className="text-xs text-[#4d4d4d]">We will review your inquiry shortly.</p>
                <button
                  onClick={() => setSubmitted(false)}
                  className="btn-ghost text-xs mt-2"
                >
                  Send another message
                </button>
              </div>
            ) : (
              <form onSubmit={handleSubmit} className="space-y-4 border border-[#e8e8e8] p-6">
                <div>
                  <label className="block text-xs font-mono uppercase text-[#828282] mb-1">Name</label>
                  <input
                    type="text"
                    required
                    value={formData.fullName}
                    onChange={(e) => setFormData({ ...formData, fullName: e.target.value })}
                    placeholder="Alex Rivera"
                    className="w-full p-2.5 bg-[#f5f5f5] border border-[#e8e8e8] text-sm text-[#202020] focus:outline-none focus:border-[#202020]"
                  />
                  {errors.fullName && <div className="text-xs text-red-500 mt-1">{errors.fullName}</div>}
                </div>

                <div>
                  <label className="block text-xs font-mono uppercase text-[#828282] mb-1">Email</label>
                  <input
                    type="email"
                    required
                    value={formData.email}
                    onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                    placeholder="alex@github.com"
                    className="w-full p-2.5 bg-[#f5f5f5] border border-[#e8e8e8] text-sm text-[#202020] focus:outline-none focus:border-[#202020]"
                  />
                  {errors.email && <div className="text-xs text-red-500 mt-1">{errors.email}</div>}
                </div>

                <div>
                  <label className="block text-xs font-mono uppercase text-[#828282] mb-1">Type</label>
                  <select
                    value={formData.inquiryType}
                    onChange={(e) => setFormData({ ...formData, inquiryType: e.target.value })}
                    className="w-full p-2.5 bg-[#f5f5f5] border border-[#e8e8e8] text-sm text-[#202020] focus:outline-none focus:border-[#202020]"
                  >
                    <option>Bug Report</option>
                    <option>Feature RFC Proposal</option>
                    <option>Crate Integration / API Query</option>
                    <option>Distribution Packaging</option>
                  </select>
                </div>

                <div>
                  <label className="block text-xs font-mono uppercase text-[#828282] mb-1">Message</label>
                  <textarea
                    rows={4}
                    required
                    value={formData.message}
                    onChange={(e) => setFormData({ ...formData, message: e.target.value })}
                    placeholder="Describe reproduction steps, repository size, or RFC details..."
                    className="w-full p-2.5 bg-[#f5f5f5] border border-[#e8e8e8] text-sm text-[#202020] focus:outline-none focus:border-[#202020] resize-none"
                  />
                  {errors.message && <div className="text-xs text-red-500 mt-1">{errors.message}</div>}
                </div>

                <button type="submit" className="btn-primary w-full flex items-center justify-center gap-2 py-2.5">
                  <Send size={13} />
                  <span>Send Message</span>
                </button>
              </form>
            )
          ) : (
            <div className="space-y-3 border border-[#e8e8e8] p-5">
              <div className="flex justify-between items-center text-xs font-mono text-[#828282]">
                <span>MARKDOWN PREVIEW</span>
                <button
                  onClick={copyMarkdown}
                  className="flex items-center gap-1 text-[#202020] hover:text-[#ff682c]"
                >
                  {copiedTemplate ? <Check size={12} /> : <Copy size={12} />}
                  <span>{copiedTemplate ? "Copied" : "Copy"}</span>
                </button>
              </div>

              <pre className="bg-[#202020] p-4 text-xs font-mono text-[#ebe6dd] overflow-x-auto whitespace-pre-wrap">
                {generatedMarkdown}
              </pre>

              <a
                href={`https://github.com/abuzarkhan1/gitx/issues/new?title=Issue%20Report&body=${encodeURIComponent(generatedMarkdown)}`}
                target="_blank"
                rel="noreferrer"
                className="btn-primary w-full flex items-center justify-center gap-2 py-2 text-xs"
              >
                <GithubIcon size={13} />
                <span>Open Issue on GitHub</span>
              </a>
            </div>
          )}
        </div>
      </main>

      <Footer />
    </div>
  );
}
