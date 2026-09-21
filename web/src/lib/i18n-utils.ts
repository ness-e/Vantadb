/**
 * i18n-utils — shared translation helpers.
 *
 * createTt wraps the dictionary lookup `t` with a fallback: when `t`
 * returns the key itself (missing translation), the hardcoded fallback
 * is shown instead. Exposed to components via LanguageProvider's `tt`.
 */
export type TranslateFn = (key: string, params?: Record<string, string>) => string;

export function createTt(t: TranslateFn) {
  return (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };
}
