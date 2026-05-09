import type { Metadata, Viewport } from "next";

import { LangProvider } from "@/app/components/LangProvider";
import { getLang } from "@/lib/lang-server";

import "./globals.css";

export const metadata: Metadata = {
  title: "Genasis Trial",
  description:
    "Genasis Trial — interactive agentic-team demo with hosted Plane + Mattermost trial signup",
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
};

export default async function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const lang = await getLang();
  return (
    <html lang={lang}>
      <body className="min-h-screen antialiased">
        <a href="#main-content" className="skip-to-content">
          {lang === "ko" ? "본문으로 건너뛰기" : "Skip to main content"}
        </a>
        <LangProvider initialLang={lang}>{children}</LangProvider>
      </body>
    </html>
  );
}
