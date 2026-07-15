import { createFileRoute } from "@tanstack/react-router";
import { getCaseStudyBySlug } from "../../lib/case-studies";

export const Route = createFileRoute("/case-studies/$slug")({
  head: ({ params }) => {
    const study = getCaseStudyBySlug(params.slug);
    const title = study ? `${study.title} — VantaDB Case Studies` : "Case study not found";
    const desc = study?.description || "";
    const url = `https://vantadb.dev/case-studies/${params.slug}`;
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
