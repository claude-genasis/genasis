"use client";

import { useRouter } from "next/navigation";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

import { LANG_COOKIE, t as translate, type Lang } from "@/lib/i18n";

type Ctx = {
  lang: Lang;
  setLang: (l: Lang) => void;
  t: (key: string, params?: Record<string, string | number>) => string;
};

const LangCtx = createContext<Ctx | null>(null);

export function LangProvider({
  initialLang,
  children,
}: {
  initialLang: Lang;
  children: ReactNode;
}) {
  const router = useRouter();
  const [lang, setLangState] = useState<Lang>(initialLang);

  // Sync state to server-driven prop changes (e.g. after router.refresh()
  // following a cookie write from another tab or the LangSwitcher).
  useEffect(() => {
    setLangState(initialLang);
  }, [initialLang]);

  const setLang = useCallback(
    (l: Lang) => {
      setLangState(l);
      // Persist for SSR on next request.
      document.cookie = `${LANG_COOKIE}=${l}; path=/; max-age=31536000; SameSite=Lax`;
      // Trigger a server tree re-render so server components also pick
      // up the new cookie value.
      router.refresh();
    },
    [router],
  );

  const value = useMemo<Ctx>(
    () => ({
      lang,
      setLang,
      t: (key, params) => translate(lang, key, params),
    }),
    [lang, setLang],
  );

  return <LangCtx.Provider value={value}>{children}</LangCtx.Provider>;
}

export function useLang(): Ctx {
  const ctx = useContext(LangCtx);
  if (!ctx) {
    throw new Error("useLang must be called within <LangProvider>");
  }
  return ctx;
}
