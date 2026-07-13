function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

const KEYWORDS = new Set([
  "import",
  "from",
  "def",
  "return",
  "if",
  "not",
  "and",
  "or",
  "True",
  "False",
  "None",
  "as",
  "for",
  "in",
]);

export function highlightPython(cmd: string): string {
  let html = "";
  let i = 0;
  while (i < cmd.length) {
    if (cmd[i] === '"' || cmd[i] === "'") {
      const q = cmd[i];
      let j = i + 1;
      while (j < cmd.length && cmd[j] !== q) {
        if (cmd[j] === "\\") j++;
        j++;
      }
      if (j < cmd.length) j++;
      html += `<span class="qs-tok-str">${esc(cmd.slice(i, j))}</span>`;
      i = j;
    } else if (/[a-zA-Z_]/.test(cmd[i])) {
      let j = i;
      while (j < cmd.length && /\w/.test(cmd[j])) j++;
      const w = cmd.slice(i, j);
      if (KEYWORDS.has(w)) html += `<span class="qs-tok-kw">${esc(w)}</span>`;
      else html += esc(w);
      i = j;
    } else if (/\d/.test(cmd[i])) {
      let j = i;
      while (j < cmd.length && /[\d.]/.test(cmd[j])) j++;
      html += `<span class="qs-tok-num">${esc(cmd.slice(i, j))}</span>`;
      i = j;
    } else {
      html += esc(cmd[i]);
      i++;
    }
  }
  return html;
}
