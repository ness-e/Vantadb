import { createFileRoute } from "@tanstack/react-router";
import { getPostBySlug } from "../../lib/blog";

export const Route = createFileRoute("/blog/$slug")({
  head: ({ params }) => {
    const post = getPostBySlug(params.slug);
    const title = post ? `${post.title} — VantaDB Blog` : "Post not found";
    const desc = post?.description || "";
    const url = `https://vantadb.dev/blog/${params.slug}`;
    return {
      meta: [
        { title },
        { name: "description", content: desc },
        { property: "og:title", content: title },
        { property: "og:description", content: desc },
        { property: "og:url", content: url },
      ],
      links: [{ rel: "canonical", href: url }],
    };
  },
});
