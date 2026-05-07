import Link from "next/link";

export default function StatusNotFound() {
  return (
    <main
      className="mx-auto max-w-2xl space-y-4 px-6 py-16 text-center"
      data-testid="status-not-found"
    >
      <h1 className="text-2xl font-semibold">신청 정보를 찾을 수 없습니다</h1>
      <p className="text-sm text-neutral-500">
        토큰이 잘못됐거나 만료됐을 수 있습니다. URL을 다시 확인해주세요.
      </p>
      <Link
        href="/?tab=signup"
        className="inline-block rounded-md bg-neutral-900 px-4 py-2 text-sm font-medium text-white hover:bg-neutral-700 dark:bg-white dark:text-neutral-900 dark:hover:bg-neutral-200"
      >
        신청 페이지로 이동
      </Link>
    </main>
  );
}
