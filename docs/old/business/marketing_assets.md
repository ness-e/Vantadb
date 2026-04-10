# NexusDB Launch Assets (GTM Copy)

Este documento contiene los borradores estratégicos (Copywriting) para el lanzamiento v1.0, dirigidos específicamente a la comunidad de ingeniería de software (HackerNews) y a creadores (Dev.to / Medium).

---

## 1. HackerNews Launch

**Title:** Show HN: NexusDB – Rust DB unifying vectors, graphs & SQL in a single 15MB binary

**First Comment (The "Founder's Hook"):**
> *"Hi HN,*
>
> *I built NexusDB because I was exhausted by the modern AI stack. To build a semi-decent RAG application today, architectures force us to deploy a vector DB for embeddings (like Qdrant or Pinecone), a graph DB for relationships (Neo4j), and Postgres for user metadata.* 
>
> *Then we stitch them together using hundreds of lines of fragile Python "glue-code" over high-latency network calls.*
>
> *NexusDB is the opposite approach. I built a 3-in-1 engine in Rust that runs entirely in-process using PyO3 bindings (think SQLite-VSS, but treating vectors and graphs as first-class citizens in the same query execution plan).* 
>
> *The result is zero network latency, zero cross-platform serialization overhead, and it boots cold in ~15MB of RAM. If you are memory-constrained, it has a deterministic 'Survival Mode' built via Cgroups that automatically downgrades the HNSW graphs to MMAP arrays rather than crashing your host server.*
>
> *It's open source (Apache 2.0). I'd love your worst, most critical feedback on the architecture."*

---

## 2. Dev.to / Medium Article (El "Por Qué")

**Article Title:** Why I built a 3-in-1 database in Rust (and it fits in 15MB)

**Initial Paragraph (The Hook):**

*"If you’ve built any LLM application this year, you already know the pain. You start with LangChain, excitedly connect to a shiny new Vector Database to store embeddings, and for the first weekend, everything is amazing. Then, reality hits. Your vectors don't understand hierarchical relations, so you boot up a Graph DB. You need strict filtering, so you spin up PostgreSQL. Suddenly, your weekend project requires three different Docker containers, $120 a month in cloud bills, and a spaghetti-mess of Python HTTP requests just to figure out 'Which documents mention Alice AND are conceptually similar to Machine Learning?'. I decided to fix this by throwing out the network layer entirely."*

**Outline / Structure for the rest of the article:**
1.  **The Glue-Code Nightmare:** Explain network latency and serialization bottlenecks. Show the traditional stack architecture comparing REST/gRPC calls vs Shared Memory.
2.  **Why Rust + PyO3:** Explain the "SQLite-like" in-process deployment. Detail the FFI boundary and how zero-copy architecture makes local interactions completely synchronous and instantly fast.
3.  **Survival Mode under the Hood:** Dive into the memory constraints section. Explain to the audience how traditional HNSW heaps crash servers out-of-memory and how NexusDB gracefully downgrades to MMAP.
4.  **Show Me The Code:** Paste the 15-line quickstart script executing a hybrid (vector + relational) search locally.
5.  **Conclusion & Call to Action:** Provide links to GitHub, emphasize the open-core license, and ask the community for contributions and testing.
