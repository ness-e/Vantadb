import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { TextPlugin } from "gsap/TextPlugin";
import { DrawSVGPlugin } from "gsap/DrawSVGPlugin";
import { useGSAP } from "@gsap/react";

gsap.registerPlugin(ScrollTrigger, TextPlugin, DrawSVGPlugin, useGSAP);

export { gsap, useGSAP, ScrollTrigger, TextPlugin, DrawSVGPlugin };

// Dummy export to force bundlers to preserve this module and its side effects
export const __gsap_plugins_registered = true;
