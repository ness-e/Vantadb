"use client";

import { FileX2, Settings2, DatabaseZap, KeyRound, ArrowRight, Check, type LucideIcon } from "lucide-react";
import Link from "next/link";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { VANTA } from "@/components/vanta/vanta-data";
import { useLanguage } from "@/lib/language-provider";

const ICONS: Record<string, LucideIcon> = {
  FileX2,
  Settings2,
  DatabaseZap,
  KeyRound,
};

export default function ConfigPage() {
  const { tt } = useLanguage();

  const benefits = [
    {
      icon: "FileX2",
      title: tt("configPage.benefits.0.title", "No schema"),
      body: tt(
        "configPage.benefits.0.body",
        "Olvidate de CREATE TABLE, índices predefinidos, migraciones de schema. VantaDB infiere la estructura de tus documentos y vectores al vuelo. Los índices BM25 y HNSW se construyen automáticamente."
      ),
    },
    {
      icon: "Settings2",
      title: tt("configPage.benefits.1.title", "No config files"),
      body: tt(
        "configPage.benefits.1.body",
        "Cero YAML, cero TOML, cero .env. La configuración vive en código Python: argumentos del constructor, defaults sensibles. Si lo necesitás cambiar, lo cambiás en una línea de Python."
      ),
    },
    {
      icon: "DatabaseZap",
      title: tt("configPage.benefits.2.title", "No migrations"),
      body: tt(
        "configPage.benefits.2.body",
        "No hay scripts de migración para correr, no hay versiones de schema que trackear. El motor evoluciona el formato del archivo WAL y los índices derivados de forma transparente entre releases."
      ),
    },
    {
      icon: "KeyRound",
      title: tt("configPage.benefits.3.title", "No secrets"),
      body: tt(
        "configPage.benefits.3.body",
        "Sin API keys, sin tokens, sin credenciales rotativas. VantaDB corre in-process en tu máquina. Lo que almacenás se queda en tu filesystem. No hay secretos que gestionar porque no hay superficie que los requiera."
      ),
    },
  ];

  const flow = [
    "pip install vantadb-py",
    "import vantadb",
    "db = vantadb.connect()",
    "db.put(text, vector)",
  ];

  return (
    <div className="animate-rise">
      <PageHeader
        badge="§CONFIG"
        title={tt("configPage.title", "Zero Configuration")}
        subtitle={tt(
          "configPage.subtitle",
          "Schema-free por diseño. No YAML, no .env, no migraciones, no secretos. Solo `pip install vantadb-py` y connect."
        )}
        tag={tt("configPage.tag", "Schema-Free · pip install")}
      />

      {/* Benefits grid */}
      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="mb-10 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-black bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black ">
                <span className="h-1.5 w-1.5 bg-black" />
                {tt("configPage.benefitsTag", "Benefits")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                {tt("configPage.benefitsTitle", "Four things you don't need")}
              </h2>
            </div>
            <span className="font-tech text-[10px] uppercase tracking-wider text-black/50 ">
              {tt("configPage.validatedTag", "By design · not by default")}
            </span>
          </div>
        </Reveal>

        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {benefits.map((b, i) => {
            const Icon = ICONS[b.icon] ?? FileX2;
            return (
              <Reveal key={b.title} direction="up" delay={i * 80} as="article">
                <article className="press-lg group relative flex h-full flex-col border-4 border-black bg-[#F2EDE2] p-5  ">
                  <span className="absolute -left-2 -top-3 rotate-[-6deg] border-2 border-black bg-black px-2 py-0.5 font-display text-xs uppercase text-[#FF5500]   ">
                    0{i + 1}
                  </span>
                  <span className="mb-4 inline-flex h-12 w-12 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                    <Icon className="h-6 w-6" strokeWidth={2.5} />
                  </span>
                  <h3 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                    {b.title}
                  </h3>
                  <p className="mt-3 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                    {b.body}
                  </p>
                  <div className="mt-4 h-1 w-full speed-lines opacity-30" />
                </article>
              </Reveal>
            );
          })}
        </div>
      </PageSection>

      {/* Terminal flow */}
      <PageSection variant="paper">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500] ">
                <span className="h-1.5 w-1.5 bg-[#FF5500]" />
                {tt("configPage.flowTag", "Get started")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                {tt("configPage.flowTitle", "Four lines. No YAML.")}
              </h2>
              <p className="mt-2 max-w-2xl font-tech text-xs text-black/70 ">
                {tt(
                  "configPage.flowSubtitle",
                  "Eso es todo lo que hace falta para tener hybrid search local-first con crash recovery. Sin provisionamiento, sin cuentas, sin setup."
                )}
              </p>
            </div>
          </div>
        </Reveal>

        <Reveal direction="up" delay={80}>
          <div className="border-4 border-black bg-black p-4 font-tech text-sm shadow-[6px_6px_0_0_#FF5500]  sm:p-6">
            <div className="mb-3 flex items-center gap-2 border-b-2 border-[#FBF9F5]/20 pb-2">
              <span className="h-3 w-3 rounded-full bg-[#FF5500]" />
              <span className="h-3 w-3 rounded-full bg-[#FBF9F5]/40" />
              <span className="h-3 w-3 rounded-full bg-[#FBF9F5]/40" />
              <span className="ml-2 font-tech text-[10px] uppercase tracking-wider text-[#FBF9F5]/40">
                vantadb_quickstart.py
              </span>
            </div>
            <div className="flex flex-col gap-2">
              {flow.map((line, i) => (
                <div key={line} className="flex items-baseline gap-3">
                  <span className="font-tech text-[10px] text-[#FF5500]">{String(i + 1).padStart(2, "0")}</span>
                  <code className="font-tech text-sm text-[#FBF9F5]">
                    {line.startsWith("pip install") ? (
                      <span>
                        <span className="text-[#FF5500]">$</span> {line}
                      </span>
                    ) : (
                      <span>
                        <span className="text-[#FF5500]">&gt;&gt;&gt;</span> {line}
                      </span>
                    )}
                  </code>
                </div>
              ))}
              <div className="mt-2 flex items-center gap-2 border-t-2 border-[#FBF9F5]/20 pt-2 font-tech text-xs text-[#FBF9F5]/60">
                <Check className="h-3 w-3 text-[#FF5500]" strokeWidth={3} />
                {tt("configPage.flowDone", "Done. No YAML. No .env. No migrations.")}
              </div>
            </div>
          </div>
        </Reveal>

        <Reveal direction="up" delay={120}>
          <div className="mt-8 flex flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row">
            <div className="flex items-center gap-3">
              <Check className="h-5 w-5 shrink-0 text-[#FF5500]" strokeWidth={3} />
              <div>
                <h3 className="font-display text-xl uppercase leading-none text-[#FBF9F5]">
                  {tt("configPage.ctaTitle", "Install in 30 seconds")}
                </h3>
                <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                  {tt("configPage.ctaBody", "pip install vantadb-py — schema-free, no setup required.")}
                </p>
              </div>
            </div>
            <Link
              href={VANTA.pypi}
              target="_blank"
              rel="noopener noreferrer"
              className="press-neon btn-neon-glow inline-flex shrink-0 items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
            >
              pip install vantadb-py
              <ArrowRight className="h-4 w-4" strokeWidth={2.5} />
            </Link>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
