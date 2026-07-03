import { createFileRoute } from "@tanstack/react-router";
import { getPostBySlug } from "../../lib/blog";

export const Route = createFileRoute("/blog/$slug")({
  head: ({ params }) => {
    const post = getPostBySlug(params.slug);
    return {
      meta: [
        { title: post ? `${post.title} — VantaDB Blog` : "Post not found" },
        { name: "description", content: post?.description || "" },
      ],
      links: [{ rel: "canonical", href: `https://vantadb.dev/blog/${params.slug}` }],
    };
  },
});
