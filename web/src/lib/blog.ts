import { marked } from "marked";

export interface BlogPost {
  slug: string;
  title: string;
  date: string;
  description: string;
  author: string;
  tags?: string[];
  body: string;
  html: string;
}

function parseFrontmatter(raw: string): {
  data: Record<string, string | string[]>;
  content: string;
} {
  const match = raw.match(/^---\s*\n([\s\S]*?)\n---\s*\n?([\s\S]*)$/);
  if (!match) return { data: {}, content: raw };
  const data: Record<string, string | string[]> = {};
  for (const line of match[1].split("\n")) {
    const idx = line.indexOf(":");
    if (idx === -1) continue;
    const key = line.slice(0, idx).trim();
    let val: string | string[] = line
      .slice(idx + 1)
      .trim()
      .replace(/^["']|["']$/g, "");
    if (val.startsWith("[") && val.endsWith("]")) {
      val = val
        .slice(1, -1)
        .split(",")
        .map((s) => s.trim().replace(/^["']|["']$/g, ""));
    }
    data[key] = val;
  }
  return { data, content: match[2] };
}

const modules = import.meta.glob("/content/blog/*.md", {
  query: "?raw",
  import: "default",
  eager: true,
});

export function getAllPosts(): BlogPost[] {
  const posts: BlogPost[] = [];

  for (const [path, raw] of Object.entries(modules)) {
    if (typeof raw !== "string") continue;
    const slug = path.replace("/content/blog/", "").replace(/\.md$/, "");
    const { data, content } = parseFrontmatter(raw);
    const html = marked.parse(content) as string;

    posts.push({
      slug,
      title: (data.title as string) || slug,
      date: (data.date as string) || "",
      description: (data.description as string) || "",
      author: (data.author as string) || "VantaDB Team",
      tags: (data.tags as string[]) || [],
      body: content,
      html,
    });
  }

  return posts
    .filter((p) => p.date && !isNaN(new Date(p.date).getTime()))
    .sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime());
}

export function getPostBySlug(slug: string): BlogPost | undefined {
  return getAllPosts().find((p) => p.slug === slug);
}
