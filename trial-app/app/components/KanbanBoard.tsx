export type KanbanColumn = "todo" | "inprogress" | "done";

export type KanbanCard = {
  id: number;
  title: string;
  column: KanbanColumn;
};

const COLUMNS: {
  key: KanbanColumn;
  label: string;
  headerClass: string;
  countClass: string;
}[] = [
  {
    key: "todo",
    label: "Todo",
    headerClass: "bg-neutral-100 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-200",
    countClass: "bg-neutral-200 text-neutral-700 dark:bg-neutral-700 dark:text-neutral-200",
  },
  {
    key: "inprogress",
    label: "In Progress",
    headerClass: "bg-blue-100 text-blue-800 dark:bg-blue-950 dark:text-blue-200",
    countClass: "bg-blue-200 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  },
  {
    key: "done",
    label: "Done",
    headerClass: "bg-green-100 text-green-800 dark:bg-green-950 dark:text-green-200",
    countClass: "bg-green-200 text-green-800 dark:bg-green-900 dark:text-green-200",
  },
];

export function KanbanBoard({ cards }: { cards: KanbanCard[] }) {
  return (
    <div
      role="list"
      aria-label="Kanban board"
      className="grid h-[420px] grid-cols-1 gap-4 sm:grid-cols-3"
      data-testid="kanban-board"
    >
      {COLUMNS.map(({ key, label, headerClass, countClass }) => {
        const columnCards = cards.filter((card) => card.column === key);
        return (
          <section
            key={key}
            role="listitem"
            aria-label={`${label} column`}
            data-column={key}
            className="flex h-full flex-col overflow-hidden rounded-lg border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-950"
          >
            <header
              className={`flex items-center justify-between px-3 py-2 text-sm font-semibold ${headerClass}`}
            >
              <span>{label}</span>
              <span
                aria-label={`${columnCards.length} cards`}
                className={`rounded-full px-2 py-0.5 text-xs font-medium ${countClass}`}
              >
                {columnCards.length}
              </span>
            </header>
            <ol className="flex-1 space-y-2 overflow-y-auto p-3">
              {columnCards.map((card) => (
                <li
                  key={card.id}
                  data-card-id={card.id}
                  className="rounded-md border border-neutral-200 bg-white px-3 py-2 text-sm shadow-sm transition-colors dark:border-neutral-700 dark:bg-neutral-900"
                >
                  <span className="mr-2 font-mono text-xs text-neutral-500 dark:text-neutral-400">
                    #{card.id}
                  </span>
                  <span className="text-neutral-900 dark:text-neutral-100">
                    {card.title}
                  </span>
                </li>
              ))}
            </ol>
          </section>
        );
      })}
    </div>
  );
}
