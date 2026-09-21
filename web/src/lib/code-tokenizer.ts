// Shared line tokenizer for syntax highlighting (used by CodeTerminal for
// Python and CodePlayground for JavaScript). Single implementation with a
// per-language config — previously duplicated ~50 lines in each component.

export type Tok = {
  t: "plain" | "comment" | "string" | "number" | "keyword" | "builtin" | "func" | "ident" | "op";
  v: string;
};

export const TOK_CLASS: Record<Tok["t"], string> = {
  plain: "text-[#FBF9F5]",
  comment: "text-[#8a8a8a] italic",
  string: "text-[#FFB380]",
  number: "text-[#a3d9a5]",
  keyword: "text-[#FF5500] font-bold",
  builtin: "text-[#7ec7ff]",
  func: "text-[#ffd479]",
  ident: "text-[#FBF9F5]",
  op: "text-[#c9c9c9]",
};

type TokenizerConfig = {
  commentStarts: string[];
  stringRe: RegExp;
  identRe: RegExp;
  keywords: Set<string>;
  builtins: Set<string>;
  opRe: RegExp;
};

export function createTokenizer(cfg: TokenizerConfig) {
  return function tokenizeLine(line: string): Tok[] {
    const tokens: Tok[] = [];
    let i = 0;
    while (i < line.length) {
      const rest = line.slice(i);
      // comment
      if (cfg.commentStarts.some((c) => rest.startsWith(c))) {
        tokens.push({ t: "comment", v: rest });
        break;
      }
      // string (double/single quoted, optionally backtick, with escapes)
      const strMatch = rest.match(cfg.stringRe);
      if (strMatch) {
        tokens.push({ t: "string", v: strMatch[0] });
        i += strMatch[0].length;
        continue;
      }
      // number
      const numMatch = rest.match(/^\d[\d_]*(\.\d+)?/);
      if (numMatch) {
        tokens.push({ t: "number", v: numMatch[0] });
        i += numMatch[0].length;
        continue;
      }
      // identifier / keyword / builtin / function call
      const idMatch = rest.match(cfg.identRe);
      if (idMatch) {
        const word = idMatch[0];
        // Look ahead: if followed by "(" it's a function call
        const after = line[i + word.length];
        let t: Tok["t"] = "ident";
        if (cfg.keywords.has(word)) t = "keyword";
        else if (cfg.builtins.has(word)) t = "builtin";
        else if (after === "(") t = "func";
        tokens.push({ t, v: word });
        i += word.length;
        continue;
      }
      // operators / punctuation
      const opMatch = rest.match(cfg.opRe);
      if (opMatch) {
        tokens.push({ t: "op", v: opMatch[0] });
        i += opMatch[0].length;
        continue;
      }
      // whitespace run
      const wsMatch = rest.match(/^\s+/);
      if (wsMatch) {
        tokens.push({ t: "plain", v: wsMatch[0] });
        i += wsMatch[0].length;
        continue;
      }
      // single char
      tokens.push({ t: "plain", v: rest[0] });
      i += 1;
    }
    return tokens;
  };
}

export const pythonTokenizer = createTokenizer({
  commentStarts: ["#"],
  stringRe: /^("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')/,
  identRe: /^[A-Za-z_][A-Za-z0-9_]*/,
  keywords: new Set([
    "import", "as", "def", "return", "from", "class", "if", "else", "elif",
    "for", "while", "in", "not", "and", "or", "None", "True", "False",
    "with", "try", "except", "lambda", "pass", "break", "continue", "self",
  ]),
  builtins: new Set([
    "print", "len", "range", "str", "int", "float", "list", "dict", "set",
    "tuple", "bool", "open", "isinstance", "enumerate", "zip", "map", "filter",
    "sorted", "reversed", "sum", "min", "max", "abs", "round", "type", "format",
  ]),
  opRe: /^(==|!=|<=|>=|->|\+=|-=|\*=|\/\/=|\/\/|\*\*|[=+\-*/%<>:,.(){}\[\]])/,
});

export const jsTokenizer = createTokenizer({
  commentStarts: ["#", "//"],
  stringRe: /^("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|`(?:[^`\\]|\\.)*`)/,
  identRe: /^[A-Za-z_$][A-Za-z0-9_$]*/,
  keywords: new Set([
    "const", "let", "var", "function", "return", "if", "else", "for", "while",
    "do", "new", "class", "extends", "import", "export", "from", "async",
    "await", "try", "catch", "finally", "throw", "switch", "case", "break",
    "continue", "default", "typeof", "instanceof", "in", "of", "null", "true",
    "false", "undefined", "this", "delete", "void", "static", "get", "set",
  ]),
  builtins: new Set([
    "console", "Math", "JSON", "Object", "Array", "String", "Number",
    "Boolean", "Promise", "fetch", "performance", "parseInt", "parseFloat",
    "setTimeout", "setInterval", "Date",
  ]),
  opRe: /^(==|!=|<=|>=|->|=>|\+=|-=|\*=|\/\/=|\/\/|\*\*|[=+\-*/%<>:,.(){}\[\]])/,
});