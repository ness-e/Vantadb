import { useGSAP } from "@gsap/react";
import { gsap } from "gsap";
import { DrawSVGPlugin } from "gsap/DrawSVGPlugin";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { TextPlugin } from "gsap/TextPlugin";

gsap.registerPlugin(ScrollTrigger, TextPlugin, DrawSVGPlugin, useGSAP);

export { DrawSVGPlugin, gsap, ScrollTrigger, TextPlugin, useGSAP };

// Dummy export to force bundlers to preserve this module and its side effects
export const __gsap_plugins_registered = true;
