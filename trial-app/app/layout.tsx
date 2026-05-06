import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Genasis Trial",
  description:
    "Genasis Trial — interactive agentic-team demo with hosted Plane + Mattermost trial signup",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="ko">
      <body className="min-h-screen antialiased">{children}</body>
    </html>
  );
}
