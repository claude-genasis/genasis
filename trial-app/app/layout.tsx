import type { Metadata } from "next";

import { LangProvider } from "@/app/components/LangProvider";
import { getLang } from "@/lib/lang-server";

import "./globals.css";

export const metadata: Metadata = {
  title: "Genasis Trial",
  description:
    "Genasis Trial — interactive agentic-team demo with hosted Plane + Mattermost trial signup",
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
        <LangProvider initialLang={lang}>{children}</LangProvider>
      </body>
    </html>
  );
}
