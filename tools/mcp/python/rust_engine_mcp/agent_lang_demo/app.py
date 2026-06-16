"""AGENT-LANG demo UI — tkinter proof of multi-agent BLANG workflow."""

from __future__ import annotations

import json
import tkinter as tk
from tkinter import messagebox, scrolledtext, ttk

from .workflow import (
    AUTH_SPINE,
    AGENTS,
    DEMO_SCRIPT,
    REF_DEFAULT,
    WITNESS_PATH,
    DemoSession,
    _compact_json,
    delimiter,
    execute_blang,
    handoff,
    write_demo_witness,
)


class AgentLangDemoApp(tk.Tk):
    def __init__(self) -> None:
        super().__init__()
        self.title("AGENT-LANG — Agent Workflow Demo")
        self.geometry("960x720")
        self.minsize(780, 560)

        self.session = DemoSession()
        self._script_idx = 0

        self._agent_var = tk.StringVar(value="orchestrator")
        self._ref_var = tk.StringVar(value=REF_DEFAULT)
        self._status_var = tk.StringVar(value="Ready — Run Full Demo or step BLANG actions")

        self._build_header()
        self._build_agents()
        self._build_blang_bar()
        self._build_delimiters()
        self._build_output()
        self._build_log()

    def _build_header(self) -> None:
        hdr = ttk.Frame(self, padding=8)
        hdr.pack(fill=tk.X)
        ttk.Label(hdr, text="DSM", font=("", 9, "bold")).pack(side=tk.LEFT)
        ttk.Label(hdr, text=AUTH_SPINE, font=("Consolas", 10)).pack(side=tk.LEFT, padx=(8, 0))
        ttk.Label(self, textvariable=self._status_var, padding=(8, 0)).pack(fill=tk.X)

    def _build_agents(self) -> None:
        row = ttk.LabelFrame(self, text="Active agent", padding=8)
        row.pack(fill=tk.X, padx=8, pady=4)
        for i, name in enumerate(AGENTS):
            ttk.Radiobutton(
                row,
                text=f"@{name}",
                value=name,
                variable=self._agent_var,
                command=self._on_agent_change,
            ).grid(row=0, column=i, padx=6, sticky=tk.W)
        ttk.Label(row, text="$ref path:").grid(row=1, column=0, sticky=tk.W, pady=(8, 0))
        ttk.Entry(row, textvariable=self._ref_var, width=72).grid(
            row=1, column=1, columnspan=3, sticky=tk.EW, pady=(8, 0)
        )
        row.columnconfigure(1, weight=1)

    def _build_blang_bar(self) -> None:
        bar = ttk.LabelFrame(self, text="BLANG actions (live MCP)", padding=8)
        bar.pack(fill=tk.X, padx=8, pady=4)
        actions = [
            ("PRE", "PRE"),
            ("Guide", "GUIDE"),
            ("HO", "HO"),
            ("Q+", "Q+"),
            ("REF", "REF"),
            ("WIT", "WIT"),
            ("RUN", "RUN"),
            ("Q✓", "Q✓"),
        ]
        for i, (label, action) in enumerate(actions):
            ttk.Button(bar, text=f"BLANG:{label}", width=10, command=lambda a=action: self._run(a)).grid(
                row=0, column=i, padx=2, pady=2
            )
        ttk.Button(bar, text="Run Full Demo", command=self._run_full_demo).grid(row=0, column=8, padx=12)
        ttk.Button(bar, text="Next demo step", command=self._run_next_script).grid(row=0, column=9, padx=2)
        ttk.Button(bar, text="Write witness 🟢", command=self._write_witness).grid(row=0, column=10, padx=2)

    def _build_delimiters(self) -> None:
        row = ttk.LabelFrame(self, text="Handoff & stream delimiters", padding=8)
        row.pack(fill=tk.X, padx=8, pady=4)
        for i, agent in enumerate(("planner-mcp", "coder-mcp", "orchestrator", "operator")):
            ttk.Button(
                row,
                text=f"ΔWF→@{agent}",
                command=lambda a=agent: self._do_handoff(a),
            ).grid(row=0, column=i, padx=4)
        for j, tag in enumerate(("BRK", "CONT", "DRIFT")):
            ttk.Button(row, text=f"⟨{tag}⟩", command=lambda t=tag: self._do_delim(t)).grid(
                row=1, column=j, padx=4, pady=(6, 0)
            )

    def _build_output(self) -> None:
        frame = ttk.LabelFrame(self, text="AGENT-LANG paste + MCP payload", padding=8)
        frame.pack(fill=tk.BOTH, expand=True, padx=8, pady=4)
        self._paste = tk.Text(frame, height=4, font=("Segoe UI", 10), wrap=tk.WORD)
        self._paste.pack(fill=tk.X)
        self._payload = scrolledtext.ScrolledText(frame, height=14, font=("Consolas", 9), wrap=tk.WORD)
        self._payload.pack(fill=tk.BOTH, expand=True, pady=(8, 0))

    def _build_log(self) -> None:
        frame = ttk.LabelFrame(self, text="Inter-agent log", padding=8)
        frame.pack(fill=tk.BOTH, expand=True, padx=8, pady=(0, 8))
        self._log = scrolledtext.ScrolledText(frame, height=8, font=("Consolas", 9), wrap=tk.WORD)
        self._log.pack(fill=tk.BOTH, expand=True)

    def _on_agent_change(self) -> None:
        self.session.active_agent = self._agent_var.get()

    def _show_result(self, result) -> None:
        self._paste.delete("1.0", tk.END)
        self._paste.insert(tk.END, result.paste)
        self._payload.delete("1.0", tk.END)
        self._payload.insert(tk.END, _compact_json(result.payload))
        self._log.insert(tk.END, result.paste + "\n")
        self._log.see(tk.END)
        st = "🟢" if result.ok else "🔴"
        self._status_var.set(f"{st} {result.blang} · @{result.agent} · {len(self.session.tools_called)} tools")

    def _run(self, action: str) -> None:
        self.session.active_agent = self._agent_var.get()
        result = execute_blang(action, self.session, ref=self._ref_var.get())
        self._show_result(result)

    def _do_handoff(self, to_agent: str) -> None:
        result = handoff(self.session, to_agent)
        self._agent_var.set(to_agent)
        self._show_result(result)

    def _do_delim(self, tag: str) -> None:
        result = delimiter(self.session, tag)
        self._show_result(result)

    def _run_next_script(self) -> None:
        if self._script_idx >= len(DEMO_SCRIPT):
            messagebox.showinfo("Demo", "Script complete — click Write witness or Run Full Demo to restart.")
            return
        step = DEMO_SCRIPT[self._script_idx]
        if step.agent:
            self.session.active_agent = step.agent
            self._agent_var.set(step.agent)
        result = step.fn(self.session)
        self._script_idx += 1
        if result:
            self._show_result(result)
        self._status_var.set(f"Demo step {self._script_idx}/{len(DEMO_SCRIPT)}: {step.label}")

    def _run_full_demo(self) -> None:
        self.session = DemoSession()
        self._script_idx = 0
        self._log.delete("1.0", tk.END)
        self._log.insert(tk.END, f"=== Full demo start · {AUTH_SPINE} ===\n")
        while self._script_idx < len(DEMO_SCRIPT):
            self._run_next_script()
        wit = write_demo_witness(self.session)
        self._log.insert(tk.END, f"\n=== Witness 🟢 → {wit.get('written', WITNESS_PATH)} ===\n")
        self._log.see(tk.END)
        brief = execute_blang("WIT", self.session, witness_path=WITNESS_PATH)
        self._show_result(brief)
        messagebox.showinfo(
            "Demo complete",
            f"Workflow proof written to:\n{wit.get('written', WITNESS_PATH)}\n\n"
            f"Steps: {len(self.session.steps)} · Tools: {', '.join(self.session.tools_called)}",
        )

    def _write_witness(self) -> None:
        wit = write_demo_witness(self.session)
        self._log.insert(tk.END, f"Witness written → {wit.get('written')}\n")
        self._log.see(tk.END)
        messagebox.showinfo("Witness", json.dumps(wit, indent=2)[:800])


def run_app() -> None:
    app = AgentLangDemoApp()
    app.mainloop()
